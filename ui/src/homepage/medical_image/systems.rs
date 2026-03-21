use crate::homepage::common::ContentAreaMarker;
use crate::homepage::medical_image::components::{
	AxialSliceImageMarker, CoronalSliceImageMarker, LoadCtSampleButtonMarker,
	LoadMrSampleButtonMarker, MedicalImageButtonBundle, MedicalImageCamera3dMarker,
	MedicalImageContentMarker, MedicalImageLightMarker, MedicalImagePanelBundle,
	MedicalImageSourceTextMarker, MedicalImageStatusTextMarker, MedicalImageSurfaceMeshMarker,
	MedicalImageViewportMarker, MedicalImageVolumeBoxMarker, RebuildSurfaceButtonMarker,
	SagittalSliceImageMarker, SliceImageBundle, SliceModeButtonMarker, SurfaceModeButtonMarker,
	SurfaceThresholdDecreaseButtonMarker, SurfaceThresholdIncreaseButtonMarker,
	VolumeModeButtonMarker, VolumeStepDecreaseButtonMarker, VolumeStepIncreaseButtonMarker,
	WindowCenterDecreaseButtonMarker, WindowCenterIncreaseButtonMarker,
	WindowWidthDecreaseButtonMarker, WindowWidthIncreaseButtonMarker,
};
use crate::homepage::medical_image::resources::{
	MedicalImageLoadState, MedicalImageSceneResources, MedicalImageState, MedicalImageTextures,
	RenderMode,
};
use crate::homepage::medical_image::volume_render::{
	VolumeRenderMaterial, VolumeTextureBuildInfo, build_render_params, build_volume_texture,
};
use bevy::asset::RenderAssetUsages;
use bevy::camera::{ClearColorConfig, Viewport};
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat};
use bevy::ui::UiGlobalTransform;
use bevy::window::PrimaryWindow;
use medical_image::{
	SliceAxis, SurfaceExtractOptions, SurfaceMeshData, extract_isosurface, extract_slice,
	load_nifti_file, normalize_slice_to_u8,
};
use std::path::{Path, PathBuf};

const SLICE_IMAGE_SIZE: f32 = 240.0;
const SLICE_PANEL_SIZE: f32 = 280.0;
const VIEWPORT_HEIGHT: f32 = 360.0;
const SLICE_PANEL_MIN_WIDTH: f32 = 260.0;
const SURFACE_THRESHOLD_STEP: f32 = 25.0;
const VOLUME_STEP_FACTOR: f32 = 0.85;

