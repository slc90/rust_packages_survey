use bevy::prelude::*;

/// 范式页面根节点标记。
#[derive(Component)]
pub struct ParadigmContentMarker;

/// GIF 预览图片标记。
#[derive(Component)]
pub struct ParadigmGifPreviewMarker;

/// GIF 信息文本标记。
#[derive(Component)]
pub struct ParadigmGifInfoTextMarker;

/// 范式状态文本标记。
#[derive(Component)]
pub struct ParadigmStatusTextMarker;

/// 显示器信息文本标记。
#[derive(Component)]
pub struct ParadigmMonitorTextMarker;

/// 目标字符文本标记。
#[derive(Component)]
pub struct ParadigmTargetTextMarker;

/// 循环切换显示器按钮标记。
#[derive(Component)]
pub struct ParadigmCycleMonitorButtonMarker;

/// 切换目标字符按钮标记。
#[derive(Component)]
pub struct ParadigmCycleTargetButtonMarker;

/// 开始播放按钮标记。
#[derive(Component)]
pub struct ParadigmStartButtonMarker;

/// 暂停恢复按钮标记。
#[derive(Component)]
pub struct ParadigmPauseResumeButtonMarker;

/// 暂停恢复按钮文字标记。
#[derive(Component)]
pub struct ParadigmPauseResumeTextMarker;

/// 停止播放按钮标记。
#[derive(Component)]
pub struct ParadigmStopButtonMarker;

/// 播放窗口根节点标记。
#[derive(Component)]
pub struct ParadigmPresentationRootMarker;

/// 播放窗口单元格标记。
#[derive(Component, Clone, Copy)]
pub struct ParadigmPresentationCellMarker {
	/// 当前单元格所在行。
	pub row: usize,

	/// 当前单元格所在列。
	pub col: usize,
}
