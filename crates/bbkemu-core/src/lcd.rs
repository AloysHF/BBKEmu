//! LCD display emulation (159x96 monochrome)

pub const LCD_WIDTH: usize = 159;
pub const LCD_HEIGHT: usize = 96;
pub const FRAMEBUFFER_SIZE: usize = LCD_WIDTH * LCD_HEIGHT;

/// LCD display orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LcdOrientation {
    /// Normal portrait orientation (159x96)
    Portrait,
    /// Swapped landscape orientation (96x159)
    Landscape,
}

impl LcdOrientation {
    /// Get the display width for this orientation
    pub fn width(&self) -> usize {
        match self {
            Self::Portrait => LCD_WIDTH,
            Self::Landscape => LCD_HEIGHT,
        }
    }

    /// Get the display height for this orientation
    pub fn height(&self) -> usize {
        match self {
            Self::Portrait => LCD_HEIGHT,
            Self::Landscape => LCD_WIDTH,
        }
    }
}

/// LCD color theme
#[derive(Clone, Copy)]
pub struct LcdTheme {
    /// Background color (RGB565)
    pub bg: u16,
    /// Foreground color (RGB565)
    pub fg: u16,
    /// Ghosting effect intensity (0 = off, higher = more ghosting)
    pub ghosting: u8,
}

impl LcdTheme {
    pub const GREY: Self = Self {
        bg: 0xD6DA,
        fg: 0x0000,
        ghosting: 20,
    };
    pub const GREEN: Self = Self {
        bg: 0x96E1,
        fg: 0x0882,
        ghosting: 20,
    };
    pub const BLUE: Self = Self {
        bg: 0x3EDD,
        fg: 0x09A8,
        ghosting: 20,
    };
    pub const YELLOW: Self = Self {
        bg: 0xF72C,
        fg: 0x2920,
        ghosting: 20,
    };

    /// Generate a random LCD color theme with sufficient contrast.
    ///
    /// Ensures each RGB channel of the background is at least 18 units
    /// brighter than the foreground, matching the hardware LCD behavior.
    pub fn random() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        // Simple LCG seeded from system time
        let mut state = seed as u64;
        let mut next = || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            ((state >> 33) & 0xFFFF) as u16
        };

        loop {
            let bg = next();
            let fg = next();
            let bg_r = ((bg >> 11) & 0x1F) as i16;
            let bg_g = ((bg >> 6) & 0x1F) as i16;
            let bg_b = (bg & 0x1F) as i16;
            let fg_r = ((fg >> 11) & 0x1F) as i16;
            let fg_g = ((fg >> 6) & 0x1F) as i16;
            let fg_b = (fg & 0x1F) as i16;
            if bg_r >= fg_r + 18 && bg_g >= fg_g + 18 && bg_b >= fg_b + 18 {
                return Self {
                    bg,
                    fg,
                    ghosting: 20,
                };
            }
        }
    }
}

/// LCD framebuffer
pub struct Lcd {
    /// Pixel buffer (true = foreground, false = background)
    pixels: [bool; FRAMEBUFFER_SIZE],
    /// Ghosting effect accumulator
    ghosting: [i8; FRAMEBUFFER_SIZE],
    /// Cursor X position
    cursor_x: u8,
    /// Cursor Y position
    cursor_y: u8,
    /// Whether the display has been updated since last render
    dirty: bool,
}

impl Default for Lcd {
    fn default() -> Self {
        Self::new()
    }
}

impl Lcd {
    pub fn new() -> Self {
        Self {
            pixels: [false; FRAMEBUFFER_SIZE],
            ghosting: [0; FRAMEBUFFER_SIZE],
            cursor_x: 0,
            cursor_y: 0,
            dirty: true,
        }
    }

    /// Clear the entire display
    pub fn clear(&mut self) {
        self.pixels.fill(false);
        self.dirty = true;
    }

    /// Set a single pixel
    pub fn set_pixel(&mut self, x: u8, y: u8, color: bool) {
        if (x as usize) < LCD_WIDTH && (y as usize) < LCD_HEIGHT {
            self.pixels[y as usize * LCD_WIDTH + x as usize] = color;
            self.dirty = true;
        }
    }

    /// Get a single pixel
    pub fn get_pixel(&self, x: u8, y: u8) -> bool {
        if (x as usize) < LCD_WIDTH && (y as usize) < LCD_HEIGHT {
            self.pixels[y as usize * LCD_WIDTH + x as usize]
        } else {
            false
        }
    }

    /// Set cursor position
    pub fn set_cursor(&mut self, x: u8, y: u8) {
        self.cursor_x = x;
        self.cursor_y = y;
    }

    /// Get cursor X position
    pub fn cursor_x(&self) -> u8 {
        self.cursor_x
    }

    /// Get cursor Y position
    pub fn cursor_y(&self) -> u8 {
        self.cursor_y
    }

    /// Draw a character at cursor position and advance cursor
    pub fn draw_char(&mut self, ch: u8, font: &FontData) {
        if let Some(glyph) = font.get_glyph(ch) {
            for row in 0..font.height {
                for col in 0..font.width {
                    if glyph.get_pixel(col, row) {
                        self.set_pixel(self.cursor_x + col, self.cursor_y + row, true);
                    }
                }
            }
            self.cursor_x = self.cursor_x.saturating_add(font.width);
        }
    }

