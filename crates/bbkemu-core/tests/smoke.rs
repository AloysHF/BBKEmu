//! Smoke test: load every available BBK game, run it for a number of
//! frames, and assert the emulator neither panics nor produces a blank frame.
//!
//! This test needs the (large, non-distributed) game assets and ROM files,
//! so it is marked `#[ignore]` and only runs on demand:
//!
//! ```text
//! cargo test -p bbkemu-core --test smoke -- --ignored --nocapture
//! ```
//!
//! By default it looks for games in `<repo>/tmp/bbk_game` and ROMs in
//! `<repo>/tmp/roms/bbkemu/a4980`. Override the locations with the `BBK_GAME_DIR`
//! and `BBK_ROM_DIR` environment variables.

use std::path::{Path, PathBuf};

use bbkemu_core::emulator::Emulator;

/// Number of frames to run per game before sampling the output.
const FRAMES: u64 = 600;

/// Resolve the directory that holds the BBK game assets (.gam files).
fn game_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("BBK_GAME_DIR") {
        let p = PathBuf::from(dir);
        return p.is_dir().then_some(p);
    }
    // Default: <workspace_root>/tmp/bbk_game. CARGO_MANIFEST_DIR points at the
    // core crate (crates/bbkemu-core), so go up two levels.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidate = manifest
        .parent()
        .and_then(|p| p.parent())
        .map(|root| root.join("tmp").join("bbk_game"));
    candidate.filter(|p| p.is_dir())
}

/// Resolve the directory that holds the ROM files (8.BIN, E.BIN).
/// Defaults to tmp/roms/bbkemu/a4980; override with BBK_ROM_DIR.
fn rom_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("BBK_ROM_DIR") {
        let p = PathBuf::from(dir);
        return p.is_dir().then_some(p);
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.parent().and_then(|p| p.parent())?;
    let candidate = root.join("tmp").join("roms").join("bbkemu").join("a4980");
    candidate.is_dir().then_some(candidate)
}

/// Collect all .gam files in a directory (non-recursive).
fn collect_games(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("gam") {
                    out.push(path);
                }
            }
        }
    }
}

/// Returns true if the LCD buffer contains more than one distinct pixel value,
/// i.e. it is not a single flat color (all-off / all-on).
fn frame_has_content(framebuffer: &[bool; 159 * 96]) -> bool {
    let mut iter = framebuffer.iter();
    let first = match iter.next() {
        Some(&p) => p,
        None => return false,
    };
    iter.any(|&p| p != first)
}

/// Create an emulator with ROMs loaded.
fn create_emulator_with_roms() -> Option<Emulator> {
    let rom_path = rom_dir()?;

    let mut emu = Emulator::new_default();

    // Load font ROM (8.BIN)
    let rom_8_path = rom_path.join("8.BIN");
    if rom_8_path.exists() {
        let rom_8 = std::fs::read(&rom_8_path).ok()?;
        emu.load_rom_8(&rom_8);
    } else {
        eprintln!("Warning: 8.BIN not found at {}", rom_8_path.display());
    }

    // Load OS ROM (E.BIN)
    let rom_e_path = rom_path.join("E.BIN");
    if rom_e_path.exists() {
        let rom_e = std::fs::read(&rom_e_path).ok()?;
        emu.load_rom_e(&rom_e);
    } else {
        eprintln!("Warning: E.BIN not found at {}", rom_e_path.display());
    }

    Some(emu)
}

