use bevy::asset::RenderAssetUsages;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use bevy::window::{
	PrimaryWindow, RawHandleWrapper, WindowClosed, WindowLevel, WindowPosition, WindowRef,
	WindowResolution,
};
use bevy::{
	camera::{RenderTarget, visibility::RenderLayers},
	input::ButtonInput,
	math::IVec2,
	prelude::*,
	render::{
		render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
		view::window::screenshot::{Screenshot, ScreenshotCaptured},
	},
};
use raw_window_handle::RawWindowHandle;
use screenshot::{
	crop::crop_image,
	request::{CaptureOutputKind, CaptureRegion},
	save::{build_output_path, save_bevy_image, save_dynamic_image},
	windows_capture::{DisplayCapture, capture_display_for_window},
};

use crate::homepage::{
	common::ContentAreaMarker,
	screenshot::{
		components::{
			PendingCropScreenshotTask, PendingRenderScreenshotTask, PendingWindowScreenshotTask,
			ScreenRegionOverlayCameraMarker, ScreenRegionOverlayRootMarker,
			ScreenRegionOverlaySelectionMarker, ScreenRegionOverlayWindowMarker,
			ScreenshotContentMarker, ScreenshotCropAreaMarker,
			ScreenshotCurrentDisplayButtonMarker, ScreenshotRegionRenderButtonMarker,
			ScreenshotRenderCameraMarker, ScreenshotRenderPreviewMarker,
			ScreenshotRenderSceneMarker, ScreenshotScreenRegionButtonMarker,
			ScreenshotStatusTextMarker, ScreenshotWindowButtonMarker,
		},
		constants::{
			RENDER_PREVIEW_HEIGHT, RENDER_PREVIEW_WIDTH, RENDER_TARGET_HEIGHT, RENDER_TARGET_WIDTH,
			SCREENSHOT_RENDER_LAYER,
		},
		resources::{ScreenRegionOverlayState, ScreenshotPageState, ScreenshotStatusMessage},
	},
};

/// 创建独立 Render 测试区使用的离屏纹理。
fn create_render_target() -> Image {
	let size = Extent3d {
		width: RENDER_TARGET_WIDTH,
		height: RENDER_TARGET_HEIGHT,
		depth_or_array_layers: 1,
	};
	let mut image = Image::new_fill(
		size,
		TextureDimension::D2,
		&[240, 235, 218, 255],
		TextureFormat::Bgra8UnormSrgb,
		RenderAssetUsages::default(),
	);
	image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
		| TextureUsages::COPY_DST
		| TextureUsages::COPY_SRC
		| TextureUsages::RENDER_ATTACHMENT;
	image
}

/// 创建截图测试页操作按钮。
fn spawn_action_button<T: Component>(marker: T, label: &str) -> impl Bundle {
	(
		Button,
		marker,
		Node {
			height: Val::Px(38.0),
			padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			..default()
		},
		BackgroundColor(Color::srgb(0.16, 0.39, 0.76)),
		children![(
			Text::new(label),
			TextFont {
				font_size: 14.0,
				..default()
			},
			TextColor(Color::WHITE),
		)],
	)
}

/// 获取主窗口对应的 Win32 HWND。
fn primary_window_hwnd(
	window_handle_query: &Query<&RawHandleWrapper, With<PrimaryWindow>>,
) -> Result<isize, String> {
	let raw_handle = window_handle_query
		.single()
		.map_err(|error| format!("获取主窗口句柄失败: {error}"))?;

	match raw_handle.get_window_handle() {
		RawWindowHandle::Win32(handle) => Ok(handle.hwnd.get()),
		_ => Err("当前主窗口不是 Win32 HWND，无法执行系统级截图".to_string()),
	}
}

