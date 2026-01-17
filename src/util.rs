use std::fmt::Display;

pub trait ResultExt<T> {
    fn stringify_err(self) -> Result<T, String>;
}

impl<T, E: Display> ResultExt<T> for Result<T, E> {
    fn stringify_err(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}

pub fn wrap_around(index: usize, offset: isize, max: usize) -> usize {
    let max_rest =
        max.saturating_add_signed(offset.saturating_add(1).saturating_add_unsigned(index));
    index.checked_add_signed(offset).unwrap_or(max_rest) % (max + 1)
}

pub fn wrap_around_optional(index: Option<usize>, offset: isize, max: usize) -> Option<usize> {
    match index {
        Some(idx) => Some(wrap_around(idx, offset, max)),
        None if max > 0 => Some(max),
        None => None,
    }
}

pub fn args<const N: usize>(items: [impl Into<String>; N]) -> Vec<String> {
    items.into_iter().map(Into::into).collect()
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_wrap_to_zero() {
        assert_eq!(wrap_around(1, -1, 10), 0);
    }

    #[test]
    fn test_wrap_below_zero() {
        assert_eq!(wrap_around(1, -3, 10), 9);
    }

    #[test]
    fn test_wrap_to_max() {
        assert_eq!(wrap_around(1, 9, 10), 10);
    }

    #[test]
    fn test_wrap_above_max() {
        assert_eq!(wrap_around(1, 11, 10), 1);
    }
}
