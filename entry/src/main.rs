// 只在 Windows 且 release 模式下隐藏控制台
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![allow(clippy::type_complexity)]

use bevy::{
	log::{Level, LogPlugin},
	prelude::*,
	render::{
		RenderPlugin,
		settings::{Backends, WgpuSettings},
	},
	ui::IsDefaultUiCamera,
	window::{PrimaryWindow, WindowResolution},
};
use config::{ConfigPlugin, Setting, data_structure::read_from_file_or_default};
use embedded_assets::{
	const_assets_path::{CLOSE_ICON, LOGO, MAXIMIZE_ICON, MINIMIZE_ICON},
	plugin::EmbeddedAssetPlugin,
};
use i18n::{I18nPlugin, LanguageManager, data_structure::LanguageKey};
use logger::{custom_layer, fmt_layer};
use std::{env, ffi::OsString};
use ui::{
	homepage::{common::ContentAreaMarker, plugin::HomepagePlugin},
	menu_bar::{plugin::MenuBarPlugin, systems::build_menu_bar},
	title_bar::{
		components::{
			TitleBarBundle, TitleBarButtonBundle, TitleBarButtonEnum, TitleBarLogoBundle,
			TitleBarMarker, TitleBarPlaceholderBundle, TitleBarTextBundle,
		},
		plugin::TitleBarPlugin,
	},
};

fn main() {
	configure_packaged_runtime_environment();
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
			})
			.set(RenderPlugin {
				render_creation: WgpuSettings {
					backends: preferred_backends(),
					..default()
				}
				.into(),
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
	app.add_systems(Startup, setup);
	app.add_systems(Update, sync_root_layout);
	app.run();
}

fn configure_packaged_runtime_environment() {
	let Ok(current_exe) = env::current_exe() else {
		return;
	};
	let Some(exe_dir) = current_exe.parent() else {
		return;
	};

	let gst_bin_dir = exe_dir.join("gstreamer").join("bin");
	let gst_plugin_dir = exe_dir.join("gstreamer").join("lib").join("gstreamer-1.0");
	let gst_libexec_dir = exe_dir
		.join("gstreamer")
		.join("libexec")
		.join("gstreamer-1.0");
	let cuda_bin_dir = exe_dir.join("cuda").join("bin");
	let mut path_entries = Vec::new();

	if gst_bin_dir.exists() {
		path_entries.push(gst_bin_dir.clone());
	}
	if cuda_bin_dir.exists() {
		path_entries.push(cuda_bin_dir);
	}

	if !path_entries.is_empty() {
		let mut combined_path_entries = path_entries;
		if let Some(existing_path) = env::var_os("PATH") {
			combined_path_entries.extend(env::split_paths(&existing_path));
		}
		if let Ok(joined_runtime_path) = env::join_paths(&combined_path_entries) {
			unsafe {
				env::set_var("PATH", joined_runtime_path);
			}
		}
	}

	if gst_plugin_dir.exists() {
		unsafe {
			env::set_var("GST_PLUGIN_PATH_1_0", &gst_plugin_dir);
			env::set_var("GST_PLUGIN_SYSTEM_PATH_1_0", &gst_plugin_dir);
		}
	}

	let gst_plugin_scanner = gst_libexec_dir.join("gst-plugin-scanner.exe");
	if gst_plugin_scanner.exists() {
		unsafe {
			env::set_var("GST_PLUGIN_SCANNER", OsString::from(&gst_plugin_scanner));
			env::set_var("GST_PLUGIN_SCANNER_1_0", gst_plugin_scanner);
		}
	}
}

fn preferred_backends() -> Option<Backends> {
	if cfg!(target_os = "windows") {
		Some(Backends::DX12)
	} else {
		None
	}
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
	commands.spawn((Camera2d, IsDefaultUiCamera));
	// 标题栏作为独立根节点
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
	// 内容区作为独立根节点
	commands.spawn((
		ContentAreaMarker,
		Node {
			position_type: PositionType::Absolute,
			left: Val::Px(0.0),
			top: Val::Px(40.0),
			overflow: Overflow::scroll(),
			..default()
		},
		BackgroundColor(Color::WHITE),
	));
}

fn sync_root_layout(
	window_query: Query<&Window, With<PrimaryWindow>>,
	mut title_bar_query: Query<&mut Node, With<TitleBarMarker>>,
	mut content_area_query: Query<&mut Node, (With<ContentAreaMarker>, Without<TitleBarMarker>)>,
) {
	let Some(window) = window_query.iter().next() else {
		return;
	};
	let window_width = window.resolution.width();
	let content_height = (window.resolution.height() - 40.0).max(0.0);

	for mut node in &mut title_bar_query {
		node.position_type = PositionType::Absolute;
		node.left = Val::Px(0.0);
		node.top = Val::Px(0.0);
		node.width = Val::Px(window_width);
		node.height = Val::Px(40.0);
	}

	for mut node in &mut content_area_query {
		node.position_type = PositionType::Absolute;
		node.left = Val::Px(0.0);
		node.top = Val::Px(40.0);
		node.width = Val::Px(window_width);
		node.height = Val::Px(content_height);
	}
}
