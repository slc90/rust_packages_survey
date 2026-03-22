//! DICOM Series 加载工具

use crate::volume::{MedicalImageError, VolumeData, VolumeModality};
use dicom_object::open_file;
use dicom_pixeldata::PixelDecoder;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// DICOM 序列基本信息
#[derive(Debug, Clone, PartialEq)]
pub struct DicomSeriesInfo {
	/// 序列 UID
	pub series_instance_uid: String,
	/// 模态
	pub modality: VolumeModality,
	/// 切片文件列表
	pub files: Vec<PathBuf>,
}

#[derive(Debug)]
struct DicomSlice {
	modality: VolumeModality,
	rows: usize,
	cols: usize,
	pixels: Vec<f32>,
	instance_number: i32,
	position: Vec<f32>,
	spacing_xy: [f32; 2],
	slice_thickness: f32,
	orientation: Vec<f32>,
}

/// 从目录中加载 DICOM 序列
pub fn load_dicom_series<P: AsRef<Path>>(directory: P) -> Result<VolumeData, MedicalImageError> {
	let directory = directory.as_ref();
	let files = collect_dicom_files(directory)?;
	info!(
		directory = %directory.display(),
		file_count = files.len(),
		"开始加载 DICOM 序列目录"
	);
	if files.is_empty() {
		return Err(MedicalImageError::Format(format!(
			"DICOM 目录中没有可用文件: {}",
			directory.display()
		)));
	}

	let mut slices = Vec::new();
	for file in files {
		let object =
			open_file(&file).map_err(|error| MedicalImageError::Format(error.to_string()))?;
		let modality_text = object
			.element_by_name("Modality")
			.map_err(|error| MedicalImageError::Format(error.to_string()))?
			.to_str()
			.map_err(|error| MedicalImageError::Format(error.to_string()))?;
		let modality = VolumeModality::parse_supported(modality_text.as_ref())?;
		let position = parse_multi_f32(
			object
				.element_by_name("ImagePositionPatient")
				.ok()
				.and_then(|element| element.to_str().ok())
				.as_deref(),
		);
		let instance_number = object
			.element_by_name("InstanceNumber")
			.ok()
			.and_then(|element| element.to_str().ok())
			.and_then(|value| value.parse::<i32>().ok())
			.unwrap_or_default();
		let spacing_values = parse_multi_f32(
			object
				.element_by_name("PixelSpacing")
				.ok()
				.and_then(|element| element.to_str().ok())
				.as_deref(),
		);
		let slice_thickness = object
			.element_by_name("SliceThickness")
			.ok()
			.and_then(|element| element.to_str().ok())
			.and_then(|value| value.parse::<f32>().ok())
			.unwrap_or(1.0);
		let orientation = parse_multi_f32(
			object
				.element_by_name("ImageOrientationPatient")
				.ok()
				.and_then(|element| element.to_str().ok())
				.as_deref(),
		);
		let rows = parse_usize_element(&object, "Rows")?
			.ok_or_else(|| MedicalImageError::Format("DICOM 切片缺少 Rows 字段".to_string()))?;
		let cols = parse_usize_element(&object, "Columns")?
			.ok_or_else(|| MedicalImageError::Format("DICOM 切片缺少 Columns 字段".to_string()))?;
		let frames = parse_usize_element(&object, "NumberOfFrames")?.unwrap_or(1);
		let samples_per_pixel = parse_usize_element(&object, "SamplesPerPixel")?.unwrap_or(1);
		let photometric_interpretation =
			parse_string_element(&object, "PhotometricInterpretation")?
				.unwrap_or_else(|| "-".to_string());
		if frames != 1 {
			warn!(
				file = %file.display(),
				rows,
				cols,
				frames,
				samples_per_pixel,
				photometric_interpretation,
				"检测到当前未支持的 DICOM 帧数或通道数"
			);
			return Err(MedicalImageError::Format(format!(
				"当前仅支持单帧 DICOM 切片，实际 Rows={rows}, Columns={cols}, NumberOfFrames={frames}, SamplesPerPixel={samples_per_pixel}, PhotometricInterpretation={photometric_interpretation}"
			)));
		}

		let pixel_data = object
			.decode_pixel_data()
			.map_err(|error| MedicalImageError::Format(error.to_string()))?;
		let array = pixel_data
			.to_ndarray::<f32>()
			.map_err(|error| MedicalImageError::Format(error.to_string()))?;
		let decoded_dims = format!("{:?}", array.dim());
		let pixels: Vec<f32> = array.iter().copied().collect();
		info!(
			file = %file.display(),
			rows,
			cols,
			frames,
			samples_per_pixel,
			photometric_interpretation,
			decoded_dims = %decoded_dims,
			decoded_pixel_count = pixels.len(),
			"完成 DICOM 切片像素解码"
		);
		let pixels = if samples_per_pixel == 1 && pixels.len() == rows * cols {
			pixels
		} else if samples_per_pixel == 3 && pixels.len() == rows * cols * 3 {
			info!(
				file = %file.display(),
				rows,
				cols,
				decoded_dims = %decoded_dims,
				"检测到 RGB DICOM 切片，转换为灰度体素"
			);
			convert_rgb_pixels_to_grayscale(&pixels)
		} else {
			warn!(
				file = %file.display(),
				rows,
				cols,
				expected_pixel_count = rows * cols * samples_per_pixel,
				decoded_pixel_count = pixels.len(),
				decoded_dims = %decoded_dims,
				samples_per_pixel,
				photometric_interpretation,
				"解码后的像素数量与单帧单通道切片不一致"
			);
			return Err(MedicalImageError::Format(format!(
				"当前仅支持单帧单通道或 RGB DICOM 切片，实际 Rows={rows}, Columns={cols}, NumberOfFrames={frames}, SamplesPerPixel={samples_per_pixel}, PhotometricInterpretation={photometric_interpretation}, 解码维度={decoded_dims}, 解码像素数={}",
				pixels.len()
			)));
		};

		slices.push(DicomSlice {
			modality,
			rows,
			cols,
			pixels,
			instance_number,
			position,
			spacing_xy: [
				spacing_values.first().copied().unwrap_or(1.0),
				spacing_values.get(1).copied().unwrap_or(1.0),
			],
			slice_thickness,
			orientation,
		});
	}

	slices.sort_by(|left, right| {
		let left_key = left
			.position
			.get(2)
			.copied()
			.unwrap_or(left.instance_number as f32);
		let right_key = right
			.position
			.get(2)
			.copied()
			.unwrap_or(right.instance_number as f32);
		left_key
			.partial_cmp(&right_key)
			.unwrap_or(std::cmp::Ordering::Equal)
	});

	let first = slices
		.first()
		.ok_or_else(|| MedicalImageError::Format("没有可用的 DICOM 切片".to_string()))?;
	let rows = first.rows;
	let cols = first.cols;
	let dims = [cols, rows, slices.len()];
	let mut voxels = Vec::with_capacity(cols * rows * slices.len());
	for slice in &slices {
		if slice.rows != rows || slice.cols != cols {
			return Err(MedicalImageError::Format(
				"DICOM 序列中的切片尺寸不一致".to_string(),
			));
		}
		voxels.extend(slice.pixels.iter().copied());
	}

	let spacing = [
		first.spacing_xy[1],
		first.spacing_xy[0],
		first.slice_thickness.max(1.0),
	];
	let origin = [
		first.position.first().copied().unwrap_or(0.0),
		first.position.get(1).copied().unwrap_or(0.0),
		first.position.get(2).copied().unwrap_or(0.0),
	];
	let direction = build_direction(&first.orientation);
	let affine = build_affine(spacing, origin, direction);

	VolumeData::new(
		dims,
		spacing,
		origin,
		direction,
		affine,
		voxels,
		first.modality,
	)
}

