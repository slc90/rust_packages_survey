use std::{fmt, io, path::PathBuf};

/// æˆªå›¾æ¨¡å—ç»Ÿä¸€é”™è¯¯ç±»åž‹ã€‚
#[derive(Debug)]
pub enum ScreenshotError {
	/// æ–‡ä»¶ç³»ç»Ÿæˆ–ç³»ç»Ÿ IO é”™è¯¯ã€‚
	Io(io::Error),

	/// å›¾åƒç¼–è§£ç é”™è¯¯ã€‚
	Image(image::ImageError),

	/// Bevy å›¾åƒè½¬æ¢å¤±è´¥ã€‚
	DynamicImage(String),

	/// Windows API è°ƒç”¨å¤±è´¥ã€‚
	WindowsApi(String),

	/// å½“å‰çª—å£å¥æŸ„ä¸æ˜¯ Win32 HWNDã€‚
	UnsupportedWindowHandle,

	/// æ— æ³•ç”¨åŽŸå§‹åƒç´ æž„å»ºå›¾åƒç¼“å†²åŒºã€‚
	InvalidImageBuffer {
		/// å›¾åƒå®½åº¦ã€‚
		width: u32,

		/// å›¾åƒé«˜åº¦ã€‚
		height: u32,
	},

	/// è£å‰ªçŸ©å½¢è¶…å‡ºå›¾åƒèŒƒå›´ã€‚
	InvalidCropRegion {
		/// åŽŸå›¾å®½åº¦ã€‚
		width: u32,

		/// åŽŸå›¾é«˜åº¦ã€‚
		height: u32,

		/// è£å‰ªåŒºåŸŸèµ·ç‚¹ Xã€‚
		x: u32,

		/// è£å‰ªåŒºåŸŸèµ·ç‚¹ Yã€‚
		y: u32,

		/// è£å‰ªåŒºåŸŸå®½åº¦ã€‚
		crop_width: u32,

		/// è£å‰ªåŒºåŸŸé«˜åº¦ã€‚
		crop_height: u32,
	},

	/// æ— æ³•ç¡®å®šæˆªå›¾è¾“å‡ºç›®å½•ã€‚
	InvalidOutputDirectory(PathBuf),
}

impl fmt::Display for ScreenshotError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Io(error) => write!(f, "IO 错误: {error}"),
			Self::Image(error) => write!(f, "图片编码错误: {error}"),
			Self::DynamicImage(error) => write!(f, "图片转换错误: {error}"),
			Self::WindowsApi(error) => write!(f, "Windows API 错误: {error}"),
			Self::UnsupportedWindowHandle => {
				write!(f, "当前窗口句柄不是 Win32 HWND，无法执行桌面截图")
			}
			Self::InvalidImageBuffer { width, height } => {
				write!(f, "无法构建图像缓冲区: {width}x{height}")
			}
			Self::InvalidCropRegion {
				width,
				height,
				x,
				y,
				crop_width,
				crop_height,
			} => write!(
				f,
				"截图区域超出范围: 原图={}x{}, 区域=({}, {}) {}x{}",
				width, height, x, y, crop_width, crop_height
			),
			Self::InvalidOutputDirectory(path) => {
				write!(f, "无法确定截图输出目录: {}", path.display())
			}
		}
	}
}

impl std::error::Error for ScreenshotError {}

impl From<io::Error> for ScreenshotError {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

impl From<image::ImageError> for ScreenshotError {
	fn from(value: image::ImageError) -> Self {
		Self::Image(value)
	}
}
