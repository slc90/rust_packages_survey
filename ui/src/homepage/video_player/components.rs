use bevy::prelude::*;

/// 视频播放页面根节点
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct VideoPlayerContentMarker;

/// 主窗口视频纹理显示节点
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct VideoDisplayMarker;

/// 弹窗视频纹理显示节点
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PopupVideoDisplayMarker;

/// 主窗口状态文本
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct VideoStatusTextMarker;

/// 弹窗状态文本
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PopupVideoStatusTextMarker;

/// 主窗口文件文本
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct VideoFileTextMarker;

/// 弹窗文件文本
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PopupVideoFileTextMarker;

/// 主窗口播放按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct VideoPlayPauseButtonMarker;

/// 弹窗播放按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PopupVideoPlayPauseButtonMarker;

/// 主窗口播放按钮文字
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct VideoPlayPauseTextMarker;

/// 弹窗播放按钮文字
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PopupVideoPlayPauseTextMarker;

/// 主窗口关闭按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct VideoCloseButtonMarker;

/// 弹窗关闭按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PopupVideoCloseButtonMarker;

/// 弹窗按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct VideoPopupButtonMarker;

/// 主窗口打开文件按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct VideoOpenFileButtonMarker;

/// 弹窗打开文件按钮
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PopupVideoOpenFileButtonMarker;

/// 弹窗 UI 根节点
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PopupVideoPlayerRootMarker;
