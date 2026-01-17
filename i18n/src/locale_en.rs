use crate::data_structure::LanguageKey;

/// 放所有的英文翻译
pub fn lookup(language_key: LanguageKey) -> &'static str {
	match language_key {
		LanguageKey::Title => "Rust Package Survey",
		LanguageKey::Function => "Function",
		LanguageKey::Language => "Language",
		LanguageKey::About => "About",
		LanguageKey::Setting => "Settings",
	}
}
