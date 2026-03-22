use bevy::prelude::*;
use deep_learning::{
	runtime::initialize_runtime_directories,
	task::{
		DlTaskId, DlTaskKind, DlTaskPayload, DlTaskRequestMessage, DlTaskResultMessage,
		DlTaskState, DlTaskStatusMessage,
	},
	translation::{
		TranslationSourceLanguage, ensure_translation_model_ready,
		save_translation_request_snapshot,
	},
	tts::{TtsLanguage, ensure_tts_model_ready, save_tts_request_snapshot},
	whisper::{WhisperLanguageHint, ensure_whisper_model_ready, save_whisper_request_snapshot},
};

use crate::{
	file_dialog::pick_single_file,
	homepage::{
		common::ContentAreaMarker,
		deep_learning::{
			components::{
				DeepLearningContentMarker, DeepLearningResultTextMarker,
				DeepLearningSmokeTestButtonMarker, DeepLearningStatusTextMarker,
				DeepLearningTranslationConfigTextMarker, DeepLearningTranslationFileTextMarker,
				DeepLearningTranslationLanguageCycleButtonMarker,
				DeepLearningTranslationOpenFileButtonMarker,
				DeepLearningTranslationStartButtonMarker, DeepLearningTtsConfigTextMarker,
				DeepLearningTtsFileTextMarker, DeepLearningTtsLanguageCycleButtonMarker,
				DeepLearningTtsOpenFileButtonMarker, DeepLearningTtsSpeedCycleButtonMarker,
				DeepLearningTtsStartButtonMarker, DeepLearningWhisperConfigTextMarker,
				DeepLearningWhisperFileTextMarker, DeepLearningWhisperLanguageCycleButtonMarker,
				DeepLearningWhisperOpenFileButtonMarker, DeepLearningWhisperStartButtonMarker,
				DeepLearningWhisperTimestampToggleButtonMarker,
			},
			resources::{DeepLearningPageState, DeepLearningPendingTasks, PendingMockTask},
		},
	},
};

/// 预检任务参数。
struct PreflightTaskArgs {
	/// 任务 ID。
	id: DlTaskId,

	/// 任务类型。
	kind: DlTaskKind,

	/// 模型预检结果。
	model_ready:
		Result<deep_learning::model::ModelDescriptor, deep_learning::error::DeepLearningError>,

	/// 快照输出路径结果。
	output_path: Result<String, deep_learning::error::DeepLearningError>,

	/// 成功摘要。
	success_summary: &'static str,

	/// 模型错误摘要。
	model_error_summary: &'static str,

	/// 输出错误摘要。
	output_error_summary: &'static str,
}

/// 创建操作按钮。
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

/// 构建 Whisper 配置文本。
fn whisper_config_text(state: &DeepLearningPageState) -> String {
	format!(
		"Whisper 配置：language_hint={} / with_timestamps={}",
		state.whisper_language_hint.as_label(),
		state.whisper_with_timestamps
	)
}

/// 构建翻译配置文本。
fn translation_config_text(state: &DeepLearningPageState) -> String {
	format!(
		"翻译配置：source={} / target=Chinese",
		state.translation_source_language.as_label()
	)
}

/// 构建 TTS 配置文本。
fn tts_config_text(state: &DeepLearningPageState) -> String {
	format!(
		"TTS 配置：language={} / speaker={} / speed={:.1}",
		state.tts_language.as_label(),
		state.tts_speaker,
		state.tts_speed
	)
}

/// 获取默认数据目录。
fn default_data_directory() -> Option<std::path::PathBuf> {
	let directory = std::path::PathBuf::from("data");
	if directory.exists() {
		return Some(directory);
	}
	None
}

/// 选择 Whisper 输入文件。
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

/// 选择文本文件。
fn pick_text_input_file(title: &str) -> Option<std::path::PathBuf> {
	pick_single_file(
		None,
		title,
		&[("Text", &["txt", "md"]), ("All Supported", &["txt", "md"])],
	)
}

/// 切换 Whisper 语言提示。
fn next_whisper_language_hint(current: WhisperLanguageHint) -> WhisperLanguageHint {
	match current {
		WhisperLanguageHint::Auto => WhisperLanguageHint::Chinese,
		WhisperLanguageHint::Chinese => WhisperLanguageHint::Japanese,
		WhisperLanguageHint::Japanese => WhisperLanguageHint::English,
		WhisperLanguageHint::English => WhisperLanguageHint::Auto,
	}
}

