use crate::{
	error::SignalProcessError,
	fft::compute_fft,
	types::{SignalBuffer, SpectrumPoint},
};

/// 计算单边功率谱。
pub fn compute_power_spectrum(
	signal: &SignalBuffer,
) -> Result<Vec<SpectrumPoint>, SignalProcessError> {
	let fft_output = compute_fft(signal)?;
	let sample_count = fft_output.bins.len();
	if sample_count == 0 {
		return Err(SignalProcessError::EmptyInput);
	}

	let half = sample_count / 2;
	let normalization = sample_count as f32;
	let mut points = Vec::with_capacity(half + 1);

	for index in 0..=half {
		let bin = fft_output.bins[index];
		let power = (bin.norm_sqr() / normalization).max(0.0);
		let frequency = index as f32 * fft_output.sample_rate / sample_count as f32;
		points.push(SpectrumPoint {
			frequency,
			value: power,
		});
	}

	Ok(points)
}

#[cfg(test)]
mod tests {
	use crate::{SineWaveConfig, generate_sine_wave};

	use super::*;

	#[test]
	fn should_find_peak_near_target_frequency() {
		let signal = generate_sine_wave(&SineWaveConfig {
			sample_rate: 256.0,
			frequency: 10.0,
			amplitude: 1.0,
			phase: 0.0,
			duration_secs: 4.0,
		})
		.unwrap_or_else(|error| panic!("生成测试正弦波失败: {error}"));

		let spectrum =
			compute_power_spectrum(&signal).unwrap_or_else(|error| panic!("功率谱失败: {error}"));
		let peak = spectrum
			.iter()
			.max_by(|left, right| left.value.total_cmp(&right.value))
			.unwrap_or_else(|| panic!("功率谱为空"));

		assert!((peak.frequency - 10.0).abs() < 0.5);
	}
}
