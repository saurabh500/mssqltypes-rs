// Licensed under the MIT License. See LICENSE file in the project root for full license information.

//! `SqlCompareOptions` — comparison options for `SqlString` values.
//!
//! Equivalent to C# `System.Data.SqlTypes.SqlCompareOptions`, simplified
//! to 4 mutually exclusive variants (no locale/bitflag support).

/// Controls how `SqlString` comparisons are performed.
///
/// - `None` — case-sensitive ordinal comparison
/// - `IgnoreCase` — case-insensitive ASCII comparison (default)
/// - `BinarySort` — raw UTF-8 byte comparison
/// - `BinarySort2` — identical to `BinarySort` (C# legacy distinction)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum SqlCompareOptions {
    /// Case-sensitive ordinal comparison.
    None,
    /// Case-insensitive ASCII comparison (default).
    #[default]
    IgnoreCase,
    /// Raw UTF-8 byte comparison.
    BinarySort,
    /// Identical to `BinarySort` (C# legacy compatibility).
    BinarySort2,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::hash::{DefaultHasher, Hash, Hasher};

    // ── T008: SqlCompareOptions variants, traits ─────────────────────────────

    #[test]
    fn all_four_variants_exist() {
        let _none = SqlCompareOptions::None;
        let _ic = SqlCompareOptions::IgnoreCase;
        let _bs = SqlCompareOptions::BinarySort;
        let _bs2 = SqlCompareOptions::BinarySort2;
    }

    #[test]
    fn default_is_ignore_case() {
        assert_eq!(SqlCompareOptions::default(), SqlCompareOptions::IgnoreCase);
    }

    #[test]
    fn copy_clone_work() {
        let a = SqlCompareOptions::BinarySort;
        let b = a; // Copy
        let c = a.clone(); // Clone
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn debug_format() {
        let s = format!("{:?}", SqlCompareOptions::IgnoreCase);
        assert_eq!(s, "IgnoreCase");
    }

    #[test]
    fn partial_eq_and_eq() {
        assert_eq!(SqlCompareOptions::None, SqlCompareOptions::None);
        assert_ne!(SqlCompareOptions::None, SqlCompareOptions::IgnoreCase);
        assert_ne!(
            SqlCompareOptions::BinarySort,
            SqlCompareOptions::BinarySort2
        );
    }

    #[test]
    fn hash_works_in_hashset() {
        let mut set = HashSet::new();
        set.insert(SqlCompareOptions::None);
        set.insert(SqlCompareOptions::IgnoreCase);
        set.insert(SqlCompareOptions::BinarySort);
        set.insert(SqlCompareOptions::BinarySort2);
        assert_eq!(set.len(), 4);
        assert!(set.contains(&SqlCompareOptions::None));
    }

    #[test]
    fn equal_variants_hash_equal() {
        let hash = |v: SqlCompareOptions| {
            let mut h = DefaultHasher::new();
            v.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash(SqlCompareOptions::None), hash(SqlCompareOptions::None));
        assert_eq!(
            hash(SqlCompareOptions::IgnoreCase),
            hash(SqlCompareOptions::IgnoreCase)
        );
    }
}