/// 切换翻译源语言。
fn next_translation_source_language(
	current: TranslationSourceLanguage,
) -> TranslationSourceLanguage {
	match current {
		TranslationSourceLanguage::English => TranslationSourceLanguage::Japanese,
		TranslationSourceLanguage::Japanese => TranslationSourceLanguage::English,
	}
}

/// 切换 TTS 语言。
fn next_tts_language(current: TtsLanguage) -> TtsLanguage {
	match current {
		TtsLanguage::Chinese => TtsLanguage::Japanese,
		TtsLanguage::Japanese => TtsLanguage::Chinese,
	}
}

/// 切换 TTS 语速。
fn next_tts_speed(current: f32) -> f32 {
	if current < 1.0 {
		1.0
	} else if current < 1.2 {
		1.2
	} else {
		0.8
	}
}

/// 在页面写入指定文本。
fn update_single_text<T: Component>(query: &mut Query<&mut Text, With<T>>, value: &str) {
	for mut text in query.iter_mut() {
		text.0 = value.to_string();
	}
}

/// 进入深度学习测试页。
pub fn on_enter(
	mut commands: Commands,
	content_area_query: Query<Entity, With<ContentAreaMarker>>,
) {
	info!("进入深度学习测试页面");

	let directories = match initialize_runtime_directories() {
		Ok(directories) => directories,
		Err(error) => {
			error!("初始化深度学习目录失败: {error}");
			return;
		}
	};

	let page_state = DeepLearningPageState::new(&directories);

	commands.insert_resource(page_state.clone());
	commands.insert_resource(DeepLearningPendingTasks::default());

	if let Ok(content_area) = content_area_query.single() {
		commands.entity(content_area).with_children(|parent| {
			parent
				.spawn((
					DeepLearningContentMarker,
					Node {
						width: Val::Percent(100.0),
						height: Val::Percent(100.0),
						flex_direction: FlexDirection::Column,
						padding: UiRect::all(Val::Px(16.0)),
						row_gap: Val::Px(16.0),
						..default()
					},
					BackgroundColor(Color::srgb(0.95, 0.96, 0.97)),
				))
				.with_children(|column| {
					column.spawn((
						Text::new("深度学习测试页（Phase 3）"),
						TextFont {
							font_size: 24.0,
							..default()
						},
						TextColor(Color::BLACK),
					));

					column.spawn((
						Text::new(format!(
							"模型目录：{}\n输出目录：{}",
							directories.model_root.display(),
							directories.output_root.display()
						)),
						TextFont {
							font_size: 16.0,
							..default()
						},
						TextColor(Color::srgb(0.18, 0.18, 0.18)),
					));

					column
						.spawn((
							Node {
								width: Val::Percent(100.0),
								flex_direction: FlexDirection::Column,
								padding: UiRect::all(Val::Px(12.0)),
								row_gap: Val::Px(10.0),
								border: UiRect::all(Val::Px(1.0)),
								..default()
							},
							BorderColor::all(Color::srgb(0.62, 0.72, 0.82)),
							BackgroundColor(Color::srgb(0.91, 0.96, 0.99)),
						))
						.with_children(|section| {
							section.spawn((
								Text::new("Whisper"),
								TextFont {
									font_size: 18.0,
									..default()
								},
								TextColor(Color::BLACK),
							));
							section.spawn((
								Node {
									width: Val::Percent(100.0),
									column_gap: Val::Px(8.0),
									flex_wrap: FlexWrap::Wrap,
									..default()
								},
								children![
									spawn_action_button(
										DeepLearningWhisperOpenFileButtonMarker,
										"Whisper 选择文件",
									),
									spawn_action_button(
										DeepLearningWhisperLanguageCycleButtonMarker,
										"Whisper 切换语言",
									),
									spawn_action_button(
										DeepLearningWhisperTimestampToggleButtonMarker,
										"Whisper 时间戳开关",
									),
									spawn_action_button(
										DeepLearningWhisperStartButtonMarker,
										"Whisper 开始任务",
									),
								],
							));
							section.spawn((
								Text::new("Whisper 文件：未选择"),
								TextFont {
									font_size: 16.0,
									..default()
								},
								TextColor(Color::srgb(0.12, 0.12, 0.12)),
								DeepLearningWhisperFileTextMarker,
							));
							section.spawn((
								Text::new(whisper_config_text(&page_state)),
								TextFont {
									font_size: 16.0,
									..default()
								},
								TextColor(Color::srgb(0.12, 0.12, 0.12)),
								DeepLearningWhisperConfigTextMarker,
							));
						});

					column
						.spawn((
							Node {
								width: Val::Percent(100.0),
								flex_direction: FlexDirection::Column,
								padding: UiRect::all(Val::Px(12.0)),
								row_gap: Val::Px(10.0),
								border: UiRect::all(Val::Px(1.0)),
								..default()
							},
							BorderColor::all(Color::srgb(0.73, 0.68, 0.56)),
							BackgroundColor(Color::srgb(0.98, 0.96, 0.90)),
						))
						.with_children(|section| {
							section.spawn((
								Text::new("本地翻译"),
								TextFont {
									font_size: 18.0,
									..default()
								},
								TextColor(Color::BLACK),
							));
							section.spawn((
								Node {
									width: Val::Percent(100.0),
									column_gap: Val::Px(8.0),
									flex_wrap: FlexWrap::Wrap,
									..default()
								},
								children![
									spawn_action_button(
										DeepLearningTranslationOpenFileButtonMarker,
										"翻译 选择文本",
									),
									spawn_action_button(
										DeepLearningTranslationLanguageCycleButtonMarker,
										"翻译 切换源语言",
									),
									spawn_action_button(
										DeepLearningTranslationStartButtonMarker,
										"翻译 开始任务",
									),
								],
							));
							section.spawn((
								Text::new("翻译文件：未选择"),
								TextFont {
									font_size: 16.0,
									..default()
								},
								TextColor(Color::srgb(0.12, 0.12, 0.12)),
								DeepLearningTranslationFileTextMarker,
							));
							section.spawn((
								Text::new(translation_config_text(&page_state)),
								TextFont {
									font_size: 16.0,
									..default()
								},
								TextColor(Color::srgb(0.12, 0.12, 0.12)),
								DeepLearningTranslationConfigTextMarker,
							));
						});

					column
						.spawn((
							Node {
								width: Val::Percent(100.0),
								flex_direction: FlexDirection::Column,
								padding: UiRect::all(Val::Px(12.0)),
								row_gap: Val::Px(10.0),
								border: UiRect::all(Val::Px(1.0)),
								..default()
							},
							BorderColor::all(Color::srgb(0.59, 0.68, 0.60)),
							BackgroundColor(Color::srgb(0.93, 0.98, 0.94)),
						))
						.with_children(|section| {
							section.spawn((
								Text::new("语音生成"),
								TextFont {
									font_size: 18.0,
									..default()
								},
								TextColor(Color::BLACK),
							));
							section.spawn((
								Node {
									width: Val::Percent(100.0),
									column_gap: Val::Px(8.0),
									flex_wrap: FlexWrap::Wrap,
									..default()
								},
								children![
									spawn_action_button(
										DeepLearningTtsOpenFileButtonMarker,
										"TTS 选择文本",
									),
									spawn_action_button(
										DeepLearningTtsLanguageCycleButtonMarker,
										"TTS 切换语言",
									),
									spawn_action_button(
										DeepLearningTtsSpeedCycleButtonMarker,
										"TTS 切换语速",
									),
									spawn_action_button(
										DeepLearningTtsStartButtonMarker,
										"TTS 开始任务",
									),
								],
							));
							section.spawn((
								Text::new("TTS 文件：未选择"),
								TextFont {
									font_size: 16.0,
									..default()
								},
								TextColor(Color::srgb(0.12, 0.12, 0.12)),
								DeepLearningTtsFileTextMarker,
							));
							section.spawn((
								Text::new(tts_config_text(&page_state)),
								TextFont {
									font_size: 16.0,
									..default()
								},
								TextColor(Color::srgb(0.12, 0.12, 0.12)),
								DeepLearningTtsConfigTextMarker,
							));
						});

					column.spawn((
						Node {
							width: Val::Percent(100.0),
							column_gap: Val::Px(8.0),
							..default()
						},
						children![spawn_action_button(
							DeepLearningSmokeTestButtonMarker,
							"空任务测试",
						)],
					));

					column.spawn((
						Text::new("状态：等待任务"),
						TextFont {
							font_size: 16.0,
							..default()
						},
						TextColor(Color::srgb(0.14, 0.14, 0.14)),
						DeepLearningStatusTextMarker,
					));

					column.spawn((
						Text::new("结果：暂无结果"),
						TextFont {
							font_size: 16.0,
							..default()
						},
						TextColor(Color::srgb(0.14, 0.14, 0.14)),
						DeepLearningResultTextMarker,
					));
				});
		});
	}
}

