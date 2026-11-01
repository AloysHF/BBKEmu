//! BBKEmu standalone frontend

use std::fs;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;

use bbkemu_core::input::BbkKey;
use bbkemu_core::lcd::LcdOrientation;
use bbkemu_core::save::SaveState;
use bbkemu_core::{model, BbkModel, Emulator};
use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowId};

#[derive(Parser)]
#[command(name = "bbkemu")]
#[command(about = "BBK electronic dictionary game emulator")]
struct Cli {
    /// GAM file to load
    game: PathBuf,

    /// System directory that holds bbkemu/<model>/
    #[arg(long, value_name = "DIR", default_value = "system")]
    system_dir: PathBuf,

    /// Display scale factor
    #[arg(short, long, default_value = "4", value_parser = clap::value_parser!(u32).range(1..=16))]
    scale: u32,

    /// Start in fullscreen mode
    #[arg(short, long)]
    fullscreen: bool,

    /// BBK model: a4980 or a4988
    #[arg(short, long, default_value = "a4980")]
    model: String,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Output PNG file path
    #[arg(short, long, default_value = "output.png")]
    output: PathBuf,

    /// Run headless for this many frames and write --output
    #[arg(long)]
    frames: Option<u64>,

    /// Swap LCD width and height for landscape display
    #[arg(long)]
    swap_lcd: bool,

    /// CPU clock rate multiplier (0.25 - 8.0)
    #[arg(long, default_value = "1.0")]
    cpu_rate: f32,

    /// Timer clock rate multiplier (0.25 - 8.0)
    #[arg(long, default_value = "1.0")]
    timer_rate: f32,

    /// Minimum key repeat interval in milliseconds (0 = no limit)
    #[arg(long, default_value = "0")]
    key_repeat_interval: u64,

    /// Cheat codes (format: AAAAAAVV for address=value, can specify multiple)
    #[arg(long)]
    cheat: Vec<String>,

    /// Take a screenshot after N frames and exit (saves as PNG)
    #[arg(short = 'S', long = "screenshot", value_name = "PATH")]
    screenshot: Option<PathBuf>,