    /// Draw a string at cursor position
    pub fn draw_string(&mut self, s: &[u8], font: &FontData) {
        for &ch in s {
            self.draw_char(ch, font);
        }
    }

    /// Fill a rectangle
    pub fn fill_rect(&mut self, x: u8, y: u8, w: u8, h: u8, color: bool) {
        for dy in 0..h {
            for dx in 0..w {
                self.set_pixel(x + dx, y + dy, color);
            }
        }
    }

    /// Scroll the display up by N lines
    pub fn scroll_up(&mut self, lines: u8) {
        let offset = lines as usize * LCD_WIDTH;
        if offset < FRAMEBUFFER_SIZE {
            self.pixels.copy_within(offset.., 0);
            self.pixels[FRAMEBUFFER_SIZE - offset..].fill(false);
        }
        self.dirty = true;
    }

    /// Render to RGB565 buffer for frontend display
    pub fn render(&self, buf: &mut [u16], theme: &LcdTheme) {
        for (i, pixel) in buf.iter_mut().enumerate().take(FRAMEBUFFER_SIZE) {
            let color = if self.pixels[i] { theme.fg } else { theme.bg };
            *pixel = color;
        }
    }

    /// Render to RGB565 buffer with specified orientation
    pub fn render_with_orientation(
        &self,
        buf: &mut [u16],
        theme: &LcdTheme,
        orientation: LcdOrientation,
    ) {
        match orientation {
            LcdOrientation::Portrait => self.render(buf, theme),
            LcdOrientation::Landscape => {
                // Rotate 90 degrees clockwise: pixel[x][y] -> pixel[y][LCD_WIDTH-1-x]
                for y in 0..LCD_HEIGHT {
                    for x in 0..LCD_WIDTH {
                        let src_idx = y * LCD_WIDTH + x;
                        let dst_idx = x * LCD_HEIGHT + (LCD_HEIGHT - 1 - y);
                        buf[dst_idx] = if self.pixels[src_idx] {
                            theme.fg
                        } else {
                            theme.bg
                        };
                    }
                }
            }
        }
    }

    /// Render with ghosting effect
    pub fn render_with_ghosting(
        &mut self,
        buf: &mut [u16],
        pixels: &[bool; FRAMEBUFFER_SIZE],
        theme: &LcdTheme,
    ) {
        let bg_r = ((theme.bg >> 11) & 0x1F) as i16;
        let bg_g = ((theme.bg >> 6) & 0x1F) as i16;
        let bg_b = (theme.bg & 0x1F) as i16;
        let fg_r = ((theme.fg >> 11) & 0x1F) as i16;
        let fg_g = ((theme.fg >> 6) & 0x1F) as i16;
        let fg_b = (theme.fg & 0x1F) as i16;

        for (i, pixel) in buf.iter_mut().enumerate().take(FRAMEBUFFER_SIZE) {
            // Update ghosting accumulator
            if pixels[i] {
                if self.ghosting[i] < theme.ghosting as i8 {
                    self.ghosting[i] += 1;
                }
            } else if self.ghosting[i] > 0 {
                self.ghosting[i] -= 1;
            }

            // Blend colors
            let a = self.ghosting[i] as f32 / theme.ghosting.max(1) as f32;
            let r = ((1.0 - a) * bg_r as f32 + a * fg_r as f32) as u16 & 0x1F;
            let g = ((1.0 - a) * bg_g as f32 + a * fg_g as f32) as u16 & 0x1F;
            let b = ((1.0 - a) * bg_b as f32 + a * fg_b as f32) as u16 & 0x1F;
            *pixel = (r << 11) | (g << 6) | b;
        }
    }

    /// Get pixel buffer reference
    pub fn pixels(&self) -> &[bool; FRAMEBUFFER_SIZE] {
        &self.pixels
    }

    /// Check if display has been updated
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear dirty flag
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

/// Bitmap font data
pub struct FontData {
    pub width: u8,
    pub height: u8,
    /// Glyph data, each glyph is width*height bits packed into bytes
    glyphs: Vec<u8>,
}

impl FontData {
    pub fn new(width: u8, height: u8) -> Self {
        Self {
            width,
            height,
            glyphs: vec![0; 256 * (width as usize * height as usize).div_ceil(8)],
        }
    }

    /// Get glyph bitmap for a character
    pub fn get_glyph(&self, ch: u8) -> Option<GlyphView<'_>> {
        let glyph_size = (self.width as usize * self.height as usize).div_ceil(8);
        let offset = ch as usize * glyph_size;
        if offset + glyph_size <= self.glyphs.len() {
            Some(GlyphView {
                data: &self.glyphs[offset..offset + glyph_size],
                width: self.width,
                height: self.height,
            })
        } else {
            None
        }
    }
}

/// View into a single glyph's bitmap data
pub struct GlyphView<'a> {
    data: &'a [u8],
    width: u8,
    height: u8,
}

impl<'a> GlyphView<'a> {
    /// Check if a pixel is set in the glyph
    pub fn get_pixel(&self, x: u8, y: u8) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let bit = y as usize * self.width as usize + x as usize;
        let byte = bit / 8;
        let bit = bit % 8;
        if byte < self.data.len() {
            (self.data[byte] >> (7 - bit)) & 1 != 0
        } else {
            false
        }
    }
}
