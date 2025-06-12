mod defaults;

pub use defaults::*;
use home::home_dir;

use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::display::Color;
use crate::paths::get_default_path;
use crate::AppResult;

pub type ConfigParseResult<T> = Result<T, ConfigReadError>;

/// The default config file name
pub const DEFAULT_CFG_NAME: &str = ".tueconf.toml";

/// Represents an error during reading a config file
#[derive(Debug, Error)]
pub enum ConfigReadError {
    #[error("Failed to get home directory!")]
    NoHome,

    #[error("File does not exist: {0}")]
    NonexistantFile(PathBuf),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("TOML Deserialization error: {0}")]
    TOMLDeserializeErr(#[from] toml::de::Error),

    #[error("Color parse error: {0}")]
    ColorParseErr(String),
}

pub struct BlueprintsConfig {
    pub(crate) store_path: PathBuf,
}

impl Default for BlueprintsConfig {
    fn default() -> Self {
        Self {
            store_path: DEFAULT_BLUEPRINTS_STORE_PATH
                .replace(
                    "$HOME",
                    &home_dir()
                        .expect("failed to get home directory")
                        .to_string_lossy(),
                )
                .into(),
        }
    }
}

pub struct GraphConfig {
    pub(crate) auto_clean: bool,
    /// In percentage, how much of the total graph node count should the [None] nodes composite before auto clean is activated
    pub(crate) auto_clean_threshold: u8,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            auto_clean: DEFAULT_GRAPH_AUTO_CLEAN,
            auto_clean_threshold: DEFAULT_GRAPH_AUTO_CLEAN_THRESHOLD,
        }
    }
}

pub struct Icon {
    pub(crate) value: String,
    pub(crate) color: Color,
}

impl Icon {
    pub fn colorize<T: Into<Color>>(mut self, color: T) -> Self {
        self.color = color.into();
        self
    }
}

impl Display for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<&str> for Icon {
    fn from(value: &str) -> Self {
        Icon {
            value: value.to_string(),
            color: Color::default(),
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
            arm_multiparent: Icon::from(DEFAULT_ICON_ARM_MULTIPARENT)
                .colorize(DEFAULT_COLOR_ARM_MULTIPARENT),
            arm_multiparent_last: Icon::from(DEFAULT_ICON_ARM_MULTIPARENT_LAST)
                .colorize(DEFAULT_COLOR_ARM_MULTIPARENT_LAST),
            arm_bar: Icon::from(DEFAULT_ICON_ARM_BAR).colorize(DEFAULT_COLOR_ARM_BAR),

            node_none: Icon::from(DEFAULT_ICON_NODE_NONE).colorize(DEFAULT_COLOR_NODE_NONE),
            node_checked: Icon::from(DEFAULT_ICON_NODE_CHECKED)
                .colorize(DEFAULT_COLOR_NODE_CHECKED),
            node_partial: Icon::from(DEFAULT_ICON_NODE_PARTIAL)
                .colorize(DEFAULT_COLOR_NODE_PARTIAL),
            node_pseudo: Icon::from(DEFAULT_ICON_NODE_PSEUDO).colorize(DEFAULT_COLOR_NODE_PSEUDO),
            node_date: Icon::from(DEFAULT_ICON_NODE_DATE).colorize(DEFAULT_COLOR_NODE_DATE),
        }
    }
}

pub struct CalendarConfig {
    pub(crate) heatmap_palette: [Color; 5],
}

impl Default for CalendarConfig {
    fn default() -> Self {
        Self {
            heatmap_palette: DEFAULT_HEATMAP_PALETTE,
        }
    }
}

pub struct DisplayConfig {
    pub(crate) date_fmt: String,
    pub(crate) show_connections: bool,
    pub(crate) icons: DisplayIconConfig,
    pub(crate) calendar_config: CalendarConfig,
    pub(crate) bar_indent: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_connections: DEFAULT_SHOW_CONNECTIONS,
            bar_indent: DEFAULT_BAR_INDENT,
            date_fmt: DEFAULT_DATE_FORMAT.to_string(),
            calendar_config: CalendarConfig::default(),
            icons: DisplayIconConfig::default(),
        }
    }
}

