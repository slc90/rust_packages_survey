use std::{fs, path::PathBuf};

use plotters::prelude::*;
use signal_processing::{SineWaveConfig, generate_sine_wave};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let signal = generate_sine_wave(&SineWaveConfig {
		sample_rate: 256.0,
		frequency: 10.0,
		amplitude: 1.0,
		phase: 0.0,
		duration_secs: 2.0,
	})?;

	let output_dir = prepare_output_directory()?;
	let output_path = output_dir.join("generate_sine.png");

	let root = BitMapBackend::new(&output_path, (1280, 720)).into_drawing_area();
	root.fill(&WHITE)?;

	let time_axis: Vec<f32> = (0..signal.samples.len())
		.map(|index| index as f32 / signal.sample_rate)
		.collect();

	let mut chart = ChartBuilder::on(&root)
		.caption("10 Hz Sine Wave", ("sans-serif", 32))
		.margin(20)
		.x_label_area_size(40)
		.y_label_area_size(50)
		.build_cartesian_2d(0.0f32..2.0f32, -1.2f32..1.2f32)?;

	chart
		.configure_mesh()
		.x_desc("Time (s)")
		.y_desc("Amplitude")
		.draw()?;
	chart.draw_series(LineSeries::new(
		time_axis.into_iter().zip(signal.samples.iter().copied()),
		&BLUE,
	))?;
	root.present()?;

	println!("示例图已输出到 {}", output_path.display());
	Ok(())
}

fn prepare_output_directory() -> Result<PathBuf, std::io::Error> {
	let output_dir = PathBuf::from("signal_processing_output");
	fs::create_dir_all(&output_dir)?;
	Ok(output_dir)
}