/// 进入医学影像页面
pub fn on_enter(
	mut commands: Commands,
	content_area_query: Query<Entity, With<ContentAreaMarker>>,
	mut images: ResMut<Assets<Image>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	info!("进入医学影像页面");

	let textures = create_slice_textures(&mut images);
	commands.insert_resource(MedicalImageTextures {
		axial: textures[0].clone(),
		coronal: textures[1].clone(),
		sagittal: textures[2].clone(),
	});

	let surface_material = materials.add(StandardMaterial {
		base_color: Color::srgb(0.90, 0.92, 0.96),
		perceptual_roughness: 0.55,
		metallic: 0.05,
		double_sided: true,
		..default()
	});
	commands.insert_resource(MedicalImageSceneResources {
		surface_material,
		cached_surface_mesh: None,
		cached_surface_threshold: None,
		cached_surface_revision: None,
		cached_volume_texture: None,
		cached_volume_material: None,
		cached_volume_revision: None,
	});

	commands.spawn((
		MedicalImageCamera3dMarker,
		Camera3d::default(),
		Camera {
			order: 1,
			is_active: false,
			clear_color: ClearColorConfig::None,
			..default()
		},
		Transform::from_xyz(0.0, 0.0, 400.0).looking_at(Vec3::ZERO, Vec3::Y),
	));
	commands.spawn((
		MedicalImageLightMarker,
		PointLight {
			intensity: 8_000_000.0,
			shadows_enabled: true,
			range: 20_000.0,
			..default()
		},
		Transform::from_xyz(300.0, 300.0, 300.0),
	));

	let mut state = MedicalImageState::default();
	let default_ct = sample_nifti_path("SubjectUCI29_CT_acpc_f.nii");
	if let Err(error) = load_sample_into_state(&default_ct, &mut state) {
		state.status_text = format!("默认 CT 加载失败: {error}");
	}
	commands.insert_resource(state);

	if let Ok(content_area) = content_area_query.single() {
		commands.entity(content_area).with_children(|parent| {
			parent
				.spawn((
					MedicalImageContentMarker,
					Node {
						width: Val::Percent(100.0),
						height: Val::Percent(100.0),
						flex_direction: FlexDirection::Column,
						align_items: AlignItems::Stretch,
						padding: UiRect::all(Val::Px(12.0)),
						row_gap: Val::Px(12.0),
						overflow: Overflow::scroll(),
						..default()
					},
					BackgroundColor(Color::srgb(0.95, 0.96, 0.98)),
				))
				.with_children(|root| {
					root.spawn((
						Text::new("文件: -"),
						TextFont {
							font_size: 16.0,
							..default()
						},
						TextColor(Color::BLACK),
						MedicalImageSourceTextMarker,
					));

					root.spawn((
						Text::new("尚未加载医学影像数据"),
						TextFont {
							font_size: 14.0,
							..default()
						},
						TextColor(Color::srgb(0.25, 0.25, 0.25)),
						MedicalImageStatusTextMarker,
					));

					root.spawn((Node {
						width: Val::Percent(100.0),
						flex_shrink: 0.0,
						flex_wrap: FlexWrap::Wrap,
						column_gap: Val::Px(8.0),
						row_gap: Val::Px(8.0),
						..default()
					},))
						.with_children(|buttons| {
							spawn_button(buttons, LoadCtSampleButtonMarker, "加载 CT 样例");
							spawn_button(buttons, LoadMrSampleButtonMarker, "加载 MR 样例");
							spawn_button(buttons, SliceModeButtonMarker, "切片模式");
							spawn_button(buttons, SurfaceModeButtonMarker, "表面模式");
							spawn_button(buttons, VolumeModeButtonMarker, "体渲染模式");
							spawn_button(buttons, RebuildSurfaceButtonMarker, "重建表面");
							spawn_button(buttons, SurfaceThresholdDecreaseButtonMarker, "阈值 -");
							spawn_button(buttons, SurfaceThresholdIncreaseButtonMarker, "阈值 +");
							spawn_button(buttons, VolumeStepDecreaseButtonMarker, "步长 -");
							spawn_button(buttons, VolumeStepIncreaseButtonMarker, "步长 +");
							spawn_button(buttons, WindowCenterDecreaseButtonMarker, "窗位 -");
							spawn_button(buttons, WindowCenterIncreaseButtonMarker, "窗位 +");
							spawn_button(buttons, WindowWidthDecreaseButtonMarker, "窗宽 -");
							spawn_button(buttons, WindowWidthIncreaseButtonMarker, "窗宽 +");
						});

					root.spawn((Node {
						width: Val::Percent(100.0),
						flex_shrink: 0.0,
						flex_direction: FlexDirection::Row,
						flex_wrap: FlexWrap::Wrap,
						column_gap: Val::Px(12.0),
						row_gap: Val::Px(12.0),
						align_items: AlignItems::Start,
						..default()
					},))
						.with_children(|panels| {
							panels
								.spawn(MedicalImagePanelBundle::new(
									SLICE_PANEL_MIN_WIDTH,
									SLICE_PANEL_SIZE,
								))
								.with_children(|panel| {
									panel.spawn((
										Text::new("轴状"),
										TextFont {
											font_size: 16.0,
											..default()
										},
										TextColor(Color::BLACK),
									));
									panel.spawn(SliceImageBundle::new(
										AxialSliceImageMarker,
										textures[0].clone(),
										SLICE_IMAGE_SIZE,
									));
								});
							panels
								.spawn(MedicalImagePanelBundle::new(
									SLICE_PANEL_MIN_WIDTH,
									SLICE_PANEL_SIZE,
								))
								.with_children(|panel| {
									panel.spawn((
										Text::new("冠状"),
										TextFont {
											font_size: 16.0,
											..default()
										},
										TextColor(Color::BLACK),
									));
									panel.spawn(SliceImageBundle::new(
										CoronalSliceImageMarker,
										textures[1].clone(),
										SLICE_IMAGE_SIZE,
									));
								});
							panels
								.spawn(MedicalImagePanelBundle::new(
									SLICE_PANEL_MIN_WIDTH,
									SLICE_PANEL_SIZE,
								))
								.with_children(|panel| {
									panel.spawn((
										Text::new("矢状"),
										TextFont {
											font_size: 16.0,
											..default()
										},
										TextColor(Color::BLACK),
									));
									panel.spawn(SliceImageBundle::new(
										SagittalSliceImageMarker,
										textures[2].clone(),
										SLICE_IMAGE_SIZE,
									));
								});
						});

					root.spawn(MedicalImagePanelBundle::responsive(
						760.0,
						VIEWPORT_HEIGHT + 56.0,
					))
					.with_children(|panel| {
						panel.spawn((
							Text::new("三维预览"),
							TextFont {
								font_size: 16.0,
								..default()
							},
							TextColor(Color::BLACK),
						));
						panel.spawn((
							Text::new("方向键旋转，PageUp/PageDown 缩放"),
							TextFont {
								font_size: 13.0,
								..default()
							},
							TextColor(Color::srgb(0.35, 0.35, 0.35)),
						));
						panel.spawn((
							MedicalImageViewportMarker,
							Node {
								width: Val::Percent(100.0),
								height: Val::Px(VIEWPORT_HEIGHT),
								border: UiRect::all(Val::Px(1.0)),
								..default()
							},
							BorderColor::all(Color::srgb(0.75, 0.78, 0.84)),
							BackgroundColor(Color::srgb(0.10, 0.12, 0.15)),
						));
					});
				});
		});
	}
}

