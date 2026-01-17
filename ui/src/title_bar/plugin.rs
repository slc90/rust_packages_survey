use crate::title_bar::{
	resources::{LeftClickAction, TitleBarButtonCooldown, WindowState},
	systems::{drag_and_move_window, handle_button_clicks, handle_window_visibility},
};
use bevy::prelude::*;

/// Plugin for the title bar system
///
/// This plugin provides a custom title bar UI for Bevy applications
/// with image support for icons and logos, including window dragging functionality
/// and control button interactions.
pub struct TitleBarPlugin;

impl Plugin for TitleBarPlugin {
	fn build(&self, app: &mut App) {
		// 注册资源
		app.insert_resource(LeftClickAction::Move);
		app.insert_resource(TitleBarButtonCooldown::default());
		app.insert_resource(WindowState::default());
		// Add window dragging, button handling, and window visibility systems
		app.add_systems(
			Update,
			(
				drag_and_move_window,
				handle_button_clicks,
				handle_window_visibility,
			),
		);
	}
}
