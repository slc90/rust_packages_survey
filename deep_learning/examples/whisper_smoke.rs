use std::path::PathBuf;

use deep_learning::{
	runtime::initialize_runtime,
	whisper::{WhisperLanguageHint, WhisperRequest, run_whisper_inference},
};

/// Whisper Base 本地烟测示例。
fn main() -> Result<(), Box<dyn std::error::Error>> {
	let input_path = std::env::args()
		.nth(1)
		.map(PathBuf::from)
		.unwrap_or_else(|| PathBuf::from("data/宮崎羽衣 - Kurenai.mp3"));
	let runtime = initialize_runtime()?;
	let output = run_whisper_inference(
		&WhisperRequest {
			input_path,
			language_hint: WhisperLanguageHint::Japanese,
			with_timestamps: false,
		},
		&runtime,
	)?;

	println!("{}", output.summary);
	if let Some(path) = output.output_path {
		println!("{}", path.display());
	}

	Ok(())
}
