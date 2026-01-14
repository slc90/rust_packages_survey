use bevy::prelude::*;

// ============================================================================
// ABOUT STATE COMPONENTS - Used for identifying About state UI elements
// ============================================================================

/// Marker component for About state UI content
/// Used to identify the About state UI elements in queries
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct AboutContentMarker;
