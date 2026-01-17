#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![allow(clippy::type_complexity)]

/// 简单的加法函数
pub fn add(a: i32, b: i32) -> i32 {
	a + b
}

#[cfg(test)]
mod tests {
	use super::*;
	use rstest::rstest;

	#[test]
	fn test_add_basic() {
		assert_eq!(add(2, 3), 5);
		assert_eq!(add(-1, 1), 0);
		assert_eq!(add(0, 0), 0);
	}

	#[rstest]
	#[case(2, 3, 5)]
	#[case(-1, 1, 0)]
	#[case(0, 0, 0)]
	#[case(100, 200, 300)]
	fn test_add_parametrized(#[case] a: i32, #[case] b: i32, #[case] expected: i32) {
		assert_eq!(add(a, b), expected);
	}
}
