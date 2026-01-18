use bevy::prelude::*;

// ============================================================================
// REALTIME_PLOT STATE COMPONENTS - Used for identifying RealtimePlot state UI elements
// ============================================================================

/// Marker component for RealtimePlot state UI content
/// Used to identify the RealtimePlot state UI elements in queries
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct RealtimePlotContentMarker;
