use es_fluent::EsFluent;
use koruma::{KorumaResult, Validate, validator};

/// Validates that a string is non-empty.
#[validator]
#[derive(Clone, Debug, EsFluent)]
pub struct NonEmptyStringValidation {
    #[koruma(value)]
    pub input: String,
}

impl Validate<String> for NonEmptyStringValidation {
    fn validate(&self, value: &String) -> KorumaResult {
        if value.is_empty() { Err(()) } else { Ok(()) }
    }
}

/// Validates that an optional string, when present, is non-empty.
#[validator]
#[derive(Clone, Debug, EsFluent)]
pub struct OptionalNonEmptyStringValidation {
    #[koruma(value)]
    pub input: String,
}

impl Validate<Option<String>> for OptionalNonEmptyStringValidation {
    fn validate(&self, value: &Option<String>) -> KorumaResult {
        match value {
            Some(s) if s.is_empty() => Err(()),
            _ => Ok(()),
        }
    }
}

/// Validates that a string is a valid email format.
#[validator]
#[derive(Clone, Debug, EsFluent)]
pub struct EmailValidation {
    #[koruma(value)]
    pub input: String,
}

impl Validate<String> for EmailValidation {
    fn validate(&self, value: &String) -> KorumaResult {
        if value.contains('@') && value.contains('.') {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// Validates that a number is within a specified range.
#[validator]
#[derive(Clone, Debug, EsFluent)]
pub struct NumberRangeValidation {
    pub min: u32,
    pub max: u32,
    #[koruma(value)]
    pub actual: u32,
}

impl Validate<u32> for NumberRangeValidation {
    fn validate(&self, value: &u32) -> KorumaResult {
        if *value < self.min || *value > self.max {
            Err(())
        } else {
            Ok(())
        }
    }
}

/// Validates that a number is positive.
#[validator]
#[derive(Clone, Debug, EsFluent)]
pub struct PositiveNumberValidation {
    #[koruma(value)]
    pub actual: f64,
}

impl Validate<f64> for PositiveNumberValidation {
    fn validate(&self, value: &f64) -> KorumaResult {
        if *value > 0.0 { Ok(()) } else { Err(()) }
    }
}
