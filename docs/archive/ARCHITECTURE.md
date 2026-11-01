# Architecture

This document describes the architecture and design of BBKEmu.

## Overview

BBKEmu is a BBK A-series electronic dictionary game emulator written in Rust.
It executes actual OS ROM code (LLE mode) to provide the runtime environment
for BBK games.

```
┌─────────────────────────────────────────────────────────────┐
│                      BBKEmu Architecture                    │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   bbkemu    │    │bbkemu-      │    │bbkemu-rom-  │     │
│  │  (frontend) │    │libretro     │    │analyzer     │     │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘     │
│         │                  │                  │             │
│         └──────────────────┼──────────────────┘             │
│                            │                                │
│                    ┌───────▼───────┐                        │
│                    │  bbkemu-core  │                        │
│                    └───────┬───────┘                        │
│         ┌──────────────────┼──────────────────┐            │
│         │                  │                  │            │
│   ┌─────▼─────┐    ┌──────▼──────┐    ┌──────▼──────┐     │
│   │    CPU    │    │   Memory    │    │  Peripherals│     │
│   │  (mos6502)│    │ (bank sw.)  │    │  LCD/Key/.. │     │
│   └───────────┘    └─────────────┘    └─────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

## Crate Structure

### bbkemu-core (Library)

Platform-independent emulator engine. Contains all emulation logic and can be
used by any frontend.

**Modules:**

| Module | File | Description |
|--------|------|-------------|
| `emulator` | emulator.rs | Main emulator orchestrator |
| `cpu` | cpu.rs | 6502 CPU wrapper (mos6502 crate) |
| `memory` | memory.rs | Memory bus with bank switching |
| `lcd` | lcd.rs | LCD framebuffer (159×96) |
| `input` | input.rs | Keyboard input handling |
| `audio` | audio.rs | Audio tone generation |
| `font` | font.rs | Font rendering |
| `font_data` | font_data.rs | Built-in font bitmap data |
| `gam` | gam.rs | GAM file loader and parser |
| `model` | model.rs | BBK model definitions |
| `debug` | debug.rs | Debugger with breakpoints |
| `save` | save.rs | Save state serialization |

### bbkemu (Binary)

Standalone frontend with window rendering and audio output.

**Files:**

| File | Description |
|------|-------------|
| main.rs | Window loop, CLI parsing, input handling |

### bbkemu-libretro (CDylib)

libretro core for RetroArch integration.

**Files:**

| File | Description |
|------|-------------|
| lib.rs | libretro API implementation |

### bbkemu-rom-analyzer (Binary)

Tool for analyzing BBK OS ROM files for emulation development and debugging.

**Files:**

| File | Description |
|------|-------------|
| main.rs | OS ROM analyzer |

## Key Types

### Emulator

Main emulator struct that owns all components and orchestrates execution.

```rust
pub struct Emulator {
    pub cpu: CpuWrapper,
    pub lcd: Lcd,
    pub input: Input,
    pub audio: Audio,
    pub debug: Debugger,
    model: &'static BbkModel,
    running: bool,
    frame_count: u64,
    timer_cycle_remainder: u32,
    far_calls: Vec<FarCall>,  // Far call return points
}
```

### CpuWrapper

Wraps the mos6502 CPU with BBK-specific functionality.

```rust
pub struct CpuWrapper {
    pub inner: CPU<Memory, Nmos6502>,
}
```

### Memory

Memory bus with bank switching, RAM, flash, and ROM support.

```rust
pub struct Memory {
    pub ram: Vec<u8>,
    pub flash: Vec<u8>,
    pub rom_8: Option<Vec<u8>>,
    pub rom_e: Option<Vec<u8>>,
    pub bank_switch: BankSwitch,
    // ...
}
```

### Lcd

159×96 monochrome framebuffer with ghosting effects.

```rust
pub struct Lcd {
    pixels: [bool; FRAMEBUFFER_SIZE],
    ghosting: [i8; FRAMEBUFFER_SIZE],
    cursor_x: u8,
    cursor_y: u8,
    dirty: bool,
}
```

### Input

Keyboard input handling with key buffer and repeat logic.

```rust
pub struct Input {
    key_buffer: VecDeque<u8>,
    key_code: u8,
    interrupt_pending: bool,
    // ...
}
```

### BbkModel

Configuration for different BBK dictionary models.

```rust
pub struct BbkModel {
    pub name: &'static str,
    pub lcd_width: u8,
    pub lcd_height: u8,
    // ...
}
```

## Execution Flow

### Initialization

1. Create `Emulator` instance with model configuration
2. Load font ROM (8.BIN) if available
3. Load OS ROM (E.BIN)
4. Load GAM file into flash memory
5. Initialize bank switch mappings
6. Execute OS initialization code

### Main Loop

```
┌─────────────────────────────────────────┐
│            Main Loop (60 fps)           │
├─────────────────────────────────────────┤
│  1. Process input events                │
│  2. Run CPU for ~66,666 cycles          │
│     (one frame at 4 MHz)                │
│  3. Update timers                       │
│  4. Handle interrupts                   │
│  5. Render LCD to framebuffer           │
│  6. Output audio samples                │
│  7. Display framebuffer                 │
└─────────────────────────────────────────┘
```

### CPU Execution

Each CPU step:

1. Fetch instruction at PC
2. Decode instruction
3. Execute instruction
4. Update cycle counter
5. Check for interrupts

### Bank Switching

The memory bus uses bank switching to access different memory regions:

- 16 × 4 KiB pages
- Each page can map to any 4 KiB aligned address
- Banks 0-4: Fixed mappings (RAM, Flash)
- Banks 5-8: Configurable (used for far calls)

## Interrupt System

The BBK hardware has multiple interrupt sources:

| Priority | Source | Vector | Description |
|----------|--------|--------|-------------|
| Highest | BRK | 0xFFFE | Break instruction |
| High | PI | 0x06 | Keyboard interrupt |
| Medium | ALM | 0x13 | Alarm interrupt |
| Low | CT | 0x12 | Counter interrupt |
| Lowest | ST2-ST4 | 0x04-0x06 | Timer interrupts |

Interrupt handling:

1. Check if interrupts are enabled (status register I flag)
2. Check interrupt status registers (ISR, TISR)
3. Check interrupt enable registers (IER, TIER)
4. If interrupt pending and enabled, jump to vector

## Audio System

The audio system generates square wave tones:

- Frequency: Configurable (Hz)
- Duration: Configurable (ms)
- Sample rate: 44,100 Hz
- Output: Mono

## Dependencies

| Crate | Purpose |
|-------|---------|
| `mos6502` | 6502 CPU emulation |
| `serde` / `bincode` | Save state serialization |
| `log` / `env_logger` | Logging |
| `anyhow` / `thiserror` | Error handling |
| `winit` / `softbuffer` | Window rendering (standalone) |
| `cpal` | Audio output (standalone) |
| `clap` | Command-line argument parsing |

## Build Commands

```bash
# Check compilation
cargo check --workspace

# Build release
cargo build --release

# Run standalone
cargo run --release -p bbkemu -- game.gam --system-dir ./system --model a4980

# Build libretro core only
cargo build --release -p bbkemu-libretro

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all
```
