use crate::homepage::{
	common::Functions,
	screenshot::{
		resources::ScreenshotStatusMessage,
		systems::{
			handle_current_display_click, handle_overlay_window_closed, handle_screen_region_click,
			handle_window_region_crop_click, handle_window_screenshot_click, on_enter, on_exit,
			sync_status_messages, update_screen_region_overlay,
		},
	},
};
use bevy::prelude::*;

pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
	fn build(&self, app: &mut App) {
		app.add_message::<ScreenshotStatusMessage>()
			.add_observer(crate::homepage::screenshot::systems::handle_screenshot_captured)
			.add_systems(OnEnter(Functions::Screenshot), on_enter)
			.add_systems(OnExit(Functions::Screenshot), on_exit)
			.add_systems(
				Update,
				(
					handle_window_screenshot_click,
					handle_window_region_crop_click,
					handle_current_display_click,
					handle_screen_region_click,
					update_screen_region_overlay,
					handle_overlay_window_closed,
					sync_status_messages,
				)
					.run_if(in_state(Functions::Screenshot)),
			);
	}
}