/// 把抓到的桌面图像转换成 Bevy 可显示纹理。
fn bevy_image_from_capture(capture: &DisplayCapture) -> Image {
	let rgba = capture.image.to_rgba8();
	Image::new(
		Extent3d {
			width: capture.width,
			height: capture.height,
			depth_or_array_layers: 1,
		},
		TextureDimension::D2,
		rgba.into_raw(),
		TextureFormat::Rgba8UnormSrgb,
		RenderAssetUsages::default(),
	)
}

/// 根据测试区 B 当前真实布局计算整窗裁剪区域。
fn crop_region_from_ui_layout(
	computed_node: &ComputedNode,
	global_transform: &UiGlobalTransform,
) -> Option<CaptureRegion> {
	let (_, _, translation) = global_transform.to_scale_angle_translation();
	let half_size = computed_node.size * 0.5;
	let left = (translation.x - half_size.x).round().max(0.0) as u32;
	let top = (translation.y - half_size.y).round().max(0.0) as u32;
	let width = computed_node.size.x.round().max(1.0) as u32;
	let height = computed_node.size.y.round().max(1.0) as u32;

	Some(CaptureRegion {
		x: left,
		y: top,
		width,
		height,
	})
}

/// 规范化拖拽区域，输出截图裁剪矩形。
fn normalized_capture_region(start: Vec2, end: Vec2, window: &Window) -> Option<CaptureRegion> {
	let width_limit = window.width().max(1.0);
	let height_limit = window.height().max(1.0);

	let min_x = start.x.min(end.x).clamp(0.0, width_limit);
	let min_y = start.y.min(end.y).clamp(0.0, height_limit);
	let max_x = start.x.max(end.x).clamp(0.0, width_limit);
	let max_y = start.y.max(end.y).clamp(0.0, height_limit);

	let region_width = (max_x - min_x).round() as u32;
	let region_height = (max_y - min_y).round() as u32;
	if region_width == 0 || region_height == 0 {
		return None;
	}

	Some(CaptureRegion {
		x: min_x.round() as u32,
		y: min_y.round() as u32,
		width: region_width,
		height: region_height,
	})
}

/// 根据当前拖拽状态同步覆盖层选框。
fn sync_overlay_selection_node(
	selection_node: &mut Node,
	selection_visibility: &mut Visibility,
	start: Option<Vec2>,
	current: Option<Vec2>,
) {
	let Some(start) = start else {
		*selection_visibility = Visibility::Hidden;
		return;
	};
	let Some(current) = current else {
		*selection_visibility = Visibility::Hidden;
		return;
	};

	let left = start.x.min(current.x);
	let top = start.y.min(current.y);
	let width = (start.x - current.x).abs();
	let height = (start.y - current.y).abs();

	if width < 1.0 || height < 1.0 {
		*selection_visibility = Visibility::Hidden;
		return;
	}

	selection_node.left = Val::Px(left);
	selection_node.top = Val::Px(top);
	selection_node.width = Val::Px(width);
	selection_node.height = Val::Px(height);
	*selection_visibility = Visibility::Visible;
}

/// 关闭桌面框选覆盖层窗口和附属实体。
fn close_overlay(commands: &mut Commands, overlay_state: &ScreenRegionOverlayState) {
	commands.entity(overlay_state.root_entity).despawn();
	commands.entity(overlay_state.camera_entity).despawn();
	commands.entity(overlay_state.window_entity).despawn();
}

