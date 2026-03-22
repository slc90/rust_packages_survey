#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![allow(clippy::type_complexity)]

use bevy::{
	log::{
		BoxedFmtLayer, BoxedLayer,
		tracing::{self},
		tracing_subscriber::{Layer, field::MakeExt},
	},
	prelude::*,
};
use std::{
	path::{Path, PathBuf},
	sync::OnceLock,
};
use tracing::level_filters::LevelFilter;
use tracing_appender::{
	non_blocking::WorkerGuard,
	rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::fmt::time::ChronoLocal;

///防止tracing_appender出了作用域被释放
static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

/// 添加文件log，按天滚动，文件名为app.YYYY-MM-DD.log
pub fn custom_layer(_app: &mut App) -> Option<BoxedLayer> {
	let logs_dir = resolve_logs_dir();
	// 按天滚动的文件写入器，文件名格式为 app.YYYY-MM-DD.log
	let file_appender = RollingFileAppender::builder()
		.rotation(Rotation::DAILY)
		.filename_prefix("app")
		.filename_suffix("log")
		.max_log_files(30)
		.build(logs_dir);
	match file_appender {
		Ok(file_appender) => {
			let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
			let _ = LOG_GUARD.set(guard);

			Some(
				bevy::log::tracing_subscriber::fmt::layer()
					.with_writer(non_blocking)
					// 使用自定义时间格式：时分秒毫秒
					.with_timer(ChronoLocal::new("%H:%M:%S%.3f".to_string()))
					// 包含日志级别
					.with_level(true)
					// 包含模块名（target）
					.with_target(true)
					// 包含文件名
					.with_file(true)
					// 包含行号
					.with_line_number(true)
					// 包含线程ID
					.with_thread_ids(true)
					// 包含线程名
					.with_thread_names(true)
					// 禁用ANSI颜色代码，避免日志文件乱码
					.with_ansi(false)
					// 使用更友好的调试格式输出字段
					.map_fmt_fields(MakeExt::debug_alt)
					.boxed(),
			)
		}
		Err(e) => {
			eprintln!("创建logs文件夹失败: {}", e);
			None
		}
	}
}

/// 解析日志目录。
///
/// 优先从当前工作目录链路向上定位工作区根目录，兼容在编辑器里以 `entry/`
/// 作为工作目录直接运行 `entry/src/main.rs` 的场景；如果失败，安装版统一回退到
/// `LOCALAPPDATA/rust_packages_survey/logs/`，避免把日志写到 `Program Files`。
fn resolve_logs_dir() -> PathBuf {
	if let Ok(current_dir) = std::env::current_dir()
		&& let Some(root) = find_workspace_root_from(&current_dir)
	{
		return root.join("logs");
	}

	if let Ok(current_exe) = std::env::current_exe()
		&& let Some(exe_dir) = current_exe.parent()
		&& let Some(root) = find_workspace_root_from(exe_dir)
	{
		return root.join("logs");
	}

	if let Some(local_app_data_dir) = local_app_data_root_dir() {
		return local_app_data_dir.join("logs");
	}

	if let Ok(current_exe) = std::env::current_exe()
		&& let Some(exe_dir) = current_exe.parent()
	{
		return exe_dir.join("logs");
	}

	PathBuf::from("logs")
}

/// 获取当前用户的本地应用数据目录。
fn local_app_data_root_dir() -> Option<PathBuf> {
	let local_app_data = std::env::var_os("LOCALAPPDATA")?;
	Some(PathBuf::from(local_app_data).join("rust_packages_survey"))
}

/// 从指定目录向上查找工作区根目录。
fn find_workspace_root_from(start: &Path) -> Option<PathBuf> {
	for directory in start.ancestors() {
		let cargo_toml = directory.join("Cargo.toml");
		let entry_manifest = directory.join("entry").join("Cargo.toml");
		let logger_manifest = directory.join("logger").join("Cargo.toml");
		if cargo_toml.exists() && entry_manifest.exists() && logger_manifest.exists() {
			return Some(directory.to_path_buf());
		}
	}

	None
}

/// 替换掉bevy默认的fmt_layer，这个用于开发时终端显示log
pub fn fmt_layer(_app: &mut App) -> Option<BoxedFmtLayer> {
	Some(Box::new(
		bevy::log::tracing_subscriber::fmt::Layer::default()
			.without_time()
			.with_level(true)
			// 输出模块-路径
			.with_target(true)
			.with_thread_ids(true)
			.with_thread_names(true)
			// 只影响使用 ? sigil 的字段
			// 可以多行输出格式化结构
			.map_fmt_fields(MakeExt::debug_alt)
			.with_filter(LevelFilter::DEBUG),
	))
}