/// 离开医学影像页面
pub fn on_exit(
	mut commands: Commands,
	content_query: Query<Entity, With<MedicalImageContentMarker>>,
	scene_query: Query<
		Entity,
		Or<(
			With<MedicalImageCamera3dMarker>,
			With<MedicalImageLightMarker>,
			With<MedicalImageSurfaceMeshMarker>,
			With<MedicalImageVolumeBoxMarker>,
		)>,
	>,
) {
	info!("离开医学影像页面");
	for entity in &content_query {
		commands.entity(entity).despawn();
	}
	for entity in &scene_query {
		commands.entity(entity).despawn();
	}
	commands.remove_resource::<MedicalImageState>();
	commands.remove_resource::<MedicalImageTextures>();
	commands.remove_resource::<MedicalImageSceneResources>();
}

/// 处理加载 CT 样例
pub fn handle_load_ct_sample(
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<LoadCtSampleButtonMarker>)>,
	mut state: ResMut<MedicalImageState>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			state.load_state = MedicalImageLoadState::Busy;
			let path = sample_nifti_path("SubjectUCI29_CT_acpc_f.nii");
			if let Err(error) = load_sample_into_state(&path, &mut state) {
				state.load_state = MedicalImageLoadState::Error;
				state.status_text = format!("加载 CT 样例失败: {error}");
			}
		}
	}
}

/// 处理加载 MR 样例
pub fn handle_load_mr_sample(
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<LoadMrSampleButtonMarker>)>,
	mut state: ResMut<MedicalImageState>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			state.load_state = MedicalImageLoadState::Busy;
			let path = sample_nifti_path("SubjectUCI29_MR_acpc.nii");
			if let Err(error) = load_sample_into_state(&path, &mut state) {
				state.load_state = MedicalImageLoadState::Error;
				state.status_text = format!("加载 MR 样例失败: {error}");
			}
		}
	}
}

/// 处理表面阈值减小
pub fn handle_surface_threshold_decrease(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<SurfaceThresholdDecreaseButtonMarker>,
		),
	>,
	mut state: ResMut<MedicalImageState>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			state.surface_threshold -= SURFACE_THRESHOLD_STEP;
			state.surface_dirty = true;
			update_status_text(&mut state);
		}
	}
}

/// 处理表面阈值增大
pub fn handle_surface_threshold_increase(
	interaction_query: Query<
		&Interaction,
		(
			Changed<Interaction>,
			With<SurfaceThresholdIncreaseButtonMarker>,
		),
	>,
	mut state: ResMut<MedicalImageState>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			state.surface_threshold += SURFACE_THRESHOLD_STEP;
			state.surface_dirty = true;
			update_status_text(&mut state);
		}
	}
}

/// 处理手动表面重建
pub fn handle_rebuild_surface(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<RebuildSurfaceButtonMarker>),
	>,
	mut state: ResMut<MedicalImageState>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			state.render_mode = RenderMode::Surface3d;
			state.surface_dirty = true;
			update_status_text(&mut state);
		}
	}
}

/// 处理显示模式切换
pub fn handle_render_mode_switch(
	slice_mode_query: Query<&Interaction, (Changed<Interaction>, With<SliceModeButtonMarker>)>,
	surface_mode_query: Query<&Interaction, (Changed<Interaction>, With<SurfaceModeButtonMarker>)>,
	volume_mode_query: Query<&Interaction, (Changed<Interaction>, With<VolumeModeButtonMarker>)>,
	mut state: ResMut<MedicalImageState>,
) {
	let mut changed = false;

	for interaction in &slice_mode_query {
		if matches!(interaction, Interaction::Pressed) {
			state.render_mode = RenderMode::SliceOnly;
			changed = true;
		}
	}

	for interaction in &surface_mode_query {
		if matches!(interaction, Interaction::Pressed) {
			state.render_mode = RenderMode::Surface3d;
			if state.surface_mesh_stats.is_none() {
				state.surface_dirty = true;
			}
			changed = true;
		}
	}

	for interaction in &volume_mode_query {
		if matches!(interaction, Interaction::Pressed) {
			state.render_mode = RenderMode::Volume3d;
			state.volume_dirty = true;
			changed = true;
		}
	}

	if changed {
		update_status_text(&mut state);
	}
}