/// 离开深度学习测试页。
pub fn on_exit(
	mut commands: Commands,
	content_query: Query<Entity, With<DeepLearningContentMarker>>,
) {
	info!("离开深度学习测试页面");

	for entity in &content_query {
		commands.entity(entity).despawn();
	}

	commands.remove_resource::<DeepLearningPageState>();
	commands.remove_resource::<DeepLearningPendingTasks>();
}

/// 处理空任务测试按钮。
pub fn handle_smoke_test_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningSmokeTestButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut writer: MessageWriter<DlTaskRequestMessage>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let task_id = state.allocate_task_id();
		writer.write(DlTaskRequestMessage {
			id: task_id,
			kind: DlTaskKind::SmokeTest,
			payload: DlTaskPayload::SmokeTest,
		});
	}
}

/// 处理 Whisper 选择文件。
pub fn handle_whisper_open_file_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningWhisperOpenFileButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningWhisperFileTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(path) = pick_whisper_input_file() else {
			continue;
		};

		state.whisper_input_file = Some(path.clone());
		update_single_text(
			&mut text_query,
			&format!("Whisper 文件：{}", path.display()),
		);
	}
}

/// 处理 Whisper 语言切换。
pub fn handle_whisper_language_cycle_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningWhisperLanguageCycleButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningWhisperConfigTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		state.whisper_language_hint = next_whisper_language_hint(state.whisper_language_hint);
		let text = whisper_config_text(&state);
		update_single_text(&mut text_query, &text);
	}
}

