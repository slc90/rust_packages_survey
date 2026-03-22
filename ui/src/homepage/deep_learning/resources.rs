use bevy::prelude::*;
use deep_learning::{runtime::RuntimeDirectories, task::DlTaskId};

/// 深度学习测试页状态。
#[derive(Resource, Debug, Clone)]
pub struct DeepLearningPageState {
	/// 模型根目录。
	pub model_root: String,

	/// 输出根目录。
	pub output_root: String,

	/// 当前状态文本。
	pub status_text: String,

	/// 当前结果文本。
	pub result_text: String,

	/// 下一个任务 ID。
	pub next_task_id: u64,
}

impl DeepLearningPageState {
	/// 根据运行时目录创建页面状态。
	pub fn new(directories: &RuntimeDirectories) -> Self {
		Self {
			model_root: directories.model_root.display().to_string(),
			output_root: directories.output_root.display().to_string(),
			status_text: "等待任务".to_string(),
			result_text: "暂无结果".to_string(),
			next_task_id: 1,
		}
	}

	/// 生成新的任务 ID。
	pub fn allocate_task_id(&mut self) -> DlTaskId {
		let task_id = DlTaskId(self.next_task_id);
		self.next_task_id += 1;
		task_id
	}
}

/// 模拟中的后台任务。
#[derive(Debug, Clone)]
pub struct PendingMockTask {
	/// 任务 ID。
	pub id: DlTaskId,

	/// 完成倒计时。
	pub timer: Timer,
}

/// 模拟任务队列资源。
#[derive(Resource, Default, Debug)]
pub struct DeepLearningPendingTasks {
	/// 当前待完成任务列表。
	pub tasks: Vec<PendingMockTask>,
}
