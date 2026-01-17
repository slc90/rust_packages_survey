use crate::homepage::{
	common::ContentAreaMarker,
	setting::components::{
		ChineseRadioButtonMarker, EnglishRadioButtonMarker, LanguageRadioGroupMarker,
		SettingContentMarker,
	},
};
use bevy::{
	picking::hover::Hovered,
	prelude::*,
	ui::Checked,
	ui_widgets::{RadioButton, RadioGroup, ValueChange, observe},
};
use config::{Setting, data_structure::save_to_file};
use i18n::{LanguageManager, data_structure::Language};

const ELEMENT_FILL: Color = Color::srgb(0.35, 0.75, 0.35);
const CHECKED_BORDER_COLOR: Color = Color::srgb(0.2, 0.6, 0.2);
const UNCHECKED_BORDER_COLOR: Color = Color::srgb(0.45, 0.45, 0.45);

/// 进入Setting页面时触发，创建设置界面
pub fn on_enter(
	mut commands: Commands,
	query: Query<Entity, With<ContentAreaMarker>>,
	language_manager: Res<LanguageManager>,
) {
	info!("进入设置页面");
	// 获取内容区域的实体
	if let Ok(content_area) = query.single() {
		// 获取当前语言状态
		let is_english = language_manager.current_language() == Language::English;
		// 在内容区域中创建Setting界面
		commands.entity(content_area).with_children(|parent| {
			// 创建设置内容实体
			let mut content_builder = parent.spawn((
				SettingContentMarker,
				Node {
					width: Val::Percent(100.0),
					height: Val::Percent(100.0),
					display: Display::Flex,
					flex_direction: FlexDirection::Column,
					align_items: AlignItems::Center,
					justify_content: JustifyContent::Center,
					row_gap: Val::Px(20.0),
					..default()
				},
				BackgroundColor(Color::WHITE),
			));
			// 添加子元素
			content_builder.with_children(|parent| {
				// 语言切换单选按钮组
				let mut radio_group = parent.spawn((
					Node {
						display: Display::Flex,
						flex_direction: FlexDirection::Column,
						align_items: AlignItems::Start,
						row_gap: Val::Px(10.0),
						..default()
					},
					Name::new("LanguageRadioGroup"),
					LanguageRadioGroupMarker,
					RadioGroup,
					observe(on_language_radio_change),
				));
				// 创建中文单选按钮
				radio_group.with_children(|parent| {
					let mut entity_commands = parent.spawn(radio_button(
						"中文",
						Language::Chinese,
						ChineseRadioButtonMarker,
						!is_english,
					));
					if !is_english {
						entity_commands.insert(Checked);
					}
				});

				// 创建英文单选按钮
				radio_group.with_children(|parent| {
					let mut entity_commands = parent.spawn(radio_button(
						"English",
						Language::English,
						EnglishRadioButtonMarker,
						is_english,
					));
					if is_english {
						entity_commands.insert(Checked);
					}
				});
			});
		});
	}
}

/// 创建单个单选按钮
fn radio_button(
	label: &str,
	_language: Language,
	marker: impl Component,
	is_checked: bool,
) -> impl Bundle {
	(
		Node {
			display: Display::Flex,
			flex_direction: FlexDirection::Row,
			justify_content: JustifyContent::FlexStart,
			align_items: AlignItems::Center,
			align_content: AlignContent::Center,
			column_gap: Val::Px(12.0),
			..default()
		},
		Name::new(format!("{}RadioButton", label)),
		Hovered::default(),
		marker,
		RadioButton,
		children![
			// 单选按钮外圈
			(
				Node {
					display: Display::Flex,
					width: Val::Px(24.0),
					height: Val::Px(24.0),
					border: UiRect::all(Val::Px(2.0)),
					border_radius: BorderRadius::MAX,
					..default()
				},
				BorderColor::all(if is_checked {
					CHECKED_BORDER_COLOR
				} else {
					UNCHECKED_BORDER_COLOR
				}),
				Name::new("RadioButtonBorder"),
				children![
					// 单选按钮内圈
					(
						Node {
							display: Display::Flex,
							width: Val::Px(12.0),
							height: Val::Px(12.0),
							position_type: PositionType::Absolute,
							left: Val::Px(4.0),
							top: Val::Px(4.0),
							border_radius: BorderRadius::MAX,
							..default()
						},
						BackgroundColor(if is_checked {
							ELEMENT_FILL
						} else {
							Color::NONE
						}),
						Name::new("RadioButtonMark"),
					),
				],
			),
			// 单选按钮标签
			(
				Text::new(label),
				TextColor(Color::BLACK),
				Name::new("RadioButtonLabel"),
			),
		],
	)
}

