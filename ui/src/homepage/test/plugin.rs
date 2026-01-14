use crate::homepage::common::Functions;
use crate::homepage::test::components::TestContentMarker;
use crate::homepage::test::systems::{on_enter, on_exit};
use bevy::prelude::*;

/// Plugin for the Test state
///
/// This plugin provides components and systems for the Test state
/// in the homepage content area. It assumes that the `Functions` state
/// has already been initialized in the app.
pub struct TestPlugin;

impl Plugin for TestPlugin {
	fn build(&self, app: &mut App) {
		// Register Test-specific components for reflection
		app.register_type::<TestContentMarker>();

		// Add Test state lifecycle systems
		// Note: This assumes that `Functions` state has already been initialized
		app.add_systems(OnEnter(Functions::Test), on_enter)
			.add_systems(OnExit(Functions::Test), on_exit);
	}
}
