use bevy::prelude::*;

/// 深度学习页面根节点标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningContentMarker;

/// 空任务测试按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningSmokeTestButtonMarker;

/// 页面状态文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningStatusTextMarker;

/// 页面结果文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningResultTextMarker;
