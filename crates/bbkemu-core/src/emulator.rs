//! Main emulator orchestrator

use anyhow::Result;

use crate::audio::Audio;
use crate::cheat::CheatEngine;
use crate::cpu::CpuWrapper;
use crate::debug::Debugger;
use crate::gam::GamFile;
use crate::input::{BbkKey, Input};
use crate::lcd::{Lcd, LcdOrientation, LcdTheme};
use crate::memory::Memory;
use crate::model::{BbkModel, MODEL_4980};
use crate::save::SaveState;

/// Main emulator struct
pub struct Emulator {
    /// 6502 CPU (owns memory)
    pub cpu: CpuWrapper,
    /// LCD display
    pub lcd: Lcd,
    /// Input system
    pub input: Input,
    /// Audio system
    pub audio: Audio,
    /// Debugger
    pub debug: Debugger,
    /// Cheat engine
    pub cheat: CheatEngine,
    /// Current model
    model: &'static BbkModel,
    /// LCD display orientation
    lcd_orientation: LcdOrientation,
    /// LCD color theme
    lcd_theme: LcdTheme,
    /// CPU clock rate multiplier (1.0 = normal speed)
    cpu_rate: f32,
    /// Timer clock rate multiplier (1.0 = normal speed)
    timer_rate: f32,
    /// Whether the emulator is running
    running: bool,
    /// Frame counter
    frame_count: u64,
    /// CPU cycles accumulated toward the next 400-cycle timer tick.
    timer_cycle_remainder: u32,
    /// Return points for compiler-runtime far calls handled by HLE.
    hle_far_calls: Vec<HleFarCall>,
}

struct HleFarCall {
    return_pc: u16,
    banks: [u32; 4],
}

const HLE_FAR_RETURN: u16 = 0x02F0;

impl Emulator {
    /// Create a new emulator instance
    pub fn new(model: &'static BbkModel) -> Self {
        let audio = Audio::new(44100);
        let memory = Memory::new();
        let cpu = CpuWrapper::new(memory);

        Self {
            cpu,
            lcd: Lcd::new(),
            input: Input::new(),
            audio,
            debug: Debugger::new(),
            cheat: CheatEngine::new(),
            model,
            lcd_orientation: LcdOrientation::Portrait,
            lcd_theme: LcdTheme::GREY,
            cpu_rate: 1.0,
            timer_rate: 1.0,
            running: false,
            frame_count: 0,
            timer_cycle_remainder: 0,
            hle_far_calls: Vec::new(),
        }
    }

    /// Create emulator with default model (A4980)
    pub fn new_default() -> Self {
        Self::new(&MODEL_4980)
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new(&MODEL_4980)
    }
}