/// 处理 Whisper 时间戳开关。
pub fn handle_whisper_timestamp_toggle_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningWhisperTimestampToggleButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningWhisperConfigTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		state.whisper_with_timestamps = !state.whisper_with_timestamps;
		let text = whisper_config_text(&state);
		update_single_text(&mut text_query, &text);
	}
}

/// 处理 Whisper 开始按钮。
pub fn handle_whisper_start_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningWhisperStartButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut writer: MessageWriter<DlTaskRequestMessage>,
	mut status_writer: MessageWriter<DlTaskStatusMessage>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(request) = state.build_whisper_request() else {
			status_writer.write(DlTaskStatusMessage {
				id: DlTaskId(0),
				kind: DlTaskKind::Whisper,
				state: DlTaskState::Failed,
				progress: 0.0,
				message: "请先选择 Whisper 输入文件".to_string(),
			});
			continue;
		};

		let task_id = state.allocate_task_id();
		writer.write(DlTaskRequestMessage {
			id: task_id,
			kind: DlTaskKind::Whisper,
			payload: DlTaskPayload::Whisper(request),
		});
	}
}

/// 处理翻译选择文件。
pub fn handle_translation_open_file_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningTranslationOpenFileButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningTranslationFileTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(path) = pick_text_input_file("选择翻译文本文件") else {
			continue;
		};

		state.translation_input_file = Some(path.clone());
		update_single_text(&mut text_query, &format!("翻译文件：{}", path.display()));
	}
}

/// 处理翻译语言切换。
pub fn handle_translation_language_cycle_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningTranslationLanguageCycleButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningTranslationConfigTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		state.translation_source_language =
			next_translation_source_language(state.translation_source_language);
		let text = translation_config_text(&state);
		update_single_text(&mut text_query, &text);
	}
}

/// 处理翻译开始按钮。
pub fn handle_translation_start_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningTranslationStartButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut writer: MessageWriter<DlTaskRequestMessage>,
	mut status_writer: MessageWriter<DlTaskStatusMessage>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(request) = state.build_translation_request() else {
			status_writer.write(DlTaskStatusMessage {
				id: DlTaskId(0),
				kind: DlTaskKind::Translation,
				state: DlTaskState::Failed,
				progress: 0.0,
				message: "请先选择翻译文本文件".to_string(),
			});
			continue;
		};

		let task_id = state.allocate_task_id();
		writer.write(DlTaskRequestMessage {
			id: task_id,
			kind: DlTaskKind::Translation,
			payload: DlTaskPayload::Translation(request),
		});
	}
}

/// 处理 TTS 选择文件。
pub fn handle_tts_open_file_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningTtsOpenFileButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningTtsFileTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(path) = pick_text_input_file("选择 TTS 文本文件") else {
			continue;
		};

		state.tts_input_file = Some(path.clone());
		update_single_text(&mut text_query, &format!("TTS 文件：{}", path.display()));
	}
}

