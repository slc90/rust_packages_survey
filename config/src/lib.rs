pub mod data_structure;

use bevy::app::Plugin;
pub use data_structure::Setting;

/// 配置插件
pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
	fn build(&self, app: &mut bevy::app::App) {
		app.init_resource::<Setting>();
	}
}
