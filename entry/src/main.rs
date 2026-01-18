// 只在 Windows 且 release 模式下隐藏控制台
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![allow(clippy::type_complexity)]

use bevy::{
	log::{Level, LogPlugin},
	prelude::*,
	window::WindowResolution,
};
use config::{ConfigPlugin, Setting, data_structure::read_from_file_or_default};
use embedded_assets::{
	const_assets_path::{CLOSE_ICON, LOGO, MAXIMIZE_ICON, MINIMIZE_ICON},
	plugin::EmbeddedAssetPlugin,
};
use i18n::{I18nPlugin, LanguageManager, data_structure::LanguageKey};
use logger::{custom_layer, fmt_layer};
use std::env;
use ui::{
	homepage::{common::ContentAreaMarker, plugin::HomepagePlugin},
	menu_bar::{plugin::MenuBarPlugin, systems::build_menu_bar},
	title_bar::{
		components::{
			TitleBarBundle, TitleBarButtonBundle, TitleBarButtonEnum, TitleBarLogoBundle,
			TitleBarPlaceholderBundle, TitleBarTextBundle,
		},
		plugin::TitleBarPlugin,
	},
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
				filter: "warn,my_crate=debug,ui=debug,i18n=debug,config=debug,utils=debug"
					.to_string(),
			})
			.set(WindowPlugin {
				primary_window: Some(Window {
					// 自定义窗口
					decorations: false,
					position: WindowPosition::Centered(MonitorSelection::Primary),
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
	// 翻译
	app.add_plugins(I18nPlugin);
	// 主页状态管理插件
	app.add_plugins(HomepagePlugin);
	// 菜单栏插件
	app.add_plugins(MenuBarPlugin);
	// 配置插件
	app.add_plugins(ConfigPlugin);
	// 初始化，设置默认进入的页面
	app.add_systems(
		Startup,
		(setup, ui::homepage::realtime_plot::systems::on_enter).chain(),
	);
	app.run();
}

fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut language_manager: ResMut<LanguageManager>,
	mut setting: ResMut<Setting>,
) {
	// 获取当前 exe 路径，如果失败就直接返回
	if let Ok(exe_path) = env::current_exe() {
		// 从路径读取配置
		let config_path = exe_path.join("../config_file/config.json");
		match config_path.to_str() {
			Some(config_path) => *setting = read_from_file_or_default(config_path),
			None => error!("解析配置路径出错,使用默认配置"),
		}
	} else {
		error!("解析当前exe路径出错,使用默认配置");
	};

	// 设置语言
	language_manager.set_current_language(setting.language);
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
			(
				TitleBarTextBundle::new(language_manager.lookup(LanguageKey::Title)),
				TextColor::BLACK
			),
			// 添加菜单栏组件
			build_menu_bar(language_manager.into()),
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
	// 创建内容区域（位于标题栏下方）
	commands.spawn((
		ContentAreaMarker,
		Node {
			width: Val::Percent(100.0),
			height: Val::Percent(100.0),
			position_type: PositionType::Absolute,
			top: Val::Px(40.0),
			bottom: Val::Px(0.0),
			left: Val::Px(0.0),
			right: Val::Px(0.0),
			..default()
		},
		BackgroundColor(Color::WHITE),
	));
}