    /// Number of frames to run before taking screenshot (default: 30)
    #[arg(long = "screenshot-frames", default_value = "30")]
    screenshot_frames: u32,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.debug {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("bbkemu", log_level)
        .filter_module("bbkemu_core", log_level)
        .init();

    log::info!("BBKEmu starting...");

    // Select model
    let bbk_model: &'static BbkModel = model::from_id(&cli.model)
        .ok_or_else(|| anyhow::anyhow!("unknown model '{}': expected a4980 or a4988", cli.model))?;

    log::info!("Creating emulator...");
    let mut emu = Emulator::new(bbk_model);
    if cli.swap_lcd {
        emu.set_lcd_orientation(LcdOrientation::Landscape);
        log::info!("LCD orientation: Landscape (swapped)");
    }
    emu.set_cpu_rate(cli.cpu_rate);
    emu.set_timer_rate(cli.timer_rate);
    emu.set_key_repeat_interval(cli.key_repeat_interval);
    log::info!(
        "CPU rate: {}x, Timer rate: {}x, Key repeat interval: {}ms",
        emu.cpu_rate(),
        emu.timer_rate(),
        emu.key_repeat_interval()
    );

    // Add cheat codes
    for cheat_code in &cli.cheat {
        if let Some(id) = emu.cheat.add_cheat(cheat_code) {
            log::info!("Added cheat {}: {}", id, cheat_code);
        } else {
            log::warn!("Invalid cheat code: {}", cheat_code);
        }
    }

    log::info!("Emulator created");

    // Load the firmware from the fixed per-model directory.
    let firmware = bbkemu_core::firmware::load(&cli.system_dir, bbk_model)
        .map_err(|error| anyhow::anyhow!("{}: {error}", bbk_model.name))?;
    emu.load_rom_8(&firmware.font);
    emu.load_rom_e(&firmware.os);

    // Load game
    let game_data = fs::read(&cli.game)?;
    emu.load_gam(&game_data)?;

    log::info!("Starting BBKEmu with model {}", bbk_model.name);
    log::info!("Game: {}", cli.game.display());
    log::info!("Scale: {}x", cli.scale);

    if let Some(frames) = cli.frames {
        run_headless(&mut emu, frames, &cli.output, cli.scale)?;
        return Ok(());
    }

    // Screenshot mode: run N frames then save screenshot and exit
    if let Some(ref screenshot_path) = cli.screenshot {
        log::info!(
            "Running {} frames before taking screenshot...",
            cli.screenshot_frames
        );
        for _ in 0..cli.screenshot_frames {
            emu.run_frame();
        }
        let lcd_buffer = emu.render_lcd_buffer();
        save_png(
            screenshot_path,
            &lcd_buffer,
            cli.scale,
            emu.lcd_orientation(),
        )?;
        log::info!("Screenshot saved to {}", screenshot_path.display());
        return Ok(());
    }

    // Save file path: same as game file with .sav extension
    let save_path = cli.game.with_extension("sav");
    log::info!("Save file: {}", save_path.display());

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new(emu, cli.scale, cli.fullscreen, save_path);
    event_loop.run_app(&mut app)?;
    Ok(())
}

fn run_headless(emu: &mut Emulator, frames: u64, output: &PathBuf, scale: u32) -> Result<()> {
    for _ in 0..frames {
        emu.run_frame();
    }
    let lcd_buffer = emu.render_lcd_buffer();
    save_png(output, &lcd_buffer, scale, emu.lcd_orientation())?;
    log::info!(
        "Emulation complete after {} frames at PC=0x{:04X} ({} cycles)",
        emu.frame_count(),
        emu.cpu.pc(),
        emu.cpu.cycles()
    );
    log::info!("Output saved to {}", output.display());
    Ok(())
}

struct App {
    emu: Emulator,
    scale: u32,
    fullscreen: bool,
    window: Option<Rc<Window>>,
    context: Option<Context<Rc<Window>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    next_frame: Instant,
    save_path: PathBuf,
}

impl App {
    fn new(emu: Emulator, scale: u32, fullscreen: bool, save_path: PathBuf) -> Self {
        Self {
            emu,
            scale,
            fullscreen,
            window: None,
            context: None,
            surface: None,
            next_frame: Instant::now(),
            save_path,
        }
    }

    fn draw(&mut self) -> std::result::Result<(), String> {
        let window = self.window.as_ref().expect("window must exist");
        let surface = self.surface.as_mut().expect("surface must exist");
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        surface
            .resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            )
            .map_err(|error| error.to_string())?;

        let pixels = self.emu.render_lcd_buffer();
        let display_width = self.emu.display_width();
        let display_height = self.emu.display_height();
        let orientation = self.emu.lcd_orientation();
        let mut buffer = surface.buffer_mut().map_err(|error| error.to_string())?;

