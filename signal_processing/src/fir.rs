use std::f32::consts::PI;

use crate::{
	error::SignalProcessError,
	types::{FilterKind, FirDesignConfig},
};

/// 设计窗函数法 FIR 系数。
pub fn design_fir(config: &FirDesignConfig) -> Result<Vec<f32>, SignalProcessError> {
	validate_fir_config(config)?;

	let last_index = config.tap_count - 1;
	let center = last_index as f32 / 2.0;
	let cutoff = normalized_cutoff(config)?;
	let mut coeffs = Vec::with_capacity(config.tap_count);

	for tap_index in 0..config.tap_count {
		let n = tap_index as f32 - center;
		let window =
			0.54 - 0.46 * (2.0 * PI * tap_index as f32 / last_index.max(1) as f32).cos();

		let ideal = match config.filter_kind {
			FilterKind::LowPass => low_pass_kernel(cutoff[0], n),
			FilterKind::HighPass => {
				let delta = if n == 0.0 { 1.0 } else { 0.0 };
				delta - low_pass_kernel(cutoff[0], n)
			}
			FilterKind::BandPass => low_pass_kernel(cutoff[1], n) - low_pass_kernel(cutoff[0], n),
			FilterKind::BandStop => {
				let delta = if n == 0.0 { 1.0 } else { 0.0 };
				delta - (low_pass_kernel(cutoff[1], n) - low_pass_kernel(cutoff[0], n))
			}
		};

		coeffs.push(ideal * window);
	}

	Ok(coeffs)
}

fn low_pass_kernel(cutoff: f32, n: f32) -> f32 {
	if n == 0.0 {
		2.0 * cutoff
	} else {
		(2.0 * cutoff * PI * n).sin() / (PI * n)
	}
}

fn normalized_cutoff(config: &FirDesignConfig) -> Result<[f32; 2], SignalProcessError> {
	let nyquist = config.sample_rate / 2.0;
	match config.filter_kind {
		FilterKind::LowPass | FilterKind::HighPass => Ok([config.cutoff_hz[0] / nyquist, 0.0]),
		FilterKind::BandPass | FilterKind::BandStop => {
			Ok([config.cutoff_hz[0] / nyquist, config.cutoff_hz[1] / nyquist])
		}
	}
}

fn validate_fir_config(config: &FirDesignConfig) -> Result<(), SignalProcessError> {
	if config.sample_rate <= 0.0 {
		return Err(SignalProcessError::InvalidArgument(
			"采样率必须大于 0".to_string(),
		));
	}
	if config.tap_count < 3 {
		return Err(SignalProcessError::InvalidArgument(
			"FIR tap 数至少为 3".to_string(),
		));
	}
	if config.tap_count.is_multiple_of(2) {
		return Err(SignalProcessError::InvalidArgument(
			"FIR tap 数建议使用奇数".to_string(),
		));
	}

	let nyquist = config.sample_rate / 2.0;
	match config.filter_kind {
		FilterKind::LowPass | FilterKind::HighPass => {
			if config.cutoff_hz.len() != 1 {
				return Err(SignalProcessError::InvalidArgument(
					"低通或高通需要一个截止频率".to_string(),
				));
			}
			if !(0.0 < config.cutoff_hz[0] && config.cutoff_hz[0] < nyquist) {
				return Err(SignalProcessError::InvalidArgument(
					"截止频率必须位于 (0, Nyquist) 内".to_string(),
				));
			}
		}
		FilterKind::BandPass | FilterKind::BandStop => {
			if config.cutoff_hz.len() != 2 {
				return Err(SignalProcessError::InvalidArgument(
					"带通或带阻需要两个截止频率".to_string(),
				));
			}
			if !(0.0 < config.cutoff_hz[0]
				&& config.cutoff_hz[0] < config.cutoff_hz[1]
				&& config.cutoff_hz[1] < nyquist)
			{
				return Err(SignalProcessError::InvalidArgument(
					"两个截止频率必须满足 0 < low < high < Nyquist".to_string(),
				));
			}
		}
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn fir_length_should_match_tap_count() {
		let coeffs = design_fir(&FirDesignConfig {
			sample_rate: 256.0,
			filter_kind: FilterKind::LowPass,
			cutoff_hz: vec![20.0],
			tap_count: 101,
		})
		.unwrap_or_else(|error| panic!("FIR 设计失败: {error}"));

		assert_eq!(coeffs.len(), 101);
	}
}