/// 进入截图测试页。
pub fn on_enter(
	mut commands: Commands,
	content_area_query: Query<Entity, With<ContentAreaMarker>>,
	mut images: ResMut<Assets<Image>>,
) {
	info!("进入截图测试页面");

	let render_target_handle = images.add(create_render_target());

	commands.insert_resource(ScreenshotPageState {
		render_target: Some(render_target_handle.clone()),
		..Default::default()
	});

	commands.spawn((
		Camera2d,
		Camera {
			clear_color: ClearColorConfig::Custom(Color::srgb(0.95, 0.92, 0.84)),
			..default()
		},
		RenderTarget::Image(render_target_handle.clone().into()),
		RenderLayers::layer(SCREENSHOT_RENDER_LAYER),
		ScreenshotRenderCameraMarker,
	));

	commands.spawn((
		ScreenshotRenderSceneMarker,
		RenderLayers::layer(SCREENSHOT_RENDER_LAYER),
		Sprite {
			color: Color::srgb(0.85, 0.31, 0.20),
			custom_size: Some(Vec2::new(220.0, 120.0)),
			..default()
		},
		Transform::from_xyz(-120.0, 24.0, 0.0),
	));

	commands.spawn((
		ScreenshotRenderSceneMarker,
		RenderLayers::layer(SCREENSHOT_RENDER_LAYER),
		Sprite {
			color: Color::srgb(0.16, 0.56, 0.48),
			custom_size: Some(Vec2::new(180.0, 180.0)),
			..default()
		},
		Transform::from_xyz(120.0, -30.0, 0.0),
	));

	if let Ok(content_area) = content_area_query.single() {
		commands.entity(content_area).with_children(|parent| {
			parent
				.spawn((
					ScreenshotContentMarker,
					Node {
						width: Val::Percent(100.0),
						height: Val::Percent(100.0),
						flex_direction: FlexDirection::Column,
						padding: UiRect::all(Val::Px(16.0)),
						row_gap: Val::Px(16.0),
						..default()
					},
					BackgroundColor(Color::srgb(0.96, 0.96, 0.96)),
				))
				.with_children(|column| {
					column.spawn((
						Node {
							width: Val::Percent(100.0),
							column_gap: Val::Px(8.0),
							row_gap: Val::Px(8.0),
							flex_wrap: FlexWrap::Wrap,
							align_items: AlignItems::Center,
							..default()
						},
						children![
							spawn_action_button(ScreenshotWindowButtonMarker, "整个程序截图"),
							spawn_action_button(
								ScreenshotRegionRenderButtonMarker,
								"固定区域截图（独立 Render）",
							),
							spawn_action_button(
								ScreenshotCurrentDisplayButtonMarker,
								"程序所在桌面截图",
							),
							spawn_action_button(
								ScreenshotScreenRegionButtonMarker,
								"鼠标选定区域截图",
							),
						],
					));

					column.spawn((
						Text::new("等待截图操作"),
						TextFont {
							font_size: 16.0,
							..default()
						},
						TextColor(Color::BLACK),
						ScreenshotStatusTextMarker,
					));

					column
						.spawn((Node {
							width: Val::Percent(100.0),
							height: Val::Px(520.0),
							column_gap: Val::Px(16.0),
							..default()
						},))
						.with_children(|row| {
							row.spawn((
								Node {
									width: Val::Percent(50.0),
									height: Val::Percent(100.0),
									flex_direction: FlexDirection::Column,
									row_gap: Val::Px(10.0),
									padding: UiRect::all(Val::Px(12.0)),
									border: UiRect::all(Val::Px(2.0)),
									..default()
								},
								BorderColor::all(Color::srgb(0.70, 0.46, 0.18)),
								BackgroundColor(Color::srgb(0.99, 0.96, 0.89)),
							))
							.with_children(|panel| {
								panel.spawn((
									Text::new("测试区 A：独立 Render 区"),
									TextFont {
										font_size: 20.0,
										..default()
									},
									TextColor(Color::BLACK),
								));
								panel.spawn((
									Text::new(
										"该区域使用离屏 RenderTarget 渲染，点击第二个按钮会同时导出独立 Render 和整窗裁剪结果。",
									),
									TextFont {
										font_size: 14.0,
										..default()
									},
									TextColor(Color::srgb(0.18, 0.18, 0.18)),
								));
								panel.spawn((
									Node {
										width: Val::Px(RENDER_PREVIEW_WIDTH),
										height: Val::Px(RENDER_PREVIEW_HEIGHT),
										align_self: AlignSelf::Center,
										border: UiRect::all(Val::Px(2.0)),
										..default()
									},
									BorderColor::all(Color::srgb(0.70, 0.46, 0.18)),
									ImageNode::new(render_target_handle.clone()),
									ScreenshotRenderPreviewMarker,
								));
							});

							row.spawn((
								Node {
									width: Val::Percent(50.0),
									height: Val::Percent(100.0),
									flex_direction: FlexDirection::Column,
									row_gap: Val::Px(10.0),
									padding: UiRect::all(Val::Px(16.0)),
									border: UiRect::all(Val::Px(2.0)),
									..default()
								},
								BorderColor::all(Color::srgb(0.15, 0.50, 0.60)),
								BackgroundColor(Color::srgb(0.88, 0.96, 0.98)),
								ScreenshotCropAreaMarker,
							))
							.with_children(|panel| {
								panel.spawn((
									Text::new("测试区 B：整窗裁剪区"),
									TextFont {
										font_size: 20.0,
										..default()
									},
									TextColor(Color::BLACK),
								));
								panel.spawn((
									Text::new(
										"该区域不提供独立 RenderTarget，固定区域截图时走整窗截图后裁剪。",
									),
									TextFont {
										font_size: 14.0,
										..default()
									},
									TextColor(Color::srgb(0.16, 0.16, 0.16)),
								));
								panel.spawn((
									Node {
										width: Val::Percent(100.0),
										height: Val::Px(64.0),
										column_gap: Val::Px(12.0),
										align_items: AlignItems::Center,
										padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
										..default()
									},
									BackgroundColor(Color::srgb(0.16, 0.39, 0.76)),
									children![
										(
											Text::new("裁剪路径专用"),
											TextFont {
												font_size: 18.0,
												..default()
											},
											TextColor(Color::WHITE),
										),
										(
											Text::new("普通 UI 区域"),
											TextFont {
												font_size: 14.0,
												..default()
											},
											TextColor(Color::srgb(0.92, 0.92, 0.92)),
										),
									],
								));
								panel.spawn((
									Node {
										width: Val::Percent(100.0),
										height: Val::Px(180.0),
										flex_direction: FlexDirection::Column,
										row_gap: Val::Px(8.0),
										padding: UiRect::all(Val::Px(12.0)),
										border: UiRect::all(Val::Px(2.0)),
										..default()
									},
									BorderColor::all(Color::srgb(0.15, 0.50, 0.60)),
									BackgroundColor(Color::srgb(0.95, 0.98, 0.99)),
									children![
										(
											Text::new("这里故意放一些普通 UI 元素。"),
											TextFont {
												font_size: 16.0,
												..default()
											},
											TextColor(Color::BLACK),
										),
										(
											Text::new("截图时会按照固定矩形从整窗口结果中裁剪。"),
											TextFont {
												font_size: 14.0,
												..default()
											},
											TextColor(Color::srgb(0.24, 0.24, 0.24)),
										),
										(
											Node {
												width: Val::Px(180.0),
												height: Val::Px(36.0),
												align_items: AlignItems::Center,
												justify_content: JustifyContent::Center,
												..default()
											},
											BackgroundColor(Color::srgb(0.78, 0.28, 0.22)),
											children![(
												Text::new("示例按钮"),
												TextFont {
													font_size: 14.0,
													..default()
												},
												TextColor(Color::WHITE),
											)],
										),
									],
								));
							});
						});
				});
		});
	}
}

