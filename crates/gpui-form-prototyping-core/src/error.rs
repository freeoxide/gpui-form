use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrototypingError {
    InvalidIdentifier {
        kind: &'static str,
        value: String,
    },
    InvalidPath {
        kind: &'static str,
        value: String,
        error: String,
    },
    InvalidSourcePath {
        source_path: String,
    },
    InvalidType {
        field_name: String,
        value: String,
        error: String,
    },
    /// A `component(list(...))` field whose value type is not `Vec<...>`: the
    /// scaffold appends `<T as Default>::default()` items, which only exists
    /// for `Vec<T>` metadata.
    InvalidListValueType {
        field_name: String,
        value: String,
    },
    /// One or more import entries in prototyping metadata were not valid
    /// Rust paths, identifiers, or aliases. Every offender is collected
    /// before failing, so one pass reports all of them.
    InvalidImports {
        errors: Vec<PrototypingError>,
    },
}

impl fmt::Display for PrototypingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentifier { kind, value } => {
                write!(f, "invalid {kind} `{value}` in prototyping metadata")
            },
            Self::InvalidPath { kind, value, error } => {
                write!(
                    f,
                    "invalid {kind} `{value}` in prototyping metadata: {error}"
                )
            },
            Self::InvalidSourcePath { source_path } => {
                write!(
                    f,
                    "failed to derive a Rust module path from source path `{source_path}`"
                )
            },
            Self::InvalidType {
                field_name,
                value,
                error,
            } => {
                write!(
                    f,
                    "invalid value type `{value}` for field `{field_name}` in prototyping metadata: {error}"
                )
            },
            Self::InvalidListValueType { field_name, value } => {
                write!(
                    f,
                    "list field `{field_name}` must have a `Vec<...>` value type in prototyping metadata (the scaffold appends `<T as Default>::default()` items); found `{value}`"
                )
            },
            Self::InvalidImports { errors } => {
                write!(f, "invalid imports in prototyping metadata:")?;
                for error in errors {
                    write!(f, " {error};")?;
                }
                Ok(())
            },
        }
    }
}

impl std::error::Error for PrototypingError {}

pub type PrototypingResult<T> = Result<T, PrototypingError>;
