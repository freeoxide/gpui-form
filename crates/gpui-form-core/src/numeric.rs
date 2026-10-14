/// Validate signed numeric input: accepts empty strings, a lone `-`, and
/// signed numbers with optional decimal points; rejects leading zeros
/// (`"00"`, `"-01"`) and misplaced `-`. With `require_parse`, must parse as `T`.
pub fn validate_signed_numeric<T: std::str::FromStr>(value: &str, require_parse: bool) -> bool {
    if value.is_empty() {
        return true;
    }

    let bytes = value.as_bytes();

    if bytes.len() == 1 && bytes[0] == b'-' {
        return true;
    }

    if !bytes[0].is_ascii_digit() && bytes[0] != b'-' {
        return false;
    }

    if bytes.len() > 1 {
        let rest = if bytes[0] == b'-' { &bytes[1..] } else { bytes };

        if rest[0] == b'0' && rest.len() > 1 && rest[1].is_ascii_digit() {
            return false;
        }

        if !rest.iter().all(|&b| b.is_ascii_digit() || b == b'.') {
            return false;
        }
    }

    if require_parse {
        value.parse::<T>().is_ok()
    } else {
        true
    }
}

/// Validate unsigned numeric input: accepts empty strings and digit-only
/// values with no leading zeros; rejects `-` and any non-digit. With
/// `require_parse`, the value must also parse as `T`.
pub fn validate_unsigned_numeric<T: std::str::FromStr>(value: &str, require_parse: bool) -> bool {
    if value.is_empty() {
        return true;
    }

    let bytes = value.as_bytes();

    if !bytes[0].is_ascii_digit() {
        return false;
    }

    if bytes[0] == b'0' && bytes.len() > 1 && bytes[1].is_ascii_digit() {
        return false;
    }

    if !bytes.iter().all(|&b| b.is_ascii_digit()) {
        return false;
    }

    if require_parse {
        value.parse::<T>().is_ok()
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_signed_numeric() {
        assert!(validate_signed_numeric::<i32>("", true));
        assert!(validate_signed_numeric::<i32>("-", true));
        assert!(validate_signed_numeric::<i32>("0", true));
        assert!(validate_signed_numeric::<i32>("-0", true));
        assert!(validate_signed_numeric::<i32>("123", true));
        assert!(validate_signed_numeric::<i32>("-123", true));

        assert!(validate_signed_numeric::<f64>("0.5", true));
        assert!(validate_signed_numeric::<f64>("-0.5", true));
        assert!(validate_signed_numeric::<f64>("123.456", true));

        assert!(!validate_signed_numeric::<i32>("00", true));
        assert!(!validate_signed_numeric::<i32>("01", true));
        assert!(!validate_signed_numeric::<i32>("-00", true));
        assert!(!validate_signed_numeric::<i32>("-01", true));

        assert!(!validate_signed_numeric::<i32>("1-2", true));
        assert!(!validate_signed_numeric::<i32>("--1", true));
    }

    #[test]
    fn test_validate_signed_numeric_uses_parse_semantics() {
        assert!(validate_signed_numeric::<f64>("1.", true));
        assert!(!validate_signed_numeric::<f64>("1.2.3", true));
        assert!(!validate_signed_numeric::<f64>("-1.2.3", true));
        assert!(!validate_signed_numeric::<i32>("1.5", true));
    }

    #[test]
    fn test_validate_unsigned_numeric() {
        assert!(validate_unsigned_numeric::<u32>("", true));
        assert!(validate_unsigned_numeric::<u32>("0", true));
        assert!(validate_unsigned_numeric::<u32>("123", true));

        assert!(!validate_unsigned_numeric::<u32>("00", true));
        assert!(!validate_unsigned_numeric::<u32>("01", true));
        assert!(!validate_unsigned_numeric::<u32>("001", true));

        assert!(!validate_unsigned_numeric::<u32>("-", true));
        assert!(!validate_unsigned_numeric::<u32>("-1", true));

        assert!(!validate_unsigned_numeric::<u32>("1.5", true));
        assert!(!validate_unsigned_numeric::<u32>("1a", true));
    }

    #[test]
    fn test_validate_unsigned_numeric_uses_parse_semantics() {
        assert!(validate_unsigned_numeric::<u8>("255", true));
        assert!(!validate_unsigned_numeric::<u8>("256", true));
        assert!(!validate_unsigned_numeric::<u8>(
            "999999999999999999999",
            true
        ));
    }

    #[test]
    fn test_validate_without_parse() {
        assert!(validate_signed_numeric::<i32>(
            "999999999999999999999",
            false
        ));
        assert!(validate_unsigned_numeric::<u32>(
            "999999999999999999999",
            false
        ));

        assert!(!validate_signed_numeric::<i32>("00", false));
        assert!(!validate_unsigned_numeric::<u32>("00", false));
    }
}
