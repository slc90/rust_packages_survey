use std::{
	collections::HashMap,
	fs::File,
	path::{Path, PathBuf},
	time::{SystemTime, UNIX_EPOCH},
};

use byteorder::{ByteOrder, LittleEndian};
use candle_core::{D, Device, IndexOp, Tensor};
use candle_nn::{VarBuilder, ops::softmax};
use rodio::{Decoder, Source};

use crate::{
	error::DeepLearningError,
	model::{ModelCapability, ModelDescriptor, ensure_model_weights_exist, model_root_dir},
	output::output_root_dir,
	runtime::{CandleRuntime, InferenceOutput},
	whisper_impl::{self as whisper_modeling, Config, audio, model::Whisper},
};

/// Whisper 语言提示。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WhisperLanguageHint {
	/// 自动检测语言。
	Auto,

	/// 中文提示。
	Chinese,

	/// 日文提示。
	Japanese,

	/// 英文提示。
	English,
}

impl WhisperLanguageHint {
	/// 获取语言提示文本。
	pub fn as_label(self) -> &'static str {
		match self {
			Self::Auto => "Auto",
			Self::Chinese => "Chinese",
			Self::Japanese => "Japanese",
			Self::English => "English",
		}
	}

	/// 获取 Whisper 语言代码。
	pub fn as_language_code(self) -> Option<&'static str> {
		match self {
			Self::Auto => None,
			Self::Chinese => Some("zh"),
			Self::Japanese => Some("ja"),
			Self::English => Some("en"),
		}
	}
}

/// Whisper 请求参数。
#[derive(Debug, Clone)]
pub struct WhisperRequest {
	/// 输入音频或视频文件路径。
	pub input_path: PathBuf,

	/// 语言提示。
	pub language_hint: WhisperLanguageHint,

	/// 是否输出时间戳。
	pub with_timestamps: bool,
}

/// Whisper Base 模型描述。
pub fn whisper_base_descriptor() -> ModelDescriptor {
	ModelDescriptor {
		id: "openai/whisper-base",
		capability: ModelCapability::Whisper,
		model_subdir: "whisper-base",
		weights_relative_path: "model.safetensors",
	}
}

/// Whisper 模型目录信息。
#[derive(Debug, Clone)]
pub struct WhisperModelPaths {
	/// 模型描述。
	pub descriptor: ModelDescriptor,

	/// 模型目录。
	pub directory: PathBuf,

	/// 权重文件。
	pub weights: PathBuf,

	/// 配置文件。
	pub config: PathBuf,

	/// 生成配置文件。
	pub generation_config: PathBuf,

	/// 预处理配置文件。
	pub preprocessor_config: PathBuf,

	/// tokenizer 文件。
	pub tokenizer: PathBuf,

	/// tokenizer 配置文件。
	pub tokenizer_config: PathBuf,

	/// tokenizer 补充 token 文件。
	pub added_tokens: PathBuf,

	/// 特殊 token 映射文件。
	pub special_tokens_map: PathBuf,

	/// merges 文件。
	pub merges: PathBuf,

	/// vocab 文件。
	pub vocab: PathBuf,
}

/// Whisper 解码片段。
#[derive(Debug, Clone)]
struct WhisperSegment {
	/// 片段起始时间。
	start_seconds: f64,

	/// 片段结束时间。
	end_seconds: f64,

	/// 片段文本。
	text: String,
}

/// Whisper 最小 tokenizer。
#[derive(Debug, Clone)]
struct WhisperTokenizer {
	/// token 到 id 的映射表。
	token_to_id: HashMap<String, u32>,

	/// id 到 token 的映射表。
	id_to_token: Vec<String>,

	/// GPT-2 字节映射的反向查找表。
	byte_decoder: HashMap<char, u8>,
}

/// Whisper 最小解码器。
struct WhisperDecoder {
	/// Whisper 模型。
	model: Whisper,

	/// tokenizer。
	tokenizer: WhisperTokenizer,

	/// 设备。
	device: Device,

	/// 起始 token。
	sot_token: u32,

	/// 转写 token。
	transcribe_token: u32,

