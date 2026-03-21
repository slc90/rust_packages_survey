use std::{fs::File, path::Path};

use docx_rs::{Docx, Paragraph, Pic, Run, Table, TableCell, TableRow};

use crate::{
	error::ReportError,
	image::read_image_bytes,
	model::{ReportBlock, ReportDocument},
};

/// 导出 DOCX 报告。
pub fn export_docx(document: &ReportDocument, path: &Path) -> Result<(), ReportError> {
	validate_document(document)?;

	let mut docx = Docx::new().add_paragraph(
		Paragraph::new().add_run(Run::new().add_text(&document.title).bold().size(36)),
	);

	if let Some(subtitle) = &document.subtitle {
		docx = docx.add_paragraph(
			Paragraph::new().add_run(Run::new().add_text(subtitle).italic().size(24)),
		);
	}

	for section in &document.sections {
		docx = docx.add_paragraph(
			Paragraph::new().add_run(Run::new().add_text(&section.heading).bold().size(28)),
		);

		for block in &section.blocks {
			match block {
				ReportBlock::Paragraph(text) => {
					docx = docx.add_paragraph(
						Paragraph::new().add_run(Run::new().add_text(text).size(22)),
					);
				}
				ReportBlock::KeyValueTable(rows) => {
					let table_rows = rows
						.iter()
						.map(|(key, value)| {
							TableRow::new(vec![
								TableCell::new().add_paragraph(
									Paragraph::new().add_run(Run::new().add_text(key).bold()),
								),
								TableCell::new().add_paragraph(
									Paragraph::new().add_run(Run::new().add_text(value)),
								),
							])
						})
						.collect::<Vec<_>>();
					docx = docx.add_table(Table::new(table_rows));
				}
				ReportBlock::Image(image) => {
					let image_bytes = read_image_bytes(&image.path)?;
					let pic = Pic::new(&image_bytes);
					docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_image(pic)));
					if let Some(caption) = &image.caption {
						docx = docx.add_paragraph(
							Paragraph::new().add_run(Run::new().add_text(caption).italic().size(20)),
						);
					}
				}
			}
		}
	}

	let file = File::create(path)?;
	docx.build()
		.pack(file)
		.map_err(|error| ReportError::Docx(error.to_string()))
}

fn validate_document(document: &ReportDocument) -> Result<(), ReportError> {
	if document.title.trim().is_empty() {
		return Err(ReportError::InvalidDocument(
			"报告标题不能为空".to_string(),
		));
	}
	Ok(())
}
