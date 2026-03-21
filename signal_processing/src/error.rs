use thiserror::Error;

/// 信号处理统一错误类型。
#[derive(Debug, Error)]
pub enum SignalProcessError {
	#[error("参数无效: {0}")]
	InvalidArgument(String),
	#[error("输入数据为空")]
	EmptyInput,
	#[error("IO 错误: {0}")]
	Io(#[from] std::io::Error),
}