	/// 结束 token。
	eot_token: u32,

	/// 禁用时间戳 token。
	no_timestamps_token: u32,

	/// 语言 token。
	language_token: Option<u32>,
}

/// 校验 Whisper 模型目录和核心文件。
pub fn ensure_whisper_model_ready() -> Result<WhisperModelPaths, DeepLearningError> {
	let descriptor = whisper_base_descriptor();

	for directory in whisper_model_directory_candidates(&descriptor) {
		if !directory.exists() {
			continue;
		}

		let paths = WhisperModelPaths {
			descriptor: descriptor.clone(),
			weights: directory.join("model.safetensors"),
			config: directory.join("config.json"),
			generation_config: directory.join("generation_config.json"),
			preprocessor_config: directory.join("preprocessor_config.json"),
			tokenizer: directory.join("tokenizer.json"),
			tokenizer_config: directory.join("tokenizer_config.json"),
			added_tokens: directory.join("added_tokens.json"),
			special_tokens_map: directory.join("special_tokens_map.json"),
			merges: directory.join("merges.txt"),
			vocab: directory.join("vocab.json"),
			directory,
		};

		ensure_whisper_support_files_exist(&paths)?;
		return Ok(paths);
	}

	Err(DeepLearningError::ModelDirectoryMissing {
		path: model_root_dir().join("whisper_base"),
	})
}

/// 构建 Whisper 任务快照输出路径。
pub fn build_whisper_request_snapshot_path() -> PathBuf {
	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|duration| duration.as_millis())
		.unwrap_or(0);

	output_root_dir()
		.join("whisper")
		.join(format!("whisper_request_{timestamp}.txt"))
}

/// 将 Whisper 请求写出为任务快照文件。
pub fn save_whisper_request_snapshot(
	request: &WhisperRequest,
) -> Result<PathBuf, DeepLearningError> {
	let output_path = build_whisper_request_snapshot_path();
	let content = format!(
		"Whisper 任务快照\ninput={}\nlanguage_hint={}\nwith_timestamps={}\n",
		request.input_path.display(),
		request.language_hint.as_label(),
		request.with_timestamps
	);
	std::fs::write(&output_path, content)?;
	Ok(output_path)
}

/// 构建 Whisper 结果输出路径。
pub fn build_whisper_output_path() -> PathBuf {
	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|duration| duration.as_millis())
		.unwrap_or(0);

	output_root_dir()
		.join("whisper")
		.join(format!("whisper_result_{timestamp}.txt"))
}

/// 执行 Whisper Base 实际推理。
pub fn run_whisper_inference(
	request: &WhisperRequest,
	runtime: &CandleRuntime,
) -> Result<InferenceOutput, DeepLearningError> {
	save_whisper_request_snapshot(request)?;
	let model_paths = ensure_whisper_model_ready()?;
	let config = load_whisper_config(&model_paths.config)?;
	let tokenizer = load_whisper_tokenizer(&model_paths)?;
	let mel_filters = load_mel_filters(config.num_mel_bins)?;
	let pcm = load_audio_as_mono_16k(&request.input_path)?;
	let mel = build_mel_tensor(&config, &pcm, &mel_filters, runtime.device())?;
	let mut model = load_whisper_model(&model_paths.weights, &config, runtime.device())?;
	let language_token =
		resolve_language_token(request.language_hint, &tokenizer, &mut model, &mel)?;
	let detected_language = language_token
		.and_then(|token_id| tokenizer.id_to_token(token_id))
		.map(|token| token.trim_matches(['<', '|', '>']).to_string())
		.unwrap_or_else(|| "unknown".to_string());
	let mut decoder =
		WhisperDecoder::new(model, tokenizer, runtime.device().clone(), language_token)?;
	let segments = decoder.run(&mel)?;
	let transcript = format_segments(&segments, request.with_timestamps);
	let output_path = build_whisper_output_path();
	let content = format!(
		"Whisper Inference Result\nmodel={}\nmodel_dir={}\ndevice={}\nlanguage={}\nsegments={}\n\n{}\n",
		model_paths.descriptor.id,
		model_paths.directory.display(),
		runtime.device_label,
		detected_language,
		segments.len(),
		transcript
	);
	std::fs::write(&output_path, content)?;

	Ok(InferenceOutput {
		summary: format!("Whisper Base 推理已完成，识别出 {} 个片段", segments.len()),
		output_path: Some(output_path),
	})
}

