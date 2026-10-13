use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use quote::ToTokens as _;
use syn::{GenericArgument, PathArguments, Type};

/// Canonical Rust spelling of a type (`Vec<String>`, not the proc-macro
/// printer's `Vec < String >`). Schema export stores the result verbatim, so
/// this output is pinned; unlike [`canonical_token_string`], Joint puncts never glue.
pub fn canonical_type_string(ty: &Type) -> String {
    let mut out = String::new();
    write_canonical_tokens(&mut out, ty.to_token_stream(), false);
    out
}

/// Canonical Rust spelling of an arbitrary schema-export token stream
/// (default expressions, converter/validator paths): like
/// [`canonical_type_string`], but Joint puncts glue (`==`, `->`) and `.` hugs (`a.b`).
pub fn canonical_token_string(tokens: TokenStream) -> String {
    let mut out = String::new();
    write_canonical_tokens(&mut out, tokens, true);
    out
}

fn write_canonical_tokens(out: &mut String, tokens: TokenStream, glue_after_joint_punct: bool) {
    let mut after_joint_punct = false;
    let mut prev_was_punct = false;
    for token in tokens {
        let skip_spacing_decision = glue_after_joint_punct && after_joint_punct;
        if !skip_spacing_decision {
            let prefix_glue = !(glue_after_joint_punct && prev_was_punct);
            match &token {
                TokenTree::Group(group) if group.delimiter() != Delimiter::None => {
                    push_token_spacing(out, delimiter_open(group.delimiter()), prefix_glue);
                },
                other => {
                    if let Some(first) = token_first_char(other) {
                        push_token_spacing(out, first, prefix_glue);
                    }
                },
            }
        }
        match token {
            TokenTree::Group(group) => match group.delimiter() {
                Delimiter::None => {
                    write_canonical_tokens(out, group.stream(), glue_after_joint_punct)
                },
                delimiter => {
                    out.push(delimiter_open(delimiter));
                    write_canonical_tokens(out, group.stream(), glue_after_joint_punct);
                    push_token_spacing(out, delimiter_close(delimiter), true);
                    out.push(delimiter_close(delimiter));
                },
            },
            TokenTree::Ident(ident) => out.push_str(&ident.to_string()),
            TokenTree::Punct(punct) => {
                out.push(punct.as_char());
                // proc-macro2 marks `.` Alone before an ident, but canonical
                // Rust always hugs `a.b` / `x.0` / `a..b`.
                after_joint_punct = punct.spacing() == Spacing::Joint
                    || (glue_after_joint_punct && punct.as_char() == '.');
                prev_was_punct = true;
                continue;
            },
            TokenTree::Literal(literal) => out.push_str(&literal.to_string()),
        }
        after_joint_punct = false;
        prev_was_punct = false;
    }
}

fn delimiter_open(delimiter: Delimiter) -> char {
    match delimiter {
        Delimiter::Parenthesis => '(',
        Delimiter::Bracket => '[',
        _ => '{',
    }
}

fn delimiter_close(delimiter: Delimiter) -> char {
    match delimiter {
        Delimiter::Parenthesis => ')',
        Delimiter::Bracket => ']',
        _ => '}',
    }
}

fn token_first_char(token: &TokenTree) -> Option<char> {
    match token {
        TokenTree::Ident(ident) => ident.to_string().chars().next(),
        TokenTree::Punct(punct) => Some(punct.as_char()),
        TokenTree::Literal(literal) => literal.to_string().chars().next(),
        TokenTree::Group(_) => None,
    }
}

fn push_token_spacing(out: &mut String, next: char, prefix_glue: bool) {
    let needs_space = match (out.chars().next_back(), next) {
        (None, _) => false,
        (_, ')') | (_, ']') | (_, '>') | (_, ',') | (_, ';') | (_, '.') => false,
        (_, '(') | (_, '[') | (_, '<') => false,
        (Some('('), _) | (Some('['), _) | (Some('<'), _) => false,
        (_, ':') | (Some(':'), _) => false,
        (Some('&'), _) | (Some('*'), _) | (Some('\''), _) if prefix_glue => false,
        _ => true,
    };
    if needs_space {
        out.push(' ');
    }
}

