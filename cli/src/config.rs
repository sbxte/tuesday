mod defaults;

use defaults::*;

use std::fs::OpenOptions;
use std::io::Read;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::display::Color;
use crate::AppResult;

pub type ConfigParseResult<T> = Result<T, ConfigReadError>;

/// The default config file name
pub const DEFAULT_FILENAME: &str = ".tueconf.toml";

/// Represents an error during reading a config file
#[derive(Debug, Error)]
pub enum ConfigReadError {
    #[error("File does not exist: {0}")]
    NonexistantFile(PathBuf),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("TOML Deserialization error: {0}")]
    TOMLDeserializeErr(#[from] toml::de::Error),

    #[error("Color parse error: {0}")]
    ColorParseErr(String),
}

pub struct GraphConfig {
    pub(crate) auto_clean: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            auto_clean: DEFAULT_GRAPH_AUTO_CLEAN
        }
    }
}

pub struct Icon {
    pub(crate) value: String,
    pub(crate) color: Color
}

impl Icon {
    pub fn colorize<T: Into<Color>>(mut self, color: T) -> Self {
        self.color = color.into();
        self
    }
}

impl From<&str> for Icon {
    fn from(value: &str) -> Self {
        Icon {
            value: value.to_string(),
            color: Color::default()
        }
    }
}

pub struct DisplayIconConfig {
    pub(crate) arm: Icon,
    pub(crate) arm_last: Icon,
    pub(crate) arm_multiparent: Icon,
    pub(crate) arm_multiparent_last: Icon,
    pub(crate) arm_bar: Icon,

    pub(crate) node_none: Icon,
    pub(crate) node_checked: Icon,
    pub(crate) node_partial: Icon,
    pub(crate) node_pseudo: Icon,
    pub(crate) node_date: Icon,
}

impl Default for DisplayIconConfig {
    fn default() -> Self {
        Self {
            arm: Icon::from(DEFAULT_ICON_ARM).colorize(DEFAULT_COLOR_ARM),
            arm_last: Icon::from(DEFAULT_ICON_ARM_LAST).colorize(DEFAULT_COLOR_ARM_LAST),
            arm_multiparent: Icon::from(DEFAULT_ICON_ARM_MULTIPARENT).colorize(DEFAULT_COLOR_ARM_MULTIPARENT),
            arm_multiparent_last: Icon::from(DEFAULT_ICON_ARM_MULTIPARENT_LAST).colorize(DEFAULT_COLOR_ARM_MULTIPARENT_LAST),
            arm_bar: Icon::from(DEFAULT_ICON_ARM_BAR).colorize(DEFAULT_COLOR_ARM_BAR),

            node_none: Icon::from(DEFAULT_ICON_NODE_NONE).colorize(DEFAULT_COLOR_NODE_NONE),
            node_checked: Icon::from(DEFAULT_ICON_NODE_CHECKED).colorize(DEFAULT_COLOR_NODE_CHECKED),
            node_partial: Icon::from(DEFAULT_ICON_NODE_PARTIAL).colorize(DEFAULT_COLOR_NODE_PARTIAL),
            node_pseudo: Icon::from(DEFAULT_ICON_NODE_PSEUDO).colorize(DEFAULT_COLOR_NODE_PSEUDO),
            node_date: Icon::from(DEFAULT_ICON_NODE_DATE).colorize(DEFAULT_COLOR_NODE_DATE),
        }
    }

}

pub struct CalendarConfig {
    pub(crate) heatmap_palette: [Color; 5]
}

impl Default for CalendarConfig {
    fn default() -> Self {
        Self {
            heatmap_palette: DEFAULT_HEATMAP_PALETTE
        }
    }
}

pub struct DisplayConfig {
    pub(crate) date_fmt: String,
    pub(crate) show_connections: bool,
    pub(crate) icons: DisplayIconConfig,
    pub(crate) calendar_config: CalendarConfig
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_connections: DEFAULT_SHOW_CONNECTIONS,
            date_fmt: DEFAULT_DATE_FORMAT.to_string(),
            calendar_config: CalendarConfig::default(),
            icons: DisplayIconConfig::default()
        }
    }
}

#[derive(Default)]
pub struct CliConfig {
    pub(crate) graph: GraphConfig,
    pub(crate) display: DisplayConfig,
}

