# Contributing

Thank you for your interest in contributing to BBKEmu! This document provides guidelines and information for contributors.

## How to Contribute

1. **Fork** this repository
2. **Create** a feature branch (`git checkout -b feature/your-feature`)
3. **Commit** your changes (`git commit -m 'Add your feature'`)
4. **Push** to the branch (`git push origin feature/your-feature`)
5. **Open** a Pull Request

## Development Setup

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable, 1.70+)
- Git

### Building from Source

```bash
# Clone the repository
git clone https://github.com/AloysHF/BBKEmu.git
cd BBKEmu

# Build the project
cargo build --release

# Run tests
cargo test --workspace

# Run with debug logging
RUST_LOG=bbkemu=debug cargo run --release -p bbkemu -- game.gam --debug
```

### Project Structure

```
BBKEmu/
├── crates/
│   ├── bbkemu/           # Standalone frontend (window, input, audio)
│   ├── bbkemu-core/      # Platform-independent emulation core
│   ├── bbkemu-libretro/  # Libretro core wrapper
│   └── bbkemu-rom-analyzer/  # ROM analysis tool
├── docs/                 # Documentation
├── system/               # System ROM files (not in repo)
└── tmp/                  # Test ROMs and games (not in repo)
```

### Key Components

- **bbkemu-core**: The heart of the emulator. Contains CPU, memory, LCD, keyboard, audio, and timer emulation. Platform-independent and can be driven headlessly.
- **bbkemu**: Standalone frontend using winit + softbuffer for windowed display.
- **bbkemu-libretro**: Libretro core wrapper for RetroArch integration.
- **bbkemu-rom-analyzer**: Development tool for analyzing BBK OS ROM files. Useful for emulation development and debugging.

```bash
# Build the ROM analyzer
cargo build -p bbkemu-rom-analyzer --release

# Run it on a ROM file
cargo run -p bbkemu-rom-analyzer --release -- path/to/E.BIN
```

## Areas That Need Help

- **Game compatibility testing** — test more games and report issues with screenshots
- **6502 CPU accuracy** — improve cycle-accurate emulation
- **Platform ports** — macOS and Linux testing and packaging
- **Documentation** — improve docs and code comments
- **Bug reports** — if you find a game that doesn't work correctly, please open an issue
- **RetroArch integration** — compatibility testing across frontends

## Code Style

- Use English for all comments and documentation
- Use `snake_case` for functions and variables
- Use `PascalCase` for types and structs
- Prefer `anyhow::Result` for error handling
- Use `log` crate for logging (not `println!`)

## Testing

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run core tests only
cargo test -p bbkemu-core

# Run with output
cargo test --workspace -- --nocapture
```

### Writing Tests

When adding new features or fixing bugs, please include tests:

```rust
#[test]
fn test_specific_feature() {
    let mut emu = Emulator::new_default();
    // Setup test state
    // Execute the code path
    // Assert expected behavior
}
```

### Headless Testing

Use the `--frames` option for headless testing:

```bash
# Run for 100 frames and save screenshot
cargo run --release -p bbkemu -- game.gam --frames 100 --output test.png
```

### Smoke Tests

Smoke tests verify that all games load and run without panicking. These tests require game assets and ROM files, so they are marked as `#[ignore]` and must be run explicitly.

**Prerequisites:**
- Game files (`.gam`) in `tmp/games/`
- ROM files (`8.BIN`, `E.BIN`) in `tmp/roms/4980/`

**Run smoke tests:**

```bash
# Basic smoke test: load all games and check for panics (600 frames)
cargo test -p bbkemu-core --test smoke smoke_all_games -- --ignored --nocapture

# Screenshot test: generate BMP screenshots for visual inspection (600 frames)
cargo test -p bbkemu-core --test smoke smoke_screenshot_all_games -- --ignored --nocapture
```

**Environment variables:**
- `BBK_GAME_DIR` — override game directory (default: `tmp/games`)
- `BBK_ROM_DIR` — override ROM directory (default: `tmp/roms/4980`)

**Output:**
- Screenshots are saved to `tmp/smoke_screenshots/` (or `tmp/smoke_screenshots_extended/` for extended test)
- Each game produces a 159×96 BMP file with green-tinted monochrome LCD simulation

## Submitting Changes

### Pull Request Process

1. Update documentation if you're changing public APIs
2. Add tests for new functionality
3. Ensure all tests pass: `cargo test --workspace`
4. Update the README if needed
5. Write a clear PR description explaining the changes

### Commit Messages

Use clear, descriptive commit messages:

```
feat: add support for landscape LCD orientation
fix: correct timer interrupt frequency for 4988 model
docs: update CLI options documentation
test: add regression test for save state loading
```

## Reporting Issues

When reporting bugs, please include:

1. **Game file**: Which game are you running?
2. **Model**: Which BBK model (4980, 4988)?
3. **Steps to reproduce**: What did you do?
4. **Expected behavior**: What should happen?
5. **Actual behavior**: What actually happened?
6. **Screenshots**: If visual, include screenshots
7. **Logs**: Run with `--debug` and include relevant log output

## Getting Started

Check the [open issues](https://github.com/AloysHF/BBKEmu/issues) for tasks labeled `good first issue` or `help wanted`. If you have questions, feel free to open a discussion issue.

To understand the BBK game file format (`.gam`), see [Game File Formats](Game-File-Formats.md).
