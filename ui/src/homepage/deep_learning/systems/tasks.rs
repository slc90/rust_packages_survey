use super::*;
use futures_lite::future;
use std::time::Instant;

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
			progress: 0.1,
			message: format!("任务 {} 已提交后台推理队列", message.id.0),
		});

		let payload = message.payload.clone();
		let task = AsyncComputeTaskPool::get().spawn(async move { execute_task(payload) });
		pending_tasks.tasks.push(PendingInferenceTask {
			id: message.id,
			kind: message.kind,
			task,
			started_at: Instant::now(),
		});
	}
}

/// 轮询后台推理任务。
pub fn update_pending_tasks(
	mut pending_tasks: ResMut<DeepLearningPendingTasks>,
	mut status_writer: MessageWriter<DlTaskStatusMessage>,
	mut result_writer: MessageWriter<DlTaskResultMessage>,
) {
	let mut finished_ids = Vec::new();

	for task in &mut pending_tasks.tasks {
		let Some(result) = future::block_on(future::poll_once(&mut task.task)) else {
			if task.kind == DlTaskKind::Whisper {
				let elapsed_seconds = task.started_at.elapsed().as_secs_f32();
				let progress = (0.15 + elapsed_seconds / 20.0).clamp(0.15, 0.95);
				status_writer.write(DlTaskStatusMessage {
					id: task.id,
					kind: task.kind,
					state: DlTaskState::Running,
					progress,
					message: format!("任务 {} 正在执行 Whisper 推理", task.id.0),
				});
			}
			continue;
		};

		match result {
			Ok(output) => {
				status_writer.write(DlTaskStatusMessage {
					id: task.id,
					kind: task.kind,
					state: DlTaskState::Completed,
					progress: 1.0,
					message: format!("任务 {} 已完成真实推理", task.id.0),
				});
				result_writer.write(DlTaskResultMessage {
					id: task.id,
					kind: task.kind,
					summary: output.summary,
					output_path: output.output_path.map(|path| path.display().to_string()),
				});
			}
			Err(error) => {
				status_writer.write(DlTaskStatusMessage {
					id: task.id,
					kind: task.kind,
					state: DlTaskState::Failed,
					progress: 0.0,
					message: format!("任务 {} 执行失败: {error}", task.id.0),
				});
				result_writer.write(DlTaskResultMessage {
					id: task.id,
					kind: task.kind,
					summary: format!("推理失败: {error}"),
					output_path: None,
				});
			}
		}

		finished_ids.push(task.id);
	}

	pending_tasks
		.tasks
		.retain(|task| !finished_ids.contains(&task.id));
}
