use bevy::prelude::*;

// ============================================================================
// REALTIME_PLOT STATE COMPONENTS - Used for identifying RealtimePlot state UI elements
// ============================================================================

/// Marker component for RealtimePlot state UI content
/// Used to identify the RealtimePlot state UI elements in queries
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct RealtimePlotContentMarker;

// ============================================================================
// WAVEFORM DISPLAY COMPONENTS - Used for identifying waveform rendering elements
// ============================================================================

/// Marker component for waveform mesh entities
/// Used to identify the waveform line rendering entities in queries
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct WaveformMeshMarker;

/// Marker component for control panel container
/// Used to identify the control panel UI container in queries
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ControlPanelMarker;

/// Marker component for channel count slider
/// Used to identify the channel slider in queries
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ChannelSliderMarker;

/// Marker component for sample rate dropdown
/// Used to identify the sample rate dropdown in queries
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct SampleRateDropdownMarker;

/// Marker component for individual channel label
/// Used to identify channel labels in queries
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ChannelLabelMarker;

/// Marker component for the waveform display area
/// Used to identify the main waveform area in queries
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct WaveformAreaMarker;
