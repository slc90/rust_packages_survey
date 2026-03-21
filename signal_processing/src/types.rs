use rustfft::num_complex::Complex32;

/// 单通道信号缓冲区。
#[derive(Debug, Clone)]
pub struct SignalBuffer {
	pub sample_rate: f32,
	pub samples: Vec<f32>,
}

/// 正弦波生成配置。
#[derive(Debug, Clone, Copy)]
pub struct SineWaveConfig {
	pub sample_rate: f32,
	pub frequency: f32,
	pub amplitude: f32,
	pub phase: f32,
	pub duration_secs: f32,
}

/// 复合信号的单个分量。
#[derive(Debug, Clone, Copy)]
pub struct CompositeComponent {
	pub frequency: f32,
	pub amplitude: f32,
	pub phase: f32,
}

/// FFT 输出结果。
#[derive(Debug, Clone)]
pub struct FftOutput {
	pub sample_rate: f32,
	pub bins: Vec<Complex32>,
}

/// 频谱点。
#[derive(Debug, Clone, Copy)]
pub struct SpectrumPoint {
	pub frequency: f32,
	pub value: f32,
}

/// 滤波器类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterKind {
	LowPass,
	HighPass,
	BandPass,
	BandStop,
}

/// FIR 设计配置。
#[derive(Debug, Clone)]
pub struct FirDesignConfig {
	pub sample_rate: f32,
	pub filter_kind: FilterKind,
	pub cutoff_hz: Vec<f32>,
	pub tap_count: usize,
}

/// IIR 设计配置。
#[derive(Debug, Clone, Copy)]
pub struct IirDesignConfig {
	pub sample_rate: f32,
	pub filter_kind: FilterKind,
	pub cutoff_hz: [f32; 2],
	pub q: f32,
}

/// 二阶 IIR 系数。
#[derive(Debug, Clone, Copy)]
pub struct IirCoefficients {
	pub b: [f32; 3],
	pub a: [f32; 3],
}
