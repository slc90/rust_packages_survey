/// 解码后的视频帧
#[derive(Debug, Clone)]
pub struct VideoFrame {
	pub width: u32,
	pub height: u32,
	pub pixels_rgba: Vec<u8>,
	pub pts_ns: u64,
}
