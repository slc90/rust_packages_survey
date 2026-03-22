use bevy::prelude::*;

use crate::homepage::{
	common::Functions,
	paradigm::systems::{
		cleanup_closed_presentation_window, handle_cycle_monitor_click, handle_cycle_target_click,
		handle_pause_resume_click, handle_start_click, handle_stop_click, on_enter, on_exit,
		sync_gif_preview_image, sync_monitor_text, sync_status_text, sync_target_text,
		sync_window_button_text, tick_gif_preview, tick_p300_playback, update_monitor_inventory,
		update_p300_presentation,
	},
};

/// 范式页面插件。
pub struct ParadigmPlugin;

impl Plugin for ParadigmPlugin {
	/// 构建范式页面插件。
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(Functions::Paradigm), on_enter)
			.add_systems(OnExit(Functions::Paradigm), on_exit)
			.add_systems(
				Update,
				(
					update_monitor_inventory,
					handle_cycle_monitor_click,
					handle_cycle_target_click,
					handle_start_click,
					handle_pause_resume_click,
					handle_stop_click,
					cleanup_closed_presentation_window,
					tick_gif_preview,
					sync_gif_preview_image,
					sync_monitor_text,
					sync_target_text,
					sync_window_button_text,
					sync_status_text,
					update_p300_presentation,
				)
					.run_if(in_state(Functions::Paradigm)),
			)
			.add_systems(
				FixedUpdate,
				tick_p300_playback.run_if(in_state(Functions::Paradigm)),
			);
	}
}
