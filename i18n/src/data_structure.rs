/// 所有可用的语言
use bevy::prelude::Reflect;

#[derive(Debug, Clone, Copy, Reflect)]
pub enum Language {
	English,

	Chinese,
}

/// 所有需要翻译的Key
/// 每次添加新的Key时，可以让编译器检查是不是有遗漏翻译
pub enum LanguageKey {
	Title,
}