impl Emulator {
    /// Load a GAM file
    pub fn load_gam(&mut self, data: &[u8]) -> Result<()> {
        let gam = GamFile::parse(data)?;
        log::info!(
            "Loading game: {} (entry: 0x{:04X})",
            gam.name(),
            gam.entry_point
        );

        // Initialize memory and run OS init
        self.cpu.memory_mut().init();
        self.cpu.reset();
        self.run_os_init();

        // Load game into flash at 0x20D000
        let flash_offset = 0xD000;
        let game_data = &gam.data;
        let end = (flash_offset + game_data.len()).min(self.cpu.memory().flash.len());
        self.cpu.memory_mut().flash[flash_offset..end]
            .copy_from_slice(&game_data[..end - flash_offset]);

        // Setup flash headers
        let sys_hdr = [
            0xC0u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
            0x00, 0x2F,
        ];
        let mut gam_hdr = [0u8; 16];
        gam_hdr[0] = 0xD0;
        gam_hdr[1] = 0x00;
        gam_hdr[2..12].copy_from_slice(&gam.info);
        let size = game_data.len();
        gam_hdr[12] = (size & 0xFF) as u8;
        gam_hdr[13] = ((size >> 8) & 0xFF) as u8;
        gam_hdr[14] = ((size >> 16) & 0xFF) as u8;
        gam_hdr[15] = 0x3D;

        let flash_base = 0x8000;
        self.cpu.memory_mut().flash[flash_base..flash_base + 16].copy_from_slice(&sys_hdr);
        self.cpu.memory_mut().flash[flash_base + 16..flash_base + 32].copy_from_slice(&gam_hdr);

        // Setup bank mappings
        self.cpu.memory_mut().bank_switch.set(0x5, 0x20D);
        self.cpu.memory_mut().bank_switch.set(0x6, 0x20E);
        self.cpu.memory_mut().bank_switch.set(0x7, 0x20F);
        self.cpu.memory_mut().bank_switch.set(0x8, 0x210);

        let data_bank = 0x20D + (gam.data_offset >> 12);
        self.cpu.memory_mut().bank_switch.set(0x9, data_bank);
        self.cpu.memory_mut().bank_switch.set(0xA, data_bank + 1);
        self.cpu.memory_mut().bank_switch.set(0xB, data_bank + 2);
        self.cpu.memory_mut().bank_switch.set(0xC, data_bank + 3);

        // Setup save area (offset differs between models)
        let save_base = if self.model.bank_sys_d == 0x0E88 {
            0x8000 // A4988
        } else {
            0x7000 // A4980
        };
        self.cpu.memory_mut().flash[flash_base + save_base + 0xF8] = 0x02;
        self.cpu.memory_mut().flash[flash_base + save_base + 0xF9] = 0x02;
        self.cpu.memory_mut().flash[flash_base + save_base + 0xFA] = 0x02;
        self.cpu.memory_mut().flash[flash_base + save_base + 0xFB] = 0x02;
        self.cpu.memory_mut().flash[flash_base + save_base + 0xFC] = 0x02;
        self.cpu.memory_mut().flash[flash_base + save_base + 0xFD] = 0x02;
        self.cpu.memory_mut().flash[flash_base + save_base + 0xFE] = 0x03;
        self.cpu.memory_mut().flash[flash_base + save_base + 0xFF] = 0x02;

        // Set system control
        self.cpu.memory_mut().write(0x2029, 0x0D);
        self.cpu.memory_mut().write(0x202A, 0x02);

        // Push BRK exit handler address onto stack
        let sp = self.cpu.sp();
        self.cpu.memory_mut().ram[0x100 | sp as usize] = 0x02;
        self.cpu.memory_mut().ram[0x100 | sp.wrapping_sub(1) as usize] = 0x60;
        self.cpu.set_sp(sp.wrapping_sub(2));
        self.cpu.set_pc(gam.entry_point);

        // Debug: check what's at the entry point
        let entry_opcode = self.cpu.memory().read(gam.entry_point);
        log::info!(
            "Entry point 0x{:04X} opcode: 0x{:02X}",
            gam.entry_point,
            entry_opcode
        );

        // Debug: check bank mapping
        let bank5 = self.cpu.memory().bank_switch.banks[5];
        log::info!("Bank 5 mapped to: 0x{:04X}", bank5);

        // Debug: check flash content at expected location
        let flash_offset = 0xD000 + (gam.entry_point & 0x0FFF) as usize;
        if flash_offset < self.cpu.memory().flash.len() {
            log::info!(
                "Flash[0x{:04X}] = 0x{:02X}",
                flash_offset,
                self.cpu.memory().flash[flash_offset]
            );
        }

        // Debug: check physical address translation
        let paddr = self.cpu.memory().bank_switch.translate(gam.entry_point);
        log::info!(
            "Physical address for 0x{:04X}: 0x{:08X}",
            gam.entry_point,
            paddr
        );

        // Debug: check read_physical
        let read_result = self.cpu.memory().read_physical(paddr);
        log::info!("read_physical(0x{:08X}) = 0x{:02X}", paddr, read_result);

        self.running = true;
        log::info!(
            "Game loaded, starting execution at 0x{:04X}",
            gam.entry_point
        );

        Ok(())
    }

    /// Load font ROM (8.BIN) - optional
    pub fn load_rom_8(&mut self, data: &[u8]) {
        self.cpu.memory_mut().load_rom_8(data);
        log::info!("Font ROM loaded ({} bytes)", data.len());
    }