#[derive(Default)]
pub struct CliConfig {
    pub(crate) graph: GraphConfig,
    pub(crate) display: DisplayConfig,
    pub(crate) blueprints: BlueprintsConfig,
}

impl CliConfig {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
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
const KEY_AUTO_CLEAN_THRESHOLD: &str = "auto_clean_threshold";
const KEY_BAR_INDENT: &str = "bar_indent";
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
const KEY_BLUEPRINTS: &str = "blueprints";
const KEY_BLUEPRINTS_STORE_PATH: &str = "store_path";

/// Parses core configurations from a toml table
/// Any missing or malformed values will be replaced with defaults.
pub fn parse_config(toml: &toml::Table) -> ConfigParseResult<CliConfig> {
    let mut conf = CliConfig::new();

    // Graph configuration
    if let Some(graph_cfg) = toml.get(KEY_GRAPH) {
        if let Some(val) = graph_cfg.get(KEY_AUTO_CLEAN).and_then(toml::Value::as_bool) {
            conf.graph.auto_clean = val;
        }
        if let Some(val) = graph_cfg
            .get(KEY_AUTO_CLEAN_THRESHOLD)
            .and_then(toml::Value::as_integer)
        {
            conf.graph.auto_clean_threshold = val as u8;
        }
    }

    // Display configuration
    if let Some(display_cfg) = toml.get(KEY_DISPLAY) {
        if let Some(val) = display_cfg.get(KEY_DATE_FMT).and_then(toml::Value::as_str) {
            conf.display.date_fmt = val.to_string();
        }

        if let Some(val) = display_cfg
            .get(KEY_BAR_INDENT)
            .and_then(toml::Value::as_bool)
        {
            conf.display.bar_indent = val;
        }

        if let Some(val) = display_cfg
            .get(KEY_SHOW_CONNECTIONS)
            .and_then(toml::Value::as_bool)
        {
            conf.display.show_connections = val;
        }

        // Icons configuration
        if let Some(icons) = display_cfg.get(KEY_DISPLAY_ICONS) {
            if let Some(arm) = icons.get(KEY_DISPLAY_ICONS_ARM) {
                if let Some(val) = arm.get(KEY_DISPLAY_ICON).and_then(toml::Value::as_str) {
                    conf.display.icons.arm.value = val.to_string();
                }

                if let Some(val) = arm.get(KEY_DISPLAY_COLOR).and_then(toml::Value::as_str) {
                    conf.display.icons.arm.color = Color::from_str(val).map_err(|_| {
                        ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {KEY_DISPLAY}{KEY_DISPLAY_ICONS}{KEY_DISPLAY_ICONS_ARM}: {val}"
                        ))
                    })?;
                }
            }

            if let Some(arm_last) = icons.get(KEY_DISPLAY_ICONS_ARM_LAST) {
                if let Some(val) = arm_last.get(KEY_DISPLAY_ICON).and_then(toml::Value::as_str) {
                    conf.display.icons.arm_last.value = val.to_string();
                }

                if let Some(val) = arm_last
                    .get(KEY_DISPLAY_COLOR)
                    .and_then(toml::Value::as_str)
                {
                    conf.display.icons.arm_last.color = Color::from_str(val).map_err(|_| {
                        ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {KEY_DISPLAY}{KEY_DISPLAY_ICONS}{KEY_DISPLAY_ICONS_ARM_LAST}: {val}"
                        ))
                    })?;
                }
            }

            if let Some(arm_multiparent) = icons.get(KEY_DISPLAY_ICONS_ARM_MULTIPARENT) {
                if let Some(val) = arm_multiparent
                    .get(KEY_DISPLAY_ICON)
                    .and_then(toml::Value::as_str)
                {
                    conf.display.icons.arm_multiparent.value = val.to_string();
                }

                if let Some(val) = arm_multiparent
                    .get(KEY_DISPLAY_COLOR)
                    .and_then(toml::Value::as_str)
                {
                    conf.display.icons.arm_multiparent.color =
                        Color::from_str(val).map_err(|_| {
                            ConfigReadError::ColorParseErr(format!(
                                "Invalid color for {KEY_DISPLAY}{KEY_DISPLAY_ICONS}{KEY_DISPLAY_ICONS_ARM_MULTIPARENT}: {val}"
                            ))
                        })?;
                }
            }

            if let Some(arm_multiparent_last) = icons.get(KEY_DISPLAY_ICONS_ARM_MULTIPARENT_LAST) {
                if let Some(val) = arm_multiparent_last
                    .get(KEY_DISPLAY_ICON)
                    .and_then(toml::Value::as_str)
                {
                    conf.display.icons.arm_multiparent_last.value = val.to_string();
                }

                if let Some(val) = arm_multiparent_last
                    .get(KEY_DISPLAY_COLOR)
                    .and_then(toml::Value::as_str)
                {
                    conf.display.icons.arm_multiparent_last.color =
                        Color::from_str(val).map_err(|_| {
                            ConfigReadError::ColorParseErr(format!(
                                "Invalid color for {KEY_DISPLAY}{KEY_DISPLAY_ICONS}{KEY_DISPLAY_ICONS_ARM_MULTIPARENT_LAST}: {val}"
                            ))
                        })?;
                }
            }

            if let Some(arm_bar) = icons.get(KEY_DISPLAY_ICONS_ARM_BAR) {
                if let Some(val) = arm_bar.get(KEY_DISPLAY_ICON).and_then(toml::Value::as_str) {
                    conf.display.icons.arm_bar.value = val.to_string();
                }

                if let Some(val) = arm_bar.get(KEY_DISPLAY_COLOR).and_then(toml::Value::as_str) {
                    conf.display.icons.arm_bar.color = Color::from_str(val).map_err(|_| {
                        ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {KEY_DISPLAY}{KEY_DISPLAY_ICONS}{KEY_DISPLAY_ICONS_ARM_BAR}: {val}"
                        ))
                    })?;
                }
            }

            if let Some(node_none) = icons.get(KEY_DISPLAY_ICONS_NODE_NONE) {
                if let Some(val) = node_none
                    .get(KEY_DISPLAY_ICON)
                    .and_then(toml::Value::as_str)
                {
                    conf.display.icons.node_none.value = val.to_string();
                }

                if let Some(val) = node_none
                    .get(KEY_DISPLAY_COLOR)
                    .and_then(toml::Value::as_str)
                {
                    conf.display.icons.node_none.color = Color::from_str(val).map_err(|_| {
                        ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {KEY_DISPLAY}{KEY_DISPLAY_ICONS}{KEY_DISPLAY_ICONS_NODE_NONE}: {val}"
                        ))
                    })?;
                }
            }

            if let Some(node_checked) = icons.get(KEY_DISPLAY_ICONS_NODE_CHECKED) {
                if let Some(val) = node_checked
                    .get(KEY_DISPLAY_ICON)
                    .and_then(toml::Value::as_str)
                {
                    conf.display.icons.node_checked.value = val.to_string();
                }

                if let Some(val) = node_checked
                    .get(KEY_DISPLAY_COLOR)
                    .and_then(toml::Value::as_str)
                {
                    conf.display.icons.node_checked.color = Color::from_str(val).map_err(|_| {
                        ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {KEY_DISPLAY}{KEY_DISPLAY_ICONS}{KEY_DISPLAY_ICONS_NODE_CHECKED}: {val}"
                        ))
                    })?;
                }
            }

            if let Some(node_partial) = icons.get(KEY_DISPLAY_ICONS_NODE_PARTIAL) {
                if let Some(val) = node_partial
                    .get(KEY_DISPLAY_ICON)
                    .and_then(toml::Value::as_str)
                {
                    conf.display.icons.node_partial.value = val.to_string();
                }

                if let Some(val) = node_partial
                    .get(KEY_DISPLAY_COLOR)
                    .and_then(toml::Value::as_str)
                {
                    conf.display.icons.node_partial.color = Color::from_str(val).map_err(|_| {
                        ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {KEY_DISPLAY}{KEY_DISPLAY_ICONS}{KEY_DISPLAY_ICONS_NODE_PARTIAL}: {val}"
                        ))
                    })?;
                }
            }

            if let Some(node_pseudo) = icons.get(KEY_DISPLAY_ICONS_NODE_PSEUDO) {
                if let Some(val) = node_pseudo
                    .get(KEY_DISPLAY_ICON)
                    .and_then(toml::Value::as_str)
                {
                    conf.display.icons.node_pseudo.value = val.to_string();
                }

                if let Some(val) = node_pseudo
                    .get(KEY_DISPLAY_COLOR)
                    .and_then(toml::Value::as_str)
                {
                    conf.display.icons.node_pseudo.color = Color::from_str(val).map_err(|_| {
                        ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {KEY_DISPLAY}{KEY_DISPLAY_ICONS}{KEY_DISPLAY_ICONS_NODE_PSEUDO}: {val}"
                        ))
                    })?;
                }
            }

            if let Some(node_date) = icons.get(KEY_DISPLAY_ICONS_NODE_DATE) {
                if let Some(val) = node_date
                    .get(KEY_DISPLAY_ICON)
                    .and_then(toml::Value::as_str)
                {
                    conf.display.icons.node_date.value = val.to_string();
                }

                if let Some(val) = node_date
                    .get(KEY_DISPLAY_COLOR)
                    .and_then(toml::Value::as_str)
                {
                    conf.display.icons.node_date.color = Color::from_str(val).map_err(|_| {
                        ConfigReadError::ColorParseErr(format!(
                            "Invalid color for {KEY_DISPLAY}{KEY_DISPLAY_ICONS}{KEY_DISPLAY_ICONS_NODE_DATE}: {val}"
                        ))
                    })?;
                }
            }
        }

        // Calendar configuration
        if let Some(calendar_cfg) = display_cfg.get(KEY_DISPLAY_CALENDAR) {
            if let Some(heatmap) = calendar_cfg.get(KEY_DISPLAY_CALENDAR_HEATMAP) {
                if let Some(val) = heatmap
                    .get(KEY_DISPLAY_CALENDAR_HEATMAP_PALETTE)
                    .and_then(toml::Value::as_array)
                {
                    for (i, color_str) in val.iter().enumerate().take(5) {
                        if let Some(color) = color_str.as_str() {
                            if let Ok(parsed_color) = Color::from_str(color) {
                                conf.display.calendar_config.heatmap_palette[i] = parsed_color;
                            } else {
                                return Err(ConfigReadError::ColorParseErr(format!(
                                    "Invalid color in heatmap palette: {color}"
                                )));
                            }
                        }
                    }
                }
            }
        }
    }

    // Blueprints configuration
    if let Some(blueprints_cfg) = toml.get(KEY_BLUEPRINTS) {
        if let Some(path) = blueprints_cfg
            .get(KEY_BLUEPRINTS_STORE_PATH)
            .and_then(toml::Value::as_str)
        {
            conf.blueprints.store_path = path
                .replace(
                    "$HOME",
                    &home_dir().ok_or(ConfigReadError::NoHome)?.to_string_lossy(),
                )
                .into();
        };
    };

    Ok(conf)
}

pub fn get_config(config_path: Option<&PathBuf>) -> AppResult<CliConfig> {
    let conf;

    if let Some(path) = config_path {
        if let Ok(res) = read_file(path) {
            return Ok(parse_config(&res)?);
        }
    };
    if let Some(path) = get_default_path(DEFAULT_CFG_NAME.into()) {
        let toml = read_file(&path)?;
        conf = parse_config(&toml)?;
    } else {
        conf = CliConfig::default();
    };

    Ok(conf)
}
