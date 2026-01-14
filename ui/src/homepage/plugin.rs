use crate::homepage::about::plugin::AboutPlugin;
use crate::homepage::common::{ContentAreaMarker, Functions};
use crate::homepage::test::plugin::TestPlugin;
use bevy::prelude::*;

/// Plugin for the homepage system
///
/// This plugin provides state management for the homepage content area
/// with About and Test states that display different UI content below the title bar.
/// It serves as the main plugin that orchestrates all homepage functionality
/// by including both AboutPlugin and TestPlugin as sub-plugins.
pub struct HomepagePlugin;

impl Plugin for HomepagePlugin {
	fn build(&self, app: &mut App) {
		// Register common components for reflection
		app.register_type::<ContentAreaMarker>();

		// Initialize the main state enum for the homepage
		app.init_state::<Functions>();

		// Add About and Test state plugins
		// These plugins handle their own component registration and system setup
		app.add_plugins(AboutPlugin);
		app.add_plugins(TestPlugin);
	}
}
