use crate::{
	error::SignalProcessError,
	types::{IirCoefficients, SignalBuffer},
};

/// 对序列执行 FIR 滤波。
pub fn apply_fir(
	signal: &SignalBuffer,
	coeffs: &[f32],
) -> Result<SignalBuffer, SignalProcessError> {
	if signal.samples.is_empty() {
		return Err(SignalProcessError::EmptyInput);
	}
	if coeffs.is_empty() {
		return Err(SignalProcessError::InvalidArgument(
			"FIR 系数不能为空".to_string(),
		));
	}

	let mut output = vec![0.0; signal.samples.len()];
	for (output_index, output_value) in output.iter_mut().enumerate() {
		let mut value = 0.0;
		for (coeff_index, coeff) in coeffs.iter().enumerate() {
			if output_index >= coeff_index {
				value += coeff * signal.samples[output_index - coeff_index];
			}
		}
		*output_value = value;
	}

	Ok(SignalBuffer {
		sample_rate: signal.sample_rate,
		samples: output,
	})
}

/// 对序列执行二阶 IIR 滤波。
pub fn apply_iir(
	signal: &SignalBuffer,
	coeffs: &IirCoefficients,
) -> Result<SignalBuffer, SignalProcessError> {
	if signal.samples.is_empty() {
		return Err(SignalProcessError::EmptyInput);
	}

	let mut output = vec![0.0; signal.samples.len()];
	for index in 0..signal.samples.len() {
		let x0 = signal.samples[index];
		let x1 = if index >= 1 { signal.samples[index - 1] } else { 0.0 };
		let x2 = if index >= 2 { signal.samples[index - 2] } else { 0.0 };
		let y1 = if index >= 1 { output[index - 1] } else { 0.0 };
		let y2 = if index >= 2 { output[index - 2] } else { 0.0 };

		output[index] = coeffs.b[0] * x0 + coeffs.b[1] * x1 + coeffs.b[2] * x2
			- coeffs.a[1] * y1
			- coeffs.a[2] * y2;
	}

	Ok(SignalBuffer {
		sample_rate: signal.sample_rate,
		samples: output,
	})
}

#[cfg(test)]
mod tests {
	use crate::{
		FilterKind, FirDesignConfig, IirDesignConfig, design_fir, design_iir,
		generate_composite_signal,
	};

	use super::*;

	#[test]
	fn fir_filter_should_keep_same_length() {
		let signal = generate_composite_signal(
			256.0,
			1.0,
			&[
				crate::CompositeComponent {
					frequency: 10.0,
					amplitude: 1.0,
					phase: 0.0,
				},
				crate::CompositeComponent {
					frequency: 40.0,
					amplitude: 0.4,
					phase: 0.0,
				},
			],
		)
		.unwrap_or_else(|error| panic!("复合信号生成失败: {error}"));
		let coeffs = design_fir(&FirDesignConfig {
			sample_rate: 256.0,
			filter_kind: FilterKind::LowPass,
			cutoff_hz: vec![20.0],
			tap_count: 101,
		})
		.unwrap_or_else(|error| panic!("FIR 设计失败: {error}"));

		let filtered =
			apply_fir(&signal, &coeffs).unwrap_or_else(|error| panic!("FIR 滤波失败: {error}"));
		assert_eq!(filtered.samples.len(), signal.samples.len());
	}

	#[test]
	fn iir_filter_should_keep_same_length() {
		let signal = SignalBuffer {
			sample_rate: 256.0,
			samples: vec![1.0; 128],
		};
		let coeffs = design_iir(&IirDesignConfig {
			sample_rate: 256.0,
			filter_kind: FilterKind::LowPass,
			cutoff_hz: [20.0, 0.0],
			q: 1.0 / 2.0_f32.sqrt(),
		})
		.unwrap_or_else(|error| panic!("IIR 设计失败: {error}"));

		let filtered =
			apply_iir(&signal, &coeffs).unwrap_or_else(|error| panic!("IIR 滤波失败: {error}"));
		assert_eq!(filtered.samples.len(), signal.samples.len());
	}
}
