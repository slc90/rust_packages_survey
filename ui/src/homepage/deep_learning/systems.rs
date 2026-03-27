use bevy::{
	asset::RenderAssetUsages,
	prelude::*,
	render::render_resource::{Extent3d, TextureDimension, TextureFormat},
	tasks::AsyncComputeTaskPool,
};
use deep_learning::{
	image_generation::{ImageGenerationModelKind, ImageGenerationResolution},
	runtime::execute_task,
	runtime::initialize_runtime_directories,
	task::{
		DlTaskId, DlTaskKind, DlTaskPayload, DlTaskRequestMessage, DlTaskResultMessage,
		DlTaskState, DlTaskStatusMessage,
	},
	translation::TranslationSourceLanguage,
	tts::TtsLanguage,
	whisper::{WhisperLanguageHint, WhisperModelKind},
};

use crate::{
	file_dialog::pick_single_file,
	homepage::{
		common::ContentAreaMarker,
		deep_learning::{
			components::{
				DeepLearningContentMarker, DeepLearningImageGenerationConfigTextMarker,
				DeepLearningImageGenerationFileTextMarker,
				DeepLearningImageGenerationModelCycleButtonMarker,
				DeepLearningImageGenerationOpenFileButtonMarker,
				DeepLearningImageGenerationPreviewImageMarker,
				DeepLearningImageGenerationPreviewTextMarker,
				DeepLearningImageGenerationResolutionCycleButtonMarker,
				DeepLearningImageGenerationSeedCycleButtonMarker,
				DeepLearningImageGenerationStartButtonMarker,
				DeepLearningImageGenerationStepsCycleButtonMarker, DeepLearningResultTextMarker,
				DeepLearningSeparationFileTextMarker, DeepLearningSeparationOpenFileButtonMarker,
				DeepLearningSeparationStartButtonMarker, DeepLearningSmokeTestButtonMarker,
				DeepLearningStatusTextMarker, DeepLearningTranslationConfigTextMarker,
				DeepLearningTranslationFileTextMarker,
				DeepLearningTranslationLanguageCycleButtonMarker,
				DeepLearningTranslationOpenFileButtonMarker,
				DeepLearningTranslationStartButtonMarker, DeepLearningTtsConfigTextMarker,
				DeepLearningTtsFileTextMarker, DeepLearningTtsLanguageCycleButtonMarker,
				DeepLearningTtsOpenFileButtonMarker, DeepLearningTtsSpeedCycleButtonMarker,
				DeepLearningTtsStartButtonMarker, DeepLearningWhisperConfigTextMarker,
				DeepLearningWhisperFileTextMarker, DeepLearningWhisperLanguageCycleButtonMarker,
				DeepLearningWhisperModelCycleButtonMarker, DeepLearningWhisperOpenFileButtonMarker,
				DeepLearningWhisperProgressFillMarker, DeepLearningWhisperProgressTextMarker,
				DeepLearningWhisperStartButtonMarker,
				DeepLearningWhisperTimestampToggleButtonMarker,
			},
			resources::{DeepLearningPageState, DeepLearningPendingTasks, PendingInferenceTask},
		},
	},
};

mod controls;
mod controls_extra;
mod layout;
mod sync;
mod tasks;

pub use controls::{
	handle_smoke_test_click, handle_translation_language_cycle_click,
	handle_translation_open_file_click, handle_translation_start_click,
	handle_whisper_language_cycle_click, handle_whisper_model_cycle_click,
	handle_whisper_open_file_click, handle_whisper_start_click,
	handle_whisper_timestamp_toggle_click,
};
pub use controls_extra::{
	handle_image_generation_model_cycle_click, handle_image_generation_open_file_click,
	handle_image_generation_resolution_cycle_click, handle_image_generation_seed_cycle_click,
	handle_image_generation_start_click, handle_image_generation_steps_cycle_click,
	handle_separation_open_file_click, handle_separation_start_click,
	handle_tts_language_cycle_click, handle_tts_open_file_click, handle_tts_speed_cycle_click,
	handle_tts_start_click,
};
pub use layout::{on_enter, on_exit};
pub use sync::{sync_result_messages, sync_status_messages};
pub use tasks::{handle_task_requests, update_pending_tasks};

fn spawn_action_button<T: Component>(marker: T, label: &str) -> impl Bundle {
	(
		Button,
		marker,
		Node {
			height: Val::Px(38.0),
			padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			..default()
		},
		BackgroundColor(Color::srgb(0.16, 0.39, 0.76)),
		children![(
			Text::new(label),
			TextFont {
				font_size: 14.0,
				..default()
			},
			TextColor(Color::WHITE),
		)],
	)
}

fn spawn_section_card(border: Color, background: Color) -> impl Bundle {
	(
		Node {
			width: Val::Percent(100.0),
			flex_direction: FlexDirection::Column,
			padding: UiRect::all(Val::Px(12.0)),
			row_gap: Val::Px(10.0),
			border: UiRect::all(Val::Px(1.0)),
			..default()
		},
		BorderColor::all(border),
		BackgroundColor(background),
	)
}

fn whisper_config_text(state: &DeepLearningPageState) -> String {
	format!(
		"Whisper 配置：模型={} / 语言提示={} / 原生时间轴={}",
		state.whisper_model.as_label(),
		state.whisper_language_hint.as_label(),
		if state.whisper_with_timestamps {
			"开启"
		} else {
			"关闭"
		}
	)
}

fn translation_config_text(state: &DeepLearningPageState) -> String {
	format!(
		"翻译配置：source={} / target=Chinese",
		state.translation_source_language.as_label()
	)
}