impl WhisperDecoder {
	/// 创建 Whisper 解码器。
	fn new(
		model: Whisper,
		tokenizer: WhisperTokenizer,
		device: Device,
		language_token: Option<u32>,
	) -> Result<Self, DeepLearningError> {
		let sot_token = token_id(&tokenizer, whisper_modeling::SOT_TOKEN)?;
		let transcribe_token = token_id(&tokenizer, whisper_modeling::TRANSCRIBE_TOKEN)?;
		let eot_token = token_id(&tokenizer, whisper_modeling::EOT_TOKEN)?;
		let no_timestamps_token = token_id(&tokenizer, whisper_modeling::NO_TIMESTAMPS_TOKEN)?;

		Ok(Self {
			model,
			tokenizer,
			device,
			sot_token,
			transcribe_token,
			eot_token,
			no_timestamps_token,
			language_token,
		})
	}

	/// 执行多片段转写。
	fn run(&mut self, mel: &Tensor) -> Result<Vec<WhisperSegment>, DeepLearningError> {
		let (_, _, content_frames) = mel
			.dims3()
			.map_err(|error| map_candle_error("读取 mel 维度", error))?;
		let mut seek = 0;
		let mut segments = Vec::new();

		while seek < content_frames {
			let segment_size = usize::min(content_frames - seek, whisper_modeling::N_FRAMES);
			let mel_segment = mel
				.narrow(2, seek, segment_size)
				.map_err(|error| map_candle_error("截取 mel 片段", error))?;
			let text = self.decode_segment(&mel_segment)?;
			let start_seconds =
				(seek * whisper_modeling::HOP_LENGTH) as f64 / whisper_modeling::SAMPLE_RATE as f64;
			let end_seconds = ((seek + segment_size) * whisper_modeling::HOP_LENGTH) as f64
				/ whisper_modeling::SAMPLE_RATE as f64;
			seek += segment_size;

			let clean_text = text.trim().to_string();
			if clean_text.is_empty() {
				continue;
			}

			segments.push(WhisperSegment {
				start_seconds,
				end_seconds,
				text: clean_text,
			});
		}

		Ok(segments)
	}

	/// 对单个 mel 片段执行贪心解码。
	fn decode_segment(&mut self, mel: &Tensor) -> Result<String, DeepLearningError> {
		let audio_features = self
			.model
			.encoder
			.forward(mel, true)
			.map_err(|error| map_candle_error("执行 Whisper 编码器", error))?;
		let sample_len = self.model.config.max_target_positions / 2;
		let mut tokens = vec![self.sot_token];
		if let Some(language_token) = self.language_token {
			tokens.push(language_token);
		}
		tokens.push(self.transcribe_token);
		tokens.push(self.no_timestamps_token);

		for step in 0..sample_len {
			let tokens_t = Tensor::new(tokens.as_slice(), &self.device)
				.map_err(|error| map_candle_error("创建 token 张量", error))?
				.unsqueeze(0)
				.map_err(|error| map_candle_error("扩展 token batch 维度", error))?;
			let ys = self
				.model
				.decoder
				.forward(&tokens_t, &audio_features, step == 0)
				.map_err(|error| map_candle_error("执行 Whisper 解码器", error))?;
			let (_, seq_len, _) = ys
				.dims3()
				.map_err(|error| map_candle_error("读取解码器输出维度", error))?;
			let logits = self
				.model
				.decoder
				.final_linear(
					&ys.i((..1, seq_len - 1..))
						.map_err(|error| map_candle_error("截取最后一步 logits", error))?,
				)
				.map_err(|error| map_candle_error("执行最终线性层", error))?
				.i(0)
				.map_err(|error| map_candle_error("提取 batch 维度", error))?
				.i(0)
				.map_err(|error| map_candle_error("提取 token 维度", error))?;
			let logits = logits
				.to_vec1::<f32>()
				.map_err(|error| map_candle_error("读取 logits 向量", error))?;
			let next_token = logits
				.iter()
				.enumerate()
				.max_by(|(_, left), (_, right)| left.total_cmp(right))
				.map(|(index, _)| index as u32)
				.unwrap_or(self.eot_token);

			if next_token == self.eot_token || tokens.len() > self.model.config.max_target_positions
			{
				break;
			}

			tokens.push(next_token);
		}

		self.tokenizer
			.decode(&tokens, true)
			.map_err(|error| DeepLearningError::InferenceFailed {
				message: format!("解码 Whisper token 失败: {error}"),
			})
	}
}

