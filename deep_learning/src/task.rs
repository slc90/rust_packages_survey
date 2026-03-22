use bevy::prelude::*;

use crate::translation::TranslationRequest;
use crate::tts::TtsRequest;
use crate::whisper::WhisperRequest;

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

	/// 任务负载。
	pub payload: DlTaskPayload,
}

/// 深度学习任务负载。
#[derive(Debug, Clone)]
pub enum DlTaskPayload {
	/// 空任务测试。
	SmokeTest,

	/// 翻译请求。
	Translation(TranslationRequest),

	/// Whisper 请求。
	Whisper(WhisperRequest),

	/// TTS 请求。
	Tts(TtsRequest),
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

	/// 可选输出路径。
	pub output_path: Option<String>,
}
