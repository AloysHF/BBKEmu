# Standalone Emulator

This guide covers installing and running the standalone `bbkemu` binary,
loading games, keyboard controls, cheats, display options, and every
command-line option.

## Installation

Download the latest standalone binary for your platform from the
[Releases](https://github.com/AloysHF/BBKEmu/releases) page.

You can also build it from source:

```bash
cargo build -p bbkemu --release
```

The binary is produced at `target/release/bbkemu` (`.exe` on Windows).

## Loading Games

The standalone emulator accepts `.gam` files. ROM files (`8.BIN` and `E.BIN`)
are required and searched in the following order:

1. Paths specified via `-8` / `--rom8` and `-e` / `--rome` flags.
2. `system/BBKEmu/<model>/` relative to the current directory.

```bash
# Load a game with explicit ROM paths
bbkemu -8 8.BIN -e E.BIN path/to/game.gam

# Load a game (ROMs auto-discovered from system/BBKEmu/<model>/)
bbkemu path/to/game.gam
```

You can always print the built-in help with:

```bash
bbkemu --help
```

## Synopsis

```text
bbkemu [OPTIONS] <GAME_PATH>
```

## Options

| Option | Value | Default | Description |
|---|---|---|---|
| `<GAME_PATH>` | path | *required* | Path to the game file (`.gam`). |
| `-8, --rom8 <PATH>` | path | — | Font ROM file (`8.BIN`). If not specified, searches in `system/BBKEmu/<model>/`. |
| `-e, --rome <PATH>` | path | — | OS ROM file (`E.BIN`). Required for LLE mode. If not specified, searches in `system/BBKEmu/<model>/`. |
| `-s, --scale <N>` | `1`–`16` | `4` | Integer scaling factor for the window. |
| `-f, --fullscreen` | flag | off | Run in borderless fullscreen at the desktop resolution. |
| `-m, --model <MODEL>` | `4980`, `4988` | `4980` | BBK model to emulate. |
| `-d, --debug` | flag | off | Enable debug logging output. |
| `-o, --output <PATH>` | path | `output.png` | Output file path for headless mode screenshots. |
| `--frames <N>` | integer | — | Run headless for N frames, save screenshot, then exit. |
| `--swap-lcd` | flag | off | Swap LCD width and height for landscape display. |
| `--cpu-rate <N>` | `0.25`–`8.0` | `1.0` | CPU clock rate multiplier. |
| `--timer-rate <N>` | `0.25`–`8.0` | `1.0` | Timer clock rate multiplier. |
| `--key-repeat-interval <N>` | integer (ms) | `0` | Minimum key repeat interval in milliseconds (0 = no limit). |
| `--cheat <CODE>` | string | — | Cheat code in format `AAAAAAVV` (address=value). Repeatable for multiple cheats. |
| `-S, --screenshot <PATH>` | path | — | Take a screenshot after N frames and exit (saves as PNG). |
| `--screenshot-frames <N>` | integer | `30` | Number of frames to run before taking screenshot. |

`--screenshot-frames` only has an effect together with `--screenshot`.

## Default Key Mappings

| BBK Key | Physical Key | Action |
|---|---|---|
| Up | ↑ Arrow | Navigate up |
| Down | ↓ Arrow | Navigate down |
| Left | ← Arrow | Navigate left |
| Right | → Arrow | Navigate right |
| Enter | Enter | Confirm / Select |
| Exit | Backspace | Back / Cancel |
| Del | Delete | Delete character |
| Space | Space | Space character |
| PgUp | Page Up | Previous page |
| PgDn | Page Down | Next page |
| 0-9 | 0-9 | Number input |
| A-Z | A-Z | Letter input |

## Hotkeys

| Key | Action |
|---|---|
| F5 | Save state (to `<game>.sav`) |
| F8 | Load state (from `<game>.sav`) |
| F12 | Take screenshot (saves as `bbkemu-screenshot.png`) |
| Escape | Exit emulator |

## Headless Mode

BBKEmu supports headless execution for automated testing and CI:

```bash
# Run for 100 frames and save screenshot
bbkemu game.gam --frames 100 --output test.png

# Run with custom scale
bbkemu game.gam --frames 50 --scale 2 --output screenshot.png
```

Headless mode is useful for:
- Automated game compatibility testing
- CI/CD pipelines
- Batch screenshot generation
- Regression testing

## Screenshot Mode

Take a screenshot after running a specific number of frames:

```bash
# Take screenshot after 30 frames (default)
bbkemu game.gam --screenshot screenshot.png

# Take screenshot after 60 frames
bbkemu game.gam --screenshot screenshot.png --screenshot-frames 60
```

## LCD Orientation

Some games are designed for landscape display. Use `--swap-lcd` to rotate the LCD:

```bash
bbkemu game.gam --swap-lcd
```

## Clock Rate Adjustment

Fine-tune emulation speed for specific games:

```bash
# Run CPU at 2x speed
bbkemu game.gam --cpu-rate 2.0

# Run timers at half speed
bbkemu game.gam --timer-rate 0.5

# Combined adjustment
bbkemu game.gam --cpu-rate 1.5 --timer-rate 0.75
```

## Cheat Codes

Apply cheat codes using the `AAAAAAVV` format (6-digit address + 2-digit value):

```bash
# Single cheat
bbkemu game.gam --cheat "001234FF"

# Multiple cheats
bbkemu game.gam --cheat "001234FF" --cheat "005678AB"
```

## Examples

```bash
# Basic usage
bbkemu path/to/game.gam

# 2x scaling with specific model
bbkemu --scale 2 --model 4988 path/to/game.gam

# Fullscreen with landscape LCD
bbkemu --fullscreen --swap-lcd game.gam

# Headless testing
bbkemu game.gam --frames 200 --output test.png --debug

# Screenshot after 60 frames
bbkemu game.gam --screenshot screenshot.png --screenshot-frames 60
```
