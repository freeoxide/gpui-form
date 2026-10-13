/// Validates numeric input for signed types (i*, f*) with custom type support.
///
/// This validation allows:
/// - Empty strings
/// - Just "-" for intermediate input
/// - Valid signed numbers with optional decimal points
///
/// This validation rejects:
/// - Leading zeros before digits (e.g., "00", "01", "-00", "-01")
/// - Multiple "-" signs
/// - "-" anywhere except the first character
///
/// When `require_parse` is true, also validates that the value can be parsed
/// to the target type T.
///
/// The scan is allocation-free: it runs over the input's UTF-8 bytes rather
/// than a collected `Vec<char>`. Byte and char scans accept/reject
/// identically here because no byte of a multi-byte UTF-8 character is an
/// ASCII digit, `.`, or `-`, so a non-ASCII character rejects at its first
/// byte exactly as it would at its `char` boundary.
pub fn validate_signed_numeric<T: std::str::FromStr>(value: &str, require_parse: bool) -> bool {
    if value.is_empty() {
        return true;
    }

    let bytes = value.as_bytes();

    // Allow just "-" for intermediate input
    if bytes.len() == 1 && bytes[0] == b'-' {
        return true;
    }

    // First character: must be 0-9 or '-'
    if !bytes[0].is_ascii_digit() && bytes[0] != b'-' {
        return false;
    }

    // If there are more characters, validate from second character onwards
    if bytes.len() > 1 {
        let rest = if bytes[0] == b'-' { &bytes[1..] } else { bytes };

        // Check for invalid leading zeros: "0X" where X is a digit (not decimal point)
        // Allow: "0", "0.", "0.5", "-0", "-0.", "-0.5"
        // Reject: "00", "01", "-00", "-01"
        if rest[0] == b'0' && rest.len() > 1 && rest[1].is_ascii_digit() {
            return false;
        }

        // Second character onwards: can't be '-'
        if !rest.iter().all(|&b| b.is_ascii_digit() || b == b'.') {
            return false;
        }
    }

    // If required, check if it can parse
    if require_parse {
        value.parse::<T>().is_ok()
    } else {
        true
    }
}

/// Validates numeric input for unsigned types (u*) with custom type support.
///
/// This validation allows:
/// - Empty strings
/// - Valid unsigned numbers (digits only)
///
/// This validation rejects:
/// - Leading zeros before digits (e.g., "00", "01", "001")
/// - Any non-digit characters (including "-")
///
/// When `require_parse` is true, also validates that the value can be parsed
/// to the target type T.
///
/// Like [`validate_signed_numeric`], the scan is allocation-free over the
/// input's UTF-8 bytes; byte and char scans accept/reject identically
/// because no byte of a multi-byte UTF-8 character is an ASCII digit.
pub fn validate_unsigned_numeric<T: std::str::FromStr>(value: &str, require_parse: bool) -> bool {
    if value.is_empty() {
        return true;
    }

    let bytes = value.as_bytes();

    // For unsigned types, first character must be 0-9
    if !bytes[0].is_ascii_digit() {
        return false;
    }

    // Check for invalid leading zeros: "0X" where X is a digit
    // Allow: "0"
    // Reject: "00", "01", "001"
    if bytes[0] == b'0' && bytes.len() > 1 && bytes[1].is_ascii_digit() {
        return false;
    }

    // All characters must be digits
    if !bytes.iter().all(|&b| b.is_ascii_digit()) {
        return false;
    }

    // If required, check if it can parse
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
        // Valid inputs
        assert!(validate_signed_numeric::<i32>("", true));
        assert!(validate_signed_numeric::<i32>("-", true));
        assert!(validate_signed_numeric::<i32>("0", true));
        assert!(validate_signed_numeric::<i32>("-0", true));
        assert!(validate_signed_numeric::<i32>("123", true));
        assert!(validate_signed_numeric::<i32>("-123", true));

        // Valid floats
        assert!(validate_signed_numeric::<f64>("0.5", true));
        assert!(validate_signed_numeric::<f64>("-0.5", true));
        assert!(validate_signed_numeric::<f64>("123.456", true));

        // Invalid: leading zeros
        assert!(!validate_signed_numeric::<i32>("00", true));
        assert!(!validate_signed_numeric::<i32>("01", true));
        assert!(!validate_signed_numeric::<i32>("-00", true));
        assert!(!validate_signed_numeric::<i32>("-01", true));

        // Invalid: multiple minus signs or in wrong position
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
        // Valid inputs
        assert!(validate_unsigned_numeric::<u32>("", true));
        assert!(validate_unsigned_numeric::<u32>("0", true));
        assert!(validate_unsigned_numeric::<u32>("123", true));

        // Invalid: leading zeros
        assert!(!validate_unsigned_numeric::<u32>("00", true));
        assert!(!validate_unsigned_numeric::<u32>("01", true));
        assert!(!validate_unsigned_numeric::<u32>("001", true));

        // Invalid: minus sign
        assert!(!validate_unsigned_numeric::<u32>("-", true));
        assert!(!validate_unsigned_numeric::<u32>("-1", true));

        // Invalid: non-digits
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
        // Custom types without parse check
        assert!(validate_signed_numeric::<i32>(
            "999999999999999999999",
            false
        ));
        assert!(validate_unsigned_numeric::<u32>(
            "999999999999999999999",
            false
        ));

        // Should still reject invalid patterns
        assert!(!validate_signed_numeric::<i32>("00", false));
        assert!(!validate_unsigned_numeric::<u32>("00", false));
    }
}
