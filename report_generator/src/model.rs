use std::path::PathBuf;

/// 统一报告数据模型。
#[derive(Debug, Clone)]
pub struct ReportDocument {
	pub title: String,
	pub subtitle: Option<String>,
	pub sections: Vec<ReportSection>,
}

/// 报告章节。
#[derive(Debug, Clone)]
pub struct ReportSection {
	pub heading: String,
	pub blocks: Vec<ReportBlock>,
}

/// 报告块。
#[derive(Debug, Clone)]
pub enum ReportBlock {
	Paragraph(String),
	KeyValueTable(Vec<(String, String)>),
	Image(ReportImage),
}

/// 报告图片。
#[derive(Debug, Clone)]
pub struct ReportImage {
	pub path: PathBuf,
	pub caption: Option<String>,
	pub width_px: Option<u32>,
}
