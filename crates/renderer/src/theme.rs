/// OneDark Pro inspired color palette
pub struct ModernTheme {
    // Background colors
    pub bg_primary: (u8, u8, u8),   // Main background
    pub bg_secondary: (u8, u8, u8), // Secondary elements
    pub bg_highlight: (u8, u8, u8), // Hover/selection

    // Foreground colors
    pub fg_primary: (u8, u8, u8),   // Main text
    pub fg_secondary: (u8, u8, u8), // Secondary text
    pub fg_muted: (u8, u8, u8),     // Muted text

    // Accent colors
    pub accent_blue: (u8, u8, u8),    // Links, info
    pub accent_cyan: (u8, u8, u8),    // Highlights
    pub accent_green: (u8, u8, u8),   // Success
    pub accent_yellow: (u8, u8, u8),  // Warnings
    pub accent_red: (u8, u8, u8),     // Errors
    pub accent_magenta: (u8, u8, u8), // Special
    pub accent_orange: (u8, u8, u8),  // Secondary accent

    // Terminal ANSI colors (16 colors)
    pub ansi_colors: [(u8, u8, u8); 16],
}

impl Default for ModernTheme {
    fn default() -> Self {
        Self {
            // Backgrounds - OneDark Pro style
            bg_primary: (40, 44, 52),   // #282c34 - Main background
            bg_secondary: (33, 37, 43), // #21252b - Darker
            bg_highlight: (57, 63, 74), // #393f4a - Selection/hover

            // Foreground - OneDark Pro text colors
            fg_primary: (171, 178, 191),   // #abb2bf - Main text
            fg_secondary: (145, 150, 161), // #9196a1 - Secondary text
            fg_muted: (92, 99, 112),       // #5c6370 - Comments/muted

            // Accents - OneDark Pro vibrant colors
            accent_blue: (97, 175, 239),     // #61afef - Blue
            accent_cyan: (86, 182, 194),     // #56b6c2 - Cyan
            accent_green: (152, 195, 121),   // #98c379 - Green
            accent_yellow: (229, 192, 123),  // #e5c07b - Yellow
            accent_red: (224, 108, 117),     // #e06c75 - Red
            accent_magenta: (198, 120, 221), // #c678dd - Purple/Magenta
            accent_orange: (209, 154, 102),  // #d19a66 - Orange

            // ANSI colors (0-15) - OneDark Pro palette
            ansi_colors: [
                // Dark variants (0-7)
                (40, 44, 52),    // 0: Black
                (224, 108, 117), // 1: Red
                (152, 195, 121), // 2: Green
                (229, 192, 123), // 3: Yellow
                (97, 175, 239),  // 4: Blue
                (198, 120, 221), // 5: Magenta
                (86, 182, 194),  // 6: Cyan
                (171, 178, 191), // 7: White
                // Bright variants (8-15)
                (92, 99, 112),   // 8: Bright Black (gray/comments)
                (240, 128, 137), // 9: Bright Red
                (172, 215, 141), // 10: Bright Green
                (245, 212, 143), // 11: Bright Yellow
                (117, 195, 255), // 12: Bright Blue
                (218, 140, 241), // 13: Bright Magenta
                (106, 202, 214), // 14: Bright Cyan
                (200, 207, 220), // 15: Bright White
            ],
        }
    }
}

impl ModernTheme {
    pub fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
        0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }

    pub fn bg_primary_u32(&self) -> u32 {
        Self::rgb_to_u32(self.bg_primary.0, self.bg_primary.1, self.bg_primary.2)
    }

    pub fn fg_primary_u32(&self) -> u32 {
        Self::rgb_to_u32(self.fg_primary.0, self.fg_primary.1, self.fg_primary.2)
    }

    pub fn get_ansi_color(&self, index: u8) -> u32 {
        let idx = (index as usize).min(15);
        let (r, g, b) = self.ansi_colors[idx];
        Self::rgb_to_u32(r, g, b)
    }

    pub fn accent_blue_u32(&self) -> u32 {
        Self::rgb_to_u32(self.accent_blue.0, self.accent_blue.1, self.accent_blue.2)
    }

    pub fn accent_cyan_u32(&self) -> u32 {
        Self::rgb_to_u32(self.accent_cyan.0, self.accent_cyan.1, self.accent_cyan.2)
    }

    pub fn accent_red_u32(&self) -> u32 {
        Self::rgb_to_u32(self.accent_red.0, self.accent_red.1, self.accent_red.2)
    }

    pub fn accent_yellow_u32(&self) -> u32 {
        Self::rgb_to_u32(
            self.accent_yellow.0,
            self.accent_yellow.1,
            self.accent_yellow.2,
        )
    }

    pub fn accent_green_u32(&self) -> u32 {
        Self::rgb_to_u32(
            self.accent_green.0,
            self.accent_green.1,
            self.accent_green.2,
        )
    }

    pub fn accent_magenta_u32(&self) -> u32 {
        Self::rgb_to_u32(
            self.accent_magenta.0,
            self.accent_magenta.1,
            self.accent_magenta.2,
        )
    }

    pub fn accent_orange_u32(&self) -> u32 {
        Self::rgb_to_u32(
            self.accent_orange.0,
            self.accent_orange.1,
            self.accent_orange.2,
        )
    }
}
