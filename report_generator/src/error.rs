use thiserror::Error;

/// 报告导出统一错误类型。
#[derive(Debug, Error)]
pub enum ReportError {
	#[error("报告内容无效: {0}")]
	InvalidDocument(String),
	#[error("IO 错误: {0}")]
	Io(#[from] std::io::Error),
	#[error("DOCX 导出失败: {0}")]
	Docx(String),
	#[error("PDF 导出失败: {0}")]
	Pdf(String),
	#[error("图片读取失败: {0}")]
	Image(String),
}
