use std::f32::consts::PI;

use crate::{
	error::SignalProcessError,
	types::{FilterKind, IirCoefficients, IirDesignConfig},
};

/// 设计二阶 IIR 系数。
pub fn design_iir(config: &IirDesignConfig) -> Result<IirCoefficients, SignalProcessError> {
	validate_iir_config(config)?;

	let sample_rate = config.sample_rate;
	let q = config.q;
	let omega = 2.0 * PI * config.cutoff_hz[0] / sample_rate;
	let cos_omega = omega.cos();
	let sin_omega = omega.sin();
	let alpha = sin_omega / (2.0 * q);

	match config.filter_kind {
		FilterKind::LowPass => normalize_coefficients(
			[
				(1.0 - cos_omega) * 0.5,
				1.0 - cos_omega,
				(1.0 - cos_omega) * 0.5,
			],
			[1.0 + alpha, -2.0 * cos_omega, 1.0 - alpha],
		),
		FilterKind::HighPass => normalize_coefficients(
			[
				(1.0 + cos_omega) * 0.5,
				-(1.0 + cos_omega),
				(1.0 + cos_omega) * 0.5,
			],
			[1.0 + alpha, -2.0 * cos_omega, 1.0 - alpha],
		),
		FilterKind::BandPass => {
			let center_frequency = (config.cutoff_hz[0] + config.cutoff_hz[1]) * 0.5;
			let bandwidth = config.cutoff_hz[1] - config.cutoff_hz[0];
			let band_q = center_frequency / bandwidth.max(f32::EPSILON);
			let band_omega = 2.0 * PI * center_frequency / sample_rate;
			let band_alpha = band_omega.sin() / (2.0 * band_q);
			normalize_coefficients(
				[band_alpha, 0.0, -band_alpha],
				[1.0 + band_alpha, -2.0 * band_omega.cos(), 1.0 - band_alpha],
			)
		}
		FilterKind::BandStop => {
			let center_frequency = (config.cutoff_hz[0] + config.cutoff_hz[1]) * 0.5;
			let bandwidth = config.cutoff_hz[1] - config.cutoff_hz[0];
			let band_q = center_frequency / bandwidth.max(f32::EPSILON);
			let band_omega = 2.0 * PI * center_frequency / sample_rate;
			let band_alpha = band_omega.sin() / (2.0 * band_q);
			normalize_coefficients(
				[1.0, -2.0 * band_omega.cos(), 1.0],
				[1.0 + band_alpha, -2.0 * band_omega.cos(), 1.0 - band_alpha],
			)
		}
	}
}

fn normalize_coefficients(b: [f32; 3], a: [f32; 3]) -> Result<IirCoefficients, SignalProcessError> {
	if a[0] == 0.0 {
		return Err(SignalProcessError::InvalidArgument(
			"IIR 系数归一化失败".to_string(),
		));
	}

	Ok(IirCoefficients {
		b: [b[0] / a[0], b[1] / a[0], b[2] / a[0]],
		a: [1.0, a[1] / a[0], a[2] / a[0]],
	})
}

fn validate_iir_config(config: &IirDesignConfig) -> Result<(), SignalProcessError> {
	if config.sample_rate <= 0.0 {
		return Err(SignalProcessError::InvalidArgument(
			"采样率必须大于 0".to_string(),
		));
	}
	if config.q <= 0.0 {
		return Err(SignalProcessError::InvalidArgument(
			"Q 值必须大于 0".to_string(),
		));
	}

	let nyquist = config.sample_rate / 2.0;
	match config.filter_kind {
		FilterKind::LowPass | FilterKind::HighPass => {
			let cutoff = config.cutoff_hz[0];
			if !(0.0 < cutoff && cutoff < nyquist) {
				return Err(SignalProcessError::InvalidArgument(
					"截止频率必须位于 (0, Nyquist) 内".to_string(),
				));
			}
		}
		FilterKind::BandPass | FilterKind::BandStop => {
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
	fn iir_should_normalize_a0_to_one() {
		let coeffs = design_iir(&IirDesignConfig {
			sample_rate: 256.0,
			filter_kind: FilterKind::LowPass,
			cutoff_hz: [20.0, 0.0],
			q: 1.0 / 2.0_f32.sqrt(),
		})
		.unwrap_or_else(|error| panic!("IIR 设计失败: {error}"));

		assert!((coeffs.a[0] - 1.0).abs() < 1e-6);
	}
}
