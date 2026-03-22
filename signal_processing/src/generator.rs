use std::f32::consts::TAU;

use crate::{
	error::SignalProcessError,
	types::{CompositeComponent, SignalBuffer, SineWaveConfig},
};

/// 生成单频正弦波。
pub fn generate_sine_wave(config: &SineWaveConfig) -> Result<SignalBuffer, SignalProcessError> {
	validate_common_generator_arguments(config.sample_rate, config.duration_secs)?;
	if config.frequency < 0.0 {
		return Err(SignalProcessError::InvalidArgument(
			"频率必须大于等于 0".to_string(),
		));
	}

	let sample_count = (config.sample_rate * config.duration_secs).round() as usize;
	let mut samples = Vec::with_capacity(sample_count);
	for index in 0..sample_count {
		let time = index as f32 / config.sample_rate;
		let value = config.amplitude * (TAU * config.frequency * time + config.phase).sin();
		samples.push(value);
	}

	Ok(SignalBuffer {
		sample_rate: config.sample_rate,
		samples,
	})
}

/// 生成多频叠加测试信号。
pub fn generate_composite_signal(
	sample_rate: f32,
	duration_secs: f32,
	components: &[CompositeComponent],
) -> Result<SignalBuffer, SignalProcessError> {
	validate_common_generator_arguments(sample_rate, duration_secs)?;
	if components.is_empty() {
		return Err(SignalProcessError::InvalidArgument(
			"至少需要一个频率分量".to_string(),
		));
	}

	let sample_count = (sample_rate * duration_secs).round() as usize;
	let mut samples = Vec::with_capacity(sample_count);
	for index in 0..sample_count {
		let time = index as f32 / sample_rate;
		let value = components.iter().fold(0.0, |accumulator, component| {
			accumulator
				+ component.amplitude * (TAU * component.frequency * time + component.phase).sin()
		});
		samples.push(value);
	}

	Ok(SignalBuffer {
		sample_rate,
		samples,
	})
}

fn validate_common_generator_arguments(
	sample_rate: f32,
	duration_secs: f32,
) -> Result<(), SignalProcessError> {
	if sample_rate <= 0.0 {
		return Err(SignalProcessError::InvalidArgument(
			"采样率必须大于 0".to_string(),
		));
	}
	if duration_secs <= 0.0 {
		return Err(SignalProcessError::InvalidArgument(
			"时长必须大于 0".to_string(),
		));
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_generate_expected_sample_count() {
		let signal = generate_sine_wave(&SineWaveConfig {
			sample_rate: 256.0,
			frequency: 10.0,
			amplitude: 1.0,
			phase: 0.0,
			duration_secs: 2.0,
		})
		.unwrap_or_else(|error| panic!("生成测试正弦波失败: {error}"));

		assert_eq!(signal.samples.len(), 512);
	}
}