/// 离开截图测试页。
pub fn on_exit(
	mut commands: Commands,
	content_query: Query<Entity, With<ScreenshotContentMarker>>,
	camera_query: Query<Entity, With<ScreenshotRenderCameraMarker>>,
	scene_query: Query<Entity, With<ScreenshotRenderSceneMarker>>,
	overlay_state: Option<Res<ScreenRegionOverlayState>>,
) {
	info!("离开截图测试页面");

	for entity in &content_query {
		commands.entity(entity).despawn();
	}
	for entity in &camera_query {
		commands.entity(entity).despawn();
	}
	for entity in &scene_query {
		commands.entity(entity).despawn();
	}

	if let Some(overlay_state) = overlay_state {
		close_overlay(&mut commands, &overlay_state);
		commands.remove_resource::<ScreenRegionOverlayState>();
	}

	commands.remove_resource::<ScreenshotPageState>();
}

/// 处理整个程序截图按钮。
pub fn handle_window_screenshot_click(
	mut commands: Commands,
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<ScreenshotWindowButtonMarker>),
	>,
	mut writer: MessageWriter<ScreenshotStatusMessage>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		match build_output_path(CaptureOutputKind::Window) {
			Ok(path) => {
				let pending_path = path.display().to_string();
				commands.spawn((
					Screenshot::primary_window(),
					PendingWindowScreenshotTask { path },
				));
				writer.write(ScreenshotStatusMessage(format!(
					"正在保存整个程序截图: {pending_path}"
				)));
			}
			Err(error) => {
				writer.write(ScreenshotStatusMessage(format!(
					"创建输出路径失败: {error}"
				)));
			}
		}
	}
}