/// 处理语言单选按钮变化事件
fn on_language_radio_change(
	value_change: On<ValueChange<Entity>>,
	mut language_manager: ResMut<LanguageManager>,
	mut setting: ResMut<Setting>,
	chinese_query: Query<(), With<ChineseRadioButtonMarker>>,
	english_query: Query<(), With<EnglishRadioButtonMarker>>,
) {
	// 根据选择的单选按钮确定语言
	let new_language = if chinese_query.contains(value_change.value) {
		Language::Chinese
	} else if english_query.contains(value_change.value) {
		Language::English
	} else {
		warn!("未知的单选按钮实体: {:?}", value_change.value);
		return;
	};
	// 更新语言管理器
	language_manager.set_current_language(new_language);
	// 更新配置文件
	setting.language = new_language;
	// 保存配置文件
	if let Ok(config_path) = std::env::current_exe() {
		let config_file_path = config_path.join("../config_file/config.json");
		if let Some(path_str) = config_file_path.to_str() {
			if let Err(e) = save_to_file(&setting, path_str) {
				error!("保存配置文件失败: {}", e);
			}
		} else {
			error!("无法将配置文件路径转换为字符串");
		}
	} else {
		error!("无法获取当前可执行文件路径");
	}
	info!("语言已切换为: {:?}", new_language);
}

/// 离开Setting页面时触发，清理资源
pub fn on_exit(mut commands: Commands, query: Query<Entity, With<SettingContentMarker>>) {
	info!("离开设置页面");
	// 清理所有Setting内容实体
	for entity in query.iter() {
		commands.entity(entity).despawn();
	}
}

/// 更新单选按钮边框颜色当选中状态变化或语言变化时
pub fn update_radio_button_border_color(
	radio_button_query: Query<
		(&Children, Has<Checked>),
		(
			Or<(Added<Checked>, Changed<Checked>)>,
			Or<(
				With<ChineseRadioButtonMarker>,
				With<EnglishRadioButtonMarker>,
			)>,
		),
	>,
	mut border_color_query: Query<&mut BorderColor>,
) {
	for (radio_children, is_checked) in radio_button_query.iter() {
		// 查找单选按钮边框实体（第一个子实体）
		if let Some(border_entity) = radio_children.first().copied() {
			// 更新边框实体的边框颜色
			if let Ok(mut border_color) = border_color_query.get_mut(border_entity) {
				border_color.set_all(if is_checked {
					CHECKED_BORDER_COLOR
				} else {
					UNCHECKED_BORDER_COLOR
				});
				debug!(
					"Updated radio button border color: is_checked = {}",
					is_checked
				);
			}
		}
	}
}

/// 补充系统：检测被移除的Checked组件并更新边框颜色
pub fn update_radio_button_border_color2(
	mut removed_checked: RemovedComponents<Checked>,
	radio_button_query: Query<
		(&Children, Has<Checked>),
		Or<(
			With<ChineseRadioButtonMarker>,
			With<EnglishRadioButtonMarker>,
		)>,
	>,
	mut border_color_query: Query<&mut BorderColor>,
) {
	for entity in removed_checked.read() {
		if let Ok((radio_children, is_checked)) = radio_button_query.get(entity) {
			// 查找单选按钮边框实体（第一个子实体）
			if let Some(border_entity) = radio_children.first().copied() {
				// 更新边框实体的边框颜色
				if let Ok(mut border_color) = border_color_query.get_mut(border_entity) {
					border_color.set_all(if is_checked {
						CHECKED_BORDER_COLOR
					} else {
						UNCHECKED_BORDER_COLOR
					});
					debug!(
						"Updated radio button border color (removed checked): is_checked = {}",
						is_checked
					);
				}
			}
		}
	}
}

/// 更新单选按钮标记颜色当选中状态变化或语言变化时
pub fn update_radio_button_mark_color(
	radio_button_query: Query<
		(&Children, Has<Checked>),
		(
			Or<(Added<Checked>, Changed<Checked>)>,
			Or<(
				With<ChineseRadioButtonMarker>,
				With<EnglishRadioButtonMarker>,
			)>,
		),
	>,
	children_query: Query<&Children>,
	mut bg_color_query: Query<(&mut BackgroundColor, &Name)>,
) {
	for (radio_children, is_checked) in radio_button_query.iter() {
		// 查找单选按钮边框实体（第一个子实体）
		if let Some(border_entity) = radio_children.first().copied() {
			// 获取边框实体的子实体
			if let Ok(border_children) = children_query.get(border_entity) {
				// 查找标记实体（边框的第一个子实体）
				if let Some(mark_entity) = border_children.first().copied() {
					// 更新标记实体的背景颜色
					if let Ok((mut bg_color, _)) = bg_color_query.get_mut(mark_entity) {
						bg_color.0 = if is_checked {
							ELEMENT_FILL
						} else {
							Color::NONE
						};
						debug!(
							"Updated radio button mark color: is_checked = {}",
							is_checked
						);
					}
				}
			}
		}
	}
}