/// 加载 Whisper 配置。
fn load_whisper_config(path: &Path) -> Result<Config, DeepLearningError> {
	let config_json =
		std::fs::read_to_string(path).map_err(|error| DeepLearningError::ModelLoadFailed {
			message: format!("读取 Whisper 配置失败: {error}"),
		})?;
	serde_json::from_str::<Config>(&config_json).map_err(|error| {
		DeepLearningError::ModelLoadFailed {
			message: format!("解析 Whisper 配置失败: {error}"),
		}
	})
}

/// 加载 Whisper tokenizer。
fn load_whisper_tokenizer(
	model_paths: &WhisperModelPaths,
) -> Result<WhisperTokenizer, DeepLearningError> {
	WhisperTokenizer::from_vocab_files(&model_paths.vocab, &model_paths.added_tokens)
}

/// 加载 Whisper 模型。
fn load_whisper_model(
	weights_path: &Path,
	config: &Config,
	device: &Device,
) -> Result<Whisper, DeepLearningError> {
	let vb = unsafe {
		VarBuilder::from_mmaped_safetensors(&[weights_path], whisper_modeling::DTYPE, device)
	}
	.map_err(|error| DeepLearningError::ModelLoadFailed {
		message: format!("映射 Whisper 权重失败: {error}"),
	})?;

	Whisper::load(&vb, config.clone()).map_err(|error| DeepLearningError::ModelLoadFailed {
		message: format!("构建 Whisper 模型失败: {error}"),
	})
}

/// 加载 mel filter 资源。
fn load_mel_filters(num_mel_bins: usize) -> Result<Vec<f32>, DeepLearningError> {
	let mel_bytes = match num_mel_bins {
		80 => include_bytes!("whisper_assets/melfilters.bytes").as_slice(),
		128 => include_bytes!("whisper_assets/melfilters128.bytes").as_slice(),
		value => {
			return Err(DeepLearningError::ModelLoadFailed {
				message: format!("不支持的 Whisper mel bin 数量: {value}"),
			});
		}
	};

	let mut mel_filters = vec![0_f32; mel_bytes.len() / 4];
	LittleEndian::read_f32_into(mel_bytes, &mut mel_filters);
	Ok(mel_filters)
}

/// 将音频解码为 16kHz 单声道 PCM。
fn load_audio_as_mono_16k(path: &Path) -> Result<Vec<f32>, DeepLearningError> {
	let file = File::open(path).map_err(|error| DeepLearningError::InferenceFailed {
		message: format!("打开音频文件失败: {error}"),
	})?;
	let decoder = Decoder::try_from(file).map_err(|error| DeepLearningError::InferenceFailed {
		message: format!("解码音频文件失败: {error}"),
	})?;
	let channel_count = usize::from(decoder.channels().get().max(1));
	let sample_rate = decoder.sample_rate().get();
	let interleaved_samples = decoder.collect::<Vec<f32>>();
	if interleaved_samples.is_empty() {
		return Err(DeepLearningError::InferenceFailed {
			message: "音频文件没有可用采样数据".to_string(),
		});
	}

	let mono_samples = if channel_count == 1 {
		interleaved_samples
	} else {
		interleaved_samples
			.chunks(channel_count)
			.map(|chunk| chunk.iter().copied().sum::<f32>() / chunk.len() as f32)
			.collect::<Vec<_>>()
	};

	if sample_rate == whisper_modeling::SAMPLE_RATE as u32 {
		return Ok(mono_samples);
	}

	Ok(linear_resample(
		&mono_samples,
		sample_rate as usize,
		whisper_modeling::SAMPLE_RATE,
	))
}