/// 处理体渲染步长减小
pub fn handle_volume_step_decrease(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<VolumeStepDecreaseButtonMarker>),
	>,
	mut state: ResMut<MedicalImageState>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			state.volume_step_size =
				(state.volume_step_size * VOLUME_STEP_FACTOR).max(1.0 / 1024.0);
			update_status_text(&mut state);
		}
	}
}

/// 处理体渲染步长增大
pub fn handle_volume_step_increase(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<VolumeStepIncreaseButtonMarker>),
	>,
	mut state: ResMut<MedicalImageState>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			state.volume_step_size = (state.volume_step_size / VOLUME_STEP_FACTOR).min(1.0 / 32.0);
			update_status_text(&mut state);
		}
	}
}

/// 处理窗位减小
pub fn handle_window_center_decrease(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<WindowCenterDecreaseButtonMarker>),
	>,
	mut state: ResMut<MedicalImageState>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			state.window_center -= 20.0;
			update_status_text(&mut state);
		}
	}
}

/// 处理窗位增大
pub fn handle_window_center_increase(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<WindowCenterIncreaseButtonMarker>),
	>,
	mut state: ResMut<MedicalImageState>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			state.window_center += 20.0;
			update_status_text(&mut state);
		}
	}
}

/// 处理窗宽减小
pub fn handle_window_width_decrease(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<WindowWidthDecreaseButtonMarker>),
	>,
	mut state: ResMut<MedicalImageState>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			state.window_width = (state.window_width - 20.0).max(1.0);
			update_status_text(&mut state);
		}
	}
}

/// 处理窗宽增大
pub fn handle_window_width_increase(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<WindowWidthIncreaseButtonMarker>),
	>,
	mut state: ResMut<MedicalImageState>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			state.window_width += 20.0;
			update_status_text(&mut state);
		}
	}
}

/// 刷新三视图纹理
pub fn update_slice_images(
	state: Res<MedicalImageState>,
	textures: Res<MedicalImageTextures>,
	mut images: ResMut<Assets<Image>>,
) {
	if !state.is_changed() {
		return;
	}

	let Some(volume) = &state.volume else {
		return;
	};

	let slice_specs = [
		(
			SliceAxis::Axial,
			state.slice_index[0],
			textures.axial.clone(),
		),
		(
			SliceAxis::Coronal,
			state.slice_index[1],
			textures.coronal.clone(),
		),
		(
			SliceAxis::Sagittal,
			state.slice_index[2],
			textures.sagittal.clone(),
		),
	];

	for (axis, index, handle) in slice_specs {
		let Ok(slice) = extract_slice(volume, axis, index) else {
			continue;
		};
		let Ok(grayscale) = normalize_slice_to_u8(&slice, state.window_center, state.window_width)
		else {
			continue;
		};
		let rgba = grayscale_to_rgba(&grayscale);
		if let Some(image) = images.get_mut(&handle) {
			*image = Image::new_fill(
				Extent3d {
					width: slice.width as u32,
					height: slice.height as u32,
					depth_or_array_layers: 1,
				},
				TextureDimension::D2,
				&rgba,
				TextureFormat::Rgba8UnormSrgb,
				RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
			);
		}
	}
}

