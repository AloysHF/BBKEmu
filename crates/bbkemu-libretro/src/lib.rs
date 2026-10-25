//! BBKEmu libretro core

use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::panic;
use std::path::{Path, PathBuf};

use bbkemu_core::input::BbkKey;
use bbkemu_core::lcd::LcdOrientation;
use bbkemu_core::{model, BbkModel, Emulator};

// libretro constants
const RETRO_API_VERSION: u32 = 1;
const RETRO_REGION_NTSC: u32 = 0;
const RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY: u32 = 9;
const RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS: u32 = 11;
const RETRO_ENVIRONMENT_SET_KEYBOARD_CALLBACK: u32 = 12;
const RETRO_ENVIRONMENT_SET_VARIABLES: u32 = 16;
const RETRO_ENVIRONMENT_GET_VARIABLE: u32 = 17;
const RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE: u32 = 18;

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

/// Input poll callback
static mut INPUT_POLL_CB: RetroInputPollT = None;

/// Input state callback
static mut INPUT_STATE_CB: RetroInputStateT = None;

/// Keyboard callback
#[allow(dead_code)]
static mut KEYBOARD_CB: RetroKeyboardCallbackT = None;

/// Keyboard callback struct for RetroArch
#[repr(C)]
struct RetroKeyboardCallback {
    callback: RetroKeyboardCallbackT,
}

/// Core option definition for RetroArch
#[repr(C)]
struct RetroVariable {
    key: *const c_char,
    value: *const c_char,
}

/// Core options array for RetroArch
static CORE_OPTIONS: &[&str] = &[
    "bbkemu_lcd_color\0LCD Color Theme\0grey;green;blue;yellow;random\0",
    "bbkemu_swap_lcd\0Swap LCD Width/Height (Restart)\0portrait;landscape\0",
    concat!(
        "bbkemu_cpu_rate\0CPU Clock Rate\0",
        "0.25;0.50;0.75;1.00;1.50;2.00;3.00;4.00;8.00\0"
    ),
    concat!(
        "bbkemu_timer_rate\0Timer Clock Rate\0",
        "0.25;0.50;0.75;1.00;1.50;2.00;3.00;4.00;8.00\0"
    ),
    concat!(
        "bbkemu_key_repeat_interval\0Key Repeat Interval (ms)\0",
        "0;50;100;150;200;250;300;400;500\0"
    ),
];

/// Get the system directory from RetroArch
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

/// Try to load ROM files for the given model.
///
/// Desktop frontends install the firmware under `<system>/BBKEmu/<model>/`.
/// EmulatorJS cannot write to that directory: it extracts an `EJS_biosUrl`
/// archive beside the content file instead, so on Emscripten also look there.
fn load_roms_for_model(emu: &mut Emulator, model: &BbkModel, content_path: Option<&Path>) {
    let mut model_dirs = Vec::new();
    if let Some(system_dir) = get_system_directory() {
        model_dirs.push(system_dir.join("BBKEmu").join(model.name));
    }

    #[cfg(target_os = "emscripten")]
    if let Some(path) = content_path {
        let dir = path.parent().unwrap_or_else(|| Path::new("/"));
        model_dirs.push(dir.to_path_buf());
        model_dirs.push(dir.join("BBKEmu").join(model.name));
    }
    #[cfg(not(target_os = "emscripten"))]
    let _ = content_path;

    let find_rom = |names: &[&str]| {
        model_dirs
            .iter()
            .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
            .find_map(|path| std::fs::read(path).ok())
    };

    if let Some(data) = find_rom(&["8.BIN", "8.bin"]) {
        emu.load_rom_8(&data);
    }
    if let Some(data) = find_rom(&["E.BIN", "E.bin", "e.bin"]) {
        emu.load_rom_e(&data);
    }
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
    // Set up panic handler to prevent UB from unwinding across FFI boundary
    panic::set_hook(Box::new(|info| {
        eprintln!("BBKEmu panic: {}", info);
    }));

    let result = panic::catch_unwind(|| unsafe {
        let mut emu = Emulator::new(&model::MODEL_4980);
        apply_core_options(&mut emu);
        EMULATOR = Some(emu);
    });

    if result.is_err() {
        eprintln!("BBKEmu: panic during retro_init");
    }
}

#[no_mangle]
pub extern "C" fn retro_deinit() {
    unsafe {
        EMULATOR = None;
    }
}

