#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use crate::data_structure::{Language, LanguageKey};
use bevy::{
	app::{App, Plugin},
	ecs::resource::Resource,
	log::info,
};

pub mod data_structure;
mod locale_en;
mod locale_zh;

pub struct I18nPlugin;

impl Plugin for I18nPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(LanguageManager::default());
	}
}

#[derive(Resource, Debug)]
pub struct LanguageManager {
	current_language: Language,
}

impl LanguageManager {
	/// Returns the current language
	pub fn current_language(&self) -> Language {
		self.current_language
	}

	/// Sets the current language
	pub fn set_current_language(&mut self, language: Language) {
		self.current_language = language;
		info!("当前语言已设置为：{:?}", language);
	}

	/// Looks up a translation key in the current language
	pub fn lookup(&self, key: LanguageKey) -> &'static str {
		match self.current_language {
			Language::English => locale_en::lookup(key),
			Language::Chinese => locale_zh::lookup(key),
		}
	}
}

impl Default for LanguageManager {
	fn default() -> Self {
		Self {
			current_language: Language::Chinese,
		}
	}
}
