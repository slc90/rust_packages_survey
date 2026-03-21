use std::{fs, path::Path};

use crate::error::ReportError;

/// 读取图片字节并做基础存在性校验。
pub fn read_image_bytes(path: &Path) -> Result<Vec<u8>, ReportError> {
	if !path.exists() {
		return Err(ReportError::Image(format!(
			"图片文件不存在: {}",
			path.display()
		)));
	}
	fs::read(path).map_err(|error| ReportError::Image(error.to_string()))
}
