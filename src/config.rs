use anyhow::{Context, Result};

pub mod ui;

pub fn load_app_config() -> Result<AppConfig> {
    let config_path = directories::ProjectDirs::from("", "", "pik")
        .map(|dirs| dirs.config_dir().join("config.toml"))
        .filter(|path| path.exists());

    match config_path {
        Some(path) => load_config_from_file(&path),
        None => Ok(AppConfig::default()),
    }
}

fn load_config_from_file(path: &std::path::PathBuf) -> Result<AppConfig> {
    let raw_toml = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to load config from file: {:?}", path))?;
    toml::from_str(&raw_toml)
        .map(|mut config: AppConfig| {
            if config.ui.use_icons.is_some() {
                println!("### WARNING ####");
                println!("ui.use_icons is deprecated and will be removed in future. Please use ui.icons instead");
                if !matches!(config.ui.icons, IconConfig::Custom(_)) {
                    config.ui.icons = IconConfig::NerdFontV3;
                }
            }
            config
        })
        .with_context(|| format!("Failed to deserialize config from file: {:?}", path))
}

use regex::Regex;
use serde::Deserialize;
use ui::{IconConfig, UIConfig};

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub screen_size: ScreenSize,
    #[serde(default)]
    pub ignore: IgnoreConfig,
    #[serde(default)]
    pub ui: UIConfig,
}

#[derive(Debug, Deserialize)]
pub struct IgnoreConfig {
    #[serde(with = "serde_regex", default)]
    pub paths: Vec<Regex>,
    #[serde(default = "set_true")]
    pub other_users: bool,
    #[serde(default = "set_true")]
    pub threads: bool,
}

const fn set_true() -> bool {
    true
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            paths: vec![],
            other_users: set_true(),
            threads: set_true(),
        }
    }
}

impl PartialEq for IgnoreConfig {
    fn eq(&self, other: &Self) -> bool {
        let mut eq = self.threads == other.threads
            && self.other_users == other.other_users
            && self.paths.len() == other.paths.len();
        if eq {
            eq = self.paths.iter().map(|r| r.as_str()).collect::<Vec<&str>>()
                == other
                    .paths
                    .iter()
                    .map(|r| r.as_str())
                    .collect::<Vec<&str>>()
        }
        eq
    }
}

impl Eq for IgnoreConfig {}

#[derive(Debug, Eq, PartialEq, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ScreenSize {
    Fullscreen,
    Height(u16),
}

pub const DEFAULT_SCREEN_SIZE: u16 = 25;

impl Default for ScreenSize {
    fn default() -> Self {
        ScreenSize::Height(DEFAULT_SCREEN_SIZE)
    }
}

#[cfg(test)]
mod tests {

    use ratatui::{
        layout::{Alignment, Margin},
        style::{Color, Modifier, Style, Stylize, palette::tailwind},
        widgets::{BorderType, block::Position},
    };
    use ui::{
        BorderTheme, CellTheme, ProcessDetailsTheme, RowTheme, ScrollbarTheme, SearchBarTheme,
        TableTheme, TitleTheme,
    };

    use super::*;

    #[test]
    fn should_deserialize_empty_configuration() {
        let default_settings = toml::from_str("");
        assert_eq!(default_settings, Ok(AppConfig::default()));
        // ensure what actual defaults are
        assert_eq!(
            default_settings,
            Ok(AppConfig {
                screen_size: ScreenSize::Height(DEFAULT_SCREEN_SIZE),
                ignore: IgnoreConfig {
                    paths: vec![],
                    other_users: true,
                    threads: true
                },
                ui: UIConfig {
                    use_icons: None,
                    icons: ui::IconConfig::Ascii,
                    process_table: TableTheme {
                        title: TitleTheme {
                            alignment: Alignment::Left,
                            position: Position::Top
                        },
                        border: BorderTheme {
                            style: Style::default().fg(tailwind::BLUE.c400),
                            _type: BorderType::Rounded
                        },
                        row: RowTheme {
                            even: Style::new()
                                .bg(tailwind::SLATE.c950)
                                .fg(tailwind::SLATE.c200),
                            odd: Style::new()
                                .bg(tailwind::SLATE.c900)
                                .fg(tailwind::SLATE.c200),
                            selected: Style::new()
                                .fg(tailwind::BLUE.c400)
                                .add_modifier(Modifier::REVERSED),
                            selected_symbol: " ".to_string(),
                        },
                        cell: CellTheme {
                            normal: Style::default(),
                            highlighted: Style::new().bg(Color::Yellow).italic(),
                        },
                        scrollbar: ScrollbarTheme {
                            style: Style::default(),
                            thumb_symbol: None,
                            track_symbol: Some("│".to_string()),
                            begin_symbol: Some("↑".to_string()),
                            end_symbol: Some("↓".to_string()),
                            margin: Margin {
                                vertical: 1,
                                horizontal: 0,
                            },
                        }
                    },
                    process_details: ProcessDetailsTheme {
                        title: TitleTheme {
                            alignment: Alignment::Left,
                            position: Position::Top
                        },
                        border: BorderTheme {
                            style: Style::default().fg(tailwind::BLUE.c400),
                            _type: BorderType::Rounded
                        },
                        scrollbar: ScrollbarTheme {
                            style: Style::default(),
                            thumb_symbol: None,
                            track_symbol: Some("│".to_string()),
                            begin_symbol: Some("↑".to_string()),
                            end_symbol: Some("↓".to_string()),
                            margin: Margin {
                                vertical: 1,
                                horizontal: 0
                            }
                        }
                    },
                    search_bar: SearchBarTheme {
                        style: Style::default(),
                        cursor_style: Style::default().add_modifier(Modifier::REVERSED)
                    }
                }
            })
        );
    }

