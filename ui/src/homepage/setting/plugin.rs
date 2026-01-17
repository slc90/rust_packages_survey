use crate::homepage::common::Functions;
use crate::homepage::setting::systems::{
	on_enter, on_exit, sync_radio_buttons_to_language, update_radio_button_border_color,
	update_radio_button_border_color2, update_radio_button_mark_color,
	update_radio_button_mark_color2,
};
use bevy::prelude::*;

/// Plugin for the Setting state
///
/// This plugin provides components and systems for the Setting state
/// in the homepage content area. It assumes that the `Functions` state
/// has already been initialized in the app.
pub struct SettingPlugin;

impl Plugin for SettingPlugin {
	fn build(&self, app: &mut App) {
		// Add Setting state lifecycle systems
		// Note: This assumes that `Functions` state has already been initialized
		app.add_systems(OnEnter(Functions::Setting), on_enter)
			.add_systems(OnExit(Functions::Setting), on_exit)
			.add_systems(
				Update,
				(
					// Radio button styling systems
					update_radio_button_border_color,
					update_radio_button_border_color2,
					update_radio_button_mark_color,
					update_radio_button_mark_color2,
					// Language synchronization system
					sync_radio_buttons_to_language,
				),
			);
	}
}