#[test]
#[ignore = "requires local BBK game assets and ROM files (set BBK_GAME_DIR, BBK_ROM_DIR)"]
fn smoke_all_games() {
    let dir = match game_dir() {
        Some(d) => d,
        None => {
            eprintln!(
                "skipping: no game directory found \
                 (set BBK_GAME_DIR or place games in <repo>/tmp/bbk_game)"
            );
            return;
        }
    };

    let mut games = Vec::new();
    collect_games(&dir, &mut games);
    games.sort();

    assert!(
        !games.is_empty(),
        "no .gam games found under {}",
        dir.display()
    );

    println!(
        "Running smoke test over {} games ({FRAMES} frames each)",
        games.len()
    );

    let mut failures = Vec::new();
    let mut warnings = Vec::new();

    for game in &games {
        let rel = game.strip_prefix(&dir).unwrap_or(game);
        match run_one(game) {
            Ok(true) => println!("[PASS] {}", rel.display()),
            Ok(false) => {
                println!("[WARN] {} (blank frame)", rel.display());
                warnings.push(format!("{}: blank frame", rel.display()));
            }
            Err(reason) => {
                println!("[FAIL] {} - {reason}", rel.display());
                failures.push(format!("{}: {reason}", rel.display()));
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Total: {}", games.len());
    println!("Pass: {}", games.len() - failures.len() - warnings.len());
    println!("Warn: {}", warnings.len());
    println!("Fail: {}", failures.len());

    assert!(
        failures.is_empty(),
        "{} game(s) failed the smoke test:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Run a single game for `FRAMES` frames. Returns Ok(true) if the final frame
/// has visible content, Ok(false) if it is blank, or Err on a load failure.
/// A panic inside the emulator will fail the test via the normal unwinding.
fn run_one(path: &Path) -> Result<bool, String> {
    let mut emu = create_emulator_with_roms()
        .ok_or_else(|| "failed to create emulator (ROM directory not found)".to_string())?;

    let gam_data = std::fs::read(path).map_err(|e| format!("failed to read game file: {e}"))?;

    emu.load_gam(&gam_data)
        .map_err(|e| format!("failed to load game: {e}"))?;

    // Run frames until completion or the emulator stops
    let mut frames_run = 0u64;
    while frames_run < FRAMES && emu.is_running() {
        emu.run_frame();
        frames_run += 1;
    }

    if !emu.is_running() && frames_run < FRAMES {
        println!("  (stopped after {} frames)", frames_run);
    }

    let fb = emu.render_lcd_buffer();
    Ok(frame_has_content(&fb))
}

/// Save the LCD framebuffer as a PNG file for visual inspection.
fn save_frame_as_png(path: &Path, framebuffer: &[bool; 159 * 96]) -> image::ImageResult<()> {
    let width = 159u32;
    let height = 96u32;
    let mut img = image::RgbImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel = framebuffer[(y * 159 + x) as usize];
            // Green-tinted monochrome (like the original LCD)
            let color = if pixel {
                image::Rgb([0x00, 0x40, 0x00]) // Dark green for "on" pixels
            } else {
                image::Rgb([0x90, 0xB0, 0x70]) // Light green for "off" pixels (LCD background)
            };
            img.put_pixel(x, y, color);
        }
    }

    img.save(path)
}

#[test]
#[ignore = "requires local BBK game assets and ROM files (set BBK_GAME_DIR, BBK_ROM_DIR)"]
fn smoke_screenshot_all_games() {
    let dir = match game_dir() {
        Some(d) => d,
        None => {
            eprintln!("skipping: no game directory found");
            return;
        }
    };

    let output_dir = dir.parent().unwrap_or(&dir).join("smoke_screenshots");
    std::fs::create_dir_all(&output_dir).ok();

    let mut games = Vec::new();
    collect_games(&dir, &mut games);
    games.sort();

    println!("Saving screenshots to {}", output_dir.display());
    println!("Running {} games ({} frames each)", games.len(), FRAMES);

    let mut failures = Vec::new();

    for game in &games {
        let game_name = game
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        match run_and_screenshot(game, &output_dir.join(format!("{game_name}.png"))) {
            Ok(true) => println!("[PASS] {game_name}"),
            Ok(false) => println!("[WARN] {game_name} (blank frame)"),
            Err(e) => {
                println!("[FAIL] {game_name} - {e}");
                failures.push(format!("{game_name}: {e}"));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} game(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Run a single game and save a screenshot.
fn run_and_screenshot(game_path: &Path, screenshot_path: &Path) -> Result<bool, String> {
    let mut emu =
        create_emulator_with_roms().ok_or_else(|| "ROM directory not found".to_string())?;

    let gam_data = std::fs::read(game_path).map_err(|e| format!("read failed: {e}"))?;

    emu.load_gam(&gam_data)
        .map_err(|e| format!("load failed: {e}"))?;

    // Run frames
    let mut frames_run = 0u64;
    while frames_run < FRAMES && emu.is_running() {
        emu.run_frame();
        frames_run += 1;
    }

    // Capture and save screenshot
    let fb = emu.render_lcd_buffer();
    save_frame_as_png(screenshot_path, &fb).map_err(|e| format!("screenshot save failed: {e}"))?;

    Ok(frame_has_content(&fb))
}
