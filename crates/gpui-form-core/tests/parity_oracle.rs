use std::collections::{HashMap, HashSet};

use gpui_form_core::numeric::{validate_signed_numeric, validate_unsigned_numeric};
use gpui_form_core::{FieldPath, PathSegment};

#[test]
fn numeric_validator_accept_reject_table_is_pinned() {
    const TABLE: &[(&str, bool, bool, bool, bool)] = &[
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

#[test]
fn numeric_validator_adjacent_semantics_are_pinned() {
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

    assert!(validate_signed_numeric::<i32>("-123", true));
    assert!(!validate_signed_numeric::<i32>("--1", true));
    assert!(!validate_signed_numeric::<i32>("1-2", true));
    assert!(!validate_signed_numeric::<i32>("1.2.3", true));
    assert!(validate_signed_numeric::<f64>("1.", true));

    assert!(!validate_unsigned_numeric::<u32>("-", true));
    assert!(!validate_unsigned_numeric::<u32>("-1", true));
    assert!(!validate_unsigned_numeric::<u32>("1.5", true));
    assert!(!validate_unsigned_numeric::<u32>("1a", true));

    assert!(validate_unsigned_numeric::<u8>("255", true));
    assert!(!validate_unsigned_numeric::<u8>("256", true));
    assert!(validate_signed_numeric::<i32>(
        "999999999999999999999",
        false
    ));
    assert!(validate_unsigned_numeric::<u32>(
        "999999999999999999999",
        false
    ));
}

#[test]
fn numeric_validator_utf8_inputs_reject_like_char_scans() {
    const MULTIBYTE_INPUTS: &[&str] = &["١٢٣", "1٢", "-1٢", "é", "-é", "1.2é", "²"];

    for &input in MULTIBYTE_INPUTS {
        assert!(
            !validate_signed_numeric::<i32>(input, true),
            "signed i32(require_parse) accepted multibyte input {input:?}"
        );
        assert!(
            !validate_signed_numeric::<f64>(input, true),
            "signed f64(require_parse) accepted multibyte input {input:?}"
        );
        assert!(
            !validate_unsigned_numeric::<u32>(input, true),
            "unsigned u32(require_parse) accepted multibyte input {input:?}"
        );
        assert!(
            !validate_signed_numeric::<f64>(input, false),
            "signed f64(no parse check) accepted multibyte input {input:?}"
        );
    }

    assert!(validate_signed_numeric::<i32>("123", true));
    assert!(validate_signed_numeric::<i32>("-12", true));
    assert!(validate_signed_numeric::<f64>("1.2", true));
    assert!(validate_unsigned_numeric::<u32>("123", true));
}

#[test]
fn field_path_display_rendering_is_pinned() {
    assert_eq!(FieldPath::new(&[]).to_string(), "");
    assert_eq!(FieldPath::new(&["email"]).to_string(), "email");
    assert_eq!(
        FieldPath::new(&["address", "city"]).to_string(),
        "address.city"
    );
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
    assert_eq!(FieldPath::new(&["contacts", "2"]).to_string(), "contacts.2");
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

    assert_eq!(city, FieldPath::new(&["address", "city"]));
    assert_eq!(city.clone(), city);
    assert_eq!(FieldPath::new(&[]), FieldPath::from_segments(&[]));

    assert_ne!(city, FieldPath::new(&["address", "zip"]));
    assert_ne!(city, FieldPath::new(&["city", "address"]));
    assert_ne!(city, FieldPath::new(&["address"]));

    assert_ne!(
        FieldPath::new(&["tags"]).push_index(2),
        FieldPath::new(&["tags", "2"])
    );

    assert_eq!(
        FieldPath::new(&["a", "b"]),
        FieldPath::from_segments(&[PathSegment::Name("a"), PathSegment::Name("b")])
    );
    assert_eq!(
        FieldPath::new(&["contacts"])
            .push_index(1)
            .push_name("email"),
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
        FieldPath::new(&["contacts"])
            .push_index(2)
            .push_name("name")
    );

    assert_eq!(PathSegment::Name("tags"), PathSegment::Name("tags"));
    assert_eq!(PathSegment::Index(3), PathSegment::Index(3));
    assert_ne!(PathSegment::Name("3"), PathSegment::Index(3));
    assert_ne!(PathSegment::Index(3), PathSegment::Index(4));
}

#[test]
fn field_path_hash_contract_is_pinned() {
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
