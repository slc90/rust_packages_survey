use rustfft::{FftPlanner, num_complex::Complex32};

use crate::{error::SignalProcessError, types::{FftOutput, SignalBuffer}};

/// 计算实数序列的 FFT。
pub fn compute_fft(signal: &SignalBuffer) -> Result<FftOutput, SignalProcessError> {
	if signal.samples.is_empty() {
		return Err(SignalProcessError::EmptyInput);
	}
	if signal.sample_rate <= 0.0 {
		return Err(SignalProcessError::InvalidArgument(
			"采样率必须大于 0".to_string(),
		));
	}

	let mut planner = FftPlanner::<f32>::new();
	let fft = planner.plan_fft_forward(signal.samples.len());
	let mut bins: Vec<Complex32> = signal
		.samples
		.iter()
		.map(|sample| Complex32::new(*sample, 0.0))
		.collect();
	fft.process(&mut bins);

	Ok(FftOutput {
		sample_rate: signal.sample_rate,
		bins,
	})
}

#[cfg(test)]
mod tests {
	use crate::{SineWaveConfig, generate_sine_wave};

	use super::*;

	#[test]
	fn fft_bin_count_should_match_input_length() {
		let signal = generate_sine_wave(&SineWaveConfig {
			sample_rate: 256.0,
			frequency: 10.0,
			amplitude: 1.0,
			phase: 0.0,
			duration_secs: 1.0,
		})
		.unwrap_or_else(|error| panic!("生成测试正弦波失败: {error}"));

		let output = compute_fft(&signal).unwrap_or_else(|error| panic!("FFT 失败: {error}"));
		assert_eq!(output.bins.len(), signal.samples.len());
	}
}
