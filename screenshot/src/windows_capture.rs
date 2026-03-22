/// Windows 系统级截图实现。
#[cfg(target_os = "windows")]
mod imp {
	use image::DynamicImage;
	use windows::Win32::{
		Foundation::HWND,
		Graphics::Gdi::{
			BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap,
			CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits,
			GetMonitorInfoW, HDC, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
			ReleaseDC, SRCCOPY, SelectObject,
		},
	};

	use crate::error::ScreenshotError;

	/// 当前显示器抓图结果。
	#[derive(Debug)]
	pub struct DisplayCapture {
		/// 显示器左上角屏幕坐标。
		pub origin_x: i32,

		/// 显示器左上角屏幕坐标。
		pub origin_y: i32,

		/// 抓图宽度。
		pub width: u32,

		/// 抓图高度。
		pub height: u32,

		/// 桌面图像。
		pub image: DynamicImage,
	}

	/// 根据主窗口句柄抓取其所在显示器的完整桌面。
	pub fn capture_display_for_window(hwnd: isize) -> Result<DisplayCapture, ScreenshotError> {
		unsafe {
			let window = HWND(hwnd as *mut core::ffi::c_void);
			let monitor = MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST);
			if monitor.is_invalid() {
				return Err(last_windows_error("无法定位当前窗口所在显示器"));
			}

			let mut monitor_info = MONITORINFO {
				cbSize: core::mem::size_of::<MONITORINFO>() as u32,
				..Default::default()
			};
			GetMonitorInfoW(monitor, &mut monitor_info)
				.ok()
				.map_err(|error| ScreenshotError::WindowsApi(error.to_string()))?;

			let width = monitor_info.rcMonitor.right - monitor_info.rcMonitor.left;
			let height = monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top;
			if width <= 0 || height <= 0 {
				return Err(ScreenshotError::WindowsApi("显示器尺寸无效".to_string()));
			}

			let desktop_dc = GetDC(HWND::default());
			if desktop_dc.is_invalid() {
				return Err(last_windows_error("无法获取桌面 DC"));
			}

			let image = capture_monitor_image(
				desktop_dc,
				monitor_info.rcMonitor.left,
				monitor_info.rcMonitor.top,
				width,
				height,
			);
			let _ = ReleaseDC(HWND::default(), desktop_dc);

			image.map(|image| DisplayCapture {
				origin_x: monitor_info.rcMonitor.left,
				origin_y: monitor_info.rcMonitor.top,
				width: width as u32,
				height: height as u32,
				image,
			})
		}
	}

	/// 使用 GDI 把目标显示器内容读回成图像。
	fn capture_monitor_image(
		desktop_dc: HDC,
		left: i32,
		top: i32,
		width: i32,
		height: i32,
	) -> Result<DynamicImage, ScreenshotError> {
		unsafe {
			let memory_dc = CreateCompatibleDC(desktop_dc);
			if memory_dc.is_invalid() {
				return Err(last_windows_error("无法创建内存 DC"));
			}

			let bitmap = CreateCompatibleBitmap(desktop_dc, width, height);
			if bitmap.is_invalid() {
				let _ = DeleteDC(memory_dc);
				return Err(last_windows_error("无法创建兼容位图"));
			}

			let old_object = SelectObject(memory_dc, bitmap);
			if old_object.is_invalid() {
				let _ = DeleteObject(bitmap);
				let _ = DeleteDC(memory_dc);
				return Err(last_windows_error("无法把位图绑定到内存 DC"));
			}

			let copy_result = BitBlt(
				memory_dc, 0, 0, width, height, desktop_dc, left, top, SRCCOPY,
			);
			if let Err(error) = copy_result {
				let _ = SelectObject(memory_dc, old_object);
				let _ = DeleteObject(bitmap);
				let _ = DeleteDC(memory_dc);
				return Err(ScreenshotError::WindowsApi(error.to_string()));
			}

			let mut bitmap_info = BITMAPINFO {
				bmiHeader: BITMAPINFOHEADER {
					biSize: core::mem::size_of::<BITMAPINFOHEADER>() as u32,
					biWidth: width,
					biHeight: -height,
					biPlanes: 1,
					biBitCount: 32,
					biCompression: BI_RGB.0,
					..Default::default()
				},
				..Default::default()
			};
			let mut bgra_pixels = vec![0_u8; width as usize * height as usize * 4];
			let lines = GetDIBits(
				memory_dc,
				bitmap,
				0,
				height as u32,
				Some(bgra_pixels.as_mut_ptr().cast()),
				&mut bitmap_info,
				DIB_RGB_COLORS,
			);
			if lines == 0 {
				let _ = SelectObject(memory_dc, old_object);
				let _ = DeleteObject(bitmap);
				let _ = DeleteDC(memory_dc);
				return Err(last_windows_error("无法读取显示器像素"));
			}

			let _ = SelectObject(memory_dc, old_object);
			let _ = DeleteObject(bitmap);
			let _ = DeleteDC(memory_dc);

			let rgba_pixels = bgra_to_rgba(bgra_pixels);
			let image = image::RgbaImage::from_raw(width as u32, height as u32, rgba_pixels)
				.ok_or(ScreenshotError::InvalidImageBuffer {
					width: width as u32,
					height: height as u32,
				})?;
			Ok(DynamicImage::ImageRgba8(image))
		}
	}

	/// GDI 返回 BGRA 像素，这里转成 RGBA。
	fn bgra_to_rgba(bgra_pixels: Vec<u8>) -> Vec<u8> {
		let mut rgba_pixels = Vec::with_capacity(bgra_pixels.len());
		for chunk in bgra_pixels.chunks_exact(4) {
			rgba_pixels.push(chunk[2]);
			rgba_pixels.push(chunk[1]);
			rgba_pixels.push(chunk[0]);
			rgba_pixels.push(255);
		}
		rgba_pixels
	}

	/// 统一格式化 Win32 错误。
	fn last_windows_error(context: &str) -> ScreenshotError {
		let error = std::io::Error::last_os_error();
		ScreenshotError::WindowsApi(format!("{context}: {error}"))
	}
}

/// 当前显示器抓图结果。
#[cfg(target_os = "windows")]
pub use imp::DisplayCapture;

/// 根据主窗口句柄抓取其所在显示器的完整桌面。
#[cfg(target_os = "windows")]
pub use imp::capture_display_for_window;

/// 非 Windows 平台的占位抓图结果。
#[cfg(not(target_os = "windows"))]
#[derive(Debug)]
pub struct DisplayCapture {
	pub origin_x: i32,

	pub origin_y: i32,

	pub width: u32,

	pub height: u32,

	pub image: image::DynamicImage,
}

/// 非 Windows 平台暂不支持系统级截图。
#[cfg(not(target_os = "windows"))]
pub fn capture_display_for_window(
	_hwnd: isize,
) -> Result<DisplayCapture, crate::error::ScreenshotError> {
	Err(crate::error::ScreenshotError::WindowsApi(
		"当前平台暂不支持程序所在桌面截图".to_string(),
	))
}
