use bevy::prelude::*;
use deep_learning::{
	runtime::initialize_runtime_directories,
	task::{
		DlTaskKind, DlTaskRequestMessage, DlTaskResultMessage, DlTaskState, DlTaskStatusMessage,
	},
};

use crate::homepage::{
	common::ContentAreaMarker,
	deep_learning::{
		components::{
			DeepLearningContentMarker, DeepLearningResultTextMarker,
			DeepLearningSmokeTestButtonMarker, DeepLearningStatusTextMarker,
		},
		resources::{DeepLearningPageState, DeepLearningPendingTasks, PendingMockTask},
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

	commands.insert_resource(DeepLearningPageState::new(&directories));
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
							..default()
						},
						children![spawn_action_button(
							DeepLearningSmokeTestButtonMarker,
							"空任务测试",
						)],
					));

					column.spawn((
						Text::new("深度学习测试页（Phase 1）"),
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
		});
	}
}

/// 处理深度学习任务请求。
pub fn handle_task_requests(
	mut messages: MessageReader<DlTaskRequestMessage>,
	mut pending_tasks: ResMut<DeepLearningPendingTasks>,
	mut status_writer: MessageWriter<DlTaskStatusMessage>,
) {
	for message in messages.read() {
		status_writer.write(DlTaskStatusMessage {
			id: message.id,
			kind: message.kind,
			state: DlTaskState::Created,
			progress: 0.0,
			message: format!("任务 {} 已创建", message.id.0),
		});
		status_writer.write(DlTaskStatusMessage {
			id: message.id,
			kind: message.kind,
			state: DlTaskState::Running,
			progress: 0.2,
			message: format!("任务 {} 正在模拟执行", message.id.0),
		});
		pending_tasks.tasks.push(PendingMockTask {
			id: message.id,
			timer: Timer::from_seconds(0.5, TimerMode::Once),
		});
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
				kind: DlTaskKind::SmokeTest,
				state: DlTaskState::Completed,
				progress: 1.0,
				message: format!("任务 {} 已完成", task.id.0),
			});
			result_writer.write(DlTaskResultMessage {
				id: task.id,
				kind: DlTaskKind::SmokeTest,
				summary: "Phase 1 消息链路已打通".to_string(),
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

	state.result_text = format!(
		"结果：任务 {} / {:?} / {}",
		last_message.id.0, last_message.kind, last_message.summary
	);

	for mut text in &mut text_query {
		text.0 = state.result_text.clone();
	}
}
