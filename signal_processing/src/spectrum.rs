use rustfft::{FftPlanner, num_complex::Complex32};

use crate::{
	error::SignalProcessError,
	types::{SignalBuffer, SpectrumPoint, WelchSpectrumConfig},
};

/// 计算 Welch 单边功率谱。
///
/// 这里采用 Welch, P. D. 1967 提出的分段加窗平均方法，
/// 参考文献：The use of fast Fourier transform for the estimation of power spectra,
/// doi: 10.1109/TAU.1967.1161901
pub fn compute_power_spectrum(
	signal: &SignalBuffer,
) -> Result<Vec<SpectrumPoint>, SignalProcessError> {
	let default_config = build_default_welch_config(signal.samples.len())?;
	compute_power_spectrum_with_config(signal, default_config)
}

/// 按指定配置计算 Welch 单边功率谱。
pub fn compute_power_spectrum_with_config(
	signal: &SignalBuffer,
	config: WelchSpectrumConfig,
) -> Result<Vec<SpectrumPoint>, SignalProcessError> {
	validate_welch_arguments(signal, config)?;

	let step = config.segment_length - config.overlap_length;
	let segment_starts = build_segment_starts(signal.samples.len(), config.segment_length, step);
	let half = config.segment_length / 2;
	let window = build_hann_window(config.segment_length);
	let window_power = window.iter().map(|value| value * value).sum::<f32>();
	let normalization = signal.sample_rate * window_power;
	let mut planner = FftPlanner::<f32>::new();
	let fft = planner.plan_fft_forward(config.segment_length);
	let mut averaged_psd = vec![0.0f32; half + 1];

	for start in segment_starts.iter().copied() {
		let segment = &signal.samples[start..start + config.segment_length];
		let mut bins: Vec<Complex32> = segment
			.iter()
			.zip(window.iter())
			.map(|(sample, weight)| Complex32::new(sample * weight, 0.0))
			.collect();
		fft.process(&mut bins);

		for index in 0..=half {
			let mut power = bins[index].norm_sqr() / normalization;
			if index != 0 && !(config.segment_length.is_multiple_of(2) && index == half) {
				power *= 2.0;
			}
			averaged_psd[index] += power;
		}
	}

	let segment_count = segment_starts.len() as f32;
	let points = averaged_psd
		.into_iter()
		.enumerate()
		.map(|(index, power)| SpectrumPoint {
			frequency: index as f32 * signal.sample_rate / config.segment_length as f32,
			value: power / segment_count,
		})
		.collect();
	Ok(points)
}

/// 构造默认 Welch 配置。
fn build_default_welch_config(
	sample_count: usize,
) -> Result<WelchSpectrumConfig, SignalProcessError> {
	if sample_count == 0 {
		return Err(SignalProcessError::EmptyInput);
	}

	let segment_length = sample_count.clamp(8, 256);
	let overlap_length = segment_length / 2;
	Ok(WelchSpectrumConfig {
		segment_length,
		overlap_length,
	})
}

/// 校验 Welch 参数。
fn validate_welch_arguments(
	signal: &SignalBuffer,
	config: WelchSpectrumConfig,
) -> Result<(), SignalProcessError> {
	if signal.samples.is_empty() {
		return Err(SignalProcessError::EmptyInput);
	}
	if signal.sample_rate <= 0.0 {
		return Err(SignalProcessError::InvalidArgument(
			"采样率必须大于 0".to_string(),
		));
	}
	if config.segment_length < 8 {
		return Err(SignalProcessError::InvalidArgument(
			"Welch 分段长度必须大于等于 8".to_string(),
		));
	}
	if config.segment_length > signal.samples.len() {
		return Err(SignalProcessError::InvalidArgument(
			"Welch 分段长度不能超过输入样本数量".to_string(),
		));
	}
	if config.overlap_length >= config.segment_length {
		return Err(SignalProcessError::InvalidArgument(
			"Welch 重叠长度必须小于分段长度".to_string(),
		));
	}
	Ok(())
}

/// 构造分段起点列表。
fn build_segment_starts(sample_count: usize, segment_length: usize, step: usize) -> Vec<usize> {
	let mut starts = Vec::new();
	let mut start = 0usize;
	while start + segment_length <= sample_count {
		starts.push(start);
		start += step;
	}
	starts
}

/// 构造 Hann 窗。
fn build_hann_window(length: usize) -> Vec<f32> {
	if length == 1 {
		return vec![1.0];
	}

	(0..length)
		.map(|index| {
			let phase = 2.0 * std::f32::consts::PI * index as f32 / (length - 1) as f32;
			0.5 - 0.5 * phase.cos()
		})
		.collect()
}

#[cfg(test)]
mod tests {
	use crate::{
		CompositeComponent, generate_composite_signal, generate_sine_wave, types::SineWaveConfig,
	};

	use super::*;

	#[test]
	fn should_find_peak_near_target_frequency_with_welch() {
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

		assert!((peak.frequency - 10.0).abs() < 1.0);
	}

	#[test]
	fn should_find_two_major_peaks_for_composite_signal() {
		let signal = generate_composite_signal(
			256.0,
			4.0,
			&[
				CompositeComponent {
					frequency: 10.0,
					amplitude: 1.0,
					phase: 0.0,
				},
				CompositeComponent {
					frequency: 40.0,
					amplitude: 0.4,
					phase: 0.0,
				},
			],
		)
		.unwrap_or_else(|error| panic!("生成复合信号失败: {error}"));

		let spectrum = compute_power_spectrum_with_config(
			&signal,
			WelchSpectrumConfig {
				segment_length: 256,
				overlap_length: 128,
			},
		)
		.unwrap_or_else(|error| panic!("Welch 功率谱失败: {error}"));

		let mut strongest_bins = spectrum.clone();
		strongest_bins.sort_by(|left, right| right.value.total_cmp(&left.value));
		let frequencies: Vec<f32> = strongest_bins
			.iter()
			.take(6)
			.map(|point| point.frequency)
			.collect();

		assert!(
			frequencies
				.iter()
				.any(|frequency| (*frequency - 10.0).abs() < 1.0)
		);
		assert!(
			frequencies
				.iter()
				.any(|frequency| (*frequency - 40.0).abs() < 1.0)
		);
	}
}
