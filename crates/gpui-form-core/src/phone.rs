//! Phone-number validation helpers.
//!
//! These helpers wrap the `phonenumber` parser and add the extra country check
//! form UIs usually need: a globally valid `+1...` number is not accepted when
//! the user selected France.

pub use phonenumber::country;

use phonenumber::Mode;

/// Result of validating a phone number against a selected country.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PhoneNumberValidation {
    /// Empty input. Treat this separately from invalid input so optional phone
    /// fields can decide whether empty is acceptable.
    Empty,
    /// The number parsed, passed libphonenumber validity checks, and belongs
    /// to the selected country.
    Valid(ValidatedPhoneNumber),
    /// The number was non-empty but failed parsing, validity, or country match.
    Invalid(PhoneNumberValidationError),
}

impl PhoneNumberValidation {
    /// Whether the result is [`PhoneNumberValidation::Valid`].
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid(_))
    }

    /// The parsed E.164 representation when valid.
    pub fn e164(&self) -> Option<&str> {
        match self {
            Self::Valid(number) => Some(number.e164()),
            Self::Empty | Self::Invalid(_) => None,
        }
    }

    /// Human-readable status text suitable for demos, logs, and simple forms.
    pub fn message(&self) -> String {
        match self {
            Self::Empty => "Enter a phone number".to_string(),
            Self::Valid(number) => format!("Valid. E.164: {}", number.e164()),
            Self::Invalid(error) => error.to_string(),
        }
    }
}

/// A phone number that passed parser, validity, and selected-country checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedPhoneNumber {
    e164: String,
    country: country::Id,
}

impl ValidatedPhoneNumber {
    pub fn e164(&self) -> &str {
        &self.e164
    }

    pub fn country(&self) -> country::Id {
        self.country
    }
}

/// Why a non-empty phone number failed validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PhoneNumberValidationError {
    WrongCountry {
        selected_country: country::Id,
        selected_label: String,
        parsed_country: Option<country::Id>,
    },
    InvalidForCountry {
        selected_country: country::Id,
        selected_label: String,
    },
    Parse {
        selected_country: country::Id,
        selected_label: String,
        message: String,
    },
}

impl core::fmt::Display for PhoneNumberValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::WrongCountry {
                selected_label,
                parsed_country,
                ..
            } => write!(
                f,
                "Phone number country {parsed_country:?} does not match selected country: {selected_label}"
            ),
            Self::InvalidForCountry { selected_label, .. } => {
                write!(f, "Not a valid {selected_label} phone number")
            },
            Self::Parse {
                selected_label,
                message,
                ..
            } => write!(
                f,
                "Could not parse as {selected_label} phone number: {message}"
            ),
        }
    }
}

impl std::error::Error for PhoneNumberValidationError {}

/// Validate a phone number against a selected country id.
///
/// This is intentionally stricter than `phonenumber::parse(Some(country), raw)`
/// alone. International input such as `+1 415 550 2222` can parse successfully
/// even when the selected country is France; this helper rejects it because the
/// parsed country does not match the selected country.
pub fn validate_phone_number_for_country(
    raw: &str,
    selected_country: country::Id,
) -> PhoneNumberValidation {
    validate_phone_number_for_country_label(raw, selected_country, format!("{selected_country:?}"))
}

/// Validate a phone number against a selected country id, using `selected_label`
/// in user-facing error messages.
pub fn validate_phone_number_for_country_label(
    raw: &str,
    selected_country: country::Id,
    selected_label: impl Into<String>,
) -> PhoneNumberValidation {
    let selected_label = selected_label.into();
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return PhoneNumberValidation::Empty;
    }

    match phonenumber::parse(Some(selected_country), trimmed) {
        Ok(number) if number.country().id() != Some(selected_country) => {
            PhoneNumberValidation::Invalid(PhoneNumberValidationError::WrongCountry {
                selected_country,
                selected_label,
                parsed_country: number.country().id(),
            })
        },
        Ok(number) if number.is_valid() => PhoneNumberValidation::Valid(ValidatedPhoneNumber {
            e164: number.format().mode(Mode::E164).to_string(),
            country: selected_country,
        }),
        Ok(_) => PhoneNumberValidation::Invalid(PhoneNumberValidationError::InvalidForCountry {
            selected_country,
            selected_label,
        }),
        Err(error) => PhoneNumberValidation::Invalid(PhoneNumberValidationError::Parse {
            selected_country,
            selected_label,
            message: error.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_national_number_for_selected_country() {
        let result = validate_phone_number_for_country("415 555 2671", country::US);

        assert!(result.is_valid());
        assert_eq!(result.e164(), Some("+14155552671"));
    }

    #[test]
    fn rejects_international_number_for_wrong_selected_country() {
        let result =
            validate_phone_number_for_country_label("+1 415 550 2222", country::FR, "France");

        assert!(matches!(
            result,
            PhoneNumberValidation::Invalid(PhoneNumberValidationError::WrongCountry {
                selected_country: country::FR,
                parsed_country: Some(country::US),
                ..
            })
        ));
    }

    #[test]
    fn validates_french_number_for_france_not_us() {
        assert!(validate_phone_number_for_country("01 42 68 53 00", country::FR).is_valid());
        assert!(!validate_phone_number_for_country("01 42 68 53 00", country::US).is_valid());
    }
}
