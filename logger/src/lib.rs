#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use bevy::{
	log::{
		BoxedFmtLayer, BoxedLayer,
		tracing::{self},
		tracing_subscriber::{Layer, field::MakeExt},
	},
	prelude::*,
};
use std::sync::OnceLock;
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
	// 按天滚动的文件写入器，文件名格式为 app.YYYY-MM-DD.log
	let file_appender = RollingFileAppender::builder()
		.rotation(Rotation::DAILY)
		.filename_prefix("app")
		.filename_suffix("log")
		.max_log_files(30)
		.build("logs");
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
