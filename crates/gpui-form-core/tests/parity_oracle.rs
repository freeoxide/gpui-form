//! Parity oracle (campaign round 0): characterization tests pinning the
//! CURRENT behavior of `gpui-form-core`'s numeric validators and typed field
//! paths.
//!
//! These tests exist to make every later optimization a visible diff: they
//! must keep passing UNCHANGED across the whole campaign. When an
//! optimization legitimately changes behavior, the change must be a
//! deliberate, reviewed oracle update — never a silent drift.
//!
//! Every expected value below was pinned against the pre-optimization tree
//! (accept/reject traced through `numeric.rs` and confirmed against its
//! existing unit tests).

use std::collections::{HashMap, HashSet};

use gpui_form_core::numeric::{validate_signed_numeric, validate_unsigned_numeric};
use gpui_form_core::{FieldPath, PathSegment};

// ── Numeric validator accept/reject table ──────────────────────────────────

/// The campaign's canonical keystroke set: `''`, `'-'`, `'0.'`, `'1e5'`,
/// `'+3'`, `'12a'` — the intermediate-editing and malformed shapes users
/// type, where the CURRENT contract is:
///
/// - `""` always accepted (empty input is "no value yet"),
/// - `"-"` accepted for signed types (minus-sign intermediate state),
/// - `"0."` shape-valid (trailing dot allowed) but only parseable by floats,
/// - `'e'`/`'+'`/letters rejected by SHAPE even when the target type would
///   parse the string (`1e5` parses as f64; the validator still rejects it).
#[test]
fn numeric_validator_accept_reject_table_is_pinned() {
    // (input, signed i32 require_parse, signed f64 require_parse,
    //  unsigned u32 require_parse, signed f64 no parse check)
    const TABLE: &[(&str, bool, bool, bool, bool)] = &[
        // input, i32+parse, f64+parse, u32+parse, f64-no-parse
        ("", true, true, true, true),
        ("-", true, true, false, true),
        ("0.", false, true, false, true),
        ("1e5", false, false, false, false),
        ("+3", false, false, false, false),
        ("12a", false, false, false, false),
    ];

    for &(input, want_i32, want_f64, want_u32, want_f64_no_parse) in TABLE {
        assert_eq!(
            validate_signed_numeric::<i32>(input, true),
            want_i32,
            "signed i32(require_parse) changed for input {input:?}"
        );
        assert_eq!(
            validate_signed_numeric::<f64>(input, true),
            want_f64,
            "signed f64(require_parse) changed for input {input:?}"
        );
        assert_eq!(
            validate_unsigned_numeric::<u32>(input, true),
            want_u32,
            "unsigned u32(require_parse) changed for input {input:?}"
        );
        assert_eq!(
            validate_signed_numeric::<f64>(input, false),
            want_f64_no_parse,
            "signed f64(no parse check) changed for input {input:?}"
        );
    }
}

/// Adjacent semantics the table rides on, pinned so an optimization cannot
/// shift them while keeping the six canonical rows green: leading-zero
/// rejection, sign placement, and integer/float parse boundaries.
#[test]
fn numeric_validator_adjacent_semantics_are_pinned() {
    // Leading zeros before a digit are rejected; a lone or dot-following
    // zero is fine.
    assert!(validate_signed_numeric::<i32>("0", true));
    assert!(validate_signed_numeric::<i32>("-0", true));
    assert!(validate_signed_numeric::<f64>("0.5", true));
    assert!(validate_signed_numeric::<f64>("-0.5", true));
    assert!(!validate_signed_numeric::<i32>("00", true));
    assert!(!validate_signed_numeric::<i32>("01", true));
    assert!(!validate_signed_numeric::<i32>("-00", true));
    assert!(!validate_signed_numeric::<i32>("-01", true));
    assert!(!validate_unsigned_numeric::<u32>("00", true));
    assert!(!validate_unsigned_numeric::<u32>("001", true));

    // Only ONE leading minus; never anywhere else.
    assert!(validate_signed_numeric::<i32>("-123", true));
    assert!(!validate_signed_numeric::<i32>("--1", true));
    assert!(!validate_signed_numeric::<i32>("1-2", true));
    assert!(!validate_signed_numeric::<i32>("1.2.3", true));
    assert!(validate_signed_numeric::<f64>("1.", true));

    // Unsigned rejects signs and separators outright.
    assert!(!validate_unsigned_numeric::<u32>("-", true));
    assert!(!validate_unsigned_numeric::<u32>("-1", true));
    assert!(!validate_unsigned_numeric::<u32>("1.5", true));
    assert!(!validate_unsigned_numeric::<u32>("1a", true));

    // Parse semantics reach the target type: u8 range is enforced when
    // require_parse is on, and skipped when it is off.
    assert!(validate_unsigned_numeric::<u8>("255", true));
    assert!(!validate_unsigned_numeric::<u8>("256", true));
    assert!(validate_signed_numeric::<i32>("999999999999999999999", false));
    assert!(validate_unsigned_numeric::<u32>("999999999999999999999", false));
}