/// 处理固定区域截图按钮。
pub fn handle_window_region_crop_click(
	mut commands: Commands,
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<ScreenshotRegionRenderButtonMarker>,
		),
	>,
	state: Res<ScreenshotPageState>,
	crop_area_query: Query<(&ComputedNode, &UiGlobalTransform), With<ScreenshotCropAreaMarker>>,
	mut writer: MessageWriter<ScreenshotStatusMessage>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let mut triggered = Vec::new();

		if let Some(handle) = state.render_target.clone() {
			match build_output_path(CaptureOutputKind::RegionRender) {
				Ok(path) => {
					commands.spawn((
						Screenshot::image(handle),
						PendingRenderScreenshotTask { path },
					));
					triggered.push("独立 Render".to_string());
				}
				Err(error) => {
					writer.write(ScreenshotStatusMessage(format!(
						"创建输出路径失败: {error}"
					)));
				}
			}
		}

		match build_output_path(CaptureOutputKind::RegionCrop) {
			Ok(path) => {
				let crop_region = match crop_area_query.single() {
					Ok((computed_node, global_transform)) => {
						match crop_region_from_ui_layout(computed_node, global_transform) {
							Some(region) => region,
							None => {
								writer.write(ScreenshotStatusMessage(
									"获取测试区 B 裁剪区域失败".to_string(),
								));
								continue;
							}
						}
					}
					Err(error) => {
						writer.write(ScreenshotStatusMessage(format!(
							"读取测试区 B 布局失败: {error}"
						)));
						continue;
					}
				};
				commands.spawn((
					Screenshot::primary_window(),
					PendingCropScreenshotTask {
						path,
						region: crop_region,
					},
				));
				triggered.push("整窗裁剪".to_string());
			}
			Err(error) => {
				writer.write(ScreenshotStatusMessage(format!(
					"创建输出路径失败: {error}"
				)));
			}
		}

		if !triggered.is_empty() {
			writer.write(ScreenshotStatusMessage(format!(
				"已触发固定区域截图路径: {}",
				triggered.join(" + ")
			)));
		}
	}
}

