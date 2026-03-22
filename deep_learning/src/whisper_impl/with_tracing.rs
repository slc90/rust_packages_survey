use candle_core::{Module, Result, Tensor};
use candle_nn::VarBuilder;

#[derive(Debug, Clone)]
pub struct Linear {
	inner: candle_nn::Linear,
	span: tracing::Span,
}

pub fn linear(d1: usize, d2: usize, vb: VarBuilder) -> Result<Linear> {
	let inner = candle_nn::linear(d1, d2, vb)?;
	let span = tracing::span!(tracing::Level::TRACE, "linear");
	Ok(Linear { inner, span })
}

pub fn linear_no_bias(d1: usize, d2: usize, vb: VarBuilder) -> Result<Linear> {
	let inner = candle_nn::linear_no_bias(d1, d2, vb)?;
	let span = tracing::span!(tracing::Level::TRACE, "linear");
	Ok(Linear { inner, span })
}

impl Module for Linear {
	fn forward(&self, xs: &Tensor) -> Result<Tensor> {
		let _enter = self.span.enter();
		self.inner.forward(xs)
	}
}
