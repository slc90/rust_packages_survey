use std::path::PathBuf;

use deep_learning::{
	runtime::initialize_runtime,
	whisper::{WhisperLanguageHint, WhisperModelKind, WhisperRequest, run_whisper_inference},
};

/// Whisper 本地烟测示例。
fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = std::env::args().collect::<Vec<_>>();
	let model = parse_model_kind(args.get(1).map(String::as_str));
	let with_timestamps = parse_with_timestamps(args.get(2).map(String::as_str));
	let input_path = args
		.get(3)
		.map(PathBuf::from)
		.unwrap_or_else(|| PathBuf::from("data/宮崎羽衣 - Kurenai.mp3"));
	let runtime = initialize_runtime()?;
	let output = run_whisper_inference(
		&WhisperRequest {
			input_path,
			model,
			language_hint: WhisperLanguageHint::Japanese,
			with_timestamps,
		},
		&runtime,
	)?;

	println!("{}", output.summary);
	if let Some(path) = output.output_path {
		println!("{}", path.display());
	}

	Ok(())
}

/// 解析示例命令行中的 Whisper 模型类型。
fn parse_model_kind(value: Option<&str>) -> WhisperModelKind {
	match value {
		Some("large-v3") | Some("v3") => WhisperModelKind::LargeV3,
		_ => WhisperModelKind::Base,
	}
}

/// 解析示例命令行中的时间戳开关。
fn parse_with_timestamps(value: Option<&str>) -> bool {
	matches!(value, Some("timestamps") | Some("ts") | Some("true"))
}
