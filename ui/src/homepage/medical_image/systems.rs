use crate::homepage::common::ContentAreaMarker;
use crate::homepage::medical_image::components::{
	AxialSliceImageMarker, CoronalSliceImageMarker, LoadCtSampleButtonMarker,
	LoadMrSampleButtonMarker, MedicalImageButtonBundle, MedicalImageContentMarker,
	MedicalImagePanelBundle, MedicalImageSourceTextMarker, MedicalImageStatusTextMarker,
	SagittalSliceImageMarker, SliceImageBundle, WindowCenterDecreaseButtonMarker,
	WindowCenterIncreaseButtonMarker, WindowWidthDecreaseButtonMarker,
	WindowWidthIncreaseButtonMarker,
};
use crate::homepage::medical_image::resources::{MedicalImageState, MedicalImageTextures};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use medical_image::{SliceAxis, extract_slice, load_nifti_file, normalize_slice_to_u8};
use std::path::{Path, PathBuf};

const SLICE_PANEL_SIZE: f32 = 320.0;

/// 进入医学影像页面
pub fn on_enter(
	mut commands: Commands,
	content_area_query: Query<Entity, With<ContentAreaMarker>>,
	mut images: ResMut<Assets<Image>>,
) {
	info!("进入医学影像页面");

	let textures = create_slice_textures(&mut images);
	commands.insert_resource(MedicalImageTextures {
		axial: textures[0].clone(),
		coronal: textures[1].clone(),
		sagittal: textures[2].clone(),
	});

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
						padding: UiRect::all(Val::Px(12.0)),
						row_gap: Val::Px(12.0),
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
						flex_direction: FlexDirection::Row,
						column_gap: Val::Px(8.0),
						..default()
					},))
						.with_children(|buttons| {
							buttons
								.spawn(MedicalImageButtonBundle::new(LoadCtSampleButtonMarker))
								.with_children(|button| {
									button.spawn((
										Text::new("加载 CT 样例"),
										TextFont {
											font_size: 14.0,
											..default()
										},
										TextColor(Color::WHITE),
									));
								});
							buttons
								.spawn(MedicalImageButtonBundle::new(LoadMrSampleButtonMarker))
								.with_children(|button| {
									button.spawn((
										Text::new("加载 MR 样例"),
										TextFont {
											font_size: 14.0,
											..default()
										},
										TextColor(Color::WHITE),
									));
								});
							buttons
								.spawn(MedicalImageButtonBundle::new(
									WindowCenterDecreaseButtonMarker,
								))
								.with_children(|button| {
									button.spawn((
										Text::new("窗位 -"),
										TextFont {
											font_size: 14.0,
											..default()
										},
										TextColor(Color::WHITE),
									));
								});
							buttons
								.spawn(MedicalImageButtonBundle::new(
									WindowCenterIncreaseButtonMarker,
								))
								.with_children(|button| {
									button.spawn((
										Text::new("窗位 +"),
										TextFont {
											font_size: 14.0,
											..default()
										},
										TextColor(Color::WHITE),
									));
								});
							buttons
								.spawn(MedicalImageButtonBundle::new(
									WindowWidthDecreaseButtonMarker,
								))
								.with_children(|button| {
									button.spawn((
										Text::new("窗宽 -"),
										TextFont {
											font_size: 14.0,
											..default()
										},
										TextColor(Color::WHITE),
									));
								});
							buttons
								.spawn(MedicalImageButtonBundle::new(
									WindowWidthIncreaseButtonMarker,
								))
								.with_children(|button| {
									button.spawn((
										Text::new("窗宽 +"),
										TextFont {
											font_size: 14.0,
											..default()
										},
										TextColor(Color::WHITE),
									));
								});
						});

					root.spawn((Node {
						width: Val::Percent(100.0),
						flex_direction: FlexDirection::Row,
						column_gap: Val::Px(12.0),
						justify_content: JustifyContent::SpaceBetween,
						..default()
					},))
						.with_children(|panels| {
							panels
								.spawn(MedicalImagePanelBundle::new(
									SLICE_PANEL_SIZE,
									SLICE_PANEL_SIZE + 40.0,
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
										SLICE_PANEL_SIZE - 16.0,
									));
								});
							panels
								.spawn(MedicalImagePanelBundle::new(
									SLICE_PANEL_SIZE,
									SLICE_PANEL_SIZE + 40.0,
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
										SLICE_PANEL_SIZE - 16.0,
									));
								});
							panels
								.spawn(MedicalImagePanelBundle::new(
									SLICE_PANEL_SIZE,
									SLICE_PANEL_SIZE + 40.0,
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
										SLICE_PANEL_SIZE - 16.0,
									));
								});
						});
				});
		});
	}
}

/// 离开医学影像页面
pub fn on_exit(mut commands: Commands, query: Query<Entity, With<MedicalImageContentMarker>>) {
	info!("离开医学影像页面");
	for entity in &query {
		commands.entity(entity).despawn();
	}
	commands.remove_resource::<MedicalImageState>();
	commands.remove_resource::<MedicalImageTextures>();
}

/// 处理加载 CT 样例
pub fn handle_load_ct_sample(
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<LoadCtSampleButtonMarker>)>,
	mut state: ResMut<MedicalImageState>,
) {
	for interaction in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			let path = sample_nifti_path("SubjectUCI29_CT_acpc_f.nii");
			if let Err(error) = load_sample_into_state(&path, &mut state) {
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
			let path = sample_nifti_path("SubjectUCI29_MR_acpc.nii");
			if let Err(error) = load_sample_into_state(&path, &mut state) {
				state.status_text = format!("加载 MR 样例失败: {error}");
			}
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
			state.status_text = format!(
				"窗位/窗宽: {:.1} / {:.1}",
				state.window_center, state.window_width
			);
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
			state.status_text = format!(
				"窗位/窗宽: {:.1} / {:.1}",
				state.window_center, state.window_width
			);
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
			state.status_text = format!(
				"窗位/窗宽: {:.1} / {:.1}",
				state.window_center, state.window_width
			);
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
			state.status_text = format!(
				"窗位/窗宽: {:.1} / {:.1}",
				state.window_center, state.window_width
			);
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
	state.volume = Some(volume);
	state.modality = Some(modality);
	state.source_text = format!("文件: {}", path.display());
	state.status_text = format!(
		"已加载 {:?} | 尺寸: {} x {} x {} | 模式: 三视图",
		modality, dims[0], dims[1], dims[2]
	);
	state.reset_slice_index();
	state.apply_default_windowing();
	Ok(())
}
