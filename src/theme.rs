use crossterm::style::Color;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Light vs dark Herdr appearance used to pick picker contrast colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Appearance {
    Dark,
    Light,
}

/// Terminal colors for unmatched text, matched tokens, and hint badges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerPalette {
    pub unmatched_fg: Color,
    pub dim_unmatched: bool,
    pub match_fg: Color,
    pub hint_fg: Color,
    pub hint_bg: Color,
}

impl PickerPalette {
    /// Loads colors from the current Herdr theme, defaulting to dark ANSI styling.
    pub fn load() -> Self {
        Self::for_appearance(load_appearance())
    }

    /// Returns high-contrast picker colors for a known appearance.
    ///
    /// Dark keeps the original ANSI Black-on-Cyan hints. Light uses truecolor
    /// because Catppuccin Latte remaps those ANSI slots to similar luminances.
    pub fn for_appearance(appearance: Appearance) -> Self {
        match appearance {
            Appearance::Dark => Self {
                unmatched_fg: Color::DarkGrey,
                dim_unmatched: true,
                match_fg: Color::Yellow,
                hint_fg: Color::Black,
                hint_bg: Color::Cyan,
            },
            Appearance::Light => Self {
                unmatched_fg: rgb(108, 111, 133),
                dim_unmatched: false,
                match_fg: rgb(30, 102, 245),
                hint_fg: rgb(255, 255, 255),
                hint_bg: rgb(30, 102, 245),
            },
        }
    }
}

fn load_appearance() -> Appearance {
    let Ok(contents) = fs::read_to_string(herdr_config_path()) else {
        return Appearance::Dark;
    };
    let Ok(value) = contents.parse::<toml::Value>() else {
        return Appearance::Dark;
    };
    appearance_from_herdr_config(&value)
}

fn appearance_from_herdr_config(value: &toml::Value) -> Appearance {
    let name = value
        .get("theme")
        .and_then(|theme| theme.get("name"))
        .and_then(|name| name.as_str())
        .unwrap_or("");
    if is_light_theme_name(name) {
        Appearance::Light
    } else {
        Appearance::Dark
    }
}

fn is_light_theme_name(name: &str) -> bool {
    let normalized = name
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "catppuccinlatte"
            | "latte"
            | "light"
            | "tokyonightday"
            | "tokyoday"
            | "gruvboxlight"
            | "onelight"
            | "solarizedlight"
            | "kanagawalotus"
            | "lotus"
            | "rosepinedawn"
            | "dawn"
    ) || normalized.contains("light")
        || normalized.contains("latte")
        || normalized.contains("dawn")
        || normalized.contains("lotus")
        || normalized.ends_with("day")
}

fn herdr_config_path() -> PathBuf {
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        return Path::new(&xdg).join("herdr/config.toml");
    }
    home_dir().join(".config/herdr/config.toml")
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_herdr_theme_names_select_light_appearance() {
        for name in [
            "catppuccin-latte",
            "tokyo-night-day",
            "gruvbox-light",
            "one-light",
            "solarized-light",
            "kanagawa-lotus",
            "rose-pine-dawn",
        ] {
            let value = format!("[theme]\nname = \"{name}\"")
                .parse::<toml::Value>()
                .unwrap();
            assert_eq!(
                appearance_from_herdr_config(&value),
                Appearance::Light,
                "{name}"
            );
        }
    }

    #[test]
    fn dark_herdr_theme_names_select_dark_appearance() {
        for name in ["catppuccin", "tokyo-night", "dracula", ""] {
            let value = if name.is_empty() {
                "[theme]\n".parse::<toml::Value>().unwrap()
            } else {
                format!("[theme]\nname = \"{name}\"")
                    .parse::<toml::Value>()
                    .unwrap()
            };
            assert_eq!(
                appearance_from_herdr_config(&value),
                Appearance::Dark,
                "{name}"
            );
        }
    }

    #[test]
    fn light_palette_uses_truecolor_hint_badge() {
        let palette = PickerPalette::for_appearance(Appearance::Light);
        assert_eq!(palette.hint_fg, rgb(255, 255, 255));
        assert_eq!(palette.hint_bg, rgb(30, 102, 245));
        assert_eq!(palette.match_fg, rgb(30, 102, 245));
        assert!(!palette.dim_unmatched);
    }
}
