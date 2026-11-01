//! BBK model definitions

/// BBK electronic dictionary model configuration
pub struct BbkModel {
    /// Stable lowercase id (e.g., "a4980") used for firmware directories,
    /// core options and command-line arguments.
    pub id: &'static str,
    /// Model name (e.g., "A4980", "A4988")
    pub name: &'static str,
    /// LCD width in pixels
    pub lcd_width: u8,
    /// LCD height in pixels
    pub lcd_height: u8,
    /// Model identifier (bank_sys_d value)
    /// 0x0EA8 = A4980, 0x0E88 = A4988
    pub bank_sys_d: u16,
    /// CPU entry point after OS init
    pub entry_point: u16,
}

/// BBK A4980 model
pub const MODEL_4980: BbkModel = BbkModel {
    id: "a4980",
    name: "A4980",
    lcd_width: 159,
    lcd_height: 96,
    bank_sys_d: 0x0EA8,
    entry_point: 0x350,
};

/// BBK A4988 model
pub const MODEL_4988: BbkModel = BbkModel {
    id: "a4988",
    name: "A4988",
    lcd_width: 159,
    lcd_height: 96,
    bank_sys_d: 0x0E88,
    entry_point: 0x350,
};

/// Look up a model by its stable id. Names are matched byte-for-byte; the
/// same lowercase ids name the model folders below the system directory.
pub fn from_id(id: &str) -> Option<&'static BbkModel> {
    match id {
        "a4980" => Some(&MODEL_4980),
        "a4988" => Some(&MODEL_4988),
        _ => None,
    }
}

/// Detect model from GAM file or bank_sys_d value
pub fn detect_model(bank_sys_d: u16) -> &'static BbkModel {
    match bank_sys_d {
        0x0EA8 => &MODEL_4980,
        0x0E88 => &MODEL_4988,
        _ => {
            log::warn!(
                "Unknown model (bank_sys_d=0x{:04X}), defaulting to A4980",
                bank_sys_d
            );
            &MODEL_4980
        }
    }
}