// ── FieldPath Display / equality ───────────────────────────────────────────

#[test]
fn field_path_display_rendering_is_pinned() {
    // Empty path renders as the empty string.
    assert_eq!(FieldPath::new(&[]).to_string(), "");
    // Flat name-only paths.
    assert_eq!(FieldPath::new(&["email"]).to_string(), "email");
    assert_eq!(
        FieldPath::new(&["address", "city"]).to_string(),
        "address.city"
    );
    // Typed index chains render as dotted numbers.
    assert_eq!(
        FieldPath::new(&["contacts"])
            .push_index(2)
            .push_name("name")
            .to_string(),
        "contacts.2.name"
    );
    assert_eq!(
        FieldPath::from_segments(&[PathSegment::Name("tags"), PathSegment::Index(4)]).to_string(),
        "tags.4"
    );
    // A literal name "2" renders IDENTICALLY to an index 2 segment even
    // though the two paths never compare equal (see the equality oracle):
    // Display is lossy about segment kind. Pinned as current behavior.
    assert_eq!(FieldPath::new(&["contacts", "2"]).to_string(), "contacts.2");
    // Join composes with dots in segment order.
    assert_eq!(
        FieldPath::new(&["address"])
            .join(&FieldPath::new(&["city", "zip"]))
            .to_string(),
        "address.city.zip"
    );
}

#[test]
fn field_path_equality_is_pinned() {
    let city = FieldPath::new(&["address", "city"]);

    // Equality is by segment sequence: same segments, same path.
    assert_eq!(city, FieldPath::new(&["address", "city"]));
    assert_eq!(city.clone(), city);
    assert_eq!(FieldPath::new(&[]), FieldPath::from_segments(&[]));

    // Different names, different order, different path.
    assert_ne!(city, FieldPath::new(&["address", "zip"]));
    assert_ne!(city, FieldPath::new(&["city", "address"]));
    assert_ne!(city, FieldPath::new(&["address"]));

    // An Index segment never equals a Name segment, even with equal Display.
    assert_ne!(
        FieldPath::new(&["tags"]).push_index(2),
        FieldPath::new(&["tags", "2"])
    );

    // Distinct construction routes converge on equal paths.
    assert_eq!(
        FieldPath::new(&["a", "b"]),
        FieldPath::from_segments(&[PathSegment::Name("a"), PathSegment::Name("b")])
    );
    assert_eq!(
        FieldPath::new(&["contacts"]).push_index(1).push_name("email"),
        FieldPath::from_segments(&[
            PathSegment::Name("contacts"),
            PathSegment::Index(1),
            PathSegment::Name("email"),
        ])
    );
    assert_eq!(
        FieldPath::new(&["contacts"])
            .push_index(2)
            .join(&FieldPath::new(&["name"])),
        FieldPath::new(&["contacts"]).push_index(2).push_name("name")
    );

    // Segment-level equality underlies path equality.
    assert_eq!(PathSegment::Name("tags"), PathSegment::Name("tags"));
    assert_eq!(PathSegment::Index(3), PathSegment::Index(3));
    assert_ne!(PathSegment::Name("3"), PathSegment::Index(3));
    assert_ne!(PathSegment::Index(3), PathSegment::Index(4));
}

#[test]
fn field_path_hash_contract_is_pinned() {
    // Equal paths must hash equal (the HashSet/HashMap contract the derive's
    // error maps and touched-field tracking rely on).
    let mut set = HashSet::new();
    set.insert(FieldPath::new(&["address", "city"]));
    assert!(set.contains(&FieldPath::new(&["address", "city"])));
    assert!(!set.contains(&FieldPath::new(&["address", "zip"])));
    assert!(!set.contains(&FieldPath::new(&["tags"]).push_index(2)));

    let mut map = HashMap::new();
    map.insert(FieldPath::new(&["contacts"]).push_index(2), "second");
    assert_eq!(
        map.get(&FieldPath::new(&["contacts"]).push_index(2)),
        Some(&"second")
    );
    assert_eq!(
        map.get(&FieldPath::new(&["contacts", "2"])),
        None,
        "a Name(\"2\") path must not hit an Index(2) key"
    );
}
