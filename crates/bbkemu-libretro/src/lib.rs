//! BBKEmu libretro core

use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::panic;
use std::path::{Path, PathBuf};

use bbkemu_core::input::BbkKey;
use bbkemu_core::lcd::LcdOrientation;
use bbkemu_core::{firmware, model, BbkModel, Emulator};

// libretro constants
const RETRO_API_VERSION: u32 = 1;
const RETRO_REGION_NTSC: u32 = 0;
const RETRO_ENVIRONMENT_SET_MESSAGE: u32 = 6;
#[cfg(not(target_os = "emscripten"))]
const RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY: u32 = 9;
const RETRO_ENVIRONMENT_SET_PIXEL_FORMAT: u32 = 10;
const RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS: u32 = 11;
const RETRO_ENVIRONMENT_SET_KEYBOARD_CALLBACK: u32 = 12;
const RETRO_ENVIRONMENT_SET_VARIABLES: u32 = 16;
const RETRO_ENVIRONMENT_GET_VARIABLE: u32 = 17;
const RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE: u32 = 18;
#[cfg(not(target_os = "emscripten"))]
const RETRO_ENVIRONMENT_GET_LOG_INTERFACE: u32 = 27;
const RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY: u32 = 31;

// libretro log levels
const RETRO_LOG_INFO: u32 = 1;
const RETRO_LOG_WARN: u32 = 2;
const RETRO_LOG_ERROR: u32 = 3;

// libretro pixel formats
const RETRO_PIXEL_FORMAT_RGB565: u32 = 2;

/// Sample rate reported in the AV info and used by the emulated sound chip.
const AUDIO_SAMPLE_RATE: usize = 44100;
/// The emulator runs at 60 fps, so one video frame carries 735 stereo frames.
const AUDIO_FRAMES_PER_VIDEO_FRAME: usize = AUDIO_SAMPLE_RATE / 60;

// libretro memory constants
const RETRO_MEMORY_SAVE_RAM: u32 = 0;
const RETRO_MEMORY_SYSTEM_RAM: u32 = 2;

// RetroArch keyboard constants
const RETROK_RETURN: u32 = 13;
const RETROK_ESCAPE: u32 = 27;
const RETROK_SPACE: u32 = 32;
const RETROK_LEFT: u32 = 0x250;
const RETROK_UP: u32 = 0x251;
const RETROK_RIGHT: u32 = 0x252;
const RETROK_DOWN: u32 = 0x253;
#[allow(dead_code)]
const RETROK_A: u32 = 97;
#[allow(dead_code)]
const RETROK_Z: u32 = 122;
const RETROK_BACKSPACE: u32 = 8;
const RETROK_DELETE: u32 = 127;
const RETROK_PAGEUP: u32 = 0x254;
const RETROK_PAGEDOWN: u32 = 0x255;

// libretro input constants
const RETRO_DEVICE_JOYPAD: u32 = 1;
const RETRO_DEVICE_ID_JOYPAD_UP: u32 = 4;
const RETRO_DEVICE_ID_JOYPAD_DOWN: u32 = 5;
const RETRO_DEVICE_ID_JOYPAD_LEFT: u32 = 6;
const RETRO_DEVICE_ID_JOYPAD_RIGHT: u32 = 7;
const RETRO_DEVICE_ID_JOYPAD_A: u32 = 8;
const RETRO_DEVICE_ID_JOYPAD_B: u32 = 0;
const RETRO_DEVICE_ID_JOYPAD_X: u32 = 9;
const RETRO_DEVICE_ID_JOYPAD_Y: u32 = 1;
const RETRO_DEVICE_ID_JOYPAD_L: u32 = 10;
const RETRO_DEVICE_ID_JOYPAD_R: u32 = 11;
const RETRO_DEVICE_ID_JOYPAD_SELECT: u32 = 2;
const RETRO_DEVICE_ID_JOYPAD_START: u32 = 3;

// Type aliases for libretro callbacks
type RetroEnvironmentT = Option<unsafe extern "C" fn(cmd: u32, data: *mut c_void) -> bool>;
type RetroVideoRefreshT =
    Option<unsafe extern "C" fn(data: *const c_void, width: u32, height: u32, pitch: usize)>;
type RetroAudioSampleT = Option<unsafe extern "C" fn(left: i16, right: i16)>;
type RetroAudioSampleBatchT =
    Option<unsafe extern "C" fn(data: *const i16, frames: usize) -> usize>;
type RetroInputPollT = Option<unsafe extern "C" fn()>;
type RetroInputStateT =
    Option<unsafe extern "C" fn(port: u32, device: u32, index: u32, id: u32) -> i16>;
type RetroKeyboardCallbackT =
    Option<unsafe extern "C" fn(down: bool, keycode: u32, character: u32, key_modifiers: u16)>;
/// Frontend logging callback.
///
/// libretro declares it C-variadic, but over wasm the callee signature carries
/// a hidden varargs pointer, so calling it from Rust traps with "function
/// signature mismatch". Native builds call it with the message as the format
/// string and no arguments; Emscripten builds log to stderr instead.
#[cfg(not(target_os = "emscripten"))]
type RetroLogPrintfT = Option<unsafe extern "C" fn(level: u32, fmt: *const c_char)>;

// libretro structs
#[repr(C)]
pub struct RetroSystemInfo {
    library_name: *const c_char,
    library_version: *const c_char,
    valid_extensions: *const c_char,
    need_fullpath: bool,
    block_extract: bool,
}

#[repr(C)]
pub struct RetroGameGeometry {
    base_width: u32,
    base_height: u32,
    max_width: u32,
    max_height: u32,
    aspect_ratio: f32,
}

#[repr(C)]
pub struct RetroSystemTiming {
    fps: f64,
    sample_rate: f64,
}

#[repr(C)]
pub struct RetroSystemAvInfo {
    geometry: RetroGameGeometry,
    timing: RetroSystemTiming,
}

#[repr(C)]
pub struct RetroGameInfo {
    path: *const c_char,
    data: *const c_void,
    size: usize,
    meta: *const c_char,
}

/// Global emulator instance
static mut EMULATOR: Option<Emulator> = None;

/// Framebuffer for rendering (RGB565 format, max size for either orientation)
static mut FRAMEBUFFER: [u16; 159 * 96] = [0; 159 * 96];