    /// Load OS ROM (E.BIN) - optional, for LLE fallback
    pub fn load_rom_e(&mut self, data: &[u8]) {
        self.cpu.memory_mut().load_rom_e(data);
        log::info!("OS ROM loaded ({} bytes)", data.len());
    }

    /// Run OS initialization sequence
    /// This runs the OS code at 0x350 until _MTCT register becomes 0xFE
    pub fn run_os_init(&mut self) {
        log::info!("Running OS initialization...");

        // Set PC to OS entry point
        self.cpu.set_pc(0x350);

        // Run until _MTCT becomes 0xFE
        let mut max_cycles = 10_000_000; // Safety limit
        while max_cycles > 0 {
            let cycles = self.cpu.step();
            max_cycles -= cycles as i64;

            // Check if OS init is complete
            let mtct = self.cpu.memory().ram[0x22B]; // _MTCT register
            if mtct == 0xFE {
                log::info!("OS initialization complete");
                return;
            }
        }

        log::warn!("OS initialization timed out");
    }

    /// Run one frame (~16.67ms at 60fps)
    pub fn run_frame(&mut self) {
        // Apply cheats at the start of each frame
        if self.cheat.has_active_cheats() {
            let (ram, flash) = self.cpu.memory_mut().get_ram_and_flash();
            self.cheat.apply_cheats(ram, flash);
        }

        // BBK runs at 4MHz, 60fps = 66666.67 cycles per frame
        // Apply CPU rate multiplier
        let cycles_per_frame = (4_000_000.0 / 60.0 * self.cpu_rate) as u32;
        let mut cycles_run = 0u32;

        // Calculate timer step based on timer rate
        // Timer ticks every 400 CPU cycles at normal speed
        let timer_step = (400.0 / self.timer_rate) as u32;

        while cycles_run < cycles_per_frame && self.running {
            let halted = self.cpu.memory().ram[0x200] & 0x08 != 0;
            let cycles = if halted { 400 } else { self.step() };
            cycles_run += cycles;
            self.timer_cycle_remainder += cycles;
            let ticks = self.timer_cycle_remainder / timer_step;
            if ticks > 0 {
                self.cpu.memory_mut().update_timers(ticks);
                self.timer_cycle_remainder %= timer_step;
            }
        }

        // Update RTC once per second (every 60 frames)
        if self.frame_count.is_multiple_of(60) {
            self.cpu.memory_mut().update_rtc();
        }

        self.frame_count += 1;
    }

    /// Execute a single CPU step
    fn step(&mut self) -> u32 {
        let pc = self.cpu.pc();

        if pc == HLE_FAR_RETURN {
            if let Some(call) = self.hle_far_calls.pop() {
                for (index, bank) in call.banks.into_iter().enumerate() {
                    self.cpu.memory_mut().bank_switch.set(5 + index as u8, bank);
                }
                self.cpu.set_pc(call.return_pc);
                return 1;
            }
        }

        // Check for breakpoint
        if self.debug.has_breakpoint(pc) {
            log::info!("Breakpoint hit at 0x{:04X}", pc);
            self.debug.set_stepping(true);
        }

        // Check for BRK instruction (game exit)
        let opcode = self.cpu.memory().read(pc);
        if opcode == 0x00 {
            // Log the return address on stack for debugging
            let sp = self.cpu.sp();
            let ret_lo = self.cpu.memory().read(0x100 | (sp.wrapping_add(1) as u16));
            let ret_hi = self.cpu.memory().read(0x100 | (sp.wrapping_add(2) as u16));
            let ret_addr = ret_lo as u16 | (ret_hi as u16) << 8;
            log::info!(
                "BRK at 0x{:04X} SP=0x{:02X} ret=0x{:04X}, game exiting",
                pc,
                sp,
                ret_addr
            );
            self.running = false;
            return 1;
        }

        // Check for JSR instruction - potential syscall
        if opcode == 0x20 {
            let target = self.cpu.memory().read16(pc + 1);
            if target >= 0xD000 {
                log::info!(
                    "OS call caller=0x{pc:04X} target=0x{target:04X} A={:02X} X={:02X} Y={:02X}",
                    self.cpu.a(),
                    self.cpu.x(),
                    self.cpu.y()
                );
            }

            // Other JSR calls execute normally
        }

        // Execute the instruction normally
        let cycles = self.cpu.step();

        // Handle interrupts
        self.handle_interrupts();

        cycles
    }