/// 统一处理 Bevy 内部截图回调。
pub fn handle_screenshot_captured(
	capture: On<ScreenshotCaptured>,
	window_tasks: Query<&PendingWindowScreenshotTask>,
	render_tasks: Query<&PendingRenderScreenshotTask>,
	crop_tasks: Query<&PendingCropScreenshotTask>,
	mut writer: MessageWriter<ScreenshotStatusMessage>,
) {
	let entity = capture.entity;

	if let Ok(task) = window_tasks.get(entity) {
		match save_bevy_image(&capture.image, &task.path) {
			Ok(result) => {
				writer.write(ScreenshotStatusMessage(format!(
					"整个程序截图成功: {} ({}x{})",
					result.path.display(),
					result.width,
					result.height
				)));
			}
			Err(error) => {
				writer.write(ScreenshotStatusMessage(format!(
					"整个程序截图失败: {error}"
				)));
			}
		}
		return;
	}

	if let Ok(task) = render_tasks.get(entity) {
		match save_bevy_image(&capture.image, &task.path) {
			Ok(result) => {
				writer.write(ScreenshotStatusMessage(format!(
					"固定区域截图（独立 Render）成功: {} ({}x{})",
					result.path.display(),
					result.width,
					result.height
				)));
			}
			Err(error) => {
				writer.write(ScreenshotStatusMessage(format!(
					"固定区域截图（独立 Render）失败: {error}"
				)));
			}
		}
		return;
	}

	if let Ok(task) = crop_tasks.get(entity) {
		match crop_image(&capture.image, task.region)
			.and_then(|(cropped, _, _)| save_dynamic_image(&cropped, &task.path))
		{
			Ok(result) => {
				writer.write(ScreenshotStatusMessage(format!(
					"固定区域截图（整窗裁剪）成功: {} ({}x{})",
					result.path.display(),
					result.width,
					result.height
				)));
			}
			Err(error) => {
				writer.write(ScreenshotStatusMessage(format!(
					"固定区域截图（整窗裁剪）失败: {error}"
				)));
			}
		}
	}
}

/// 处理程序所在桌面截图按钮。
pub fn handle_current_display_click(
	window_handle_query: Query<&RawHandleWrapper, With<PrimaryWindow>>,
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<ScreenshotCurrentDisplayButtonMarker>,
		),
	>,
	mut writer: MessageWriter<ScreenshotStatusMessage>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		let hwnd = match primary_window_hwnd(&window_handle_query) {
			Ok(hwnd) => hwnd,
			Err(error) => {
				writer.write(ScreenshotStatusMessage(error));
				continue;
			}
		};

		let output_path = match build_output_path(CaptureOutputKind::CurrentDisplay) {
			Ok(path) => path,
			Err(error) => {
				writer.write(ScreenshotStatusMessage(format!(
					"创建输出路径失败: {error}"
				)));
				continue;
			}
		};

		match capture_display_for_window(hwnd)
			.and_then(|capture| save_dynamic_image(&capture.image, &output_path))
		{
			Ok(result) => {
				writer.write(ScreenshotStatusMessage(format!(
					"程序所在桌面截图成功: {} ({}x{})",
					result.path.display(),
					result.width,
					result.height
				)));
			}
			Err(error) => {
				writer.write(ScreenshotStatusMessage(format!(
					"程序所在桌面截图失败: {error}"
				)));
			}
		}
	}
}

