use crate::homepage::common::Functions;
use crate::homepage::medical_image::systems::{
	handle_load_ct_sample, handle_load_mr_sample, handle_window_center_decrease,
	handle_window_center_increase, handle_window_width_decrease, handle_window_width_increase,
	on_enter, on_exit, sync_medical_image_texts, update_slice_images,
};
use bevy::prelude::*;

/// 医学影像页面插件
pub struct MedicalImagePlugin;

impl Plugin for MedicalImagePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(Functions::MedicalImage), on_enter)
			.add_systems(OnExit(Functions::MedicalImage), on_exit)
			.add_systems(
				Update,
				(
					handle_load_ct_sample,
					handle_load_mr_sample,
					handle_window_center_decrease,
					handle_window_center_increase,
					handle_window_width_decrease,
					handle_window_width_increase,
					update_slice_images,
					sync_medical_image_texts,
				)
					.run_if(in_state(Functions::MedicalImage)),
			);
	}
}
