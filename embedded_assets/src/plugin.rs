use bevy::{
	asset::{embedded_asset, load_internal_binary_asset},
	prelude::*,
};

/// 嵌入式资源插件
pub struct EmbeddedAssetPlugin;

impl Plugin for EmbeddedAssetPlugin {
	fn build(&self, app: &mut App) {
		// 加载全局默认字体作为内部二进制资源
		// 字体文件路径相对于当前源文件
		load_internal_binary_asset!(
			app,
			TextFont::default().font,
			"../assets/SmileySans-Oblique.ttf",
			|bytes: &[u8], _path: String| {
				match Font::try_from_bytes(bytes.to_vec()) {
					Ok(result) => result,
					Err(e) => {
						panic!("未能加载字体:{}", e)
					}
				}
			}
		);
		//嵌入所有的资源
		embedded_asset!(app, "../assets/logo.png");
		embedded_asset!(app, "../assets/minimize.png");
		embedded_asset!(app, "../assets/maximize.png");
		embedded_asset!(app, "../assets/close.png");
	}
}