    #[test]
    fn should_allow_to_override_defaults() {
        let overrided_settings: AppConfig = toml::from_str(
            r##"
            screen_size = "fullscreen"

            [ignore]
            paths=["/usr/*"]
            other_users = false
            threads = false

            [ui]
            use_icons = true
            icons = "nerd_font_v3"

            [ui.process_table.title]
            alignment = "right"
            position = "bottom"

            [ui.process_table.border]
            type = "plain"
            style = {fg = "#6366f1", add_modifier = "BOLD | ITALIC"}

            [ui.process_table.row]
            selected_symbol = ">"
            even = {fg = "#fafaf9", bg = "#57534e", add_modifier = "BOLD"}
            odd = {fg = "#ecfdf5", bg = "#059669", add_modifier = "ITALIC"}
            selected = {fg = "#f87171"}

            [ui.process_table.cell]
            normal = {fg = "#a5f3fc", bg = "#0891b2", add_modifier = "CROSSED_OUT"}
            highlighted = {fg = "#fff7ed", bg = "#fb923c", add_modifier = "UNDERLINED"}

            [ui.process_table.scrollbar]
            style = {fg = "#f472b6", bg = "#4c1d95", add_modifier = "BOLD"}
            thumb_symbol = "x"
            track_symbol = "y"
            begin_symbol = "z"
            end_symbol = "q"
            margin = {horizontal = 10, vertical = 20}

            [ui.process_details.title]
            alignment = "center"
            position = "bottom"

            [ui.process_details.border]
            type = "double"
            style = {fg = "#6366f1", add_modifier = "UNDERLINED | ITALIC"}

            [ui.process_details.scrollbar]
            style = {fg = "#f472b6", bg = "#4c1d95", add_modifier = "BOLD"}
            thumb_symbol = "T"
            track_symbol = "="
            begin_symbol = "^"
            end_symbol = "v"
            margin = {horizontal = 2, vertical = 3}

            [ui.search_bar]
            style = {fg = "#6366f1", add_modifier = "UNDERLINED | ITALIC"}
            cursor_style = {fg = "#a5f3fc", bg = "#0891b2", add_modifier = "CROSSED_OUT"}
            "##,
        )
        .unwrap();
        assert_eq!(
            overrided_settings,
            AppConfig {
                screen_size: ScreenSize::Fullscreen,
                ignore: IgnoreConfig {
                    paths: vec![Regex::new("/usr/*").unwrap()],
                    other_users: false,
                    threads: false
                },
                ui: UIConfig {
                    use_icons: Some(true),
                    icons: ui::IconConfig::NerdFontV3,
                    process_table: TableTheme {
                        title: TitleTheme {
                            alignment: Alignment::Right,
                            position: Position::Bottom
                        },
                        border: BorderTheme {
                            style: Style::default().fg(tailwind::INDIGO.c500).bold().italic(),
                            _type: BorderType::Plain
                        },
                        row: RowTheme {
                            even: Style::new()
                                .fg(tailwind::STONE.c50)
                                .bg(tailwind::STONE.c600)
                                .bold(),
                            odd: Style::new()
                                .fg(tailwind::EMERALD.c50)
                                .bg(tailwind::EMERALD.c600)
                                .italic(),
                            selected: Style::new().fg(tailwind::RED.c400),
                            selected_symbol: ">".to_string(),
                        },
                        cell: CellTheme {
                            normal: Style::new()
                                .fg(tailwind::CYAN.c200)
                                .bg(tailwind::CYAN.c600)
                                .crossed_out(),
                            highlighted: Style::new()
                                .fg(tailwind::ORANGE.c50)
                                .bg(tailwind::ORANGE.c400)
                                .underlined(),
                        },
                        scrollbar: ScrollbarTheme {
                            style: Style::new()
                                .fg(tailwind::PINK.c400)
                                .bg(tailwind::VIOLET.c900)
                                .bold(),
                            thumb_symbol: Some("x".to_string()),
                            track_symbol: Some("y".to_string()),
                            begin_symbol: Some("z".to_string()),
                            end_symbol: Some("q".to_string()),
                            margin: Margin {
                                vertical: 20,
                                horizontal: 10
                            }
                        }
                    },
                    process_details: ProcessDetailsTheme {
                        title: TitleTheme {
                            alignment: Alignment::Center,
                            position: Position::Bottom
                        },
                        border: BorderTheme {
                            style: Style::default()
                                .fg(tailwind::INDIGO.c500)
                                .italic()
                                .underlined(),
                            _type: BorderType::Double
                        },
                        scrollbar: ScrollbarTheme {
                            style: Style::default()
                                .fg(tailwind::PINK.c400)
                                .bg(tailwind::VIOLET.c900)
                                .bold(),
                            thumb_symbol: Some("T".to_string()),
                            track_symbol: Some("=".to_string()),
                            begin_symbol: Some("^".to_string()),
                            end_symbol: Some("v".to_string()),
                            margin: Margin {
                                horizontal: 2,
                                vertical: 3
                            }
                        }
                    },
                    search_bar: SearchBarTheme {
                        style: Style::default()
                            .fg(tailwind::INDIGO.c500)
                            .italic()
                            .underlined(),
                        cursor_style: Style::default()
                            .fg(tailwind::CYAN.c200)
                            .bg(tailwind::CYAN.c600)
                            .crossed_out(),
                    }
                }
            }
        );
    }
}
