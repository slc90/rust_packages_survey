#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use bevy::{
	asset::{UnapprovedPathMode, load_internal_binary_asset},
	log::{Level, LogPlugin},
	prelude::*,
	window::WindowResolution,
};
use logger::{custom_layer, fmt_layer};
use ui::title_bar::{
	components::{
		TitleBarBundle, TitleBarButtonBundle, TitleBarButtonEnum, TitleBarLogoBundle,
		TitleBarPlaceholderBundle, TitleBarTextBundle,
	},
	plugin::TitleBarPlugin,
};

fn main() {
	let mut app = App::new();
	app.add_plugins(
		DefaultPlugins
			.set(LogPlugin {
				custom_layer,
				fmt_layer,
				level: Level::DEBUG,
				..default()
			})
			.set(WindowPlugin {
				primary_window: Some(Window {
					// 自定义窗口
					decorations: false,
					position: WindowPosition::Centered(MonitorSelection::Current),
					//初始默认大小
					resolution: WindowResolution::new(1920, 1080)
						//不要受dpi影响
						.with_scale_factor_override(1.0),
					..default()
				}),
				..default()
			})
			.set(AssetPlugin {
				file_path: "../".to_string(), // 指向项目根目录
				unapproved_path_mode: UnapprovedPathMode::Allow,
				..default()
			}),
	);

	// 加载全局默认字体
	// This needs to happen after `DefaultPlugins` is added.
	load_internal_binary_asset!(
		app,
		TextFont::default().font,
		"../../assets/SmileySans-Oblique.ttf",
		|bytes: &[u8], _path: String| {
			match Font::try_from_bytes(bytes.to_vec()) {
				Ok(result) => result,
				Err(e) => {
					panic!("未能加载字体:{}", e)
				}
			}
		}
	);
	// 自定义标题栏插件
	app.add_plugins(TitleBarPlugin);
	app.add_systems(Startup, setup);
	app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	// 生成相机用于UI渲染
	commands.spawn(Camera2d);
	// 创建标题栏
	commands.spawn((
		TitleBarBundle::default(),
		BackgroundColor(Color::WHITE),
		children![
			// 添加应用Logo
			TitleBarLogoBundle::new(asset_server.load("assets/logo.png"), 24.0),
			// 添加标题文本
			(
				TitleBarTextBundle::new("Rust Packages Survey"),
				TextColor::BLACK
			),
			// 添加填充区域
			TitleBarPlaceholderBundle::flexible(),
			// 最小化按钮
			TitleBarButtonBundle::new(
				TitleBarButtonEnum::Minimize,
				24.0,
				asset_server.load("assets/minimize.png"),
			),
			// 最大化按钮
			TitleBarButtonBundle::new(
				TitleBarButtonEnum::Maximize,
				24.0,
				asset_server.load("assets/maximize.png"),
			),
			// 关闭按钮
			TitleBarButtonBundle::new(
				TitleBarButtonEnum::Close,
				24.0,
				asset_server.load("assets/close.png"),
			)
		],
	));
}
