use crate::error::SqlTypeError;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{BitAnd, BitOr, BitXor, Not};
use std::str::FromStr;

// Internal state constants matching C# SqlBoolean layout
const X_NULL: u8 = 0;
const X_FALSE: u8 = 1;
const X_TRUE: u8 = 2;

/// A three-state boolean type representing SQL Server's BIT type with full NULL support.
///
/// Uses a `u8` internal representation (`0=Null, 1=False, 2=True`) matching the
/// C# `System.Data.SqlTypes.SqlBoolean` layout.
#[derive(Copy, Clone, Debug)]
pub struct SqlBoolean {
    m_value: u8,
}
