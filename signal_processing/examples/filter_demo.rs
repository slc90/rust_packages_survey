use std::{fs, path::PathBuf};

use plotters::prelude::*;
use signal_processing::{
	CompositeComponent, FilterKind, FirDesignConfig, apply_fir, design_fir,
	generate_composite_signal,
};

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
				frequency: 50.0,
				amplitude: 0.5,
				phase: 0.0,
			},
		],
	)?;
	let coeffs = design_fir(&FirDesignConfig {
		sample_rate: 256.0,
		filter_kind: FilterKind::LowPass,
		cutoff_hz: vec![20.0],
		tap_count: 101,
	})?;
	let filtered = apply_fir(&signal, &coeffs)?;

	let output_dir = prepare_output_directory()?;
	let output_path = output_dir.join("filter_demo.png");
	let root = BitMapBackend::new(&output_path, (1280, 720)).into_drawing_area();
	root.fill(&WHITE)?;

	let areas = root.split_evenly((2, 1));
	let time_axis: Vec<f32> = (0..signal.samples.len())
		.map(|index| index as f32 / signal.sample_rate)
		.collect();

	let mut top_chart = ChartBuilder::on(&areas[0])
		.caption("Original Signal", ("sans-serif", 26))
		.margin(20)
		.x_label_area_size(30)
		.y_label_area_size(50)
		.build_cartesian_2d(0.0f32..4.0f32, -1.8f32..1.8f32)?;
	top_chart
		.configure_mesh()
		.x_desc("Time (s)")
		.y_desc("Amplitude")
		.draw()?;
	top_chart.draw_series(LineSeries::new(
		time_axis
			.iter()
			.copied()
			.zip(signal.samples.iter().copied()),
		&BLUE,
	))?;

	let mut bottom_chart = ChartBuilder::on(&areas[1])
		.caption("Filtered Signal", ("sans-serif", 26))
		.margin(20)
		.x_label_area_size(30)
		.y_label_area_size(50)
		.build_cartesian_2d(0.0f32..4.0f32, -1.8f32..1.8f32)?;
	bottom_chart
		.configure_mesh()
		.x_desc("Time (s)")
		.y_desc("Amplitude")
		.draw()?;
	bottom_chart.draw_series(LineSeries::new(
		time_axis.into_iter().zip(filtered.samples.iter().copied()),
		&GREEN,
	))?;

	root.present()?;
	println!("滤波对比图已输出到 {}", output_path.display());
	Ok(())
}

fn prepare_output_directory() -> Result<PathBuf, std::io::Error> {
	let output_dir = PathBuf::from("signal_processing_output");
	fs::create_dir_all(&output_dir)?;
	Ok(output_dir)
}
