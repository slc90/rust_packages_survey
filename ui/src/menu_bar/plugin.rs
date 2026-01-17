use bevy::{
	input_focus::{InputDispatchPlugin, tab_navigation::TabNavigationPlugin},
	prelude::*,
	ui_widgets::UiWidgetsPlugins,
};

/// Plugin for the menu bar system
///
/// This plugin provides a menu bar UI component that allows users to switch
/// between different application functions (About, Setting, etc.) through a dropdown menu.
/// The menu bar integrates with the homepage's `Functions` state system.
pub struct MenuBarPlugin;

impl Plugin for MenuBarPlugin {
	fn build(&self, app: &mut App) {
		// Add required plugins for menu functionality
		app.add_plugins((UiWidgetsPlugins, InputDispatchPlugin, TabNavigationPlugin));
	}
}