impl CliConfig {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

/// Returns the default config file located at the user's home directory
/// If the file does not exist then it returns `None`
pub fn get_home_default() -> Option<PathBuf> {
    home::home_dir()
        .map(|mut pathbuf| {
            pathbuf.push(DEFAULT_FILENAME);
            pathbuf
        })
        .filter(|pb| pb.exists() && pb.is_file())
}

/// Returns the default config file at a given directory path
///
/// - If a/b/c is a directory, a/b/c/[`DEFAULT_FILENAME`]
/// - If none found so far, returns [`get_home_default`]
pub fn get_default_at(mut pathbuf: PathBuf) -> Option<PathBuf> {
    if pathbuf.exists() && pathbuf.is_dir() {
        pathbuf.push(DEFAULT_FILENAME);
        if pathbuf.exists() && pathbuf.is_file() {
            return Some(pathbuf);
        } else {
            return None;
        }
    }

    get_home_default()
}

/// Parses a file at path into a toml table
pub fn read_file(path: &Path) -> ConfigParseResult<toml::Table> {
    if !path.exists() {
        return Err(ConfigReadError::NonexistantFile(path.to_path_buf()));
    }

    let mut file = OpenOptions::new().read(true).open(path)?;

    let mut string = String::new();
    file.read_to_string(&mut string)?;

    Ok(string.parse::<toml::Table>()?)
}

const KEY_GRAPH: &str = "graph";
const KEY_DISPLAY: &str = "display";
const KEY_AUTO_CLEAN: &str = "auto_clean";
const KEY_DATE_FMT: &str = "date_fmt";
const KEY_SHOW_CONNECTIONS: &str = "show_connections";
const KEY_DISPLAY_ICONS: &str = "icons";
const KEY_DISPLAY_ICON: &str = "icon";
const KEY_DISPLAY_COLOR: &str = "color";
const KEY_DISPLAY_ICONS_ARM: &str = "arm";
const KEY_DISPLAY_ICONS_ARM_LAST: &str = "arm_last";
const KEY_DISPLAY_ICONS_ARM_MULTIPARENT: &str = "arm_multiparent";
const KEY_DISPLAY_ICONS_ARM_MULTIPARENT_LAST: &str = "arm_multiparent_last";
const KEY_DISPLAY_ICONS_ARM_BAR: &str = "arm_bar";
const KEY_DISPLAY_ICONS_NODE_NONE: &str = "node_none";
const KEY_DISPLAY_ICONS_NODE_CHECKED: &str = "node_checked";
const KEY_DISPLAY_ICONS_NODE_PARTIAL: &str = "node_partial";
const KEY_DISPLAY_ICONS_NODE_PSEUDO: &str = "node_pseudo";
const KEY_DISPLAY_ICONS_NODE_DATE: &str = "node_date";
const KEY_DISPLAY_CALENDAR: &str = "calendar";
const KEY_DISPLAY_CALENDAR_HEATMAP: &str = "heatmap";
const KEY_DISPLAY_CALENDAR_HEATMAP_PALETTE: &str = "palette";

/// Parses core configurations from a toml table
/// Any missing or malformed values will be replaced with defaults.
pub fn parse_config(toml: &toml::Table) -> ConfigParseResult<CliConfig> {
    let mut conf = CliConfig::new();

    // TODO: um wtf

    // Graph configuration
    if let Some(graph_cfg) = toml.get(KEY_GRAPH) {
        if let Some(val) = graph_cfg.get(KEY_AUTO_CLEAN).and_then(toml::Value::as_bool) {
            conf.graph.auto_clean = val;
        }
    }

    // Display configuration
    if let Some(display_cfg) = toml.get(KEY_DISPLAY) {
        if let Some(val) = display_cfg.get(KEY_DATE_FMT).and_then(toml::Value::as_str) {
            conf.display.date_fmt = val.to_string();
        }

        if let Some(val) = display_cfg.get(KEY_SHOW_CONNECTIONS).and_then(toml::Value::as_bool) {
            conf.display.show_connections = val;
        }

        // Icons configuration
        if let Some(icons) = display_cfg.get(KEY_DISPLAY_ICONS) {
            if let Some(arm) = icons.get(KEY_DISPLAY_ICONS_ARM) {
                if let Some(val) = arm.get(KEY_DISPLAY_ICON).and_then(toml::Value::as_str) {
                    conf.display.icons.arm.value = val.to_string();
                }

                if let Some(val) = arm.get(KEY_DISPLAY_COLOR).and_then(toml::Value::as_str) {
                    conf.display.icons.arm.color = Color::from_hex(val)
                        .map_err(|_| ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {}{}{}: {}",
                            KEY_DISPLAY, KEY_DISPLAY_ICONS, KEY_DISPLAY_ICONS_ARM, val
                        )))?;
                }
            }

            if let Some(arm_last) = icons.get(KEY_DISPLAY_ICONS_ARM_LAST) {
                if let Some(val) = arm_last.get(KEY_DISPLAY_ICON).and_then(toml::Value::as_str) {
                    conf.display.icons.arm_last.value = val.to_string();
                }

                if let Some(val) = arm_last.get(KEY_DISPLAY_COLOR).and_then(toml::Value::as_str) {
                    conf.display.icons.arm_last.color = Color::from_hex(val)
                        .map_err(|_| ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {}{}{}: {}",
                            KEY_DISPLAY, KEY_DISPLAY_ICONS, KEY_DISPLAY_ICONS_ARM_LAST, val
                        )))?;
                }
            }

            if let Some(arm_multiparent) = icons.get(KEY_DISPLAY_ICONS_ARM_MULTIPARENT) {
                if let Some(val) = arm_multiparent.get(KEY_DISPLAY_ICON).and_then(toml::Value::as_str) {
                    conf.display.icons.arm_multiparent.value = val.to_string();
                }

                if let Some(val) = arm_multiparent.get(KEY_DISPLAY_COLOR).and_then(toml::Value::as_str) {
                    conf.display.icons.arm_multiparent.color = Color::from_hex(val)
                        .map_err(|_| ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {}{}{}: {}",
                            KEY_DISPLAY, KEY_DISPLAY_ICONS, KEY_DISPLAY_ICONS_ARM_MULTIPARENT, val
                        )))?;
                }
            }

            if let Some(arm_multiparent_last) = icons.get(KEY_DISPLAY_ICONS_ARM_MULTIPARENT_LAST) {
                if let Some(val) = arm_multiparent_last.get(KEY_DISPLAY_ICON).and_then(toml::Value::as_str) {
                    conf.display.icons.arm_multiparent_last.value = val.to_string();
                }

                if let Some(val) = arm_multiparent_last.get(KEY_DISPLAY_COLOR).and_then(toml::Value::as_str) {
                    conf.display.icons.arm_multiparent_last.color = Color::from_hex(val)
                        .map_err(|_| ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {}{}{}: {}",
                            KEY_DISPLAY, KEY_DISPLAY_ICONS, KEY_DISPLAY_ICONS_ARM_MULTIPARENT_LAST, val
                        )))?;
                }
            }

            if let Some(arm_bar) = icons.get(KEY_DISPLAY_ICONS_ARM_BAR) {
                if let Some(val) = arm_bar.get(KEY_DISPLAY_ICON).and_then(toml::Value::as_str) {
                    conf.display.icons.arm_bar.value = val.to_string();
                }

                if let Some(val) = arm_bar.get(KEY_DISPLAY_COLOR).and_then(toml::Value::as_str) {
                    conf.display.icons.arm_bar.color = Color::from_hex(val)
                        .map_err(|_| ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {}{}{}: {}",
                            KEY_DISPLAY, KEY_DISPLAY_ICONS, KEY_DISPLAY_ICONS_ARM_BAR, val
                        )))?;
                }
            }

            if let Some(node_none) = icons.get(KEY_DISPLAY_ICONS_NODE_NONE) {
                if let Some(val) = node_none.get(KEY_DISPLAY_ICON).and_then(toml::Value::as_str) {
                    conf.display.icons.node_none.value = val.to_string();
                }

                if let Some(val) = node_none.get(KEY_DISPLAY_COLOR).and_then(toml::Value::as_str) {
                    conf.display.icons.node_none.color = Color::from_hex(val)
                        .map_err(|_| ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {}{}{}: {}",
                            KEY_DISPLAY, KEY_DISPLAY_ICONS, KEY_DISPLAY_ICONS_NODE_NONE, val
                        )))?;
                }
            }

            if let Some(node_checked) = icons.get(KEY_DISPLAY_ICONS_NODE_CHECKED) {
                if let Some(val) = node_checked.get(KEY_DISPLAY_ICON).and_then(toml::Value::as_str) {
                    conf.display.icons.node_checked.value = val.to_string();
                }

                if let Some(val) = node_checked.get(KEY_DISPLAY_COLOR).and_then(toml::Value::as_str) {
                    conf.display.icons.node_checked.color = Color::from_hex(val)
                        .map_err(|_| ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {}{}{}: {}",
                            KEY_DISPLAY, KEY_DISPLAY_ICONS, KEY_DISPLAY_ICONS_NODE_CHECKED, val
                        )))?;
                }
            }

            if let Some(node_partial) = icons.get(KEY_DISPLAY_ICONS_NODE_PARTIAL) {
                if let Some(val) = node_partial.get(KEY_DISPLAY_ICON).and_then(toml::Value::as_str) {
                    conf.display.icons.node_partial.value = val.to_string();
                }

                if let Some(val) = node_partial.get(KEY_DISPLAY_COLOR).and_then(toml::Value::as_str) {
                    conf.display.icons.node_partial.color = Color::from_hex(val)
                        .map_err(|_| ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {}{}{}: {}",
                            KEY_DISPLAY, KEY_DISPLAY_ICONS, KEY_DISPLAY_ICONS_NODE_PARTIAL, val
                        )))?;
                }
            }

            if let Some(node_pseudo) = icons.get(KEY_DISPLAY_ICONS_NODE_PSEUDO) {
                if let Some(val) = node_pseudo.get(KEY_DISPLAY_ICON).and_then(toml::Value::as_str) {
                    conf.display.icons.node_pseudo.value = val.to_string();
                }

                if let Some(val) = node_pseudo.get(KEY_DISPLAY_COLOR).and_then(toml::Value::as_str) {
                    conf.display.icons.node_pseudo.color = Color::from_hex(val)
                        .map_err(|_| ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {}{}{}: {}",
                            KEY_DISPLAY, KEY_DISPLAY_ICONS, KEY_DISPLAY_ICONS_NODE_PSEUDO, val
                        )))?;
                }
            }

            if let Some(node_date) = icons.get(KEY_DISPLAY_ICONS_NODE_DATE) {
                if let Some(val) = node_date.get(KEY_DISPLAY_ICON).and_then(toml::Value::as_str) {
                    conf.display.icons.node_date.value = val.to_string();
                }

                if let Some(val) = node_date.get(KEY_DISPLAY_COLOR).and_then(toml::Value::as_str) {
                    conf.display.icons.node_date.color = Color::from_hex(val)
                        .map_err(|_| ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {}{}{}: {}",
                            KEY_DISPLAY, KEY_DISPLAY_ICONS, KEY_DISPLAY_ICONS_NODE_DATE, val
                        )))?;
                }
            }
        }

        // Calendar configuration
        if let Some(calendar_cfg) = display_cfg.get(KEY_DISPLAY_CALENDAR) {
            if let Some(heatmap) = calendar_cfg.get(KEY_DISPLAY_CALENDAR_HEATMAP) {
                if let Some(val) = heatmap.get(KEY_DISPLAY_CALENDAR_HEATMAP_PALETTE).and_then(toml::Value::as_array) {
                    for (i, color_str) in val.iter().enumerate() {
                        if let Some(color) = color_str.as_str() {
                            if let Ok(parsed_color) = Color::from_hex(color) {
                                conf.display.calendar_config.heatmap_palette[i] = parsed_color;
                            } else {
                                return Err(ConfigReadError::ColorParseErr(format!(
                                    "Invalid color in heatmap palette: {}", color
                                )).into());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(conf)
}


pub fn get_config() -> AppResult<CliConfig> {
    let conf;
    if let Some(path) = get_home_default() {
        if let Some(path) = get_default_at(path) {
            let toml = read_file(&path)?;
            conf = parse_config(&toml)?;
        } else {
        conf = CliConfig::default();
        }
    } else {
        conf = CliConfig::default();
    };

    Ok(conf)
}
