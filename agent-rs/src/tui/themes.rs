use ratatui::style::Color;

use super::TuiEvent;
use super::state::ThemeName;

#[derive(Debug, Clone, Copy)]
pub struct ThemeColorSet {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub border: Color,
    pub border_focused: Color,
    pub title: Color,
    pub subtitle: Color,
    pub user_message: Color,
    pub assistant_message: Color,
    pub system_message: Color,
    pub tool_message: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub info: Color,
    pub ghost_text: Color,
    pub selection: Color,
    pub scrollbar: Color,
    pub user_message_bg: Color,
    pub assistant_message_bg: Color,
    pub tool_running: Color,
    pub tool_completed: Color,
    pub tool_failed: Color,
}

impl ThemeName {
    pub fn colors(&self) -> ThemeColorSet {
        if std::env::var("NO_COLOR").is_ok() {
            return ThemeColorSet::monochrome();
        }
        match self {
            ThemeName::Dark => ThemeColorSet::dark(),
            ThemeName::Light => ThemeColorSet::light(),
            ThemeName::Nord => ThemeColorSet::nord(),
            ThemeName::Gruvbox => ThemeColorSet::gruvbox(),
            ThemeName::Custom => load_custom_theme().unwrap_or_else(ThemeColorSet::dark),
        }
    }
}

impl ThemeColorSet {
    fn monochrome() -> Self {
        Self {
            background: Color::Reset,
            foreground: Color::Reset,
            primary: Color::Reset,
            secondary: Color::Reset,
            accent: Color::Reset,
            border: Color::Reset,
            border_focused: Color::Reset,
            title: Color::Reset,
            subtitle: Color::Reset,
            user_message: Color::Reset,
            assistant_message: Color::Reset,
            system_message: Color::Reset,
            tool_message: Color::Reset,
            error: Color::Reset,
            success: Color::Reset,
            warning: Color::Reset,
            info: Color::Reset,
            ghost_text: Color::Reset,
            selection: Color::Reset,
            scrollbar: Color::Reset,
            user_message_bg: Color::Reset,
            assistant_message_bg: Color::Reset,
            tool_running: Color::Reset,
            tool_completed: Color::Reset,
            tool_failed: Color::Reset,
        }
    }

    fn dark() -> Self {
        Self {
            background: Color::Rgb(10, 14, 20),    // #0A0E14
            foreground: Color::Rgb(212, 212, 212), // #D4D4D4
            primary: Color::Rgb(97, 175, 239),
            secondary: Color::Rgb(110, 118, 129),
            accent: Color::Rgb(0, 255, 204), // #00FFCC
            border: Color::Rgb(80, 80, 80),
            border_focused: Color::Rgb(0, 255, 204),
            title: Color::Rgb(0, 255, 204),
            subtitle: Color::Rgb(106, 111, 120), // #6A6F78
            user_message: Color::Rgb(255, 255, 255),
            assistant_message: Color::Rgb(212, 212, 212),
            system_message: Color::Rgb(106, 111, 120),
            tool_message: Color::Yellow,
            error: Color::Rgb(255, 85, 85),   // #FF5555
            success: Color::Rgb(85, 255, 85), // #55FF55
            warning: Color::Rgb(255, 165, 0), // #FFA500
            info: Color::Cyan,
            ghost_text: Color::Rgb(106, 111, 120),
            selection: Color::Rgb(60, 60, 60),
            scrollbar: Color::Gray,
            user_message_bg: Color::Rgb(42, 47, 58), // #2A2F3A
            assistant_message_bg: Color::Rgb(26, 30, 38), // #1A1E26
            tool_running: Color::Rgb(255, 215, 0),   // #FFD700
            tool_completed: Color::Rgb(85, 255, 85),
            tool_failed: Color::Rgb(255, 85, 85),
        }
    }

