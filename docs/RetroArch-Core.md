# RetroArch Core

BBKEmu is available as a libretro core for RetroArch on Windows, Linux,
macOS, Android, and iOS. This guide covers installation, loading content,
supported frontend features, controls, and core options.

## Installation

### Online Updater

1. Open RetroArch.
2. Go to **Main Menu > Online Updater > Core Downloader**.
3. Select **BBKEmu**.

### Manual Installation

Download the core from the
[Releases](https://github.com/AloysHF/BBKEmu/releases) page. Copy
`bbkemu_libretro.dll` (`.so` on Linux or `.dylib` on macOS) to
RetroArch's `cores/` directory, and copy `bbkemu_libretro.info` to its
`info/` directory.

## Loading Games

1. Open RetroArch and select **Load Core > BBKEmu**.
2. Select **Load Content**.
3. Choose a `.gam` file.

ROM files (`8.BIN` and `E.BIN`) should be placed in RetroArch's
`system/BBKEmu/<model>/` directory (e.g., `system/BBKEmu/A4980/`).

## Mobile Platforms

The same libretro core architecture is available on mobile platforms, with
platform-specific installation requirements:

- [Android Libretro Core](Android-Libretro-Core.md)
- [iOS Libretro Core](iOS-Libretro-Core.md)

## Supported Features

- ✅ Video output (RGB565 pixel format)
- ✅ Audio output (tone generation)
- ✅ RetroPad input handling
- ✅ `.gam` content loading
- ✅ Save states
- ✅ Live core options
- ✅ SRAM support (flash memory)
- ✅ Cheat codes support

## RetroPad Button Mapping

| RetroPad Button | BBK Key | Action |
|---|---|---|
| D-Pad Left | Left | Navigate left |
| D-Pad Right | Right | Navigate right |
| D-Pad Up | Up | Navigate up |
| D-Pad Down | Down | Navigate down |
| A | Enter | Confirm |
| B | Exit | Back / Cancel |

## Core Options

Options are available from **Quick Menu > Core Options**. Changes apply live
without reloading the game.

| Option | Values | Default | Effect |
|---|---|---|---|
| Swap LCD Width/Height | portrait, landscape | portrait | Swap display dimensions for landscape orientation |
| CPU Clock Rate | 0.25, 0.50, 0.75, 1.00, 1.50, 2.00, 3.00, 4.00, 8.00 | 1.00 | CPU speed multiplier |
| Timer Clock Rate | 0.25, 0.50, 0.75, 1.00, 1.50, 2.00, 3.00, 4.00, 8.00 | 1.00 | Timer speed multiplier |
| Key Repeat Interval | 0, 50, 100, 150, 200, 250, 300, 400, 500 | 0 | Minimum interval between repeated key presses (ms); 0 = no limit |

### Swap LCD Width/Height

Some BBK games are designed for landscape display. This option swaps the LCD
dimensions (159×96 becomes 96×159) for better gameplay on widescreen displays.

### CPU Clock Rate

Adjusts the CPU emulation speed. Useful for games that run too fast or too slow
on default settings. Lower values slow down the CPU, higher values speed it up.

### Timer Clock Rate

Adjusts the timer interrupt frequency independently from the CPU. Some games
use timers for animation timing, music tempo, or input repeat behavior.

### Key Repeat Interval

Sets the minimum time (in milliseconds) between repeated key presses when a
button is held down. This prevents overly rapid input in games that are
sensitive to key repeat speed.

- **0** — No limit (fastest repeat, default behavior)
- **50–500** — Increasingly slower repeat rates

## Cheats

BBKEmu supports cheat codes through RetroArch's cheat interface. Use the
`AAAAAAVV` format (6-digit address + 2-digit value).

1. Load a game and open **Quick Menu > Cheats**.
2. Select **Add New Code to Top** or **Add New Code to Bottom**.
3. Enter the cheat code (e.g., `001234FF`).
4. Set **Enabled** to **On**.
5. Return to the Cheats menu and select **Apply Changes**.

See the official
[RetroArch cheat code guide](https://docs.libretro.com/guides/cheat-codes/)
for more information.