/// 重建表面网格并同步 3D 场景
pub fn rebuild_surface_mesh(
	mut commands: Commands,
	mut state: ResMut<MedicalImageState>,
	mut scene: ResMut<MedicalImageSceneResources>,
	mut meshes: ResMut<Assets<Mesh>>,
	surface_query: Query<Entity, With<MedicalImageSurfaceMeshMarker>>,
	mut camera_query: Query<&mut Transform, With<MedicalImageCamera3dMarker>>,
	mut light_query: Query<
		&mut Transform,
		(
			With<MedicalImageLightMarker>,
			Without<MedicalImageCamera3dMarker>,
		),
	>,
) {
	if !state.surface_dirty {
		return;
	}
	state.load_state = MedicalImageLoadState::Busy;

	let Some(volume) = &state.volume else {
		state.load_state = MedicalImageLoadState::Error;
		state.status_text = "未加载体数据，无法重建表面".to_string();
		state.surface_dirty = false;
		return;
	};
	let reuse_cached_mesh = scene.cached_surface_revision == Some(state.volume_revision)
		&& scene.cached_surface_threshold == Some(state.surface_threshold)
		&& scene.cached_surface_mesh.is_some();

	let (stats, mesh_handle, center, distance) = if reuse_cached_mesh {
		let Some(mesh_handle) = scene.cached_surface_mesh.clone() else {
			return;
		};
		let stats = state
			.surface_mesh_stats
			.unwrap_or(medical_image::SurfaceMeshStats {
				vertex_count: 0,
				triangle_count: 0,
			});
		(
			stats,
			mesh_handle,
			Vec3::from_array(state.surface_focus_center),
			state.surface_camera_distance.max(120.0),
		)
	} else {
		let mesh_data = match extract_isosurface(
			volume,
			SurfaceExtractOptions {
				threshold: state.surface_threshold,
			},
		) {
			Ok(mesh_data) => mesh_data,
			Err(error) => {
				state.surface_mesh_stats = None;
				state.surface_dirty = false;
				state.load_state = MedicalImageLoadState::Error;
				state.status_text = format!("表面重建失败: {error}");
				return;
			}
		};

		let stats = mesh_data.stats();
		let center = Vec3::from_array(mesh_data.center());
		let distance = (mesh_data.diagonal_length() * 1.4).max(120.0);
		let mesh_handle = meshes.add(build_surface_mesh_asset(&mesh_data));
		scene.cached_surface_mesh = Some(mesh_handle.clone());
		scene.cached_surface_threshold = Some(state.surface_threshold);
		scene.cached_surface_revision = Some(state.volume_revision);
		(stats, mesh_handle, center, distance)
	};

	for entity in &surface_query {
		commands.entity(entity).despawn();
	}

	commands.spawn((
		MedicalImageSurfaceMeshMarker,
		Mesh3d(mesh_handle),
		MeshMaterial3d(scene.surface_material.clone()),
		Transform::default(),
	));

	state.surface_focus_center = center.to_array();
	state.surface_camera_distance = distance;
	if state.surface_camera_pitch.abs() < 0.05 {
		state.surface_camera_pitch = 0.35;
	}
	if state.surface_camera_yaw.abs() < 0.05 {
		state.surface_camera_yaw = 0.75;
	}
	state.surface_mesh_stats = Some(stats);
	state.surface_dirty = false;
	state.load_state = MedicalImageLoadState::Ready;
	state.render_mode = RenderMode::Surface3d;
	apply_orbit_camera_transform(&state, &mut camera_query);
	if let Some(mut light_transform) = light_query.iter_mut().next() {
		light_transform.translation = center + Vec3::new(distance * 0.7, distance, distance * 0.9);
	}
	update_status_text(&mut state);
}

