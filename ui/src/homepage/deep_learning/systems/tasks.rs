use super::*;

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
			DlTaskPayload::Separation(request) => {
				handle_preflight_task(
					PreflightTaskArgs {
						id: message.id,
						kind: message.kind,
						model_ready: ensure_separation_model_ready(),
						output_path: save_separation_request_snapshot(request)
							.map(|path| path.display().to_string()),
						success_summary: "Phase 4 人声分离预检完成，已生成任务快照",
						model_error_summary: "人声分离模型目录或权重缺失",
						output_error_summary: "人声分离任务快照写出失败",
					},
					&mut pending_tasks,
					&mut status_writer,
					&mut result_writer,
				);
			}
			DlTaskPayload::ImageGeneration(request) => {
				handle_preflight_task(
					PreflightTaskArgs {
						id: message.id,
						kind: message.kind,
						model_ready: ensure_image_generation_model_ready(request.model),
						output_path: prepare_image_generation_output(request)
							.map(|path| path.display().to_string()),
						success_summary: "Phase 5 图像生成预检完成，已输出 PNG 预览图",
						model_error_summary: "图像生成模型目录或权重缺失",
						output_error_summary: "图像生成 PNG 输出失败",
					},
					&mut pending_tasks,
					&mut status_writer,
					&mut result_writer,
				);
			}
		}
	}
}

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
				message: format!("{:?} 输出失败: {error}", args.kind),
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