/// 处理桌面框选截图按钮。
pub fn handle_screen_region_click(
	mut commands: Commands,
	window_handle_query: Query<&RawHandleWrapper, With<PrimaryWindow>>,
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<ScreenshotScreenRegionButtonMarker>,
		),
	>,
	overlay_state: Option<Res<ScreenRegionOverlayState>>,
	mut images: ResMut<Assets<Image>>,
	mut writer: MessageWriter<ScreenshotStatusMessage>,
) {
	for interaction in &interaction_query {
		if !matches!(interaction, Interaction::Pressed) {
			continue;
		}

		if overlay_state.is_some() {
			writer.write(ScreenshotStatusMessage(
				"桌面框选覆盖层已打开，请先完成或取消当前框选".to_string(),
			));
			continue;
		}

		let hwnd = match primary_window_hwnd(&window_handle_query) {
			Ok(hwnd) => hwnd,
			Err(error) => {
				writer.write(ScreenshotStatusMessage(error));
				continue;
			}
		};

		let output_path = match build_output_path(CaptureOutputKind::ScreenRegion) {
			Ok(path) => path,
			Err(error) => {
				writer.write(ScreenshotStatusMessage(format!(
					"创建输出路径失败: {error}"
				)));
				continue;
			}
		};

		let capture = match capture_display_for_window(hwnd) {
			Ok(capture) => capture,
			Err(error) => {
				writer.write(ScreenshotStatusMessage(format!(
					"启动桌面框选失败: {error}"
				)));
				continue;
			}
		};

		let preview_handle = images.add(bevy_image_from_capture(&capture));

		let window_entity = commands
			.spawn((
				Window {
					title: "桌面框选截图".to_string(),
					position: WindowPosition::At(IVec2::new(capture.origin_x, capture.origin_y)),
					resolution: WindowResolution::new(capture.width, capture.height)
						.with_scale_factor_override(1.0),
					decorations: false,
					resizable: false,
					window_level: WindowLevel::AlwaysOnTop,
					skip_taskbar: true,
					..default()
				},
				ScreenRegionOverlayWindowMarker,
			))
			.id();

		let camera_entity = commands
			.spawn((
				Camera2d,
				RenderTarget::Window(WindowRef::Entity(window_entity)),
				ScreenRegionOverlayCameraMarker,
			))
			.id();

		let mut selection_entity = None;
		let root_entity = commands
			.spawn((
				Node {
					width: Val::Percent(100.0),
					height: Val::Percent(100.0),
					..default()
				},
				BackgroundColor(Color::BLACK),
				UiTargetCamera(camera_entity),
				ScreenRegionOverlayRootMarker,
			))
			.with_children(|parent| {
				parent.spawn((
					Node {
						position_type: PositionType::Absolute,
						left: Val::Px(0.0),
						top: Val::Px(0.0),
						width: Val::Percent(100.0),
						height: Val::Percent(100.0),
						..default()
					},
					ImageNode::new(preview_handle),
				));

				parent.spawn((
					Node {
						position_type: PositionType::Absolute,
						left: Val::Px(16.0),
						top: Val::Px(16.0),
						padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
						..default()
					},
					BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
					children![(
						Text::new("左键拖拽选区，松开后保存；右键取消"),
						TextFont {
							font_size: 18.0,
							..default()
						},
						TextColor(Color::WHITE),
					)],
				));

				selection_entity = Some(
					parent
						.spawn((
							Node {
								position_type: PositionType::Absolute,
								left: Val::Px(0.0),
								top: Val::Px(0.0),
								width: Val::Px(0.0),
								height: Val::Px(0.0),
								border: UiRect::all(Val::Px(2.0)),
								..default()
							},
							BorderColor::all(Color::srgb(0.96, 0.76, 0.18)),
							BackgroundColor(Color::srgba(0.98, 0.82, 0.20, 0.22)),
							Visibility::Hidden,
							ScreenRegionOverlaySelectionMarker,
						))
						.id(),
				);
			})
			.id();

		let Some(selection_entity) = selection_entity else {
			writer.write(ScreenshotStatusMessage(
				"创建桌面框选覆盖层失败: 选框节点创建异常".to_string(),
			));
			continue;
		};

		commands.insert_resource(ScreenRegionOverlayState {
			window_entity,
			camera_entity,
			root_entity,
			selection_entity,
			capture_image: capture.image,
			output_path,
			drag_start: None,
			drag_current: None,
		});

		writer.write(ScreenshotStatusMessage(
			"已打开桌面框选覆盖层，请在覆盖层中拖拽选区".to_string(),
		));
	}
}

