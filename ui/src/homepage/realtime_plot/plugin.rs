use crate::homepage::common::Functions;
use crate::homepage::realtime_plot::systems::{
	generate_waveform_data, handle_channel_slider_click, handle_sample_rate_click,
	init_waveform_generator, init_waveform_rendering, on_enter, on_exit, spawn_axis_grid,
	spawn_waveform_settings_ui, update_waveform_display, update_waveform_settings,
};
use bevy::prelude::*;

/// Plugin for the RealtimePlot state
///
/// This plugin provides components and systems for the RealtimePlot state
/// in the homepage content area. It assumes that the `Functions` state
/// has already been initialized in the app.
pub struct RealtimePlotPlugin;

impl Plugin for RealtimePlotPlugin {
	fn build(&self, app: &mut App) {
		// Add RealtimePlot state lifecycle systems
		// Note: This assumes that `Functions` state has already been initialized
		app.add_systems(
			OnEnter(Functions::RealtimePlot),
			(
				on_enter,
				init_waveform_generator,
				init_waveform_rendering,
				spawn_waveform_settings_ui,
				spawn_axis_grid,
			),
		)
		.add_systems(OnExit(Functions::RealtimePlot), on_exit)
		// Add update systems
		.add_systems(
			Update,
			generate_waveform_data.run_if(in_state(Functions::RealtimePlot)),
		)
		.add_systems(
			Update,
			(update_waveform_display, update_waveform_settings)
				.run_if(in_state(Functions::RealtimePlot)),
		)
		.add_systems(
			Update,
			(handle_channel_slider_click, handle_sample_rate_click)
				.run_if(in_state(Functions::RealtimePlot)),
		);
	}
}
