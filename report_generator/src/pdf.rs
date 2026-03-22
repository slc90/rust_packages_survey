use std::{fs, path::Path};

use printpdf::{
	BuiltinFont, Mm, Op, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt, RawImage,
	TextItem, XObjectTransform,
};

use crate::{
	error::ReportError,
	image::read_image_bytes,
	model::{ReportBlock, ReportDocument},
};

/// 导出 PDF 报告。
pub fn export_pdf(document: &ReportDocument, path: &Path) -> Result<(), ReportError> {
	validate_document(document)?;

	let mut pdf = PdfDocument::new(&document.title);
	let mut ops = Vec::new();
	let mut cursor_y = 280.0_f32;

	write_text_line(
		&mut ops,
		&document.title,
		24.0,
		20.0,
		cursor_y,
		BuiltinFont::HelveticaBold,
	);
	cursor_y -= 12.0;

	if let Some(subtitle) = &document.subtitle {
		write_text_line(
			&mut ops,
			subtitle,
			14.0,
			20.0,
			cursor_y,
			BuiltinFont::HelveticaOblique,
		);
		cursor_y -= 12.0;
	}

	for section in &document.sections {
		cursor_y -= 8.0;
		write_text_line(
			&mut ops,
			&section.heading,
			18.0,
			20.0,
			cursor_y,
			BuiltinFont::HelveticaBold,
		);
		cursor_y -= 10.0;

		for block in &section.blocks {
			match block {
				ReportBlock::Paragraph(text) => {
					for line in wrap_text(text, 80) {
						write_text_line(
							&mut ops,
							&line,
							11.0,
							24.0,
							cursor_y,
							BuiltinFont::Helvetica,
						);
						cursor_y -= 6.0;
					}
				}
				ReportBlock::KeyValueTable(rows) => {
					for (key, value) in rows {
						write_text_line(
							&mut ops,
							&format!("{key}: {value}"),
							11.0,
							24.0,
							cursor_y,
							BuiltinFont::Helvetica,
						);
						cursor_y -= 6.0;
					}
				}
				ReportBlock::Image(image) => {
					let image_bytes = read_image_bytes(&image.path)?;
					let raw_image = RawImage::decode_from_bytes(&image_bytes, &mut Vec::new())
						.map_err(ReportError::Pdf)?;
					let image_id = pdf.add_image(&raw_image);

					ops.push(Op::UseXobject {
						id: image_id,
						transform: XObjectTransform {
							translate_x: Some(Pt(24.0)),
							translate_y: Some(Pt(cursor_y * 2.834_645_7)),
							scale_x: Some(0.30),
							scale_y: Some(0.30),
							..Default::default()
						},
					});
					cursor_y -= 70.0;

					if let Some(caption) = &image.caption {
						write_text_line(
							&mut ops,
							caption,
							10.0,
							24.0,
							cursor_y,
							BuiltinFont::HelveticaOblique,
						);
						cursor_y -= 8.0;
					}
				}
			}

			if cursor_y < 30.0 {
				break;
			}
		}
	}

	let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);
	let bytes = pdf
		.with_pages(vec![page])
		.save(&PdfSaveOptions::default(), &mut Vec::new());
	fs::write(path, bytes)?;
	Ok(())
}

fn write_text_line(
	ops: &mut Vec<Op>,
	text: &str,
	font_size: f32,
	x_mm: f32,
	y_mm: f32,
	font: BuiltinFont,
) {
	ops.push(Op::StartTextSection);
	ops.push(Op::SetFont {
		font: PdfFontHandle::Builtin(font),
		size: Pt(font_size),
	});
	ops.push(Op::SetTextCursor {
		pos: Point::new(Mm(x_mm), Mm(y_mm)),
	});
	ops.push(Op::ShowText {
		items: vec![TextItem::Text(text.to_string())],
	});
	ops.push(Op::EndTextSection);
}

fn validate_document(document: &ReportDocument) -> Result<(), ReportError> {
	if document.title.trim().is_empty() {
		return Err(ReportError::InvalidDocument("报告标题不能为空".to_string()));
	}
	Ok(())
}

fn wrap_text(text: &str, max_len: usize) -> Vec<String> {
	let mut lines = Vec::new();
	let mut current = String::new();

	for word in text.split_whitespace() {
		let next_len = if current.is_empty() {
			word.len()
		} else {
			current.len() + 1 + word.len()
		};

		if next_len > max_len && !current.is_empty() {
			lines.push(current);
			current = word.to_string();
		} else {
			if !current.is_empty() {
				current.push(' ');
			}
			current.push_str(word);
		}
	}

	if !current.is_empty() {
		lines.push(current);
	}

	lines
}
