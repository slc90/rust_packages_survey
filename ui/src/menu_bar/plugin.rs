use crate::menu_bar::components::*;
use bevy::{
	input_focus::{InputDispatchPlugin, tab_navigation::TabNavigationPlugin},
	prelude::*,
	ui_widgets::UiWidgetsPlugins,
};

/// Plugin for the menu bar system
///
/// This plugin provides a menu bar UI component that allows users to switch
/// between different application functions (About, Test, etc.) through a dropdown menu.
/// The menu bar integrates with the homepage's `Functions` state system.
pub struct MenuBarPlugin;

impl Plugin for MenuBarPlugin {
	fn build(&self, app: &mut App) {
		// Add required plugins for menu functionality
		app.add_plugins((UiWidgetsPlugins, InputDispatchPlugin, TabNavigationPlugin));

		// Register all components for reflection
		app.register_type::<MenuBarMarker>()
			.register_type::<FunctionMenuMarker>()
			.register_type::<LanguageMenuMarker>()
			.register_type::<MenuPopupMarker>()
			.register_type::<MenuItemMarker>()
			.register_type::<FunctionMenuItemMarker>()
			.register_type::<LanguageMenuItemMarker>()
			.register_type::<MenuBarStyle>()
			.register_type::<MenuButtonStyle>()
			.register_type::<MenuPopupStyle>()
			.register_type::<MenuItemStyle>();
	}
}
