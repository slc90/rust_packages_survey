#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use bevy::{app::Plugin, ecs::resource::Resource};

use crate::data_structure::{Language, LanguageKey};

pub mod data_structure;
mod locale_en;
mod locale_zh;

pub struct I18nPlugin;

impl Plugin for I18nPlugin {
	fn build(&self, app: &mut bevy::app::App) {
		app.insert_resource(LanguageManager::default());
	}
}

#[derive(Resource, Debug)]
pub struct LanguageManager {
	current_language: Language,
}

impl Default for LanguageManager {
	fn default() -> Self {
		Self {
			current_language: Language::Chinese,
		}
	}
}

impl LanguageManager {
	/// 返回当前的语言
	pub fn current_language(self) -> Language {
		self.current_language
	}

	/// 设置当前的语言
	pub fn set_current_language(&mut self, language: Language) {
		self.current_language = language;
	}

	/// 根据当前的语言返回对应的翻译
	pub fn lookup(&self, key: LanguageKey) -> &'static str {
		match self.current_language {
			Language::English => locale_en::lookup(key),
			Language::Chinese => locale_zh::lookup(key),
		}
	}
}
