// 隐藏 Windows 控制台窗口，让程序作为纯 GUI 应用运行
#![cfg_attr(windows, windows_subsystem = "windows")]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use bevy::{
	log::{Level, LogPlugin},
	prelude::*,
	window::WindowResolution,
};
use embedded_assets::{
	const_assets_path::{CLOSE_ICON, LOGO, MAXIMIZE_ICON, MINIMIZE_ICON},
	plugin::EmbeddedAssetPlugin,
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
	//修改默认Plugin
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
			}),
	);
	// 所有嵌入资源
	app.add_plugins(EmbeddedAssetPlugin);
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
			TitleBarLogoBundle::new(asset_server.load(LOGO), 24.0),
			// 添加标题文本
			(TitleBarTextBundle::new("Rust包调研"), TextColor::BLACK),
			// 添加填充区域
			TitleBarPlaceholderBundle::flexible(),
			// 最小化按钮
			TitleBarButtonBundle::new(
				TitleBarButtonEnum::Minimize,
				24.0,
				asset_server.load(MINIMIZE_ICON),
			),
			// 最大化按钮
			TitleBarButtonBundle::new(
				TitleBarButtonEnum::Maximize,
				24.0,
				asset_server.load(MAXIMIZE_ICON),
			),
			// 关闭按钮
			TitleBarButtonBundle::new(
				TitleBarButtonEnum::Close,
				24.0,
				asset_server.load(CLOSE_ICON),
			)
		],
	));
}
