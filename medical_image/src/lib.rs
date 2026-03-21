//! 医学影像基础库
//!
//! 当前阶段提供统一体数据结构、切片工具和窗宽窗位工具。

pub mod slice;
pub mod volume;
pub mod windowing;

pub use slice::{SliceAxis, SliceImage, extract_slice};
pub use volume::{MedicalImageError, VolumeData, VolumeModality};
pub use windowing::{normalize_slice_to_u8, window_value};