/// 对 PCM 数据执行线性重采样。
fn linear_resample(samples: &[f32], input_rate: usize, target_rate: usize) -> Vec<f32> {
	if samples.is_empty() || input_rate == target_rate {
		return samples.to_vec();
	}

	let output_len = samples.len() * target_rate / input_rate;
	let ratio = input_rate as f32 / target_rate as f32;
	let mut output = Vec::with_capacity(output_len.max(1));

	for output_index in 0..output_len.max(1) {
		let source_position = output_index as f32 * ratio;
		let left_index = source_position.floor() as usize;
		let right_index = usize::min(left_index + 1, samples.len() - 1);
		let fraction = source_position - left_index as f32;
		let sample = samples[left_index] * (1.0 - fraction) + samples[right_index] * fraction;
		output.push(sample);
	}

	output
}

/// 构建 mel 张量。
fn build_mel_tensor(
	config: &Config,
	pcm: &[f32],
	mel_filters: &[f32],
	device: &Device,
) -> Result<Tensor, DeepLearningError> {
	let mel = audio::pcm_to_mel(config, pcm, mel_filters);
	let mel_len = mel.len();
	Tensor::from_vec(
		mel,
		(1, config.num_mel_bins, mel_len / config.num_mel_bins),
		device,
	)
	.map_err(|error| map_candle_error("创建 Whisper mel 张量", error))
}

/// 根据语言提示解析 Whisper 语言 token。
fn resolve_language_token(
	language_hint: WhisperLanguageHint,
	tokenizer: &WhisperTokenizer,
	model: &mut Whisper,
	mel: &Tensor,
) -> Result<Option<u32>, DeepLearningError> {
	if let Some(language_code) = language_hint.as_language_code() {
		return Ok(Some(token_id(tokenizer, &format!("<|{language_code}|>"))?));
	}

	detect_language(model, tokenizer, mel).map(Some)
}

/// 自动检测语言 token。
fn detect_language(
	model: &mut Whisper,
	tokenizer: &WhisperTokenizer,
	mel: &Tensor,
) -> Result<u32, DeepLearningError> {
	let languages = [
		"en", "zh", "de", "es", "ru", "ko", "fr", "ja", "pt", "tr", "pl", "ca", "nl", "ar", "sv",
		"it", "id", "hi", "fi", "vi", "he", "uk", "el", "ms", "cs", "ro", "da", "hu", "ta", "no",
		"th", "ur", "hr", "bg", "lt", "la", "mi", "ml", "cy", "sk", "te", "fa", "lv", "bn", "sr",
		"az", "sl", "kn", "et", "mk", "br", "eu", "is", "hy", "ne", "mn", "bs", "kk", "sq", "sw",
		"gl", "mr", "pa", "si", "km", "sn", "yo", "so", "af", "oc", "ka", "be", "tg", "sd", "gu",
		"am", "yi", "lo", "uz", "fo", "ht", "ps", "tk", "nn", "mt", "sa", "lb", "my", "bo", "tl",
		"mg", "as", "tt", "haw", "ln", "ha", "ba", "jw", "su",
	];
	let (_batch, _, seq_len) = mel
		.dims3()
		.map_err(|error| map_candle_error("读取语言检测 mel 维度", error))?;
	let mel = mel
		.narrow(2, 0, usize::min(seq_len, model.config.max_source_positions))
		.map_err(|error| map_candle_error("裁剪语言检测 mel", error))?;
	let language_token_ids = languages
		.iter()
		.map(|code| token_id(tokenizer, &format!("<|{code}|>")))
		.collect::<Result<Vec<_>, _>>()?;
	let sot_token = token_id(tokenizer, whisper_modeling::SOT_TOKEN)?;
	let audio_features = model
		.encoder
		.forward(&mel, true)
		.map_err(|error| map_candle_error("执行语言检测编码器", error))?;
	let tokens = Tensor::new(&[[sot_token]], mel.device())
		.map_err(|error| map_candle_error("创建语言检测 token 张量", error))?;
	let language_token_ids_tensor = Tensor::new(language_token_ids.as_slice(), mel.device())
		.map_err(|error| map_candle_error("创建语言 token 列表张量", error))?;
	let ys = model
		.decoder
		.forward(&tokens, &audio_features, true)
		.map_err(|error| map_candle_error("执行语言检测解码器", error))?;
	let logits = model
		.decoder
		.final_linear(
			&ys.i(..1)
				.map_err(|error| map_candle_error("提取语言检测 logits", error))?,
		)
		.map_err(|error| map_candle_error("执行语言检测线性层", error))?
		.i(0)
		.map_err(|error| map_candle_error("提取语言检测 batch 维度", error))?
		.i(0)
		.map_err(|error| map_candle_error("提取语言检测 token 维度", error))?;
	let logits = logits
		.index_select(&language_token_ids_tensor, 0)
		.map_err(|error| map_candle_error("筛选语言 logits", error))?;
	let probabilities = softmax(&logits, D::Minus1)
		.map_err(|error| map_candle_error("执行语言 softmax", error))?
		.to_vec1::<f32>()
		.map_err(|error| map_candle_error("读取语言概率向量", error))?;

	let best_index = probabilities
		.iter()
		.enumerate()
		.max_by(|(_, left), (_, right)| left.total_cmp(right))
		.map(|(index, _)| index)
		.unwrap_or(0);
	Ok(language_token_ids[best_index])
}

