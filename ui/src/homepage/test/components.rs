use bevy::prelude::*;

// ============================================================================
// TEST STATE COMPONENTS - Used for identifying Test state UI elements
// ============================================================================

/// Marker component for Test state UI content
/// Used to identify the Test state UI elements in queries
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct TestContentMarker;
