//! 回放波形 UI 组件标记
//!
//! 定义回放波形界面的 UI 组件标记

use bevy::prelude::*;

// ============================================================================
// MARKER COMPONENTS - UI 元素标记
// ============================================================================

/// 回放波形内容区域标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PlaybackPlotContentMarker;

/// 控制面板标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PlaybackControlPanelMarker;

/// 播放按钮标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PlayButtonMarker;

/// 暂停按钮标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PauseButtonMarker;

/// 速度按钮标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct SpeedButtonMarker;

/// 播放按钮文字标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PlayButtonTextMarker;

/// 速度按钮文字标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct SpeedButtonTextMarker;

/// 上一页按钮标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PrevPageButtonMarker;

/// 下一页按钮标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct NextPageButtonMarker;

/// 文件路径显示标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct FilePathDisplayMarker;

/// 位置显示标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PositionDisplayMarker;

/// 页码显示标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PageDisplayMarker;

/// 波形网格实体标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PlaybackWaveformMeshMarker;