/// Environment callback for querying system directory
static mut ENVIRONMENT_CB: RetroEnvironmentT = None;

/// Video callback for sending frames to frontend
static mut VIDEO_CB: RetroVideoRefreshT = None;

/// Audio callback for sending samples to frontend
static mut AUDIO_CB: RetroAudioSampleT = None;

/// Audio batch callback for sending sample buffers to frontend
static mut AUDIO_BATCH_CB: RetroAudioSampleBatchT = None;

/// Frontend logging callback, queried from the frontend's environment
#[cfg(not(target_os = "emscripten"))]
static mut LOG_CB: RetroLogPrintfT = None;

/// Input poll callback
static mut INPUT_POLL_CB: RetroInputPollT = None;

/// Input state callback
static mut INPUT_STATE_CB: RetroInputStateT = None;

/// Keyboard callback
#[allow(dead_code)]
static mut KEYBOARD_CB: RetroKeyboardCallbackT = None;

/// Battery save file chosen for the currently loaded game, if any.
static mut BATTERY_SAVE_PATH: Option<PathBuf> = None;

/// Size of the battery-backed flash sector.
///
/// Save data lives in the last 32 KiB of flash; the bus rotates that sector to
/// the front of the backing store, so it is the first `0x8000` bytes here.
const BATTERY_SAVE_LEN: usize = 0x8000;

/// Keyboard callback struct for RetroArch
#[repr(C)]
struct RetroKeyboardCallback {
    callback: RetroKeyboardCallbackT,
}

/// On-screen message shown by the frontend (RETRO_ENVIRONMENT_SET_MESSAGE).
#[repr(C)]
struct RetroMessage {
    msg: *const c_char,
    /// Duration in frames.
    frames: u32,
}

/// Frontend logging callback (RETRO_ENVIRONMENT_GET_LOG_INTERFACE).
#[cfg(not(target_os = "emscripten"))]
#[repr(C)]
struct RetroLogCallback {
    log: RetroLogPrintfT,
}

/// Core option definition for RetroArch
#[repr(C)]
struct RetroVariable {
    key: *const c_char,
    value: *const c_char,
}

/// Core options in the libretro `Label; choice1|choice2` format.
static CORE_OPTIONS: &[(&str, &str)] = &[
    ("bbkemu_model", "BBK Model; a4980|a4988"),
    (
        "bbkemu_lcd_color",
        "LCD Color Theme; grey|green|blue|yellow|random",
    ),
    (
        "bbkemu_swap_lcd",
        "Swap LCD Width/Height (Restart); portrait|landscape",
    ),
    (
        "bbkemu_cpu_rate",
        "CPU Clock Rate; 0.25|0.50|0.75|1.00|1.50|2.00|3.00|4.00|8.00",
    ),
    (
        "bbkemu_timer_rate",
        "Timer Clock Rate; 0.25|0.50|0.75|1.00|1.50|2.00|3.00|4.00|8.00",
    ),
    (
        "bbkemu_key_repeat_interval",
        "Key Repeat Interval (ms); 0|50|100|150|200|250|300|400|500",
    ),
];

/// Get the system directory from RetroArch
#[cfg(not(target_os = "emscripten"))]
fn get_system_directory() -> Option<PathBuf> {
    unsafe {
        let cb = ENVIRONMENT_CB?;
        let mut dir: *const c_char = std::ptr::null();
        if cb(
            RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY,
            &mut dir as *mut _ as *mut c_void,
        ) && !dir.is_null()
        {
            let cstr = CStr::from_ptr(dir);
            return Some(PathBuf::from(cstr.to_str().ok()?));
        }
        None
    }
}

/// Mirror a message to the frontend's log and the `log` crate.
fn log_message(level: u32, message: &str) {
    match level {
        RETRO_LOG_ERROR => log::error!("{message}"),
        RETRO_LOG_WARN => log::warn!("{message}"),
        _ => log::info!("{message}"),
    }

    // EmulatorJS forwards stderr to the browser console.
    #[cfg(target_os = "emscripten")]
    eprintln!("{message}");

    // The message itself is the format string, so percent signs in game names
    // or paths have to be escaped before reaching the frontend's printf.
    #[cfg(not(target_os = "emscripten"))]
    unsafe {
        if let Some(log_cb) = LOG_CB {
            let escaped = message.replace('%', "%%");
            if let Ok(c_message) = std::ffi::CString::new(escaped) {
                log_cb(level, c_message.as_ptr());
            }
        }
    }
}

/// Ask the frontend to show a short on-screen message.
///
/// Frontends without an on-screen message facility (headless test harnesses)
/// simply return false; the message is already mirrored to the log by callers.
fn show_message(message: &str, frames: u32) {
    unsafe {
        let Some(env_cb) = ENVIRONMENT_CB else {
            return;
        };
        let Ok(c_message) = std::ffi::CString::new(message) else {
            return;
        };
        let msg = RetroMessage {
            msg: c_message.as_ptr(),
            frames,
        };
        env_cb(
            RETRO_ENVIRONMENT_SET_MESSAGE,
            &msg as *const _ as *mut c_void,
        );
    }
}

/// The error reported when the frontend supplies no content path.
const MISSING_CONTENT_PATH: &str =
    "The frontend passed no content path; this core loads content from a file path only.";

/// Report a load failure to both the log and the frontend's on-screen message.
fn fail(message: &str) -> bool {
    log_message(RETRO_LOG_ERROR, message);
    show_message(message, 240);
    false
}

/// Run `body`, turning a panic into `fallback`.
///
/// A panic must never unwind across the C ABI: the frontend is a host process
/// (RetroArch, a mobile app) that would be torn down with us. Swallowing the
/// panic keeps the session recoverable and leaves a log entry behind.
fn guard<T>(fallback: T, body: impl FnOnce() -> T) -> T {
    match panic::catch_unwind(panic::AssertUnwindSafe(body)) {
        Ok(value) => value,
        Err(payload) => {
            let detail = payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_owned())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown panic".to_owned());
            log_message(
                RETRO_LOG_ERROR,
                &format!("BBKEmu libretro entry point panicked: {detail}"),
            );
            fallback
        }
    }
}

