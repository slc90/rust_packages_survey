use bevy::prelude::*;
use bevy::state::state::States;

// ============================================================================
// STATE COMPONENTS - Used for state management
// ============================================================================

/// 所有功能的枚举，用来切换ContentArea
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default, Reflect)]
pub enum Functions {
	#[default]
	About,

	Test,
}

// ============================================================================
// MARKER COMPONENTS - Used for querying and identifying UI elements
// ============================================================================

/// Marker component for the content area container
/// Used to identify the main content area entity in queries
/// This area appears below the title bar and displays different UI content based on state
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct ContentAreaMarker;
