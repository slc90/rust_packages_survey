use bevy::prelude::*;

/// 音频播放页面根节点标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct AudioPlayerContentMarker;

/// 打开文件按钮标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct AudioOpenFileButtonMarker;

/// 播放暂停按钮标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct AudioPlayPauseButtonMarker;

/// 关闭音频按钮标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct AudioCloseButtonMarker;

/// 播放暂停按钮文字标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct AudioPlayPauseTextMarker;

/// 文件文本标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct AudioFileTextMarker;

/// 状态文本标记
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct AudioStatusTextMarker;
