use bevy::prelude::*;
use deep_learning::{
	runtime::initialize_runtime_directories,
	task::{
		DlTaskKind, DlTaskPayload, DlTaskRequestMessage, DlTaskResultMessage, DlTaskState,
		DlTaskStatusMessage,
	},
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
				DeepLearningWhisperConfigTextMarker, DeepLearningWhisperFileTextMarker,
				DeepLearningWhisperLanguageCycleButtonMarker,
				DeepLearningWhisperOpenFileButtonMarker, DeepLearningWhisperStartButtonMarker,
				DeepLearningWhisperTimestampToggleButtonMarker,
			},
			resources::{DeepLearningPageState, DeepLearningPendingTasks, PendingMockTask},
		},
	},
};

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

/// 构建 Whisper 配置展示文本。
fn whisper_config_text(state: &DeepLearningPageState) -> String {
	format!(
		"Whisper 配置：language_hint={} / with_timestamps={}",
		state.whisper_language_hint.as_label(),
		state.whisper_with_timestamps
	)
}

/// 获取默认音频目录。
fn default_audio_directory() -> Option<std::path::PathBuf> {
	let directory = std::path::PathBuf::from("data");
	if directory.exists() {
		return Some(directory);
	}
	None
}

/// 选择 Whisper 输入文件。
fn pick_whisper_input_file() -> Option<std::path::PathBuf> {
	pick_single_file(
		default_audio_directory().as_deref(),
		"选择 Whisper 输入文件",
		&[
			("Audio/Video", &["mp3", "wav", "m4a", "mp4"]),
			("All Supported", &["mp3", "wav", "m4a", "mp4"]),
		],
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
	let whisper_config = whisper_config_text(&page_state);

	commands.insert_resource(page_state);
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
						Node {
							width: Val::Percent(100.0),
							column_gap: Val::Px(8.0),
							flex_wrap: FlexWrap::Wrap,
							..default()
						},
						children![
							spawn_action_button(DeepLearningSmokeTestButtonMarker, "空任务测试"),
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

					column.spawn((
						Text::new("深度学习测试页（Phase 2）"),
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

					column.spawn((
						Text::new("Whisper 文件：未选择"),
						TextFont {
							font_size: 16.0,
							..default()
						},
						TextColor(Color::srgb(0.12, 0.12, 0.12)),
						DeepLearningWhisperFileTextMarker,
					));

					column.spawn((
						Text::new(whisper_config),
						TextFont {
							font_size: 16.0,
							..default()
						},
						TextColor(Color::srgb(0.12, 0.12, 0.12)),
						DeepLearningWhisperConfigTextMarker,
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
	mut status_writer: MessageWriter<DlTaskStatusMessage>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let Some(path) = pick_whisper_input_file() else {
			status_writer.write(DlTaskStatusMessage {
				id: deep_learning::task::DlTaskId(0),
				kind: DlTaskKind::Whisper,
				state: DlTaskState::Created,
				progress: 0.0,
				message: "已取消 Whisper 文件选择".to_string(),
			});
			continue;
		};

		state.whisper_input_file = Some(path.clone());
		for mut text in &mut text_query {
			text.0 = format!("Whisper 文件：{}", path.display());
		}
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
		let config_text = whisper_config_text(&state);
		for mut text in &mut text_query {
			text.0 = config_text.clone();
		}
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
		let config_text = whisper_config_text(&state);
		for mut text in &mut text_query {
			text.0 = config_text.clone();
		}
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
				id: deep_learning::task::DlTaskId(0),
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
				if let Err(error) = ensure_whisper_model_ready() {
					status_writer.write(DlTaskStatusMessage {
						id: message.id,
						kind: message.kind,
						state: DlTaskState::Failed,
						progress: 0.0,
						message: format!("Whisper 模型预检失败: {error}"),
					});
					result_writer.write(DlTaskResultMessage {
						id: message.id,
						kind: message.kind,
						summary: "Whisper 模型目录或权重缺失".to_string(),
						output_path: None,
					});
					continue;
				}

				let output_path = match save_whisper_request_snapshot(request) {
					Ok(path) => path,
					Err(error) => {
						status_writer.write(DlTaskStatusMessage {
							id: message.id,
							kind: message.kind,
							state: DlTaskState::Failed,
							progress: 0.0,
							message: format!("Whisper 任务快照写出失败: {error}"),
						});
						result_writer.write(DlTaskResultMessage {
							id: message.id,
							kind: message.kind,
							summary: "Whisper 任务快照写出失败".to_string(),
							output_path: None,
						});
						continue;
					}
				};

				status_writer.write(DlTaskStatusMessage {
					id: message.id,
					kind: message.kind,
					state: DlTaskState::Running,
					progress: 0.4,
					message: format!("Whisper 任务 {} 已完成模型预检", message.id.0),
				});
				pending_tasks.tasks.push(PendingMockTask {
					id: message.id,
					kind: message.kind,
					timer: Timer::from_seconds(0.3, TimerMode::Once),
					summary: Some("Whisper Phase 2 预检完成，已生成任务快照".to_string()),
					output_path: Some(output_path.display().to_string()),
				});
			}
		}
	}
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

	for mut text in &mut text_query {
		text.0 = state.status_text.clone();
	}
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

	for mut text in &mut text_query {
		text.0 = state.result_text.clone();
	}
}
