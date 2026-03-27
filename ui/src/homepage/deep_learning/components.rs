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

/// Whisper 模型切换按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningWhisperModelCycleButtonMarker;

/// Whisper 文件文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningWhisperFileTextMarker;

/// Whisper 配置文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningWhisperConfigTextMarker;

/// Whisper 进度文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningWhisperProgressTextMarker;

/// Whisper 进度条填充标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningWhisperProgressFillMarker;

/// 翻译选择文件按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTranslationOpenFileButtonMarker;

/// 翻译语言切换按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTranslationLanguageCycleButtonMarker;

/// 翻译开始按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTranslationStartButtonMarker;

/// 翻译文件文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTranslationFileTextMarker;

/// 翻译配置文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTranslationConfigTextMarker;

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

/// TTS 文件文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTtsFileTextMarker;

/// TTS 配置文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningTtsConfigTextMarker;

/// 人声分离选择文件按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningSeparationOpenFileButtonMarker;

/// 人声分离开始按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningSeparationStartButtonMarker;

/// 人声分离文件文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningSeparationFileTextMarker;

/// 图像生成选择 Prompt 按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningImageGenerationOpenFileButtonMarker;

/// 图像生成分辨率切换按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningImageGenerationResolutionCycleButtonMarker;

/// 图像生成随机种子切换按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningImageGenerationSeedCycleButtonMarker;

/// 图像生成步数切换按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningImageGenerationStepsCycleButtonMarker;

/// 图像生成模型切换按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningImageGenerationModelCycleButtonMarker;

/// 图像生成开始按钮标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningImageGenerationStartButtonMarker;

/// 图像生成 Prompt 文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningImageGenerationFileTextMarker;

/// 图像生成配置文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningImageGenerationConfigTextMarker;

/// 图像生成预览图片标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningImageGenerationPreviewImageMarker;

/// 图像生成预览说明文本标记。
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct DeepLearningImageGenerationPreviewTextMarker;