/// 重建体渲染实体和 3D 纹理
#[allow(clippy::too_many_arguments)]
pub fn rebuild_volume_render_entity(
	mut commands: Commands,
	mut state: ResMut<MedicalImageState>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut images: ResMut<Assets<Image>>,
	mut materials: ResMut<Assets<VolumeRenderMaterial>>,
	mut scene: ResMut<MedicalImageSceneResources>,
	volume_query: Query<Entity, With<MedicalImageVolumeBoxMarker>>,
	mut scene_transforms: ParamSet<(
		Query<&mut Transform, With<MedicalImageCamera3dMarker>>,
		Query<
			&mut Transform,
			(
				With<MedicalImageLightMarker>,
				Without<MedicalImageCamera3dMarker>,
			),
		>,
	)>,
) {
	if !state.volume_dirty {
		return;
	}
	state.load_state = MedicalImageLoadState::Busy;

	let Some(volume) = &state.volume else {
		state.load_state = MedicalImageLoadState::Error;
		state.status_text = "未加载体数据，无法创建体渲染".to_string();
		state.volume_dirty = false;
		return;
	};

	for entity in &volume_query {
		commands.entity(entity).despawn();
	}

	let cache_hit = scene.cached_volume_revision == Some(state.volume_revision);
	let (volume_texture, texture_build_info) = if cache_hit {
		(
			scene.cached_volume_texture.clone().unwrap_or_else(|| {
				let build_result = build_volume_texture(volume);
				images.add(build_result.image)
			}),
			VolumeTextureBuildInfo {
				texture_dims: volume.dims,
				downsample_factors: [1, 1, 1],
			},
		)
	} else {
		let build_result = build_volume_texture(volume);
		let texture = images.add(build_result.image);
		scene.cached_volume_texture = Some(texture.clone());
		scene.cached_volume_revision = Some(state.volume_revision);
		scene.cached_volume_material = None;
		(texture, build_result.info)
	};
	let bounds_min = Vec3::from_array(volume.origin);
	let size = Vec3::new(
		volume.spacing[0] * volume.dims[0] as f32,
		volume.spacing[1] * volume.dims[1] as f32,
		volume.spacing[2] * volume.dims[2] as f32,
	);
	let bounds_max = bounds_min + size;
	let center = (bounds_min + bounds_max) * 0.5;
	let material_handle = if cache_hit {
		if let Some(material_handle) = scene.cached_volume_material.clone() {
			if let Some(material) = materials.get_mut(&material_handle) {
				material.render_params = build_render_params(
					volume,
					state.window_center,
					state.window_width,
					state.volume_step_size,
				);
				material.bounds_min = bounds_min.extend(0.0);
				material.bounds_max = bounds_max.extend(0.0);
				material.volume_texture = volume_texture.clone();
			}
			material_handle
		} else {
			let material = materials.add(VolumeRenderMaterial {
				render_params: build_render_params(
					volume,
					state.window_center,
					state.window_width,
					state.volume_step_size,
				),
				bounds_min: bounds_min.extend(0.0),
				bounds_max: bounds_max.extend(0.0),
				volume_texture: volume_texture.clone(),
			});
			scene.cached_volume_material = Some(material.clone());
			material
		}
	} else {
		let material = materials.add(VolumeRenderMaterial {
			render_params: build_render_params(
				volume,
				state.window_center,
				state.window_width,
				state.volume_step_size,
			),
			bounds_min: bounds_min.extend(0.0),
			bounds_max: bounds_max.extend(0.0),
			volume_texture: volume_texture.clone(),
		});
		scene.cached_volume_material = Some(material.clone());
		material
	};
	let mesh_handle = meshes.add(Cuboid::new(
		size.x.max(1.0),
		size.y.max(1.0),
		size.z.max(1.0),
	));

	commands.spawn((
		MedicalImageVolumeBoxMarker,
		Mesh3d(mesh_handle),
		MeshMaterial3d(material_handle),
		Transform::from_translation(center),
	));

	let distance = size.length().max(120.0) * 1.2;
	state.surface_focus_center = center.to_array();
	state.surface_camera_distance = distance;
	state.volume_dirty = false;
	state.load_state = MedicalImageLoadState::Ready;
	state.render_mode = RenderMode::Volume3d;
	apply_orbit_camera_transform(&state, &mut scene_transforms.p0());
	if let Some(mut light_transform) = scene_transforms.p1().iter_mut().next() {
		light_transform.translation = center + Vec3::new(distance * 0.7, distance, distance * 0.9);
	}
	update_status_text(&mut state);
	if texture_build_info.is_downsampled() {
		state.status_text = format!(
			"{} | 体渲染纹理已降采样到 {} x {} x {}",
			state.status_text,
			texture_build_info.texture_dims[0],
			texture_build_info.texture_dims[1],
			texture_build_info.texture_dims[2],
		);
	}
}

/// 同步 3D 相机视口和网格显隐
pub fn sync_3d_viewport(
	state: Res<MedicalImageState>,
	preview_query: Query<(&ComputedNode, &UiGlobalTransform), With<MedicalImageViewportMarker>>,
	window_query: Query<&Window, With<PrimaryWindow>>,
	mut camera_query: Query<&mut Camera, With<MedicalImageCamera3dMarker>>,
	mut mesh_query: Query<
		&mut Visibility,
		(
			With<MedicalImageSurfaceMeshMarker>,
			Without<MedicalImageVolumeBoxMarker>,
		),
	>,
	mut volume_query: Query<
		&mut Visibility,
		(
			With<MedicalImageVolumeBoxMarker>,
			Without<MedicalImageSurfaceMeshMarker>,
		),
	>,
) {
	let Some(mut camera) = camera_query.iter_mut().next() else {
		return;
	};

	let show_3d = matches!(
		state.render_mode,
		RenderMode::Surface3d | RenderMode::Volume3d
	);
	camera.is_active = show_3d;

	for mut visibility in &mut mesh_query {
		*visibility = if state.render_mode == RenderMode::Surface3d {
			Visibility::Visible
		} else {
			Visibility::Hidden
		};
	}
	for mut visibility in &mut volume_query {
		*visibility = if state.render_mode == RenderMode::Volume3d {
			Visibility::Visible
		} else {
			Visibility::Hidden
		};
	}

	if !show_3d {
		return;
	}

	let Some((computed_node, ui_transform)) = preview_query.iter().next() else {
		return;
	};
	let Some(window) = window_query.iter().next() else {
		return;
	};

	let (_, _, center) = ui_transform.to_scale_angle_translation();
	let size = computed_node.size();
	let min = center - size * 0.5;
	let mut viewport = Viewport {
		physical_position: UVec2::new(min.x.max(0.0) as u32, min.y.max(0.0) as u32),
		physical_size: UVec2::new(size.x.max(1.0) as u32, size.y.max(1.0) as u32),
		depth: 0.0..1.0,
	};
	viewport.clamp_to_size(UVec2::new(
		window.physical_width(),
		window.physical_height(),
	));
	camera.viewport = Some(viewport);
}

