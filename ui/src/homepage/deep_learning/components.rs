use bevy::prelude::*;

/// 深度学习页面根节点标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningContentMarker;

/// 空任务测试按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningSmokeTestButtonMarker;

/// Whisper 打开文件按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningWhisperOpenFileButtonMarker;

/// Whisper 开始按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningWhisperStartButtonMarker;

/// Whisper 时间戳切换按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningWhisperTimestampToggleButtonMarker;

/// Whisper 语言切换按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningWhisperLanguageCycleButtonMarker;

/// 页面状态文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningStatusTextMarker;

/// 页面结果文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningResultTextMarker;

/// Whisper 文件文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningWhisperFileTextMarker;

/// Whisper 配置文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningWhisperConfigTextMarker;

/// 翻译选择文件按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTranslationOpenFileButtonMarker;

/// 翻译语言切换按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTranslationLanguageCycleButtonMarker;

/// 翻译开始按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTranslationStartButtonMarker;

/// TTS 选择文件按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTtsOpenFileButtonMarker;

/// TTS 语言切换按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTtsLanguageCycleButtonMarker;

/// TTS 语速切换按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTtsSpeedCycleButtonMarker;

/// TTS 开始按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTtsStartButtonMarker;

/// 翻译文件文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTranslationFileTextMarker;

/// 翻译配置文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTranslationConfigTextMarker;

/// TTS 文件文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTtsFileTextMarker;

/// TTS 配置文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTtsConfigTextMarker;
