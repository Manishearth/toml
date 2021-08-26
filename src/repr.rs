pub(crate) type InternalString = String;

/// A value together with its `to_string` representation,
/// including surrounding it whitespaces and comments.
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct Formatted<T> {
    value: T,
    pub(crate) repr: Repr,
    pub(crate) decor: Decor,
}

impl<T> Formatted<T> {
    pub fn raw(&self) -> &str {
        &self.repr.raw_value
    }

    pub fn prefix(&self) -> &str {
        &self.decor.prefix
    }

    pub fn suffix(&self) -> &str {
        &self.decor.suffix
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub(crate) fn new(v: T, repr: Repr, decor: Decor) -> Self {
        Self {
            value: v,
            repr,
            decor,
        }
    }
}

impl<D: std::fmt::Display> From<D> for Formatted<D> {
    fn from(other: D) -> Self {
        let repr = Repr::from(&other);
        Self {
            value: other,
            repr,
            decor: Decor::new("", ""),
        }
    }
}

// String representation of a key or a value
// together with a decoration.
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub(crate) struct Repr {
    pub(crate) raw_value: InternalString,
}

impl Repr {
    pub fn new(value: impl Into<InternalString>) -> Self {
        Repr {
            raw_value: value.into(),
        }
    }
}

impl<D: std::fmt::Display> From<&D> for Repr {
    fn from(other: &D) -> Self {
        Self::new(other.to_string())
    }
}

/// A prefix and suffix,
/// including comments, whitespaces and newlines.
#[derive(Eq, PartialEq, Clone, Default, Debug, Hash)]
pub struct Decor {
    pub(crate) prefix: InternalString,
    pub(crate) suffix: InternalString,
}

impl Decor {
    /// Creates a new decor from the given prefix and suffix.
    pub fn new(prefix: impl Into<String>, suffix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            suffix: suffix.into(),
        }
    }

    /// Get the prefix.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Get the suffix.
    pub fn suffix(&self) -> &str {
        &self.suffix
    }

    /// Render a value with its decor
    pub fn display<'d, D: std::fmt::Display + std::fmt::Debug>(
        &'d self,
        inner: &'d D,
    ) -> DecorDisplay<'d, D> {
        DecorDisplay { inner, decor: self }
    }
}

/// A prefix and suffix,
/// including comments, whitespaces and newlines.
#[derive(Debug)]
pub struct DecorDisplay<'d, D> {
    pub(crate) inner: &'d D,
    pub(crate) decor: &'d Decor,
}