    fn handle_interrupts(&mut self) {
        let isr = self.cpu.memory().ram[0x04]; // ISR register
        let ier = self.cpu.memory().ram[0x23A]; // IER register
        let tisr = self.cpu.memory().ram[0x05]; // TISR register
        let tier = self.cpu.memory().ram[0x23B]; // TIER register
        let status = self.cpu.status();

        // Check if interrupts are disabled
        if status & 0x04 != 0 {
            return;
        }

        // 1. Keyboard interrupt (PI) — highest priority
        if (isr & 0x80) != 0 && (ier & 0x80) != 0 {
            self.cpu.memory_mut().ram[0x04] &= 0x7F; // Clear PI flag
            return;
        }

        // 2. Alarm interrupt (ALM)
        if (isr & 0x01) != 0 && (ier & 0x01) != 0 {
            self.trigger_interrupt(0x13); // ALM
            return;
        }

        // 3. Counter interrupt (CT)
        if (isr & 0x02) != 0 && (ier & 0x02) != 0 {
            self.trigger_interrupt(0x12); // CT
            return;
        }

        // 4. Main timer (MT)
        if (tisr & 0x20) != 0 && (tier & 0x20) != 0 {
            self.trigger_interrupt(0x11); // MT
            return;
        }

        // 5. General timer high (GTH)
        if (tisr & 0x80) != 0 && (tier & 0x80) != 0 {
            self.trigger_interrupt(0x10); // GTH
            return;
        }

        // 6. General timer low (GTL)
        if (tisr & 0x40) != 0 && (tier & 0x40) != 0 {
            self.trigger_interrupt(0x0F); // GTL
            return;
        }

        // 7. Software timer 1 (ST1) — inline counter handling
        if (tisr & 0x01) != 0 && (tier & 0x01) != 0 {
            let ram = &mut self.cpu.memory_mut().ram;
            ram[0x05] &= 0xFE;
            ram[0x2018] = ram[0x2018].wrapping_add(1);
            if ram[0x2018] >= ram[0x2019] {
                ram[0x201E] |= 0x01;
                ram[0x2018] = 0;
            }
            return;
        }

        // 8-10. Software timers 2-4 (ST2-ST4)
        if (tisr & 0x02) != 0 && (tier & 0x02) != 0 {
            self.trigger_interrupt(0x04); // ST2
            return;
        }
        if (tisr & 0x04) != 0 && (tier & 0x04) != 0 {
            self.trigger_interrupt(0x05); // ST3
            return;
        }
        if (tisr & 0x08) != 0 && (tier & 0x08) != 0 {
            self.trigger_interrupt(0x06); // ST4
        }
    }

    /// Trigger an interrupt
    fn trigger_interrupt(&mut self, vector_idx: u8) {
        // Push PC and status to stack
        let pc = self.cpu.pc();
        let status = self.cpu.status();
        let sp = self.cpu.sp();

        // Write to stack
        self.cpu.memory_mut().ram[0x100 | sp as usize] = (pc >> 8) as u8;
        self.cpu.memory_mut().ram[0x100 | sp.wrapping_sub(1) as usize] = (pc & 0xFF) as u8;
        self.cpu.memory_mut().ram[0x100 | sp.wrapping_sub(2) as usize] = status;
        self.cpu.set_sp(sp.wrapping_sub(3));

        // Each vector slot contains an executable ROM jump stub.
        self.cpu.set_pc(0x0300 + (vector_idx as u16) * 4);
    }

