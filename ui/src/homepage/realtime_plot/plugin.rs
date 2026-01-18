use crate::homepage::common::Functions;
use crate::homepage::realtime_plot::systems::{on_enter, on_exit};
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
		app.add_systems(OnEnter(Functions::RealtimePlot), on_enter)
			.add_systems(OnExit(Functions::RealtimePlot), on_exit);
	}
}