/// 将片段列表格式化为文本。
fn format_segments(segments: &[WhisperSegment], with_timestamps: bool) -> String {
	if with_timestamps {
		return segments
			.iter()
			.map(|segment| {
				format!(
					"[{:.2}s - {:.2}s] {}",
					segment.start_seconds, segment.end_seconds, segment.text
				)
			})
			.collect::<Vec<_>>()
			.join("\n")
			.trim()
			.to_string();
	}

	segments
		.iter()
		.map(|segment| segment.text.as_str())
		.collect::<Vec<_>>()
		.join("\n")
		.trim()
		.to_string()
}

/// 获取 token id。
fn token_id(tokenizer: &WhisperTokenizer, token: &str) -> Result<u32, DeepLearningError> {
	tokenizer
		.token_to_id(token)
		.ok_or_else(|| DeepLearningError::InferenceFailed {
			message: format!("无法找到 token: {token}"),
		})
}

impl WhisperTokenizer {
	/// 从 Whisper 的 vocab 文件构建最小 tokenizer。
	fn from_vocab_files(
		vocab_path: &Path,
		added_tokens_path: &Path,
	) -> Result<Self, DeepLearningError> {
		let content = std::fs::read_to_string(vocab_path).map_err(|error| {
			DeepLearningError::ModelLoadFailed {
				message: format!("读取 Whisper vocab 失败: {error}"),
			}
		})?;
		let mut token_to_id =
			serde_json::from_str::<HashMap<String, u32>>(&content).map_err(|error| {
				DeepLearningError::ModelLoadFailed {
					message: format!("解析 Whisper vocab 失败: {error}"),
				}
			})?;
		let added_tokens_content = std::fs::read_to_string(added_tokens_path).map_err(|error| {
			DeepLearningError::ModelLoadFailed {
				message: format!("读取 Whisper added_tokens 失败: {error}"),
			}
		})?;
		let added_tokens = serde_json::from_str::<HashMap<String, u32>>(&added_tokens_content)
			.map_err(|error| DeepLearningError::ModelLoadFailed {
				message: format!("解析 Whisper added_tokens 失败: {error}"),
			})?;
		token_to_id.extend(added_tokens);
		let max_token_id = token_to_id.values().copied().max().unwrap_or(0) as usize;
		let mut id_to_token = vec![String::new(); max_token_id + 1];
		for (token, token_id) in &token_to_id {
			id_to_token[*token_id as usize] = token.clone();
		}

		Ok(Self {
			token_to_id,
			id_to_token,
			byte_decoder: gpt2_byte_decoder(),
		})
	}

