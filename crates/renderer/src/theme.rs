/// Modern color palette inspired by Tokyo Night and GitHub Dark
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
            // Backgrounds - Dark with subtle purple tint
            bg_primary: (26, 27, 38),   // #1a1b26
            bg_secondary: (30, 32, 44), // #1e202c
            bg_highlight: (42, 45, 60), // #2a2d3c

            // Foreground - Soft whites and grays
            fg_primary: (192, 202, 245),   // #c0caf5 - Bright white-blue
            fg_secondary: (169, 177, 214), // #a9b1d6 - Soft gray-blue
            fg_muted: (118, 124, 155),     // #767c9b - Muted gray

            // Accents - Vibrant but refined
            accent_blue: (122, 162, 247),    // #7aa2f7 - Bright blue
            accent_cyan: (125, 207, 255),    // #7dcfff - Cyan
            accent_green: (158, 206, 106),   // #9ece6a - Green
            accent_yellow: (224, 175, 104),  // #e0af68 - Yellow/orange
            accent_red: (247, 118, 142),     // #f7768e - Red/pink
            accent_magenta: (187, 154, 247), // #bb9af7 - Purple
            accent_orange: (255, 158, 100),  // #ff9e64 - Orange

            // ANSI colors (0-15)
            ansi_colors: [
                // Dark variants (0-7)
                (26, 27, 38),    // 0: Black
                (247, 118, 142), // 1: Red
                (158, 206, 106), // 2: Green
                (224, 175, 104), // 3: Yellow
                (122, 162, 247), // 4: Blue
                (187, 154, 247), // 5: Magenta
                (125, 207, 255), // 6: Cyan
                (192, 202, 245), // 7: White
                // Bright variants (8-15)
                (86, 95, 137),   // 8: Bright Black (gray)
                (255, 135, 157), // 9: Bright Red
                (182, 227, 133), // 10: Bright Green
                (241, 200, 135), // 11: Bright Yellow
                (142, 182, 255), // 12: Bright Blue
                (207, 184, 255), // 13: Bright Magenta
                (154, 227, 255), // 14: Bright Cyan
                (220, 228, 255), // 15: Bright White
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
}