/// 同步体渲染材质参数
pub fn sync_volume_render_material(
	state: Res<MedicalImageState>,
	volume_entity_query: Query<
		&MeshMaterial3d<VolumeRenderMaterial>,
		With<MedicalImageVolumeBoxMarker>,
	>,
	mut materials: ResMut<Assets<VolumeRenderMaterial>>,
) {
	if !state.is_changed() {
		return;
	}

	let Some(volume) = &state.volume else {
		return;
	};
	let Some(material_handle) = volume_entity_query.iter().next() else {
		return;
	};
	let Some(material) = materials.get_mut(&material_handle.0) else {
		return;
	};
	material.render_params = build_render_params(
		volume,
		state.window_center,
		state.window_width,
		state.volume_step_size,
	);
}

/// 更新三维相机的简单轨道控制
pub fn update_surface_preview_transform(
	keyboard: Res<ButtonInput<KeyCode>>,
	mut state: ResMut<MedicalImageState>,
	mut camera_query: Query<&mut Transform, With<MedicalImageCamera3dMarker>>,
) {
	if matches!(state.render_mode, RenderMode::SliceOnly) || state.volume.is_none() {
		return;
	}

	let mut changed = false;

	if keyboard.pressed(KeyCode::ArrowLeft) {
		state.surface_camera_yaw += 0.03;
		changed = true;
	}
	if keyboard.pressed(KeyCode::ArrowRight) {
		state.surface_camera_yaw -= 0.03;
		changed = true;
	}
	if keyboard.pressed(KeyCode::ArrowUp) {
		state.surface_camera_pitch = (state.surface_camera_pitch + 0.02).clamp(-1.2, 1.2);
		changed = true;
	}
	if keyboard.pressed(KeyCode::ArrowDown) {
		state.surface_camera_pitch = (state.surface_camera_pitch - 0.02).clamp(-1.2, 1.2);
		changed = true;
	}
	if keyboard.pressed(KeyCode::PageUp) {
		state.surface_camera_distance = (state.surface_camera_distance * 0.97).max(20.0);
		changed = true;
	}
	if keyboard.pressed(KeyCode::PageDown) {
		state.surface_camera_distance *= 1.03;
		changed = true;
	}

	if changed {
		apply_orbit_camera_transform(&state, &mut camera_query);
	}
}

/// 同步文本显示
pub fn sync_medical_image_texts(
	state: Res<MedicalImageState>,
	mut source_query: Query<&mut Text, With<MedicalImageSourceTextMarker>>,
	mut status_query: Query<
		&mut Text,
		(
			With<MedicalImageStatusTextMarker>,
			Without<MedicalImageSourceTextMarker>,
		),
	>,
) {
	if !state.is_changed() {
		return;
	}

	for mut text in &mut source_query {
		text.0 = state.source_text.clone();
	}

	for mut text in &mut status_query {
		text.0 = state.status_text.clone();
	}
}

/// 创建统一按钮
fn spawn_button<T: Component>(parent: &mut ChildSpawnerCommands<'_>, marker: T, label: &str) {
	parent
		.spawn(MedicalImageButtonBundle::new(marker))
		.with_children(|button| {
			button.spawn((
				Text::new(label),
				TextFont {
					font_size: 14.0,
					..default()
				},
				TextColor(Color::WHITE),
			));
		});
}

/// 创建占位纹理
fn create_slice_textures(images: &mut Assets<Image>) -> [Handle<Image>; 3] {
	let mut image = || {
		images.add(Image::new_fill(
			Extent3d {
				width: 2,
				height: 2,
				depth_or_array_layers: 1,
			},
			TextureDimension::D2,
			&[32, 32, 32, 255],
			TextureFormat::Rgba8UnormSrgb,
			RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
		))
	};

	[image(), image(), image()]
}

/// 灰度图转 RGBA
fn grayscale_to_rgba(grayscale: &[u8]) -> Vec<u8> {
	let mut rgba = Vec::with_capacity(grayscale.len() * 4);
	for value in grayscale {
		rgba.extend_from_slice(&[*value, *value, *value, 255]);
	}
	rgba
}

