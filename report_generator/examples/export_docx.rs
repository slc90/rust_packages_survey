use std::{fs, path::PathBuf};

use plotters::prelude::*;
use report_generator::{ReportBlock, ReportDocument, ReportImage, ReportSection, export_docx};
use signal_processing::{CompositeComponent, compute_power_spectrum, generate_composite_signal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = prepare_output_directory()?;
	let chart_path = output_dir.join("report_spectrum.png");
	render_spectrum_chart(&chart_path)?;

	let report = ReportDocument {
		title: "Signal Processing Report".to_string(),
		subtitle: Some("DOCX Export Example".to_string()),
		sections: vec![
			ReportSection {
				heading: "Overview".to_string(),
				blocks: vec![ReportBlock::Paragraph(
					"This report demonstrates Word export with text, key-value table and an embedded spectrum image."
						.to_string(),
				)],
			},
			ReportSection {
				heading: "Metrics".to_string(),
				blocks: vec![ReportBlock::KeyValueTable(vec![
					("Sample Rate".to_string(), "256 Hz".to_string()),
					("Duration".to_string(), "4 s".to_string()),
					("Peak Frequencies".to_string(), "10 Hz, 40 Hz".to_string()),
				])],
			},
			ReportSection {
				heading: "Spectrum".to_string(),
				blocks: vec![ReportBlock::Image(ReportImage {
					path: chart_path,
					caption: Some("Power spectrum exported from signal_processing example".to_string()),
					width_px: Some(1024),
				})],
			},
		],
	};

	let report_path = output_dir.join("example_report.docx");
	export_docx(&report, &report_path)?;
	println!("DOCX 报告已输出到 {}", report_path.display());
	Ok(())
}

fn prepare_output_directory() -> Result<PathBuf, std::io::Error> {
	let output_dir = PathBuf::from("report_output");
	fs::create_dir_all(&output_dir)?;
	Ok(output_dir)
}

fn render_spectrum_chart(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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
	let max_y = spectrum
		.iter()
		.map(|point| point.value)
		.fold(0.0f32, f32::max)
		.max(1e-6);

	let root = BitMapBackend::new(path, (1024, 512)).into_drawing_area();
	root.fill(&WHITE)?;
	let mut chart = ChartBuilder::on(&root)
		.caption("Spectrum", ("sans-serif", 28))
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
	Ok(())
}