fn tts_config_text(state: &DeepLearningPageState) -> String {
	format!(
		"TTS 配置：language={} / speaker={} / speed={:.1}",
		state.tts_language.as_label(),
		state.tts_speaker,
		state.tts_speed
	)
}

fn image_generation_config_text(state: &DeepLearningPageState) -> String {
	format!(
		"图像生成配置：model={} / size={} / seed={} / steps={}",
		state.image_generation_model.as_label(),
		state.image_generation_resolution.as_label(),
		state.image_generation_seed,
		state.image_generation_steps
	)
}

fn default_data_directory() -> Option<std::path::PathBuf> {
	let directory = std::path::PathBuf::from("data");
	if directory.exists() {
		return Some(directory);
	}
	None
}

fn pick_whisper_input_file() -> Option<std::path::PathBuf> {
	pick_single_file(
		default_data_directory().as_deref(),
		"选择 Whisper 输入文件",
		&[
			("Audio/Video", &["mp3", "wav", "m4a", "mp4"]),
			("All Supported", &["mp3", "wav", "m4a", "mp4"]),
		],
	)
}

fn pick_separation_input_file() -> Option<std::path::PathBuf> {
	pick_single_file(
		default_data_directory().as_deref(),
		"选择人声分离输入文件",
		&[
			("Audio", &["mp3", "wav", "m4a"]),
			("All Supported", &["mp3", "wav", "m4a"]),
		],
	)
}

fn pick_text_input_file(title: &str) -> Option<std::path::PathBuf> {
	pick_single_file(
		None,
		title,
		&[("Text", &["txt", "md"]), ("All Supported", &["txt", "md"])],
	)
}

fn next_whisper_language_hint(current: WhisperLanguageHint) -> WhisperLanguageHint {
	match current {
		WhisperLanguageHint::Auto => WhisperLanguageHint::Chinese,
		WhisperLanguageHint::Chinese => WhisperLanguageHint::Japanese,
		WhisperLanguageHint::Japanese => WhisperLanguageHint::English,
		WhisperLanguageHint::English => WhisperLanguageHint::Auto,
	}
}

fn next_whisper_model(current: WhisperModelKind) -> WhisperModelKind {
	match current {
		WhisperModelKind::Base => WhisperModelKind::LargeV3,
		WhisperModelKind::LargeV3 => WhisperModelKind::Base,
	}
}

fn next_translation_source_language(
	current: TranslationSourceLanguage,
) -> TranslationSourceLanguage {
	match current {
		TranslationSourceLanguage::English => TranslationSourceLanguage::Japanese,
		TranslationSourceLanguage::Japanese => TranslationSourceLanguage::English,
	}
}

fn next_tts_language(current: TtsLanguage) -> TtsLanguage {
	match current {
		TtsLanguage::Chinese => TtsLanguage::Japanese,
		TtsLanguage::Japanese => TtsLanguage::Chinese,
	}
}

fn next_tts_speed(current: f32) -> f32 {
	if current < 1.0 {
		1.0
	} else if current < 1.2 {
		1.2
	} else {
		0.8
	}
}

fn next_image_generation_resolution(
	current: ImageGenerationResolution,
) -> ImageGenerationResolution {
	match current {
		ImageGenerationResolution::Size1024x768 => ImageGenerationResolution::Size1920x1080,
		ImageGenerationResolution::Size1920x1080 => ImageGenerationResolution::Size1024x768,
	}
}

fn next_image_generation_seed(current: u64) -> u64 {
	match current {
		20260322 => 42,
		42 => 777777,
		_ => 20260322,
	}
}

fn next_image_generation_steps(current: u32) -> u32 {
	match current {
		4 => 8,
		8 => 16,
		_ => 4,
	}
}

fn next_image_generation_model(current: ImageGenerationModelKind) -> ImageGenerationModelKind {
	match current {
		ImageGenerationModelKind::SdxlTurbo => ImageGenerationModelKind::SdxlBase,
		ImageGenerationModelKind::SdxlBase => ImageGenerationModelKind::SdxlTurbo,
	}
}

fn update_single_text<T: Component>(query: &mut Query<&mut Text, With<T>>, value: &str) {
	for mut text in query.iter_mut() {
		text.0 = value.to_string();
	}
}

fn create_preview_placeholder_texture(images: &mut Assets<Image>) -> Handle<Image> {
	images.add(Image::new_fill(
		Extent3d {
			width: 32,
			height: 32,
			depth_or_array_layers: 1,
		},
		TextureDimension::D2,
		&[230, 234, 240, 255],
		TextureFormat::Rgba8UnormSrgb,
		RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
	))
}

fn bevy_image_from_dynamic(dynamic_image: image::DynamicImage) -> Image {
	let rgba = dynamic_image.to_rgba8();
	Image::new(
		Extent3d {
			width: rgba.width(),
			height: rgba.height(),
			depth_or_array_layers: 1,
		},
		TextureDimension::D2,
		rgba.into_raw(),
		TextureFormat::Rgba8UnormSrgb,
		RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
	)
}

fn load_preview_texture_from_path(
	handle: &Handle<Image>,
	path: &std::path::Path,
	images: &mut Assets<Image>,
) -> Result<(), String> {
	let dynamic_image = image::open(path).map_err(|error| format!("读取生成 PNG 失败: {error}"))?;

	if let Some(image) = images.get_mut(handle) {
		*image = bevy_image_from_dynamic(dynamic_image);
		return Ok(());
	}

	Err("图片预览纹理句柄不存在".to_string())
}