/// 处理 TTS 语言切换。
pub fn handle_tts_language_cycle_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningTtsLanguageCycleButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningTtsConfigTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		state.tts_language = next_tts_language(state.tts_language);
		let text = tts_config_text(&state);
		update_single_text(&mut text_query, &text);
	}
}

/// 处理 TTS 语速切换。
pub fn handle_tts_speed_cycle_click(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<DeepLearningTtsSpeedCycleButtonMarker>,
		),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningTtsConfigTextMarker>>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		state.tts_speed = next_tts_speed(state.tts_speed);
		let text = tts_config_text(&state);
		update_single_text(&mut text_query, &text);
	}
}

/// 处理 TTS 开始按钮。
pub fn handle_tts_start_click(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<DeepLearningTtsStartButtonMarker>),
	>,
	mut state: ResMut<DeepLearningPageState>,
	mut writer: MessageWriter<DlTaskRequestMessage>,
	mut status_writer: MessageWriter<DlTaskStatusMessage>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(request) = state.build_tts_request() else {
			status_writer.write(DlTaskStatusMessage {
				id: DlTaskId(0),
				kind: DlTaskKind::Tts,
				state: DlTaskState::Failed,
				progress: 0.0,
				message: "请先选择 TTS 文本文件".to_string(),
			});
			continue;
		};

		let task_id = state.allocate_task_id();
		writer.write(DlTaskRequestMessage {
			id: task_id,
			kind: DlTaskKind::Tts,
			payload: DlTaskPayload::Tts(request),
		});
	}
}

/// 处理深度学习任务请求。
pub fn handle_task_requests(
	mut messages: MessageReader<DlTaskRequestMessage>,
	mut pending_tasks: ResMut<DeepLearningPendingTasks>,
	mut status_writer: MessageWriter<DlTaskStatusMessage>,
	mut result_writer: MessageWriter<DlTaskResultMessage>,
) {
	for message in messages.read() {
		status_writer.write(DlTaskStatusMessage {
			id: message.id,
			kind: message.kind,
			state: DlTaskState::Created,
			progress: 0.0,
			message: format!("任务 {} 已创建", message.id.0),
		});

		match &message.payload {
			DlTaskPayload::SmokeTest => {
				status_writer.write(DlTaskStatusMessage {
					id: message.id,
					kind: message.kind,
					state: DlTaskState::Running,
					progress: 0.2,
					message: format!("任务 {} 正在模拟执行", message.id.0),
				});
				pending_tasks.tasks.push(PendingMockTask {
					id: message.id,
					kind: message.kind,
					timer: Timer::from_seconds(0.5, TimerMode::Once),
					summary: Some("Phase 1 消息链路已打通".to_string()),
					output_path: None,
				});
			}
			DlTaskPayload::Whisper(request) => {
				handle_preflight_task(
					PreflightTaskArgs {
						id: message.id,
						kind: message.kind,
						model_ready: ensure_whisper_model_ready(),
						output_path: save_whisper_request_snapshot(request)
							.map(|path| path.display().to_string()),
						success_summary: "Whisper Phase 2 预检完成，已生成任务快照",
						model_error_summary: "Whisper 模型目录或权重缺失",
						output_error_summary: "Whisper 任务快照写出失败",
					},
					&mut pending_tasks,
					&mut status_writer,
					&mut result_writer,
				);
			}
			DlTaskPayload::Translation(request) => {
				handle_preflight_task(
					PreflightTaskArgs {
						id: message.id,
						kind: message.kind,
						model_ready: ensure_translation_model_ready(),
						output_path: save_translation_request_snapshot(request)
							.map(|path| path.display().to_string()),
						success_summary: "Phase 3 翻译预检完成，已生成任务快照",
						model_error_summary: "翻译模型目录或权重缺失",
						output_error_summary: "翻译任务快照写出失败",
					},
					&mut pending_tasks,
					&mut status_writer,
					&mut result_writer,
				);
			}
			DlTaskPayload::Tts(request) => {
				handle_preflight_task(
					PreflightTaskArgs {
						id: message.id,
						kind: message.kind,
						model_ready: ensure_tts_model_ready(),
						output_path: save_tts_request_snapshot(request)
							.map(|path| path.display().to_string()),
						success_summary: "Phase 3 TTS 预检完成，已生成任务快照",
						model_error_summary: "TTS 模型目录或权重缺失",
						output_error_summary: "TTS 任务快照写出失败",
					},
					&mut pending_tasks,
					&mut status_writer,
					&mut result_writer,
				);
			}
		}
	}
}