    fn light() -> Self {
        Self {
            background: Color::Rgb(250, 250, 250),
            foreground: Color::Rgb(51, 51, 51),
            primary: Color::Rgb(0, 97, 175),
            secondary: Color::Rgb(100, 100, 100),
            accent: Color::Rgb(0, 140, 110),
            border: Color::Rgb(200, 200, 200),
            border_focused: Color::Rgb(0, 97, 175),
            title: Color::Rgb(0, 97, 175),
            subtitle: Color::Rgb(120, 120, 120),
            user_message: Color::Rgb(0, 120, 0),
            assistant_message: Color::Rgb(0, 70, 150),
            system_message: Color::Rgb(90, 90, 90),
            tool_message: Color::Rgb(130, 95, 0),
            error: Color::Rgb(170, 0, 0),
            success: Color::Rgb(0, 120, 0),
            warning: Color::Rgb(170, 110, 0),
            info: Color::Rgb(0, 110, 140),
            ghost_text: Color::Rgb(150, 150, 150),
            selection: Color::Rgb(225, 235, 245),
            scrollbar: Color::Gray,
            user_message_bg: Color::Rgb(235, 245, 250),
            assistant_message_bg: Color::Rgb(245, 245, 245),
            tool_running: Color::Rgb(180, 140, 0),
            tool_completed: Color::Rgb(0, 120, 0),
            tool_failed: Color::Rgb(170, 0, 0),
        }
    }

    fn nord() -> Self {
        Self {
            background: Color::Rgb(46, 52, 64),
            foreground: Color::Rgb(216, 222, 233),
            primary: Color::Rgb(136, 192, 208),
            secondary: Color::Rgb(129, 161, 193),
            accent: Color::Rgb(180, 142, 173),
            border: Color::Rgb(76, 86, 106),
            border_focused: Color::Rgb(143, 188, 187),
            title: Color::Rgb(143, 188, 187),
            subtitle: Color::Rgb(129, 161, 193),
            user_message: Color::Rgb(163, 190, 140),
            assistant_message: Color::Rgb(136, 192, 208),
            system_message: Color::Rgb(129, 161, 193),
            tool_message: Color::Rgb(235, 203, 139),
            error: Color::Rgb(191, 97, 106),
            success: Color::Rgb(163, 190, 140),
            warning: Color::Rgb(235, 203, 139),
            info: Color::Rgb(136, 192, 208),
            ghost_text: Color::Rgb(94, 129, 172),
            selection: Color::Rgb(67, 76, 94),
            scrollbar: Color::Rgb(94, 129, 172),
            user_message_bg: Color::Rgb(59, 66, 82),
            assistant_message_bg: Color::Rgb(46, 52, 64),
            tool_running: Color::Rgb(235, 203, 139),
            tool_completed: Color::Rgb(163, 190, 140),
            tool_failed: Color::Rgb(191, 97, 106),
        }
    }

    fn gruvbox() -> Self {
        Self {
            background: Color::Rgb(40, 40, 40),
            foreground: Color::Rgb(235, 219, 178),
            primary: Color::Rgb(184, 187, 38),
            secondary: Color::Rgb(131, 165, 152),
            accent: Color::Rgb(211, 134, 155),
            border: Color::Rgb(102, 92, 84),
            border_focused: Color::Rgb(250, 189, 47),
            title: Color::Rgb(250, 189, 47),
            subtitle: Color::Rgb(168, 153, 132),
            user_message: Color::Rgb(184, 187, 38),
            assistant_message: Color::Rgb(131, 165, 152),
            system_message: Color::Rgb(168, 153, 132),
            tool_message: Color::Rgb(250, 189, 47),
            error: Color::Rgb(251, 73, 52),
            success: Color::Rgb(184, 187, 38),
            warning: Color::Rgb(250, 189, 47),
            info: Color::Rgb(131, 165, 152),
            ghost_text: Color::Rgb(146, 131, 116),
            selection: Color::Rgb(60, 56, 54),
            scrollbar: Color::Rgb(124, 111, 100),
            user_message_bg: Color::Rgb(50, 48, 47),
            assistant_message_bg: Color::Rgb(40, 40, 40),
            tool_running: Color::Rgb(250, 189, 47),
            tool_completed: Color::Rgb(184, 187, 38),
            tool_failed: Color::Rgb(251, 73, 52),
        }
    }