/// Get the save directory from the frontend.
///
/// Saves must never be written next to the content: Android hands the core a
/// disposable cache copy of the user's document, so the save would be lost.
fn get_save_directory() -> Option<PathBuf> {
    unsafe {
        let cb = ENVIRONMENT_CB?;
        let mut dir: *const c_char = std::ptr::null();
        if cb(
            RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY,
            &mut dir as *mut _ as *mut c_void,
        ) && !dir.is_null()
        {
            let cstr = CStr::from_ptr(dir);
            return Some(PathBuf::from(cstr.to_str().ok()?));
        }
        None
    }
}

/// Battery save file for the given content, inside the frontend save directory.
fn battery_save_path(content_path: Option<&Path>) -> Option<PathBuf> {
    let stem = content_path?.file_stem()?.to_str()?;
    Some(get_save_directory()?.join(format!("{stem}.sav")))
}

/// Copy the persisted battery save into the emulated flash save sector.
fn restore_battery_save(emu: &mut Emulator, path: &Path) {
    match std::fs::read(path) {
        Ok(bytes) => {
            let len = bytes.len().min(BATTERY_SAVE_LEN);
            let flash = emu.cpu.memory_mut().flash.as_mut_slice();
            flash[..len].copy_from_slice(&bytes[..len]);
            log::info!(
                "Battery save loaded from {} ({} bytes)",
                path.display(),
                len
            );
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => log::warn!("Failed to read battery save {}: {error}", path.display()),
    }
}

/// Write the emulated flash save sector to the frontend save directory.
#[allow(static_mut_refs)]
fn persist_battery_save() {
    let Some(path) = (unsafe { BATTERY_SAVE_PATH.clone() }) else {
        return;
    };
    let Some(emu) = (unsafe { EMULATOR.as_ref() }) else {
        return;
    };
    if let Some(parent) = path.parent() {
        if let Err(error) = std::fs::create_dir_all(parent) {
            log::warn!(
                "Failed to create save directory {}: {error}",
                parent.display()
            );
            return;
        }
    }
    match std::fs::write(&path, &emu.cpu.memory().flash[..BATTERY_SAVE_LEN]) {
        Ok(()) => log::info!("Battery save written to {}", path.display()),
        Err(error) => log::warn!("Failed to write battery save {}: {error}", path.display()),
    }
}

/// Directory that holds firmware, as seen by the core.
///
/// EmulatorJS extracts `EJS_biosUrl` archives into the virtual filesystem
/// root and does not report a system directory, so `/` is the root there.
fn firmware_root() -> Option<PathBuf> {
    #[cfg(target_os = "emscripten")]
    {
        Some(PathBuf::from("/"))
    }
    #[cfg(not(target_os = "emscripten"))]
    {
        get_system_directory()
    }
}

/// Load the firmware ROMs for the selected model.
fn load_firmware(emu: &mut Emulator, model: &BbkModel) -> Result<(), String> {
    let root = firmware_root().ok_or_else(|| {
        format!(
            "the frontend did not report a system directory\n\
             Copy 8.BIN and E.BIN into <system>/bbkemu/{}.",
            model.id
        )
    })?;
    let firmware = firmware::load(&root, model)?;
    emu.load_rom_8(&firmware.font);
    emu.load_rom_e(&firmware.os);
    Ok(())
}

/// Model selected by the frontend, defaulting to the A4980.
unsafe fn selected_model() -> &'static BbkModel {
    get_variable_value(ENVIRONMENT_CB, "bbkemu_model")
        .and_then(|value| model::from_id(&value))
        .unwrap_or(&model::MODEL_4980)
}

