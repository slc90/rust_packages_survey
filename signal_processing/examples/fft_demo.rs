use std::{fs, path::PathBuf};

use plotters::prelude::*;
use signal_processing::{CompositeComponent, compute_power_spectrum, generate_composite_signal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
	)?;
	let spectrum = compute_power_spectrum(&signal)?;
	let output_dir = prepare_output_directory()?;
	let output_path = output_dir.join("fft_demo.png");

	let max_y = spectrum
		.iter()
		.map(|point| point.value)
		.fold(0.0f32, f32::max)
		.max(1e-6);

	let root = BitMapBackend::new(&output_path, (1280, 720)).into_drawing_area();
	root.fill(&WHITE)?;

	let mut chart = ChartBuilder::on(&root)
		.caption("Power Spectrum", ("sans-serif", 32))
		.margin(20)
		.x_label_area_size(40)
		.y_label_area_size(50)
		.build_cartesian_2d(0.0f32..128.0f32, 0.0f32..max_y * 1.1)?;

	chart
		.configure_mesh()
		.x_desc("Frequency (Hz)")
		.y_desc("Power")
		.draw()?;
	chart.draw_series(LineSeries::new(
		spectrum
			.into_iter()
			.map(|point| (point.frequency, point.value)),
		&RED,
	))?;
	root.present()?;

	println!("频谱图已输出到 {}", output_path.display());
	Ok(())
}

fn prepare_output_directory() -> Result<PathBuf, std::io::Error> {
	let output_dir = PathBuf::from("signal_processing_output");
	fs::create_dir_all(&output_dir)?;
	Ok(output_dir)
}
