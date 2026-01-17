use crate::data_structure::LanguageKey;

/// 放所有的中文翻译
pub fn lookup(language_key: LanguageKey) -> &'static str {
	match language_key {
		LanguageKey::Title => "Rust包调研",
		LanguageKey::Function => "功能",
		LanguageKey::Language => "语言",
		LanguageKey::About => "关于",
		LanguageKey::Setting => "设置",
	}
}