/// Model named by the only `bbkemu/<model>/` folder below `root`.
///
/// EmulatorJS exposes no core-option UI, so the firmware archive layout
/// selects the model there. Several model folders are rejected instead of
/// guessed.
#[cfg(any(target_os = "emscripten", test))]
fn model_in_root(root: &Path) -> Result<Option<&'static BbkModel>, String> {
    let Some(core_dir) =
        firmware::exact_child(root, firmware::CORE_DIR).filter(|path| path.is_dir())
    else {
        return Ok(None);
    };

    let mut models: Vec<&'static BbkModel> = Vec::new();
    let entries = std::fs::read_dir(&core_dir)
        .map_err(|error| format!("cannot read {}: {error}", core_dir.display()))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("cannot read {}: {error}", core_dir.display()))?;
        if !entry.path().is_dir() {
            continue;
        }
        if let Some(model) = entry.file_name().to_str().and_then(model::from_id) {
            models.push(model);
        }
    }
    models.sort_by_key(|model| model.id);
    models.dedup_by_key(|model| model.id);

    match models.as_slice() {
        [] => Ok(None),
        [model] => Ok(Some(*model)),
        _ => Err(format!(
            "multiple model folders in {}/{}: {}; keep only one model per archive",
            root.display(),
            firmware::CORE_DIR,
            models
                .iter()
                .map(|model| model.id)
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Read the bytes of the content being loaded.
///
/// The core advertises `need_fullpath = true`, so the frontend always hands
/// over a real file path. A memory buffer is ignored.
fn read_content(path: Option<&Path>) -> Result<Vec<u8>, String> {
    let Some(path) = path else {
        return Err(MISSING_CONTENT_PATH.to_owned());
    };
    std::fs::read(path).map_err(|error| format!("BBKEmu: cannot read {}.\n{error}", path.display()))
}

/// Helper function to get a core option value
unsafe fn get_variable_value(cb: RetroEnvironmentT, key: &str) -> Option<String> {
    let cb = cb?;
    let c_key = std::ffi::CString::new(key).ok()?;
    let mut var = RetroVariable {
        key: c_key.as_ptr(),
        value: std::ptr::null(),
    };
    if cb(
        RETRO_ENVIRONMENT_GET_VARIABLE,
        &mut var as *mut _ as *mut c_void,
    ) && !var.value.is_null()
    {
        let value = CStr::from_ptr(var.value);
        return value.to_str().ok().map(|s| s.to_string());
    }
    None
}

/// Apply core options from RetroArch to the emulator
fn apply_core_options(emu: &mut Emulator) {
    unsafe {
        let Some(cb) = ENVIRONMENT_CB else {
            return;
        };

        // Check for swap_lcd option
        if let Some(value) = get_variable_value(Some(cb), "bbkemu_swap_lcd") {
            if value == "landscape" {
                emu.set_lcd_orientation(LcdOrientation::Landscape);
            } else {
                emu.set_lcd_orientation(LcdOrientation::Portrait);
            }
        }

        // Check for lcd_color option
        if let Some(value) = get_variable_value(Some(cb), "bbkemu_lcd_color") {
            use bbkemu_core::lcd::LcdTheme;
            let theme = match value.as_str() {
                "green" => LcdTheme::GREEN,
                "blue" => LcdTheme::BLUE,
                "yellow" => LcdTheme::YELLOW,
                "random" => LcdTheme::random(),
                _ => LcdTheme::GREY,
            };
            emu.set_lcd_theme(theme);
        }

        // Check for cpu_rate option
        if let Some(value) = get_variable_value(Some(cb), "bbkemu_cpu_rate") {
            if let Ok(rate) = value.parse::<f32>() {
                emu.set_cpu_rate(rate);
            }
        }

        // Check for timer_rate option
        if let Some(value) = get_variable_value(Some(cb), "bbkemu_timer_rate") {
            if let Ok(rate) = value.parse::<f32>() {
                emu.set_timer_rate(rate);
            }
        }

        // Check for key_repeat_interval option
        if let Some(value) = get_variable_value(Some(cb), "bbkemu_key_repeat_interval") {
            if let Ok(interval) = value.parse::<u64>() {
                emu.set_key_repeat_interval(interval);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn retro_api_version() -> u32 {
    RETRO_API_VERSION
}

#[no_mangle]
pub extern "C" fn retro_init() {
    // Keep a diagnostic for panics that happen outside the guarded entry
    // points; guarded ones are logged with their payload instead.
    panic::set_hook(Box::new(|info| log::error!("BBKEmu panic: {info}")));

    guard((), || unsafe {
        let mut emu = Emulator::new(&model::MODEL_4980);
        apply_core_options(&mut emu);
        EMULATOR = Some(emu);
    });
}

#[no_mangle]
pub extern "C" fn retro_deinit() {
    guard((), || {
        persist_battery_save();
        unsafe {
            EMULATOR = None;
            BATTERY_SAVE_PATH = None;
        }
    });
}

#[no_mangle]
pub extern "C" fn retro_set_environment(cb: RetroEnvironmentT) {
    guard((), || unsafe {
        ENVIRONMENT_CB = cb;

        // Set keyboard callback
        let kbd = RetroKeyboardCallback {
            callback: Some(keyboard_callback),
        };
        if let Some(env_cb) = cb {
            // Query the log interface first so later failures reach the
            // frontend's log as well as the on-screen message. Over wasm the
            // callback is unusable (see `RetroLogPrintfT`), so it is native
            // only.
            #[cfg(not(target_os = "emscripten"))]
            {
                let mut log = RetroLogCallback { log: None };
                if env_cb(
                    RETRO_ENVIRONMENT_GET_LOG_INTERFACE,
                    &mut log as *mut _ as *mut c_void,
                ) {
                    LOG_CB = log.log;
                }
            }

            env_cb(
                RETRO_ENVIRONMENT_SET_KEYBOARD_CALLBACK,
                &kbd as *const _ as *mut c_void,
            );

            // Register core options in the libretro `Label; choice1|choice2`
            // format; the array must hold `retro_variable` pairs, not strings.
            let keys: Vec<std::ffi::CString> = CORE_OPTIONS
                .iter()
                .filter_map(|(key, _)| std::ffi::CString::new(*key).ok())
                .collect();
            let values: Vec<std::ffi::CString> = CORE_OPTIONS
                .iter()
                .filter_map(|(_, value)| std::ffi::CString::new(*value).ok())
                .collect();
            let mut variables: Vec<RetroVariable> = keys
                .iter()
                .zip(&values)
                .map(|(key, value)| RetroVariable {
                    key: key.as_ptr(),
                    value: value.as_ptr(),
                })
                .collect();
            variables.push(RetroVariable {
                key: std::ptr::null(),
                value: std::ptr::null(),
            });
            env_cb(
                RETRO_ENVIRONMENT_SET_VARIABLES,
                variables.as_mut_ptr() as *mut c_void,
            );
        }
    });
}

#[no_mangle]
pub extern "C" fn retro_set_video_refresh(cb: RetroVideoRefreshT) {
    guard((), || unsafe {
        VIDEO_CB = cb;
    });
}

#[no_mangle]
pub extern "C" fn retro_set_audio_sample(cb: RetroAudioSampleT) {
    guard((), || unsafe {
        AUDIO_CB = cb;
    });
}

#[no_mangle]
pub extern "C" fn retro_set_audio_sample_batch(cb: RetroAudioSampleBatchT) {
    guard((), || unsafe {
        AUDIO_BATCH_CB = cb;
    });
}

#[no_mangle]
pub extern "C" fn retro_set_input_poll(cb: RetroInputPollT) {
    guard((), || unsafe {
        INPUT_POLL_CB = cb;
    });
}

#[no_mangle]
pub extern "C" fn retro_set_input_state(cb: RetroInputStateT) {
    guard((), || unsafe {
        INPUT_STATE_CB = cb;
    });
}

/// Map RetroArch keyboard key to BBK key
fn map_keyboard_to_bbk_key(keycode: u32) -> Option<BbkKey> {
    match keycode {
        RETROK_RETURN => Some(BbkKey::Enter),
        RETROK_ESCAPE => Some(BbkKey::Exit),
        RETROK_SPACE => Some(BbkKey::Space),
        RETROK_LEFT => Some(BbkKey::Left),
        RETROK_RIGHT => Some(BbkKey::Right),
        RETROK_UP => Some(BbkKey::Up),
        RETROK_DOWN => Some(BbkKey::Down),
        RETROK_BACKSPACE | RETROK_DELETE => Some(BbkKey::Del),
        RETROK_PAGEUP => Some(BbkKey::PgUp),
        RETROK_PAGEDOWN => Some(BbkKey::PgDn),
        _ => None,
    }
}

/// Keyboard callback function
unsafe extern "C" fn keyboard_callback(
    down: bool,
    keycode: u32,
    _character: u32,
    _key_modifiers: u16,
) {
    if !down {
        return;
    }

    if let Some(ref mut emu) = EMULATOR {
        if let Some(bbk_key) = map_keyboard_to_bbk_key(keycode) {
            emu.key_down(bbk_key);
        }
    }
}

/// # Safety
///
/// `info` must be a valid pointer to a `RetroSystemInfo` struct.
#[no_mangle]
pub unsafe extern "C" fn retro_get_system_info(info: *mut RetroSystemInfo) {
    guard((), || {
        (*info).library_name = c"BBKEmu".as_ptr();
        (*info).library_version =
            concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char;
        (*info).valid_extensions = c"gam".as_ptr();
        (*info).need_fullpath = true;
        (*info).block_extract = false;
    });
}

/// # Safety
///
/// `info` must be a valid pointer to a `RetroSystemAvInfo` struct.
#[no_mangle]
pub unsafe extern "C" fn retro_get_system_av_info(info: *mut RetroSystemAvInfo) {
    guard((), || {
        if let Some(ref emu) = EMULATOR {
            (*info).geometry.base_width = emu.display_width() as u32;
            (*info).geometry.base_height = emu.display_height() as u32;
            (*info).geometry.max_width = emu.display_width() as u32;
            (*info).geometry.max_height = emu.display_height() as u32;
        } else {
            (*info).geometry.base_width = 159;
            (*info).geometry.base_height = 96;
            (*info).geometry.max_width = 159;
            (*info).geometry.max_height = 96;
        }
        (*info).geometry.aspect_ratio = 0.0;
        (*info).timing.fps = 60.0;
        (*info).timing.sample_rate = AUDIO_SAMPLE_RATE as f64;
    });
}

/// Input descriptor for RetroArch
#[repr(C)]
struct RetroInputDescriptor {
    port: u32,
    device: u32,
    index: u32,
    id: u32,
    description: *const c_char,
}

/// # Safety
///
/// `info` must be a valid pointer to a `RetroGameInfo` struct whose `path` is
/// either null or a valid NUL-terminated string.
#[allow(static_mut_refs)]
#[no_mangle]
pub unsafe extern "C" fn retro_load_game(info: *const RetroGameInfo) -> bool {
    guard(false, || {
        if info.is_null() || (*info).path.is_null() {
            return fail(MISSING_CONTENT_PATH);
        }
        let content_path = match CStr::from_ptr((*info).path).to_str() {
            Ok(path) => Some(PathBuf::from(path)),
            Err(_) => {
                return fail("BBKEmu: the content path is not valid UTF-8.");
            }
        };
        // Frames are emitted as u16 RGB565 samples; the frontend has to know
        // that before it receives the first frame. Declare it before any
        // content is read so an unsupported format fails without doing I/O.
        let mut pixel_format = RETRO_PIXEL_FORMAT_RGB565;
        let declared = match ENVIRONMENT_CB {
            Some(cb) => cb(
                RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
                &mut pixel_format as *mut _ as *mut c_void,
            ),
            None => false,
        };
        if !declared {
            return fail("BBKEmu: this frontend does not support RGB565 video.");
        }

        let game_data = match read_content(content_path.as_deref()) {
            Ok(data) => data,
            Err(message) => return fail(&message),
        };

        // The model decides which firmware directory is used, so it has to be
        // applied before the ROMs are read. The emulator is rebuilt only when
        // the frontend selected another model than the default.
        #[cfg(target_os = "emscripten")]
        let bbk_model = match model_in_root(Path::new("/")) {
            Ok(Some(model)) => model,
            Ok(None) => selected_model(),
            Err(message) => return fail(&format!("BBKEmu: {message}")),
        };
        #[cfg(not(target_os = "emscripten"))]
        let bbk_model = selected_model();
        let needs_model = EMULATOR
            .as_ref()
            .is_none_or(|emu| emu.model().id != bbk_model.id);
        if needs_model {
            let mut emu = Emulator::new(bbk_model);
            apply_core_options(&mut emu);
            EMULATOR = Some(emu);
        }

        let Some(ref mut emu) = EMULATOR else {
            return fail("BBKEmu: the core was not initialized by the frontend.");
        };

        // Load the firmware ROMs before loading the game.
        if let Err(message) = load_firmware(emu, bbk_model) {
            return fail(&format!("BBKEmu: {message}"));
        }

        // Set input descriptors
        let inputs = [
            RetroInputDescriptor {
                port: 0,
                device: RETRO_DEVICE_JOYPAD,
                index: 0,
                id: RETRO_DEVICE_ID_JOYPAD_B,
                description: c"EXIT".as_ptr(),
            },
            RetroInputDescriptor {
                port: 0,
                device: RETRO_DEVICE_JOYPAD,
                index: 0,
                id: RETRO_DEVICE_ID_JOYPAD_Y,
                description: c"HELP".as_ptr(),
            },
            RetroInputDescriptor {
                port: 0,
                device: RETRO_DEVICE_JOYPAD,
                index: 0,
                id: RETRO_DEVICE_ID_JOYPAD_SELECT,
                description: c"INSERT".as_ptr(),
            },
            RetroInputDescriptor {
                port: 0,
                device: RETRO_DEVICE_JOYPAD,
                index: 0,
                id: RETRO_DEVICE_ID_JOYPAD_START,
                description: c"SEARCH".as_ptr(),
            },
            RetroInputDescriptor {
                port: 0,
                device: RETRO_DEVICE_JOYPAD,
                index: 0,
                id: RETRO_DEVICE_ID_JOYPAD_UP,
                description: c"UP".as_ptr(),
            },
            RetroInputDescriptor {
                port: 0,
                device: RETRO_DEVICE_JOYPAD,
                index: 0,
                id: RETRO_DEVICE_ID_JOYPAD_DOWN,
                description: c"DOWN".as_ptr(),
            },
            RetroInputDescriptor {
                port: 0,
                device: RETRO_DEVICE_JOYPAD,
                index: 0,
                id: RETRO_DEVICE_ID_JOYPAD_LEFT,
                description: c"LEFT".as_ptr(),
            },
            RetroInputDescriptor {
                port: 0,
                device: RETRO_DEVICE_JOYPAD,
                index: 0,
                id: RETRO_DEVICE_ID_JOYPAD_RIGHT,
                description: c"RIGHT".as_ptr(),
            },
            RetroInputDescriptor {
                port: 0,
                device: RETRO_DEVICE_JOYPAD,
                index: 0,
                id: RETRO_DEVICE_ID_JOYPAD_A,
                description: c"ENTER".as_ptr(),
            },
            RetroInputDescriptor {
                port: 0,
                device: RETRO_DEVICE_JOYPAD,
                index: 0,
                id: RETRO_DEVICE_ID_JOYPAD_X,
                description: c"R".as_ptr(),
            },
            RetroInputDescriptor {
                port: 0,
                device: RETRO_DEVICE_JOYPAD,
                index: 0,
                id: RETRO_DEVICE_ID_JOYPAD_L,
                description: c"PGUP".as_ptr(),
            },
            RetroInputDescriptor {
                port: 0,
                device: RETRO_DEVICE_JOYPAD,
                index: 0,
                id: RETRO_DEVICE_ID_JOYPAD_R,
                description: c"PGDN".as_ptr(),
            },
            RetroInputDescriptor {
                port: 0,
                device: 0,
                index: 0,
                id: 0,
                description: std::ptr::null(),
            },
        ];

        if let Some(env_cb) = ENVIRONMENT_CB {
            env_cb(
                RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS,
                inputs.as_ptr() as *mut c_void,
            );
        }

        match emu.load_gam(&game_data) {
            Ok(()) => {
                // Apply model-specific joypad remapping after game is loaded
                apply_joypad_remapping_for_model(emu.model());
                log_message(RETRO_LOG_INFO, "BBKEmu: game loaded");
                // Content may live in a throwaway location (a browser MEMFS
                // tree or an Android app cache copy of a SAF document), so
                // keep battery saves in the frontend's persistent save
                // directory.
                if let Some(path) = battery_save_path(content_path.as_deref()) {
                    restore_battery_save(emu, &path);
                    BATTERY_SAVE_PATH = Some(path);
                }
                true
            }
            Err(error) => fail(&format!(
                "BBKEmu: failed to load this game.\n{error}\n\nSupported content: .gam"
            )),
        }
    })
}

#[no_mangle]
pub extern "C" fn retro_unload_game() {
    guard((), || {
        persist_battery_save();
        unsafe {
            BATTERY_SAVE_PATH = None;
        }
    });
}

/// Default joypad button to BBK key mapping (A4980)
static mut JOYK: [BbkKey; 16] = [
    BbkKey::Exit,   // RETRO_DEVICE_ID_JOYPAD_B (0) -> EXIT
    BbkKey::Help,   // RETRO_DEVICE_ID_JOYPAD_Y (1) -> HELP
    BbkKey::Insert, // RETRO_DEVICE_ID_JOYPAD_SELECT (2) -> INSERT
    BbkKey::Search, // RETRO_DEVICE_ID_JOYPAD_START (3) -> SEARCH
    BbkKey::Up,     // RETRO_DEVICE_ID_JOYPAD_UP (4) -> UP
    BbkKey::Down,   // RETRO_DEVICE_ID_JOYPAD_DOWN (5) -> DOWN
    BbkKey::Left,   // RETRO_DEVICE_ID_JOYPAD_LEFT (6) -> LEFT
    BbkKey::Right,  // RETRO_DEVICE_ID_JOYPAD_RIGHT (7) -> RIGHT
    BbkKey::Enter,  // RETRO_DEVICE_ID_JOYPAD_A (8) -> ENTER
    BbkKey::R,      // RETRO_DEVICE_ID_JOYPAD_X (9) -> R
    BbkKey::PgUp,   // RETRO_DEVICE_ID_JOYPAD_L (10) -> PGUP
    BbkKey::PgDn,   // RETRO_DEVICE_ID_JOYPAD_R (11) -> PGDN
    BbkKey::Modify, // RETRO_DEVICE_ID_JOYPAD_L2 (12) -> MODIFY
    BbkKey::Del,    // RETRO_DEVICE_ID_JOYPAD_R2 (13) -> DEL
    BbkKey::A,      // RETRO_DEVICE_ID_JOYPAD_L3 (14) -> A
    BbkKey::Z,      // RETRO_DEVICE_ID_JOYPAD_R3 (15) -> Z
];

/// Apply A4988-specific joypad remapping
unsafe fn apply_joypad_remapping_for_model(model: &BbkModel) {
    if model.bank_sys_d == 0x0E88 {
        // A4988 has a different keyboard layout
        JOYK[1] = BbkKey::Z; // Y -> Z
        JOYK[2] = BbkKey::Del; // SELECT -> DEL (SHIFT on A4988)
        JOYK[3] = BbkKey::Zy; // START -> ZY
        JOYK[12] = BbkKey::Space; // L2 -> SPACE
        JOYK[13] = BbkKey::X; // R2 -> X
        JOYK[15] = BbkKey::S; // R3 -> S
    }
}

/// Static state for joypad input handling
static mut JOYPAD_PRESSED: i32 = -1;
static mut JOYPAD_REPEAT: i32 = 0;

#[no_mangle]
pub extern "C" fn retro_run() {
    guard((), || {
        unsafe {
            if let Some(ref mut emu) = EMULATOR {
                // 0. Check for core option updates
                let mut updated = false;
                if let Some(cb) = ENVIRONMENT_CB {
                    if cb(
                        RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE,
                        &mut updated as *mut _ as *mut c_void,
                    ) && updated
                    {
                        apply_core_options(emu);
                    }
                }

                // 1. Poll input
                if let Some(poll_cb) = INPUT_POLL_CB {
                    poll_cb();
                }

                // 2. Handle joypad input
                if let Some(input_cb) = INPUT_STATE_CB {
                    // Check if previously pressed button is still held
                    if JOYPAD_PRESSED >= 0 {
                        if input_cb(0, RETRO_DEVICE_JOYPAD, 0, JOYPAD_PRESSED as u32) == 0 {
                            // Button released
                            JOYPAD_PRESSED = -1;
                            JOYPAD_REPEAT = 0;
                        } else {
                            // Button still held - handle repeat
                            JOYPAD_REPEAT += 1;
                            if JOYPAD_REPEAT > 20 {
                                JOYPAD_REPEAT -= 5;
                                emu.key_down(JOYK[JOYPAD_PRESSED as usize]);
                            }
                        }
                    }

                    // Check for new button press
                    if JOYPAD_PRESSED == -1 {
                        for i in 0..16 {
                            if input_cb(0, RETRO_DEVICE_JOYPAD, 0, i) != 0 {
                                // New button pressed
                                JOYPAD_PRESSED = i as i32;
                                JOYPAD_REPEAT = 0;
                                emu.key_down(JOYK[i as usize]);
                                break;
                            }
                        }
                    }
                }

                // 3. Run emulation frame
                emu.run_frame();

                // 4. Render LCD with orientation support
                #[allow(static_mut_refs)]
                emu.render_lcd_with_orientation(&mut FRAMEBUFFER, false);

                // 5. Send video frame to frontend
                if let Some(video_cb) = VIDEO_CB {
                    let display_width = emu.display_width() as u32;
                    let display_height = emu.display_height() as u32;
                    #[allow(static_mut_refs)]
                    video_cb(
                        FRAMEBUFFER.as_ptr() as *const c_void,
                        display_width,
                        display_height,
                        (display_width * 2) as usize, // pitch = width * bytes_per_pixel (RGB565 = 2 bytes)
                    );
                }

                // 6. Send one frame of signed 16-bit stereo audio
                if let Some(batch_cb) = AUDIO_BATCH_CB {
                    let mut mono = [0.0f32; AUDIO_FRAMES_PER_VIDEO_FRAME];
                    emu.audio.mix_samples(&mut mono);
                    let mut stereo = [0i16; AUDIO_FRAMES_PER_VIDEO_FRAME * 2];
                    for (frame, sample) in mono.iter().enumerate() {
                        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                        stereo[frame * 2] = value;
                        stereo[frame * 2 + 1] = value;
                    }
                    batch_cb(stereo.as_ptr(), AUDIO_FRAMES_PER_VIDEO_FRAME);
                }
            }
        }
    })
}

/// Bytes advertised to the frontend for the serialized payload, excluding the
/// envelope. The state itself is roughly 115 KiB (RAM plus the saved flash).
const SAVE_STATE_CAPACITY: usize = 1024 * 1024;

/// Save-state envelope: 4-byte magic followed by a little-endian format
/// version. Bump the version whenever the core's state layout changes so stale
/// states are rejected instead of silently restoring into the wrong fields.
const SAVE_STATE_MAGIC: [u8; 4] = *b"BBKS";
const SAVE_STATE_FORMAT: u32 = 1;
const SAVE_STATE_HEADER_LEN: usize = SAVE_STATE_MAGIC.len() + std::mem::size_of::<u32>();

fn write_save_state_header(buffer: &mut [u8]) {
    buffer[..SAVE_STATE_MAGIC.len()].copy_from_slice(&SAVE_STATE_MAGIC);
    buffer[SAVE_STATE_MAGIC.len()..SAVE_STATE_HEADER_LEN]
        .copy_from_slice(&SAVE_STATE_FORMAT.to_le_bytes());
}

/// Returns `Some(payload)` when the envelope matches this build's format.
fn read_save_state_payload(data: &[u8]) -> Option<&[u8]> {
    if data.len() < SAVE_STATE_HEADER_LEN || data[..SAVE_STATE_MAGIC.len()] != SAVE_STATE_MAGIC {
        return None;
    }
    let version = u32::from_le_bytes(
        data[SAVE_STATE_MAGIC.len()..SAVE_STATE_HEADER_LEN]
            .try_into()
            .ok()?,
    );
    (version == SAVE_STATE_FORMAT).then(|| &data[SAVE_STATE_HEADER_LEN..])
}

#[no_mangle]
pub extern "C" fn retro_serialize_size() -> usize {
    guard(0, || SAVE_STATE_CAPACITY + SAVE_STATE_HEADER_LEN)
}

#[no_mangle]
pub extern "C" fn retro_serialize(data: *mut c_void, size: usize) -> bool {
    guard(false, || unsafe {
        let Some(ref emu) = EMULATOR else {
            return false;
        };
        if data.is_null() || size < SAVE_STATE_CAPACITY + SAVE_STATE_HEADER_LEN {
            return false;
        }
        let bytes = emu.save_state().to_bytes();
        if bytes.len() > SAVE_STATE_CAPACITY {
            log_message(
                RETRO_LOG_ERROR,
                "BBKEmu: the save state does not fit in the advertised buffer.",
            );
            return false;
        }
        let buffer = std::slice::from_raw_parts_mut(data as *mut u8, size);
        write_save_state_header(buffer);
        buffer[SAVE_STATE_HEADER_LEN..].fill(0);
        buffer[SAVE_STATE_HEADER_LEN..SAVE_STATE_HEADER_LEN + bytes.len()].copy_from_slice(&bytes);
        true
    })
}

#[no_mangle]
pub extern "C" fn retro_unserialize(data: *const c_void, size: usize) -> bool {
    guard(false, || unsafe {
        let Some(ref mut emu) = EMULATOR else {
            return false;
        };
        if data.is_null() {
            return false;
        }
        let bytes = std::slice::from_raw_parts(data as *const u8, size);
        let Some(payload) = read_save_state_payload(bytes) else {
            log_message(
                RETRO_LOG_WARN,
                "BBKEmu: rejecting a save state with an unknown format or version.",
            );
            return false;
        };
        match bbkemu_core::save::SaveState::from_bytes(payload) {
            Ok(state) => emu.load_save_state(&state).is_ok(),
            Err(error) => {
                log_message(
                    RETRO_LOG_WARN,
                    &format!("BBKEmu: rejecting a corrupt save state ({error})."),
                );
                false
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn retro_reset() {}

#[no_mangle]
pub extern "C" fn retro_cheat_reset() {
    guard((), || unsafe {
        if let Some(ref mut emu) = EMULATOR {
            emu.cheat.clear();
        }
    });
}

/// # Safety
///
/// `code` must be a valid pointer to a null-terminated C string, or null.
#[no_mangle]
pub unsafe extern "C" fn retro_cheat_set(index: u32, enabled: bool, code: *const c_char) {
    guard((), || {
        if code.is_null() {
            return;
        }

        if let Some(ref mut emu) = EMULATOR {
            let code_str = CStr::from_ptr(code);
            if let Ok(code_str) = code_str.to_str() {
                // If the code string is empty, disable the cheat at this index
                if code_str.is_empty() {
                    emu.cheat.set_cheat_enabled(index, false);
                    return;
                }

                // Try to add the cheat code
                if let Some(id) = emu.cheat.add_cheat(code_str) {
                    emu.cheat.set_cheat_enabled(id, enabled);
                    log::info!("Cheat {} set: {} (enabled: {})", id, code_str, enabled);
                } else {
                    log::warn!("Invalid cheat code: {}", code_str);
                }
            }
        }
    });
}

#[no_mangle]
pub extern "C" fn retro_load_game_special(
    _game_type: u32,
    _info: *const RetroGameInfo,
    _num_info: usize,
) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn retro_set_controller_port_device(_port: u32, _device: u32) {}

#[no_mangle]
pub extern "C" fn retro_get_region() -> u32 {
    RETRO_REGION_NTSC
}

#[no_mangle]
pub extern "C" fn retro_get_memory_data(id: u32) -> *mut c_void {
    guard(std::ptr::null_mut(), || unsafe {
        if let Some(ref mut emu) = EMULATOR {
            match id {
                RETRO_MEMORY_SAVE_RAM => {
                    // Return flash memory pointer for save RAM
                    emu.cpu.memory_mut().flash.as_mut_ptr() as *mut c_void
                }
                RETRO_MEMORY_SYSTEM_RAM => {
                    // Return RAM pointer
                    emu.cpu.memory_mut().ram.as_mut_ptr() as *mut c_void
                }
                _ => std::ptr::null_mut(),
            }
        } else {
            std::ptr::null_mut()
        }
    })
}

#[no_mangle]
pub extern "C" fn retro_get_memory_size(id: u32) -> usize {
    guard(0, || {
        #[allow(static_mut_refs)]
        unsafe {
            if EMULATOR.is_some() {
                match id {
                    RETRO_MEMORY_SAVE_RAM => {
                        // Save area: first 0x14000 bytes of flash
                        0x14000
                    }
                    RETRO_MEMORY_SYSTEM_RAM => {
                        // Full RAM size
                        0x8000
                    }
                    _ => 0,
                }
            } else {
                0
            }
        }
    })
}

#[cfg(test)]
mod save_state_tests {
    use super::*;

    #[test]
    fn save_state_envelope_rejects_other_versions() {
        let mut buffer = [0_u8; SAVE_STATE_HEADER_LEN + 3];
        buffer[SAVE_STATE_HEADER_LEN..].copy_from_slice(&[1, 2, 3]);
        write_save_state_header(&mut buffer);
        assert_eq!(
            read_save_state_payload(&buffer),
            Some(&[1_u8, 2, 3][..]),
            "a freshly written envelope must be accepted"
        );

        buffer[SAVE_STATE_MAGIC.len()] = 0xFF;
        assert_eq!(
            read_save_state_payload(&buffer),
            None,
            "a state from another format version must be rejected"
        );

        let truncated = [0_u8; SAVE_STATE_HEADER_LEN - 1];
        assert_eq!(
            read_save_state_payload(&truncated),
            None,
            "a buffer shorter than the envelope must be rejected"
        );
    }

    #[test]
    fn save_state_envelope_rejects_foreign_magic() {
        let mut buffer = [0_u8; SAVE_STATE_HEADER_LEN];
        write_save_state_header(&mut buffer);
        buffer[..SAVE_STATE_MAGIC.len()].copy_from_slice(b"N32S");
        assert_eq!(read_save_state_payload(&buffer), None);
    }

    #[test]
    fn zero_padded_envelope_round_trips() {
        use bbkemu_core::save::{BankState, CpuState, SaveState};

        let state = SaveState {
            ram: vec![0xAB; 64],
            flash: vec![0xCD; 32],
            cpu: CpuState {
                pc: 0x0400,
                sp: 0xFD,
                a: 1,
                x: 2,
                y: 3,
                status: 0x30,
                cycles: 12345,
            },
            bank_switch: BankState {
                banks: vec![0, 1, 2, 3],
                selected: 1,
            },
            bank_sys_d: 0x1234,
            flash_cmd: 0,
            flash_cycles: 0,
        };
        let bytes = state.to_bytes();

        let mut buffer = vec![0_u8; SAVE_STATE_CAPACITY + SAVE_STATE_HEADER_LEN];
        write_save_state_header(&mut buffer);
        buffer[SAVE_STATE_HEADER_LEN..SAVE_STATE_HEADER_LEN + bytes.len()].copy_from_slice(&bytes);

        let payload = read_save_state_payload(&buffer).expect("envelope must validate");
        let restored = SaveState::from_bytes(payload).expect("padded payload must decode");
        assert_eq!(restored.ram, state.ram);
        assert_eq!(restored.flash, state.flash);
        assert_eq!(restored.cpu.cycles, state.cpu.cycles);
        assert_eq!(restored.bank_switch.banks, state.bank_switch.banks);
        assert_eq!(restored.bank_sys_d, state.bank_sys_d);
    }
}

#[cfg(test)]
mod model_scan_tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "bbkemu-model-scan-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn selects_the_only_model_folder() {
        let root = temp_root("single");
        std::fs::create_dir_all(root.join("bbkemu").join("a4988")).unwrap();
        let model = model_in_root(&root).unwrap().expect("model folder");
        assert_eq!(model.id, "a4988");
    }

    #[test]
    fn ignores_unknown_folders_and_requires_the_exact_core_dir() {
        let root = temp_root("unknown");
        std::fs::create_dir_all(root.join("BBKEmu").join("a4980")).unwrap();
        assert!(model_in_root(&root).unwrap().is_none());

        std::fs::create_dir_all(root.join("bbkemu").join("other")).unwrap();
        assert!(model_in_root(&root).unwrap().is_none());
    }

    #[test]
    fn rejects_several_model_folders() {
        let root = temp_root("several");
        std::fs::create_dir_all(root.join("bbkemu").join("a4980")).unwrap();
        std::fs::create_dir_all(root.join("bbkemu").join("a4988")).unwrap();
        let Err(error) = model_in_root(&root) else {
            panic!("several model folders must be rejected");
        };
        assert!(
            error.contains("a4980") && error.contains("a4988"),
            "{error}"
        );
    }
}
