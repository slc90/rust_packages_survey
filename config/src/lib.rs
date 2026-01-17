#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![allow(clippy::type_complexity)]

pub mod data_structure;

use bevy::app::Plugin;
pub use data_structure::{ConfigPath, Setting, save_to_file};

/// 配置插件
pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
	fn build(&self, app: &mut bevy::app::App) {
		app.init_resource::<Setting>();
	}
}