/// 处理桌面框选覆盖层交互。
pub fn update_screen_region_overlay(
	mut commands: Commands,
	overlay_state: Option<ResMut<ScreenRegionOverlayState>>,
	window_query: Query<&Window, With<ScreenRegionOverlayWindowMarker>>,
	mouse_buttons: Res<ButtonInput<MouseButton>>,
	mut selection_query: Query<
		(&mut Node, &mut Visibility),
		With<ScreenRegionOverlaySelectionMarker>,
	>,
	mut writer: MessageWriter<ScreenshotStatusMessage>,
) {
	let Some(mut overlay_state) = overlay_state else {
		return;
	};

	let Ok(window) = window_query.get(overlay_state.window_entity) else {
		return;
	};
	let Ok((mut selection_node, mut selection_visibility)) =
		selection_query.get_mut(overlay_state.selection_entity)
	else {
		return;
	};

	let cursor = window.cursor_position();

	if mouse_buttons.just_pressed(MouseButton::Right) {
		close_overlay(&mut commands, &overlay_state);
		commands.remove_resource::<ScreenRegionOverlayState>();
		writer.write(ScreenshotStatusMessage(
			"已取消鼠标选定区域截图".to_string(),
		));
		return;
	}

	if mouse_buttons.just_pressed(MouseButton::Left) {
		overlay_state.drag_start = cursor;
		overlay_state.drag_current = cursor;
	}

	if mouse_buttons.pressed(MouseButton::Left)
		&& overlay_state.drag_start.is_some()
		&& let Some(cursor) = cursor
	{
		overlay_state.drag_current = Some(cursor);
	}

	sync_overlay_selection_node(
		&mut selection_node,
		&mut selection_visibility,
		overlay_state.drag_start,
		overlay_state.drag_current,
	);

	if mouse_buttons.just_released(MouseButton::Left) && overlay_state.drag_start.is_some() {
		let start = overlay_state.drag_start;
		let end = cursor.or(overlay_state.drag_current);

		if let (Some(start), Some(end)) = (start, end) {
			if let Some(region) = normalized_capture_region(start, end, window) {
				let cropped = overlay_state.capture_image.crop_imm(
					region.x,
					region.y,
					region.width,
					region.height,
				);
				match save_dynamic_image(&cropped, &overlay_state.output_path) {
					Ok(result) => {
						writer.write(ScreenshotStatusMessage(format!(
							"鼠标选定区域截图成功: {} ({}x{})",
							result.path.display(),
							result.width,
							result.height
						)));
					}
					Err(error) => {
						writer.write(ScreenshotStatusMessage(format!(
							"鼠标选定区域截图失败: {error}"
						)));
					}
				}
			} else {
				writer.write(ScreenshotStatusMessage(
					"鼠标选定区域截图已取消：选区尺寸无效".to_string(),
				));
			}
		} else {
			writer.write(ScreenshotStatusMessage(
				"鼠标选定区域截图已取消：未获取到有效光标位置".to_string(),
			));
		}

		close_overlay(&mut commands, &overlay_state);
		commands.remove_resource::<ScreenRegionOverlayState>();
	}
}

/// 处理覆盖层窗口被关闭的情况。
pub fn handle_overlay_window_closed(
	mut commands: Commands,
	mut closed_windows: MessageReader<WindowClosed>,
	overlay_state: Option<Res<ScreenRegionOverlayState>>,
	mut writer: MessageWriter<ScreenshotStatusMessage>,
) {
	let Some(overlay_state) = overlay_state else {
		return;
	};

	let mut should_close = false;
	for event in closed_windows.read() {
		if event.window == overlay_state.window_entity {
			should_close = true;
			break;
		}
	}

	if !should_close {
		return;
	}

	commands.entity(overlay_state.root_entity).despawn();
	commands.entity(overlay_state.camera_entity).despawn();
	commands.remove_resource::<ScreenRegionOverlayState>();
	writer.write(ScreenshotStatusMessage(
		"已关闭桌面框选覆盖层，本次框选截图已取消".to_string(),
	));
}

/// 同步状态文本消息到页面。
pub fn sync_status_messages(
	mut messages: MessageReader<ScreenshotStatusMessage>,
	mut state: ResMut<ScreenshotPageState>,
	mut text_query: Query<&mut Text, With<ScreenshotStatusTextMarker>>,
) {
	let Some(last_message) = messages.read().last().cloned() else {
		return;
	};

	state.status_text = last_message.0.clone();
	for mut text in &mut text_query {
		text.0 = state.status_text.clone();
	}
}
