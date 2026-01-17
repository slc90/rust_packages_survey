use bevy::prelude::*;

// ============================================================================
// SETTING STATE COMPONENTS - Used for identifying Setting state UI elements
// ============================================================================

/// Marker component for Setting state UI content
/// Used to identify the Setting state UI elements in queries
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct SettingContentMarker;

/// 标识语言RadioGroup的Marker
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct LanguageRadioGroupMarker;

/// 标识中文RadioButton的Marker
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct ChineseRadioButtonMarker;

/// 标识英文RadioButton的Marker
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct EnglishRadioButtonMarker;