/// 递归收集 DICOM 文件
fn collect_dicom_files(directory: &Path) -> Result<Vec<PathBuf>, MedicalImageError> {
	let mut files = Vec::new();
	for entry in fs::read_dir(directory)? {
		let entry = entry?;
		let path = entry.path();
		if should_skip_path(&entry, &path) {
			continue;
		}
		if path.is_dir() {
			files.extend(collect_dicom_files(&path)?);
		} else {
			files.push(path);
		}
	}
	Ok(files)
}

/// 判断目录扫描时是否应跳过当前路径。
fn should_skip_path(entry: &fs::DirEntry, path: &Path) -> bool {
	let file_name = path
		.file_name()
		.and_then(|value| value.to_str())
		.unwrap_or_default();
	if file_name.starts_with('.') {
		return true;
	}

	if let Ok(file_type) = entry.file_type()
		&& file_type.is_dir()
		&& file_name.eq_ignore_ascii_case(".temp")
	{
		return true;
	}

	false
}

/// 解析 DICOM 多值字段
fn parse_multi_f32(value: Option<&str>) -> Vec<f32> {
	value
		.unwrap_or_default()
		.split('\\')
		.filter_map(|item| item.trim().parse::<f32>().ok())
		.collect()
}

/// 将 RGB 像素转换为灰度体素。
///
/// 这里使用 ITU-R BT.601 常见亮度加权系数，将 RGB 三通道压缩为单通道，
/// 以便复用当前仅支持标量体数据的三视图与三维渲染链路。
fn convert_rgb_pixels_to_grayscale(rgb_pixels: &[f32]) -> Vec<f32> {
	let mut grayscale_pixels = Vec::with_capacity(rgb_pixels.len() / 3);
	for rgb in rgb_pixels.chunks_exact(3) {
		let value = 0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2];
		grayscale_pixels.push(value);
	}
	grayscale_pixels
}