/// 补充系统：检测被移除的Checked组件并更新标记颜色
pub fn update_radio_button_mark_color2(
	mut removed_checked: RemovedComponents<Checked>,
	radio_button_query: Query<
		(&Children, Has<Checked>),
		Or<(
			With<ChineseRadioButtonMarker>,
			With<EnglishRadioButtonMarker>,
		)>,
	>,
	children_query: Query<&Children>,
	mut bg_color_query: Query<(&mut BackgroundColor, &Name)>,
) {
	for entity in removed_checked.read() {
		if let Ok((radio_children, is_checked)) = radio_button_query.get(entity) {
			// 查找单选按钮边框实体（第一个子实体）
			if let Some(border_entity) = radio_children.first().copied() {
				// 获取边框实体的子实体
				if let Ok(border_children) = children_query.get(border_entity) {
					// 查找标记实体（边框的第一个子实体）
					if let Some(mark_entity) = border_children.first().copied() {
						// 更新标记实体的背景颜色
						if let Ok((mut bg_color, _)) = bg_color_query.get_mut(mark_entity) {
							bg_color.0 = if is_checked {
								ELEMENT_FILL
							} else {
								Color::NONE
							};
							debug!(
								"Updated radio button mark color (removed checked): is_checked = {}",
								is_checked
							);
						}
					}
				}
			}
		}
	}
}

/// 同步单选按钮的选中状态到当前语言，并直接更新颜色
pub fn sync_radio_buttons_to_language(
	language_manager: Res<LanguageManager>,
	chinese_radio_query: Query<(Entity, Has<Checked>, &Children), With<ChineseRadioButtonMarker>>,
	english_radio_query: Query<(Entity, Has<Checked>, &Children), With<EnglishRadioButtonMarker>>,
	children_query: Query<&Children>,
	mut commands: Commands,
	mut border_color_query: Query<&mut BorderColor>,
	mut bg_color_query: Query<&mut BackgroundColor>,
) {
	if language_manager.is_changed() {
		let is_english = matches!(language_manager.current_language(), Language::English);

		// 更新中文单选按钮
		for (chinese_entity, has_checked, chinese_children) in chinese_radio_query.iter() {
			let should_be_checked = !is_english;

			// 更新Checked组件
			if should_be_checked && !has_checked {
				commands.entity(chinese_entity).insert(Checked);
			} else if !should_be_checked && has_checked {
				commands.entity(chinese_entity).remove::<Checked>();
			}

			// 直接更新颜色，不等待其他系统
			if let Some(border_entity) = chinese_children.first().copied() {
				// 更新边框颜色
				if let Ok(mut border_color) = border_color_query.get_mut(border_entity) {
					border_color.set_all(if should_be_checked {
						CHECKED_BORDER_COLOR
					} else {
						UNCHECKED_BORDER_COLOR
					});
					debug!(
						"Sync: Updated Chinese radio button border color: should_be_checked = {}",
						should_be_checked
					);
				}

				// 更新标记颜色
				if let Ok(border_children) = children_query.get(border_entity)
					&& let Some(mark_entity) = border_children.first().copied()
					&& let Ok(mut bg_color) = bg_color_query.get_mut(mark_entity)
				{
					bg_color.0 = if should_be_checked {
						ELEMENT_FILL
					} else {
						Color::NONE
					};
					debug!(
						"Sync: Updated Chinese radio button mark color: should_be_checked = {}",
						should_be_checked
					);
				}
			}
		}

		// 更新英文单选按钮
		for (english_entity, has_checked, english_children) in english_radio_query.iter() {
			let should_be_checked = is_english;

			// 更新Checked组件
			if should_be_checked && !has_checked {
				commands.entity(english_entity).insert(Checked);
			} else if !should_be_checked && has_checked {
				commands.entity(english_entity).remove::<Checked>();
			}

			// 直接更新颜色，不等待其他系统
			if let Some(border_entity) = english_children.first().copied() {
				// 更新边框颜色
				if let Ok(mut border_color) = border_color_query.get_mut(border_entity) {
					border_color.set_all(if should_be_checked {
						CHECKED_BORDER_COLOR
					} else {
						UNCHECKED_BORDER_COLOR
					});
					debug!(
						"Sync: Updated English radio button border color: should_be_checked = {}",
						should_be_checked
					);
				}

				// 更新标记颜色
				if let Ok(border_children) = children_query.get(border_entity)
					&& let Some(mark_entity) = border_children.first().copied()
					&& let Ok(mut bg_color) = bg_color_query.get_mut(mark_entity)
				{
					bg_color.0 = if should_be_checked {
						ELEMENT_FILL
					} else {
						Color::NONE
					};
					debug!(
						"Sync: Updated English radio button mark color: should_be_checked = {}",
						should_be_checked
					);
				}
			}
		}
	}
}