        for y in 0..height as usize {
            for x in 0..width as usize {
                let (source_x, source_y) = match orientation {
                    LcdOrientation::Portrait => {
                        (x * 159 / width as usize, y * 96 / height as usize)
                    }
                    LcdOrientation::Landscape => {
                        // Map rotated coordinates back to original
                        let orig_x = display_width - 1 - x * display_width / width as usize;
                        let orig_y = y * display_height / height as usize;
                        (orig_y.min(158), orig_x.min(95))
                    }
                };
                buffer[y * width as usize + x] = if pixels[source_y * 159 + source_x] {
                    0x0014_1814
                } else {
                    0x00a8_b8a0
                };
            }
        }
        buffer.present().map_err(|error| error.to_string())?;
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let display_width = self.emu.display_width() as u32;
        let display_height = self.emu.display_height() as u32;
        let mut attributes = Window::default_attributes()
            .with_title("BBKEmu")
            .with_inner_size(LogicalSize::new(
                (display_width * self.scale) as f64,
                (display_height * self.scale) as f64,
            ))
            .with_min_inner_size(LogicalSize::new(
                display_width as f64,
                display_height as f64,
            ));
        if self.fullscreen {
            attributes = attributes.with_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        let window = match event_loop.create_window(attributes) {
            Ok(window) => Rc::new(window),
            Err(error) => {
                log::error!("Failed to create window: {error}");
                event_loop.exit();
                return;
            }
        };
        let context = match Context::new(window.clone()) {
            Ok(context) => context,
            Err(error) => {
                log::error!("Failed to create display context: {error}");
                event_loop.exit();
                return;
            }
        };
        let surface = match Surface::new(&context, window.clone()) {
            Ok(surface) => surface,
            Err(error) => {
                log::error!("Failed to create display surface: {error}");
                event_loop.exit();
                return;
            }
        };
        self.window = Some(window);
        self.context = Some(context);
        self.surface = Some(surface);
        self.next_frame = Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.draw() {
                    log::error!("Failed to render frame: {error}");
                    event_loop.exit();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let PhysicalKey::Code(code) = event.physical_key else {
                    return;
                };
                if code == KeyCode::F5 && event.state == ElementState::Pressed {
                    // Save state
                    let state = self.emu.save_state();
                    match std::fs::write(&self.save_path, state.to_bytes()) {
                        Ok(()) => log::info!("State saved to {}", self.save_path.display()),
                        Err(error) => log::error!("Failed to save state: {error}"),
                    }
                    return;
                }
                if code == KeyCode::F8 && event.state == ElementState::Pressed {
                    // Load state
                    if self.save_path.exists() {
                        match std::fs::read(&self.save_path) {
                            Ok(bytes) => match SaveState::from_bytes(&bytes) {
                                Ok(state) => match self.emu.load_save_state(&state) {
                                    Ok(()) => {
                                        log::info!("State loaded from {}", self.save_path.display())
                                    }
                                    Err(error) => log::error!("Failed to load state: {error}"),
                                },
                                Err(error) => log::error!("Failed to parse state: {error}"),
                            },
                            Err(error) => log::error!("Failed to read save file: {error}"),
                        }
                    } else {
                        log::warn!("No save file found at {}", self.save_path.display());
                    }
                    return;
                }
                if code == KeyCode::F12 && event.state == ElementState::Pressed {
                    let pixels = self.emu.render_lcd_buffer();
                    let path = PathBuf::from("bbkemu-screenshot.png");
                    match save_png(&path, &pixels, 1, self.emu.lcd_orientation()) {
                        Ok(()) => log::info!("Screenshot saved to {}", path.display()),
                        Err(error) => log::error!("Failed to save screenshot: {error}"),
                    }
                    return;
                }
                if code == KeyCode::Escape && event.state == ElementState::Pressed {
                    event_loop.exit();
                    return;
                }
                if let Some(key) = map_key(code) {
                    match event.state {
                        ElementState::Pressed => self.emu.key_down(key),
                        ElementState::Released => self.emu.key_up(),
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if now >= self.next_frame {
            self.emu.run_frame();
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            self.next_frame = now + Duration::from_micros(16_667);
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame));
    }
}

fn map_key(code: KeyCode) -> Option<BbkKey> {
    Some(match code {
        KeyCode::ArrowUp => BbkKey::Up,
        KeyCode::ArrowDown => BbkKey::Down,
        KeyCode::ArrowLeft => BbkKey::Left,
        KeyCode::ArrowRight => BbkKey::Right,
        KeyCode::Enter => BbkKey::Enter,
        KeyCode::Backspace => BbkKey::Exit,
        KeyCode::Delete => BbkKey::Del,
        KeyCode::Space => BbkKey::Space,
        KeyCode::PageUp => BbkKey::PgUp,
        KeyCode::PageDown => BbkKey::PgDn,
        KeyCode::Digit0 => BbkKey::Key0,
        KeyCode::Digit1 => BbkKey::Key1,
        KeyCode::Digit2 => BbkKey::Key2,
        KeyCode::Digit3 => BbkKey::Key3,
        KeyCode::Digit4 => BbkKey::Key4,
        KeyCode::Digit5 => BbkKey::Key5,
        KeyCode::Digit6 => BbkKey::Key6,
        KeyCode::Digit7 => BbkKey::Key7,
        KeyCode::Digit8 => BbkKey::Key8,
        KeyCode::Digit9 => BbkKey::Key9,
        KeyCode::KeyQ => BbkKey::Q,
        KeyCode::KeyW => BbkKey::W,
        KeyCode::KeyE => BbkKey::E,
        KeyCode::KeyR => BbkKey::R,
        KeyCode::KeyT => BbkKey::T,
        KeyCode::KeyY => BbkKey::Y,
        KeyCode::KeyU => BbkKey::U,
        KeyCode::KeyI => BbkKey::I,
        KeyCode::KeyO => BbkKey::O,
        KeyCode::KeyP => BbkKey::P,
        KeyCode::KeyA => BbkKey::A,
        KeyCode::KeyS => BbkKey::S,
        KeyCode::KeyD => BbkKey::D,
        KeyCode::KeyF => BbkKey::F,
        KeyCode::KeyG => BbkKey::G,
        KeyCode::KeyH => BbkKey::H,
        KeyCode::KeyJ => BbkKey::J,
        KeyCode::KeyK => BbkKey::K,
        KeyCode::KeyL => BbkKey::L,
        KeyCode::KeyZ => BbkKey::Z,
        KeyCode::KeyX => BbkKey::X,
        KeyCode::KeyC => BbkKey::C,
        KeyCode::KeyV => BbkKey::V,
        KeyCode::KeyB => BbkKey::B,
        KeyCode::KeyN => BbkKey::N,
        KeyCode::KeyM => BbkKey::M,
        _ => return None,
    })
}

fn save_png(
    path: &PathBuf,
    pixels: &[bool; 159 * 96],
    scale: u32,
    orientation: LcdOrientation,
) -> Result<()> {
    let (img_width, img_height) = match orientation {
        LcdOrientation::Portrait => (159 * scale, 96 * scale),
        LcdOrientation::Landscape => (96 * scale, 159 * scale),
    };

    let mut img = image::RgbImage::new(img_width, img_height);

    // Colors (RGB format)
    let bg_color = image::Rgb([0xDA, 0xD6, 0x00]); // Grey background
    let fg_color = image::Rgb([0x00, 0x00, 0x00]); // Black foreground

    match orientation {
        LcdOrientation::Portrait => {
            for y in 0..96u32 {
                for dy in 0..scale {
                    for x in 0..159u32 {
                        let pixel = pixels[(y * 159 + x) as usize];
                        let color = if pixel { fg_color } else { bg_color };
                        for dx in 0..scale {
                            img.put_pixel(x * scale + dx, y * scale + dy, color);
                        }
                    }
                }
            }
        }
        LcdOrientation::Landscape => {
            // Rotated 90 degrees clockwise
            for x in 0..159u32 {
                for dx in 0..scale {
                    for y in (0..96u32).rev() {
                        let pixel = pixels[(y * 159 + x) as usize];
                        let color = if pixel { fg_color } else { bg_color };
                        for dy in 0..scale {
                            img.put_pixel((95 - y) * scale + dy, x * scale + dx, color);
                        }
                    }
                }
            }
        }
    }

    img.save(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_navigation_and_action_keys() {
        assert_eq!(map_key(KeyCode::ArrowUp), Some(BbkKey::Up));
        assert_eq!(map_key(KeyCode::Enter), Some(BbkKey::Enter));
        assert_eq!(map_key(KeyCode::Backspace), Some(BbkKey::Exit));
    }

    #[test]
    fn leaves_unmapped_host_keys_available_to_frontend() {
        assert_eq!(map_key(KeyCode::F1), None);
        assert_eq!(map_key(KeyCode::Escape), None);
    }
}