/// 解析 DICOM 中的单值无符号整数标签。
fn parse_usize_element(
	object: &dicom_object::DefaultDicomObject,
	name: &str,
) -> Result<Option<usize>, MedicalImageError> {
	let Some(element) = object.element_by_name(name).ok() else {
		return Ok(None);
	};
	let value = element
		.to_str()
		.map_err(|error| MedicalImageError::Format(error.to_string()))?;
	let parsed = value
		.trim()
		.parse::<usize>()
		.map_err(|error| MedicalImageError::Format(error.to_string()))?;
	Ok(Some(parsed))
}

/// 解析 DICOM 中的单值字符串标签。
fn parse_string_element(
	object: &dicom_object::DefaultDicomObject,
	name: &str,
) -> Result<Option<String>, MedicalImageError> {
	let Some(element) = object.element_by_name(name).ok() else {
		return Ok(None);
	};
	let value = element
		.to_str()
		.map_err(|error| MedicalImageError::Format(error.to_string()))?;
	Ok(Some(value.trim().to_string()))
}

/// 根据方向余弦构造方向矩阵
fn build_direction(orientation: &[f32]) -> [[f32; 3]; 3] {
	if orientation.len() >= 6 {
		let row = [orientation[0], orientation[1], orientation[2]];
		let col = [orientation[3], orientation[4], orientation[5]];
		let normal = cross(row, col);
		[row, col, normal]
	} else {
		[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
	}
}

/// 构造 affine 矩阵
fn build_affine(spacing: [f32; 3], origin: [f32; 3], direction: [[f32; 3]; 3]) -> [[f32; 4]; 4] {
	[
		[
			direction[0][0] * spacing[0],
			direction[1][0] * spacing[1],
			direction[2][0] * spacing[2],
			origin[0],
		],
		[
			direction[0][1] * spacing[0],
			direction[1][1] * spacing[1],
			direction[2][1] * spacing[2],
			origin[1],
		],
		[
			direction[0][2] * spacing[0],
			direction[1][2] * spacing[1],
			direction[2][2] * spacing[2],
			origin[2],
		],
		[0.0, 0.0, 0.0, 1.0],
	]
}

/// 三维向量叉积
fn cross(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
	[
		left[1] * right[2] - left[2] * right[1],
		left[2] * right[0] - left[0] * right[2],
		left[0] * right[1] - left[1] * right[0],
	]
}

#[cfg(test)]
mod tests {
	use std::path::Path;

	use dicom_pixeldata::PixelDecoder;

	use super::{
		convert_rgb_pixels_to_grayscale, load_dicom_series, parse_multi_f32, should_skip_path,
	};

	#[test]
	fn should_parse_multi_value_f32() {
		assert_eq!(parse_multi_f32(Some("1.0\\2.5\\3.5")), vec![1.0, 2.5, 3.5]);
		assert!(parse_multi_f32(None).is_empty());
	}

	#[test]
	fn should_skip_hidden_sync_sidecar_file() {
		let temp_dir = std::env::temp_dir().join(format!(
			"rust_packages_survey_dicom_test_{}",
			std::process::id()
		));
		std::fs::create_dir_all(&temp_dir).expect("创建临时目录失败");
		let hidden_path = temp_dir.join(".WeDrive");
		std::fs::write(&hidden_path, "sidecar").expect("写入隐藏文件失败");
		let entry = std::fs::read_dir(&temp_dir)
			.expect("读取临时目录失败")
			.next()
			.expect("缺少目录项")
			.expect("读取目录项失败");

		assert!(should_skip_path(&entry, Path::new(&hidden_path)));

		let _ = std::fs::remove_file(&hidden_path);
		let _ = std::fs::remove_dir(&temp_dir);
	}

	#[test]
	fn should_convert_rgb_pixels_to_grayscale() {
		let grayscale = convert_rgb_pixels_to_grayscale(&[255.0, 0.0, 0.0, 0.0, 255.0, 0.0]);

		assert_eq!(grayscale.len(), 2);
		assert!((grayscale[0] - 76.245).abs() < 0.01);
		assert!((grayscale[1] - 149.685).abs() < 0.01);
	}

	#[test]
	fn debug_local_ct_dicom_layout_when_available() {
		let sample_path = Path::new(env!("CARGO_MANIFEST_DIR"))
			.parent()
			.unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")))
			.join("data")
			.join("CT_DICOM")
			.join("CT_DICOM (1).dcm");
		if !sample_path.exists() {
			return;
		}

		let object = dicom_object::open_file(&sample_path).expect("打开本地 DICOM 失败");
		let rows = object
			.element_by_name("Rows")
			.expect("缺少 Rows")
			.to_int::<u16>()
			.expect("Rows 解析失败");
		let cols = object
			.element_by_name("Columns")
			.expect("缺少 Columns")
			.to_int::<u16>()
			.expect("Columns 解析失败");
		let frames = object
			.element_by_name("NumberOfFrames")
			.ok()
			.and_then(|element| element.to_str().ok())
			.map(|value| value.to_string())
			.unwrap_or_else(|| "1".to_string());
		let spp = object
			.element_by_name("SamplesPerPixel")
			.ok()
			.and_then(|element| element.to_str().ok())
			.map(|value| value.to_string())
			.unwrap_or_else(|| "1".to_string());
		let photo = object
			.element_by_name("PhotometricInterpretation")
			.ok()
			.and_then(|element| element.to_str().ok())
			.map(|value| value.to_string())
			.unwrap_or_else(|| "-".to_string());
		let pixel_data = object.decode_pixel_data().expect("像素解码失败");
		let array = pixel_data.to_ndarray::<f32>().expect("ndarray 转换失败");

		println!(
			"local ct dicom debug => rows={rows}, cols={cols}, frames={frames}, samples_per_pixel={spp}, photometric={photo}, decoded_dims={:?}, decoded_pixel_count={}",
			array.dim(),
			array.len()
		);
	}

	#[test]
	fn should_load_local_ct_dicom_series_when_available() {
		let sample_directory = Path::new(env!("CARGO_MANIFEST_DIR"))
			.parent()
			.unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")))
			.join("data")
			.join("CT_DICOM");
		if !sample_directory.exists() {
			return;
		}

		let volume = load_dicom_series(&sample_directory).expect("加载本地 CT_DICOM 目录失败");
		assert_eq!(volume.dims[0], 512);
		assert_eq!(volume.dims[1], 512);
		assert_eq!(volume.dims[2], 19);
		assert_eq!(volume.voxels.len(), 512 * 512 * 19);
	}
}