    /// Try to load custom overrides from a TOML-like config file.
    /// Parses only the supported keys: user_message_bg, assistant_message_bg, accent, error.
    pub fn apply_overrides_from_file(&mut self, path: &str) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let mut in_colors_section = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "[colors]" {
                in_colors_section = true;
                continue;
            }
            if trimmed.starts_with('[') {
                in_colors_section = false;
                continue;
            }
            if !in_colors_section {
                continue;
            }
            if let Some((key, val)) = trimmed.split_once('=') {
                let key = key.trim();
                let val = val.trim().trim_matches('"');
                if let Some(color) = parse_hex_color(val) {
                    match key {
                        "user_message_bg" => self.user_message_bg = color,
                        "assistant_message_bg" => self.assistant_message_bg = color,
                        "accent" => self.accent = color,
                        "error" => self.error = color,
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Parse a #RRGGBB hex color string.
fn parse_hex_color(s: &str) -> Option<Color> {
    let hex = s.trim().strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeManager {
    pub current: ThemeName,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            current: ThemeName::Dark,
        }
    }

    pub fn switch_to(&mut self, theme: ThemeName) -> TuiEvent {
        self.current = theme;
        TuiEvent::ThemeChanged(theme)
    }

    pub fn next(&mut self) -> TuiEvent {
        self.switch_to(self.current.next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_name_next_cycles_all_presets() {
        assert_eq!(ThemeName::Dark.next(), ThemeName::Light);
        assert_eq!(ThemeName::Light.next(), ThemeName::Nord);
        assert_eq!(ThemeName::Nord.next(), ThemeName::Gruvbox);
        assert_eq!(ThemeName::Gruvbox.next(), ThemeName::Custom);
        assert_eq!(ThemeName::Custom.next(), ThemeName::Dark);
    }

    #[test]
    fn theme_manager_next_emits_theme_changed_event() {
        let mut manager = ThemeManager::new();
        let event = manager.next();
        assert!(matches!(event, TuiEvent::ThemeChanged(ThemeName::Light)));
    }

    #[test]
    fn dark_and_light_palettes_differ() {
        let dark = ThemeColorSet::dark();
        let light = ThemeColorSet::light();
        assert_ne!(dark.background, light.background);
        assert_ne!(dark.foreground, light.foreground);
    }

    #[test]
    fn monochrome_returns_reset_colors() {
        let mono = ThemeColorSet::monochrome();
        assert_eq!(mono.background, Color::Reset);
        assert_eq!(mono.accent, Color::Reset);
    }

    #[test]
    fn parse_hex_color_valid() {
        assert_eq!(parse_hex_color("#FF5555"), Some(Color::Rgb(255, 85, 85)));
        assert_eq!(parse_hex_color("#000000"), Some(Color::Rgb(0, 0, 0)));
    }

    #[test]
    fn parse_hex_color_invalid() {
        assert_eq!(parse_hex_color("invalid"), None);
        assert_eq!(parse_hex_color("#ZZZ"), None);
        assert_eq!(parse_hex_color(""), None);
    }
}

pub fn load_custom_theme() -> Option<ThemeColorSet> {
    let home = std::env::var("HOME").ok()?;
    let path = format!("{}/.config/helix-agent/theme.toml", home);
    let content = std::fs::read_to_string(path).ok()?;

    let mut colors = ThemeColorSet::dark();
    let mut in_colors = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_colors = line == "[colors]";
            continue;
        }
        if !in_colors || line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, val)) = line.split_once('=') {
            let key = key.trim();
            let val = val.trim().trim_matches('"').trim_matches('\'');
            if let Some(col) = parse_hex_color(val) {
                match key {
                    "background" => colors.background = col,
                    "foreground" => colors.foreground = col,
                    "primary" => colors.primary = col,
                    "secondary" => colors.secondary = col,
                    "accent" => colors.accent = col,
                    "border" => colors.border = col,
                    "border_focused" => colors.border_focused = col,
                    "title" => colors.title = col,
                    "subtitle" => colors.subtitle = col,
                    "user_message" => colors.user_message = col,
                    "assistant_message" => colors.assistant_message = col,
                    "system_message" => colors.system_message = col,
                    "tool_message" => colors.tool_message = col,
                    "error" => colors.error = col,
                    "success" => colors.success = col,
                    "warning" => colors.warning = col,
                    "info" => colors.info = col,
                    "ghost_text" => colors.ghost_text = col,
                    "selection" => colors.selection = col,
                    "scrollbar" => colors.scrollbar = col,
                    _ => {}
                }
            }
        }
    }
    Some(colors)
}
