#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod docx;
pub mod error;
pub mod image;
pub mod model;
pub mod pdf;

pub use docx::export_docx;
pub use error::ReportError;
pub use model::{ReportBlock, ReportDocument, ReportImage, ReportSection};
pub use pdf::export_pdf;
