//! Firmware ROM discovery shared by the standalone and libretro frontends.
//!
//! Firmware lives at a fixed location below the frontend system directory:
//!
//! ```text
//! <system>/
//! └── bbkemu/
//!     ├── a4980/
//!     │   ├── 8.BIN
//!     │   └── E.BIN
//!     └── a4988/
//!         ├── 8.BIN
//!         └── E.BIN
//! ```
//!
//! Every component is matched byte-for-byte. `std::fs::read` cannot be used
//! for discovery because Windows resolves paths case-insensitively, so the
//! directory entries are compared directly instead.

use std::path::{Path, PathBuf};

use crate::model::BbkModel;

/// Folder below the system directory that holds BBK firmware.
pub const CORE_DIR: &str = "bbkemu";
/// Font ROM file name.
pub const FONT_ROM: &str = "8.BIN";
/// OS ROM file name.
pub const OS_ROM: &str = "E.BIN";

/// Firmware bytes for one model.
#[derive(Debug)]
pub struct Firmware {
    pub font: Vec<u8>,
    pub os: Vec<u8>,
}

/// Return the entry of `dir` whose name equals `name` exactly.
pub fn exact_child(dir: &Path, name: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .find(|entry| entry.file_name().to_str() == Some(name))
        .map(|entry| entry.path())
}

/// Directory holding the firmware of `model`, if it exists.
pub fn firmware_dir(system_dir: &Path, model: &BbkModel) -> Option<PathBuf> {
    exact_child(system_dir, CORE_DIR)
        .filter(|path| path.is_dir())
        .and_then(|core_dir| exact_child(&core_dir, model.id))
        .filter(|path| path.is_dir())
}

/// Load both ROMs for `model`, reporting every missing file at once.
pub fn load(system_dir: &Path, model: &BbkModel) -> Result<Firmware, String> {
    let dir = firmware_dir(system_dir, model).ok_or_else(|| {
        format!(
            "firmware directory not found: {}/bbkemu/{}\n\
             Copy 8.BIN and E.BIN into that directory.",
            system_dir.display(),
            model.id
        )
    })?;

    let mut missing = Vec::new();
    let font = read_rom(&dir, FONT_ROM, &mut missing)?;
    let os = read_rom(&dir, OS_ROM, &mut missing)?;
    if !missing.is_empty() {
        return Err(format!(
            "missing firmware: {} in {}",
            missing.join(", "),
            dir.display()
        ));
    }

    Ok(Firmware {
        font: font.expect("font ROM is present when nothing is missing"),
        os: os.expect("OS ROM is present when nothing is missing"),
    })
}

fn read_rom(
    dir: &Path,
    name: &'static str,
    missing: &mut Vec<&'static str>,
) -> Result<Option<Vec<u8>>, String> {
    let Some(path) = exact_child(dir, name).filter(|path| path.is_file()) else {
        missing.push(name);
        return Ok(None);
    };
    std::fs::read(&path)
        .map(Some)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "bbkemu-firmware-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn finds_the_fixed_model_directory() {
        let root = temp_dir("fixed");
        let dir = root.join("bbkemu").join("a4980");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("8.BIN"), b"font").unwrap();
        std::fs::write(dir.join("E.BIN"), b"os").unwrap();

        let firmware = load(&root, &crate::model::MODEL_4980).unwrap();
        assert_eq!(firmware.font, b"font");
        assert_eq!(firmware.os, b"os");

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_case_mismatched_names() {
        let root = temp_dir("case");
        let dir = root.join("bbkemu").join("a4980");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("8.bin"), b"font").unwrap();
        std::fs::write(dir.join("e.bin"), b"os").unwrap();

        let error = load(&root, &crate::model::MODEL_4980).unwrap_err();
        assert!(error.contains("8.BIN"), "{error}");
        assert!(error.contains("E.BIN"), "{error}");

        std::fs::remove_dir_all(root).unwrap();
    }
}
