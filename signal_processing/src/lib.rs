#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod error;
pub mod fft;
pub mod filter;
pub mod fir;
pub mod generator;
pub mod iir;
pub mod spectrum;
pub mod types;

pub use error::SignalProcessError;
pub use fft::compute_fft;
pub use filter::{apply_fir, apply_iir};
pub use fir::design_fir;
pub use generator::{generate_composite_signal, generate_sine_wave};
pub use iir::design_iir;
pub use spectrum::compute_power_spectrum;
pub use types::{
	CompositeComponent, FftOutput, FilterKind, FirDesignConfig, IirCoefficients, IirDesignConfig,
	SignalBuffer, SineWaveConfig, SpectrumPoint,
};