    /// Handle key down event
    pub fn key_down(&mut self, key: BbkKey) {
        let code = key as u8;
        let ram = &mut self.cpu.memory_mut().ram;
        ram[0x200] &= 0xF7;
        ram[0x24E] = code | 0x80;
        ram[0x04] |= 0x80;
        if ram[0x23A] & 0x80 != 0 {
            ram[0x2003] = 0;
            ram[0x2004] = 0x0F;
            ram[0x2017] = code & 0x3F;
            ram[0x24E] = 0;
        }
        self.input.key_down(key, self.frame_count * 16);
    }

    /// Handle key up event
    pub fn key_up(&mut self) {
        self.cpu.memory_mut().ram[0x24E] = 0;
        self.input.key_up();
    }

    /// Get the LCD framebuffer
    pub fn get_framebuffer(&self) -> &[bool; 159 * 96] {
        // TODO: Return actual framebuffer from RAM
        static FB: [bool; 159 * 96] = [false; 159 * 96];
        &FB
    }

    /// Set LCD display orientation
    pub fn set_lcd_orientation(&mut self, orientation: LcdOrientation) {
        self.lcd_orientation = orientation;
    }

    /// Get LCD display orientation
    pub fn lcd_orientation(&self) -> LcdOrientation {
        self.lcd_orientation
    }

    /// Set LCD color theme
    pub fn set_lcd_theme(&mut self, theme: LcdTheme) {
        self.lcd_theme = theme;
    }

    /// Get LCD color theme
    pub fn lcd_theme(&self) -> &LcdTheme {
        &self.lcd_theme
    }

    /// Get the display width considering orientation
    pub fn display_width(&self) -> usize {
        self.lcd_orientation.width()
    }

    /// Get the display height considering orientation
    pub fn display_height(&self) -> usize {
        self.lcd_orientation.height()
    }

    /// Set CPU clock rate multiplier
    pub fn set_cpu_rate(&mut self, rate: f32) {
        self.cpu_rate = rate.clamp(0.25, 8.0);
    }

    /// Get CPU clock rate multiplier
    pub fn cpu_rate(&self) -> f32 {
        self.cpu_rate
    }

    /// Set timer clock rate multiplier
    pub fn set_timer_rate(&mut self, rate: f32) {
        self.timer_rate = rate.clamp(0.25, 8.0);
    }

    /// Get timer clock rate multiplier
    pub fn timer_rate(&self) -> f32 {
        self.timer_rate
    }

    /// Set minimum key repeat interval in milliseconds
    pub fn set_key_repeat_interval(&mut self, ms: u64) {
        self.input.set_min_repeat_interval(ms);
    }

    /// Get minimum key repeat interval in milliseconds
    pub fn key_repeat_interval(&self) -> u64 {
        self.input.min_repeat_interval()
    }

    /// Render LCD to boolean buffer (true = foreground, false = background)
    pub fn render_lcd_buffer(&mut self) -> [bool; 159 * 96] {
        let mut pixels = [false; 159 * 96];

        self.cpu.memory_mut().ram[0x400] = self.cpu.memory().ram[0x1000];
        let ram = &self.cpu.memory().ram;
        let mut v = 0x400;

        for j in (-30i32..=65).rev() {
            let y = if j >= 0 { j } else { -j + 65 } as usize;
            for i in 1..20 {
                decode_lcd_byte(&mut pixels, y, i * 8, ram[v]);
                v += 1;
            }
            v += 13;
        }

        v = 0x413;
        for j in (-30i32..=64).rev() {
            let y = if j >= 0 { j } else { -j + 65 } as usize;
            decode_lcd_byte(&mut pixels, y, 0, ram[v]);
            v += 32;
        }
        decode_lcd_byte(&mut pixels, 65, 0, ram[0x0ff3]);

        pixels
    }

    /// Render LCD to RGB565 buffer
    pub fn render_lcd(&mut self, buf: &mut [u16], ghosting: bool) {
        let theme = self.lcd_theme;
        let pixels = self.render_lcd_buffer();

        if ghosting {
            self.lcd.render_with_ghosting(buf, &pixels, &theme);
        } else {
            for i in 0..159 * 96 {
                buf[i] = if pixels[i] { theme.fg } else { theme.bg };
            }
        }
    }