pub fn extract_option_inner_type(ty: &Type) -> (bool, Type) {
    if let Type::Path(type_path) = ty
        && let Some(last_segment) = type_path.path.segments.last()
        && last_segment.ident == "Option"
        && let PathArguments::AngleBracketed(args) = &last_segment.arguments
        && let Some(GenericArgument::Type(inner_type)) = args.args.first()
    {
        (true, inner_type.clone())
    } else {
        (false, ty.clone())
    }
}

/// Extracts `T` from `Vec<T>`; the caller must already have stripped any
/// `Option<..>` wrapper. `None` for non-`Vec` types.
pub fn extract_vec_inner_type(ty: &Type) -> Option<Type> {
    if let Type::Path(type_path) = ty
        && let Some(last_segment) = type_path.path.segments.last()
        && last_segment.ident == "Vec"
        && let PathArguments::AngleBracketed(args) = &last_segment.arguments
        && let Some(GenericArgument::Type(inner_type)) = args.args.first()
    {
        Some(inner_type.clone())
    } else {
        None
    }
}

/// Resolves a nested form's generated value holder by the shared naming
/// convention: `some_lib::Address` -> `AddressFormValueHolder`. `None` for
/// non-path types (the caller turns that into a derive error).
pub fn nested_form_holder_type(base: &Type) -> Option<Type> {
    let Type::Path(type_path) = base else {
        return None;
    };
    let last_segment = type_path.path.segments.last()?;
    let holder_ident = quote::format_ident!("{}FormValueHolder", last_segment.ident);
    Some(syn::parse_quote!(#holder_ident))
}

#[cfg(test)]
mod tests {
    use super::{canonical_token_string, canonical_type_string};
    use quote::ToTokens as _;

    fn canonical(source: &str) -> String {
        canonical_type_string(&syn::parse_str::<syn::Type>(source).expect("test type parses"))
    }

    #[test]
    fn nested_generics_normalize_to_canonical_spelling() {
        assert_eq!(
            canonical("Vec<Option<HashMap<String, u32>>>"),
            "Vec<Option<HashMap<String, u32>>>"
        );
        assert_eq!(
            canonical("Vec < Option < HashMap < String, u32 > > >"),
            "Vec<Option<HashMap<String, u32>>>"
        );
        assert_eq!(
            canonical("Option < Vec < String > >"),
            "Option<Vec<String>>"
        );
    }

    #[test]
    fn common_type_families_keep_their_canonical_spaces() {
        assert_eq!(canonical("String"), "String");
        assert_eq!(canonical("std :: vec :: Vec < u8 >"), "std::vec::Vec<u8>");
        assert_eq!(canonical("&str"), "&str");
        assert_eq!(canonical("& 'static str"), "&'static str");
        assert_eq!(canonical("[u8 ; 4]"), "[u8; 4]");
        assert_eq!(canonical("(A , B)"), "(A, B)");
        assert_eq!(canonical("HashMap < K , V >"), "HashMap<K, V>");
        assert_eq!(canonical("fn ( u32 ) -> bool"), "fn(u32) -> bool");
        assert_eq!(canonical("* const u8"), "*const u8");
        assert_eq!(canonical("dyn A + Send"), "dyn A + Send");
        assert_eq!(canonical("< T as Trait > :: Assoc"), "<T as Trait>::Assoc");
    }

    #[test]
    fn expression_and_path_metadata_keep_their_canonical_spaces() {
        fn canonical_expr(source: &str) -> String {
            canonical_token_string(
                syn::parse_str::<syn::Expr>(source)
                    .expect("test expression parses")
                    .to_token_stream(),
            )
        }

        assert_eq!(
            canonical_expr("crate :: defaults :: country ()"),
            "crate::defaults::country()"
        );
        assert_eq!(canonical_expr("Some (3 + 4)"), "Some(3 + 4)");
        assert_eq!(canonical_expr("a == b && c != d"), "a == b && c != d");
        assert_eq!(canonical_expr("value . field"), "value.field");
        assert_eq!(canonical_expr("items . 0"), "items.0");
        assert_eq!(
            canonical_expr("crate :: defaults :: country () . to_string ()"),
            "crate::defaults::country().to_string()"
        );
        assert_eq!(canonical_expr("0 .. 9"), "0..9");
        // syn re-emits closure `||` heads with Alone spacing (`| |`), and the
        // printer cannot tell a unary prefix from a binary operator.
        assert_eq!(canonical_expr("|| -> i32 { 1 }"), "| | -> i32 { 1 }");
    }
}