#[no_mangle]
pub extern "C" fn retro_set_environment(cb: RetroEnvironmentT) {
    unsafe {
        ENVIRONMENT_CB = cb;

        // Set keyboard callback
        let kbd = RetroKeyboardCallback {
            callback: Some(keyboard_callback),
        };
        if let Some(env_cb) = cb {
            env_cb(
                RETRO_ENVIRONMENT_SET_KEYBOARD_CALLBACK,
                &kbd as *const _ as *mut c_void,
            );

            // Register core options
            let options: Vec<*const c_char> = CORE_OPTIONS
                .iter()
                .map(|s| s.as_ptr() as *const c_char)
                .chain(std::iter::once(std::ptr::null()))
                .collect();
            env_cb(
                RETRO_ENVIRONMENT_SET_VARIABLES,
                options.as_ptr() as *mut c_void,
            );
        }
    }
}

#[no_mangle]
pub extern "C" fn retro_set_video_refresh(cb: RetroVideoRefreshT) {
    unsafe {
        VIDEO_CB = cb;
    }
}

#[no_mangle]
pub extern "C" fn retro_set_audio_sample(cb: RetroAudioSampleT) {
    unsafe {
        AUDIO_CB = cb;
    }
}

#[no_mangle]
pub extern "C" fn retro_set_audio_sample_batch(cb: RetroAudioSampleBatchT) {
    unsafe {
        AUDIO_BATCH_CB = cb;
    }
}

#[no_mangle]
pub extern "C" fn retro_set_input_poll(cb: RetroInputPollT) {
    unsafe {
        INPUT_POLL_CB = cb;
    }
}

#[no_mangle]
pub extern "C" fn retro_set_input_state(cb: RetroInputStateT) {
    unsafe {
        INPUT_STATE_CB = cb;
    }
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
    (*info).library_name = c"BBKEmu".as_ptr();
    (*info).library_version = c"0.1.0".as_ptr();
    (*info).valid_extensions = c"gam".as_ptr();
    (*info).need_fullpath = false;
    (*info).block_extract = false;
}

/// # Safety
///
/// `info` must be a valid pointer to a `RetroSystemAvInfo` struct.
#[no_mangle]
pub unsafe extern "C" fn retro_get_system_av_info(info: *mut RetroSystemAvInfo) {
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
    (*info).timing.sample_rate = 44100.0;
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
/// `info` must be a valid pointer to a `RetroGameInfo` struct.
/// `(*info).data` must be a valid pointer to `(*info).size` bytes of game data.
#[no_mangle]
pub unsafe extern "C" fn retro_load_game(info: *const RetroGameInfo) -> bool {
    let result = panic::catch_unwind(|| {
        if info.is_null() {
            return false;
        }
        let data = (*info).data as *const u8;
        let size = (*info).size;
        if data.is_null() || size == 0 {
            return false;
        }
        let game_data = std::slice::from_raw_parts(data, size);
        let content_path = if (*info).path.is_null() {
            None
        } else {
            CStr::from_ptr((*info).path)
                .to_str()
                .ok()
                .map(PathBuf::from)
        };

        if let Some(ref mut emu) = EMULATOR {
            // Load ROMs before loading the game
            let bbk_model = &model::MODEL_4980;
            load_roms_for_model(emu, bbk_model, content_path.as_deref());

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

            let result = emu.load_gam(game_data);
            if result.is_ok() {
                // Apply model-specific joypad remapping after game is loaded
                apply_joypad_remapping_for_model(emu.model());
            }
            result.is_ok()
        } else {
            false
        }
    });

    match result {
        Ok(val) => val,
        Err(_) => {
            eprintln!("BBKEmu: panic during retro_load_game");
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn retro_unload_game() {}

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
    let _ = panic::catch_unwind(|| {
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
            }
        }
    });
}

#[no_mangle]
pub extern "C" fn retro_serialize_size() -> usize {
    1024 * 1024
}

#[no_mangle]
pub extern "C" fn retro_serialize(data: *mut c_void, size: usize) -> bool {
    let result = panic::catch_unwind(|| unsafe {
        if let Some(ref emu) = EMULATOR {
            let state = emu.save_state();
            let bytes = state.to_bytes();
            if bytes.len() <= size {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), data as *mut u8, bytes.len());
                true
            } else {
                false
            }
        } else {
            false
        }
    });
    result.unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn retro_unserialize(data: *const c_void, size: usize) -> bool {
    let result = panic::catch_unwind(|| unsafe {
        if let Some(ref mut emu) = EMULATOR {
            let bytes = std::slice::from_raw_parts(data as *const u8, size);
            match bbkemu_core::save::SaveState::from_bytes(bytes) {
                Ok(state) => emu.load_save_state(&state).is_ok(),
                Err(_) => false,
            }
        } else {
            false
        }
    });
    result.unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn retro_reset() {}

#[no_mangle]
pub extern "C" fn retro_cheat_reset() {
    let _ = panic::catch_unwind(|| unsafe {
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
    let _ = panic::catch_unwind(|| {
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
    unsafe {
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
    }
}

#[no_mangle]
pub extern "C" fn retro_get_memory_size(id: u32) -> usize {
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
}