    /// Render LCD to RGB565 buffer with orientation support
    pub fn render_lcd_with_orientation(&mut self, buf: &mut [u16], ghosting: bool) {
        let theme = self.lcd_theme;
        let pixels = self.render_lcd_buffer();

        match self.lcd_orientation {
            LcdOrientation::Portrait => {
                if ghosting {
                    self.lcd.render_with_ghosting(buf, &pixels, &theme);
                } else {
                    for i in 0..159 * 96 {
                        buf[i] = if pixels[i] { theme.fg } else { theme.bg };
                    }
                }
            }
            LcdOrientation::Landscape => {
                let mut portrait = [0u16; 159 * 96];
                if ghosting {
                    self.lcd
                        .render_with_ghosting(&mut portrait, &pixels, &theme);
                } else {
                    for i in 0..159 * 96 {
                        portrait[i] = if pixels[i] { theme.fg } else { theme.bg };
                    }
                }
                // Rotate 90 degrees clockwise: portrait[y*159+x] -> buf[x*96+(95-y)]
                for y in 0..96 {
                    for x in 0..159 {
                        buf[x * 96 + (95 - y)] = portrait[y * 159 + x];
                    }
                }
            }
        }
    }

    /// Get current model
    pub fn model(&self) -> &'static BbkModel {
        self.model
    }

    /// Check if emulator is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Stop the emulator
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Get frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Create a save state
    pub fn save_state(&self) -> SaveState {
        SaveState {
            ram: self.cpu.memory().ram.to_vec(),
            flash: self.cpu.memory().flash[..0x14000].to_vec(),
            cpu: crate::save::CpuState {
                pc: self.cpu.pc(),
                sp: self.cpu.sp(),
                a: self.cpu.a(),
                x: self.cpu.x(),
                y: self.cpu.y(),
                status: self.cpu.status(),
                cycles: self.cpu.cycles(),
            },
            bank_switch: crate::save::BankState {
                banks: self.cpu.memory().bank_switch.banks.to_vec(),
                selected: self.cpu.memory().bank_switch.selected(),
            },
            bank_sys_d: self.model.bank_sys_d,
            flash_cmd: self.cpu.memory().flash_cmd(),
            flash_cycles: self.cpu.memory().flash_cycles(),
        }
    }

    /// Load a save state
    pub fn load_save_state(&mut self, state: &SaveState) -> Result<()> {
        self.cpu.memory_mut().ram.copy_from_slice(&state.ram);
        let len = state.flash.len().min(self.cpu.memory().flash.len());
        self.cpu.memory_mut().flash[..len].copy_from_slice(&state.flash[..len]);
        self.cpu.set_pc(state.cpu.pc);
        self.cpu.set_sp(state.cpu.sp);
        self.cpu
            .memory_mut()
            .set_flash_state(state.flash_cmd, state.flash_cycles);
        Ok(())
    }

    /// Get debugger
    pub fn debugger(&mut self) -> &mut Debugger {
        &mut self.debug
    }
}

fn decode_lcd_byte(pixels: &mut [bool; 159 * 96], y: usize, x: usize, byte: u8) {
    for bit in 0..8 {
        if x + bit < 159 {
            pixels[y * 159 + x + bit] = byte & (1 << (7 - bit)) != 0;
        }
    }
}

#[cfg(test)]
mod lcd_render_tests {
    use super::*;

    #[test]
    fn decodes_most_significant_bit_on_left() {
        let mut pixels = [false; 159 * 96];
        decode_lcd_byte(&mut pixels, 2, 8, 0b1000_0001);
        assert!(pixels[2 * 159 + 8]);
        assert!(pixels[2 * 159 + 15]);
        assert!(!pixels[2 * 159 + 9]);
    }

    #[test]
    fn clips_the_unused_160th_column() {
        let mut pixels = [false; 159 * 96];
        decode_lcd_byte(&mut pixels, 0, 152, 0xff);
        assert_eq!(pixels[..159].iter().filter(|pixel| **pixel).count(), 7);
    }
}
