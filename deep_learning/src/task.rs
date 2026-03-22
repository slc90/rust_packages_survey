use bevy::prelude::*;

/// 深度学习任务类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DlTaskKind {
	/// 空任务测试。
	SmokeTest,

	/// 本地翻译。
	Translation,

	/// 人声分离。
	Separation,

	/// Whisper。
	Whisper,

	/// 图像生成。
	ImageGeneration,

	/// 语音生成。
	Tts,
}

/// 深度学习任务状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DlTaskState {
	/// 已创建。
	Created,

	/// 运行中。
	Running,

	/// 已完成。
	Completed,

	/// 已失败。
	Failed,
}

/// 深度学习任务唯一标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DlTaskId(pub u64);

/// 深度学习任务请求消息。
#[derive(Message, Debug, Clone)]
pub struct DlTaskRequestMessage {
	/// 任务 ID。
	pub id: DlTaskId,

	/// 任务类型。
	pub kind: DlTaskKind,
}

/// 深度学习任务状态消息。
#[derive(Message, Debug, Clone)]
pub struct DlTaskStatusMessage {
	/// 任务 ID。
	pub id: DlTaskId,

	/// 任务类型。
	pub kind: DlTaskKind,

	/// 当前任务状态。
	pub state: DlTaskState,

	/// 当前进度。
	pub progress: f32,

	/// 当前状态文本。
	pub message: String,
}

/// 深度学习任务结果消息。
#[derive(Message, Debug, Clone)]
pub struct DlTaskResultMessage {
	/// 任务 ID。
	pub id: DlTaskId,

	/// 任务类型。
	pub kind: DlTaskKind,

	/// 结果摘要。
	pub summary: String,
}
