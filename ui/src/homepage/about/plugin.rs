use crate::homepage::about::components::AboutContentMarker;
use crate::homepage::about::systems::{on_enter, on_exit};
use crate::homepage::common::Functions;
use bevy::prelude::*;

/// Plugin for the About state
///
/// This plugin provides components and systems for the About state
/// in the homepage content area. It assumes that the `Functions` state
/// has already been initialized in the app.
pub struct AboutPlugin;

impl Plugin for AboutPlugin {
	fn build(&self, app: &mut App) {
		// Register About-specific components for reflection
		app.register_type::<AboutContentMarker>();

		// Add About state lifecycle systems
		// Note: This assumes that `Functions` state has already been initialized
		app.add_systems(OnEnter(Functions::About), on_enter)
			.add_systems(OnExit(Functions::About), on_exit);
	}
}