	/// 根据 token 文本查找 id。
	fn token_to_id(&self, token: &str) -> Option<u32> {
		self.token_to_id.get(token).copied()
	}

	/// 根据 id 查找 token 文本。
	fn id_to_token(&self, token_id: u32) -> Option<&str> {
		self.id_to_token.get(token_id as usize).map(String::as_str)
	}

	/// 将 Whisper 生成的 token 序列解码为文本。
	fn decode(
		&self,
		token_ids: &[u32],
		skip_special_tokens: bool,
	) -> Result<String, DeepLearningError> {
		let mut decoded_bytes = Vec::new();

		for token_id in token_ids {
			let Some(token) = self.id_to_token(*token_id) else {
				continue;
			};

			if skip_special_tokens && is_special_token(token) {
				continue;
			}

			for character in token.chars() {
				if let Some(byte) = self.byte_decoder.get(&character) {
					decoded_bytes.push(*byte);
				} else {
					let mut buffer = [0_u8; 4];
					let utf8 = character.encode_utf8(&mut buffer);
					decoded_bytes.extend_from_slice(utf8.as_bytes());
				}
			}
		}

		Ok(String::from_utf8_lossy(&decoded_bytes).to_string())
	}
}

/// 判断 token 是否属于 Whisper 控制 token。
fn is_special_token(token: &str) -> bool {
	token.starts_with("<|") && token.ends_with("|>")
}

/// 构建 GPT-2 字节到 Unicode 映射的反向字典。
fn gpt2_byte_decoder() -> HashMap<char, u8> {
	let mut bytes = (33_u16..=126)
		.chain(161..=172)
		.chain(174..=255)
		.map(|value| value as u8)
		.collect::<Vec<_>>();
	let mut codepoints = bytes
		.iter()
		.map(|value| u32::from(*value))
		.collect::<Vec<_>>();
	let mut extra = 0_u32;

	for value in 0_u16..=255 {
		let byte = value as u8;
		if bytes.contains(&byte) {
			continue;
		}
		bytes.push(byte);
		codepoints.push(256 + extra);
		extra += 1;
	}

	bytes
		.into_iter()
		.zip(codepoints)
		.filter_map(|(byte, codepoint)| {
			char::from_u32(codepoint).map(|character| (character, byte))
		})
		.collect()
}

/// 获取 Whisper 模型目录候选列表。
fn whisper_model_directory_candidates(descriptor: &ModelDescriptor) -> Vec<PathBuf> {
	let model_root = model_root_dir();
	vec![
		model_root.join("whisper_base"),
		model_root.join("whisper-base"),
		model_root.join("whisper").join(descriptor.model_subdir),
	]
}

/// 校验 Whisper 运行最小所需文件。
fn ensure_whisper_support_files_exist(paths: &WhisperModelPaths) -> Result<(), DeepLearningError> {
	ensure_model_weights_exist(&paths.weights)?;
	ensure_file_exists(&paths.config)?;
	ensure_file_exists(&paths.generation_config)?;
	ensure_file_exists(&paths.preprocessor_config)?;
	ensure_file_exists(&paths.tokenizer)?;
	ensure_file_exists(&paths.tokenizer_config)?;
	ensure_file_exists(&paths.added_tokens)?;
	ensure_file_exists(&paths.special_tokens_map)?;
	ensure_file_exists(&paths.merges)?;
	ensure_file_exists(&paths.vocab)?;
	Ok(())
}

/// 校验单个文件存在。
fn ensure_file_exists(path: &Path) -> Result<(), DeepLearningError> {
	if path.exists() {
		return Ok(());
	}

	Err(DeepLearningError::ModelFileMissing {
		path: path.to_path_buf(),
	})
}

/// 将 Candle 错误转换为项目统一错误。
fn map_candle_error(action: &str, error: candle_core::Error) -> DeepLearningError {
	DeepLearningError::InferenceFailed {
		message: format!("{action}失败: {error}"),
	}
}
