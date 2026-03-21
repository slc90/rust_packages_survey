//! 医学影像基础库
//!
//! 当前阶段提供统一体数据结构、切片工具和窗宽窗位工具。

pub mod dicom_loader;
pub mod nifti_loader;
pub mod slice;
pub mod surface;
pub mod volume;
pub mod windowing;

pub use dicom_loader::{DicomSeriesInfo, load_dicom_series};
pub use nifti_loader::load_nifti_file;
pub use slice::{SliceAxis, SliceImage, extract_slice};
pub use surface::{SurfaceExtractOptions, SurfaceMeshData, SurfaceMeshStats, extract_isosurface};
pub use volume::{MedicalImageError, VolumeData, VolumeModality};
pub use windowing::{normalize_slice_to_u8, window_value};