/// 统一处理预检型任务。
fn handle_preflight_task(
	args: PreflightTaskArgs,
	pending_tasks: &mut ResMut<DeepLearningPendingTasks>,
	status_writer: &mut MessageWriter<DlTaskStatusMessage>,
	result_writer: &mut MessageWriter<DlTaskResultMessage>,
) {
	if let Err(error) = args.model_ready {
		status_writer.write(DlTaskStatusMessage {
			id: args.id,
			kind: args.kind,
			state: DlTaskState::Failed,
			progress: 0.0,
			message: format!("{:?} 模型预检失败: {error}", args.kind),
		});
		result_writer.write(DlTaskResultMessage {
			id: args.id,
			kind: args.kind,
			summary: args.model_error_summary.to_string(),
			output_path: None,
		});
		return;
	}

	let output_path = match args.output_path {
		Ok(path) => path,
		Err(error) => {
			status_writer.write(DlTaskStatusMessage {
				id: args.id,
				kind: args.kind,
				state: DlTaskState::Failed,
				progress: 0.0,
				message: format!("{:?} 任务快照写出失败: {error}", args.kind),
			});
			result_writer.write(DlTaskResultMessage {
				id: args.id,
				kind: args.kind,
				summary: args.output_error_summary.to_string(),
				output_path: None,
			});
			return;
		}
	};

	status_writer.write(DlTaskStatusMessage {
		id: args.id,
		kind: args.kind,
		state: DlTaskState::Running,
		progress: 0.4,
		message: format!("{:?} 任务 {} 已完成模型预检", args.kind, args.id.0),
	});
	pending_tasks.tasks.push(PendingMockTask {
		id: args.id,
		kind: args.kind,
		timer: Timer::from_seconds(0.3, TimerMode::Once),
		summary: Some(args.success_summary.to_string()),
		output_path: Some(output_path),
	});
}

/// 推进模拟任务。
pub fn update_pending_tasks(
	time: Res<Time>,
	mut pending_tasks: ResMut<DeepLearningPendingTasks>,
	mut status_writer: MessageWriter<DlTaskStatusMessage>,
	mut result_writer: MessageWriter<DlTaskResultMessage>,
) {
	let mut finished_ids = Vec::new();

	for task in &mut pending_tasks.tasks {
		task.timer.tick(time.delta());
		if task.timer.is_finished() {
			status_writer.write(DlTaskStatusMessage {
				id: task.id,
				kind: task.kind,
				state: DlTaskState::Completed,
				progress: 1.0,
				message: format!("任务 {} 已完成", task.id.0),
			});
			result_writer.write(DlTaskResultMessage {
				id: task.id,
				kind: task.kind,
				summary: task
					.summary
					.clone()
					.unwrap_or_else(|| "任务已完成".to_string()),
				output_path: task.output_path.clone(),
			});
			finished_ids.push(task.id);
		}
	}

	pending_tasks
		.tasks
		.retain(|task| !finished_ids.contains(&task.id));
}

/// 同步状态消息到页面。
pub fn sync_status_messages(
	mut messages: MessageReader<DlTaskStatusMessage>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningStatusTextMarker>>,
) {
	let Some(last_message) = messages.read().last().cloned() else {
		return;
	};

	state.status_text = format!(
		"状态：任务 {} / {:?} / {:.0}% / {}",
		last_message.id.0,
		last_message.state,
		last_message.progress * 100.0,
		last_message.message
	);
	update_single_text(&mut text_query, &state.status_text.clone());
}

/// 同步结果消息到页面。
pub fn sync_result_messages(
	mut messages: MessageReader<DlTaskResultMessage>,
	mut state: ResMut<DeepLearningPageState>,
	mut text_query: Query<&mut Text, With<DeepLearningResultTextMarker>>,
) {
	let Some(last_message) = messages.read().last().cloned() else {
		return;
	};

	state.result_text = match &last_message.output_path {
		Some(path) => format!(
			"结果：任务 {} / {:?} / {} / 输出：{}",
			last_message.id.0, last_message.kind, last_message.summary, path
		),
		None => format!(
			"结果：任务 {} / {:?} / {}",
			last_message.id.0, last_message.kind, last_message.summary
		),
	};

	update_single_text(&mut text_query, &state.result_text.clone());
}
