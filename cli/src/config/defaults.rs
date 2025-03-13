//! Baked-in defaults for the tuecli's configuration.

use crate::display::{Color, ColorEnum};

pub const DEFAULT_HEATMAP_PALETTE: [Color; 5] = [
    Color::new(90, 126, 222), // #5A7EDE
    Color::new(120, 94, 240), // #785EF0
    Color::new(220, 38, 127), // #DC267F
    Color::new(254, 97, 0),   // #FE6100
    Color::new(224, 167, 41), // #E0A729
];

// Graph section
pub const DEFAULT_GRAPH_AUTO_CLEAN: bool = false;

// Display section
pub const DEFAULT_DATE_FORMAT: &str = "%Y-%m-%d";
pub const DEFAULT_SHOW_CONNECTIONS: bool = true;
pub const DEFAULT_BAR_INDENT: bool = false;

// Icons - arms
pub const DEFAULT_ICON_ARM: &str = "+--";
pub const DEFAULT_COLOR_ARM: Color = Color::new(255, 255, 255); // #FFFFFF

pub const DEFAULT_ICON_ARM_LAST: &str = "+--";
pub const DEFAULT_COLOR_ARM_LAST: Color = Color::new(255, 255, 255); // #FFFFFF

pub const DEFAULT_ICON_ARM_MULTIPARENT: &str = "+..";
pub const DEFAULT_COLOR_ARM_MULTIPARENT: Color = Color::new(255, 255, 255); // #FFFFFF
//
pub const DEFAULT_ICON_ARM_MULTIPARENT_LAST: &str = "+..";
pub const DEFAULT_COLOR_ARM_MULTIPARENT_LAST: Color = Color::new(255, 255, 255); // #FFFFFF

pub const DEFAULT_ICON_ARM_BAR: &str = "|";
pub const DEFAULT_COLOR_ARM_BAR: Color = Color::new(255, 255, 255); // #FFFFFF

// Icons - nodes
pub const DEFAULT_ICON_NODE_NONE: &str = "[ ]";
pub const DEFAULT_COLOR_NODE_NONE: ColorEnum = ColorEnum::Cyan;

pub const DEFAULT_ICON_NODE_CHECKED: &str = "[x]";
pub const DEFAULT_COLOR_NODE_CHECKED: ColorEnum = ColorEnum::Green;

pub const DEFAULT_ICON_NODE_PARTIAL: &str = "[~]";
pub const DEFAULT_COLOR_NODE_PARTIAL: ColorEnum = ColorEnum::Orange;

pub const DEFAULT_ICON_NODE_PSEUDO: &str = "[*]";
pub const DEFAULT_COLOR_NODE_PSEUDO: ColorEnum = ColorEnum::Yellow;

pub const DEFAULT_ICON_NODE_DATE: &str = "[#]";
pub const DEFAULT_COLOR_NODE_DATE: ColorEnum = ColorEnum::Purple;

pub const DEFAULT_CONFIG: &str = include_str!("default_cfg.toml");