/// 构造样例 NIfTI 路径
fn sample_nifti_path(file_name: &str) -> PathBuf {
	let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let base_dir = manifest_dir
		.parent()
		.map(std::path::Path::to_path_buf)
		.unwrap_or(manifest_dir);
	base_dir.join("data").join(file_name)
}

/// 将样例文件加载进页面状态
fn load_sample_into_state(path: &Path, state: &mut MedicalImageState) -> Result<(), String> {
	let volume = load_nifti_file(path).map_err(|error| error.to_string())?;
	let dims = volume.dims;
	let modality = volume.modality;
	state.volume_revision = state.volume_revision.saturating_add(1);
	state.volume = Some(volume);
	state.modality = Some(modality);
	state.load_state = MedicalImageLoadState::Ready;
	state.source_text = format!("文件: {}", path.display());
	state.reset_slice_index();
	state.apply_default_windowing();
	state.surface_mesh_stats = None;
	state.surface_dirty = false;
	state.volume_dirty = false;
	state.surface_focus_center = [0.0, 0.0, 0.0];
	state.surface_camera_distance = 400.0;
	state.surface_camera_yaw = 0.75;
	state.surface_camera_pitch = 0.45;
	state.volume_step_size = 1.0 / 256.0;
	state.render_mode = RenderMode::SliceOnly;
	update_status_text(state);
	state.status_text = format!(
		"已加载 {:?} | 尺寸: {} x {} x {} | 模式: {} | 窗位/窗宽: {:.1}/{:.1} | 阈值: {:.1} | 步长: {:.5}",
		modality,
		dims[0],
		dims[1],
		dims[2],
		render_mode_label(state.render_mode),
		state.window_center,
		state.window_width,
		state.surface_threshold,
		state.volume_step_size
	);
	Ok(())
}

/// 构建可用于 Bevy 的三角网格
fn build_surface_mesh_asset(surface_mesh: &SurfaceMeshData) -> Mesh {
	Mesh::new(
		PrimitiveTopology::TriangleList,
		RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
	)
	.with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, surface_mesh.positions.clone())
	.with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, surface_mesh.normals.clone())
	.with_inserted_indices(Indices::U32(surface_mesh.indices.clone()))
}

/// 按当前状态更新状态文本
fn update_status_text(state: &mut MedicalImageState) {
	let dims_text = state
		.volume
		.as_ref()
		.map(|volume| {
			format!(
				"{} x {} x {}",
				volume.dims[0], volume.dims[1], volume.dims[2]
			)
		})
		.unwrap_or_else(|| "-".to_string());
	let mesh_text = state
		.surface_mesh_stats
		.map(|stats| {
			format!(
				"{} 顶点 / {} 三角形",
				stats.vertex_count, stats.triangle_count
			)
		})
		.unwrap_or_else(|| "未生成表面".to_string());

	state.status_text = format!(
		"状态: {} | 尺寸: {dims_text} | 模式: {} | 窗位/窗宽: {:.1}/{:.1} | 阈值: {:.1} | 步长: {:.5} | 表面: {mesh_text}",
		load_state_label(state.load_state),
		render_mode_label(state.render_mode),
		state.window_center,
		state.window_width,
		state.surface_threshold,
		state.volume_step_size
	);
}

/// 页面状态文字
fn load_state_label(state: MedicalImageLoadState) -> &'static str {
	match state {
		MedicalImageLoadState::Empty => "空闲",
		MedicalImageLoadState::Ready => "就绪",
		MedicalImageLoadState::Busy => "处理中",
		MedicalImageLoadState::Error => "错误",
	}
}

/// 应用轨道相机参数
fn apply_orbit_camera_transform(
	state: &MedicalImageState,
	camera_query: &mut Query<&mut Transform, With<MedicalImageCamera3dMarker>>,
) {
	let Some(mut camera_transform) = camera_query.iter_mut().next() else {
		return;
	};

	let center = Vec3::from_array(state.surface_focus_center);
	let distance = state.surface_camera_distance.max(1.0);
	let yaw = state.surface_camera_yaw;
	let pitch = state.surface_camera_pitch;
	let offset = Vec3::new(
		distance * yaw.cos() * pitch.cos(),
		distance * pitch.sin(),
		distance * yaw.sin() * pitch.cos(),
	);
	camera_transform.translation = center + offset;
	camera_transform.look_at(center, Vec3::Y);
}

/// 渲染模式文字
fn render_mode_label(mode: RenderMode) -> &'static str {
	match mode {
		RenderMode::SliceOnly => "切片",
		RenderMode::Surface3d => "表面",
		RenderMode::Volume3d => "体渲染",
	}
}
