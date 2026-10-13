use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use quote::ToTokens as _;
use syn::{GenericArgument, PathArguments, Type};

/// Renders a type in canonical Rust spelling: `Vec<String>`, not the
/// proc-macro token spacing `Vec < String >`.
///
/// `TokenStream::to_string()` inserts a space between every token pair, and
/// the schema export/inventory metadata store those strings verbatim — so
/// before this helper, `#[gpui_form(component(list))] tags: Vec<String>`
/// exported `"value_type": "Vec < String >"` and JSON fixtures had to pin
/// the spacing. The printer walks the type's token trees and re-glues
/// punctuation (`<>`, `::`, `,`, `;`, `&`, `*`, lifetimes, delimiters) the
/// way rustc/rustfmt spell them, keeping single spaces where the canonical
/// form has them (`HashMap<K, V>`, `[u8; 4]`, `fn() -> T`, `*const T`).
///
/// This is the ORIGINAL spacing-table printer, byte for byte: the spacing
/// decision runs for every token regardless of `Spacing::Joint`
/// (`glue_after_joint_punct = false` below), so type output cannot shift
/// with [`canonical_token_string`]'s Joint-punct extension.
pub fn canonical_type_string(ty: &Type) -> String {
    let mut out = String::new();
    write_canonical_tokens(&mut out, ty.to_token_stream(), false);
    out
}

/// Renders an arbitrary token stream (expression or path metadata) in
/// canonical Rust spelling — the type printer plus expression extensions.
///
/// The schema export stores several non-type strings verbatim
/// (`default_expr`, `from_expr`/`into_expr`, the async-validator path), and
/// raw `to_token_stream().to_string()` would export them as
/// `"crate :: defaults :: country ()"`. Unlike a type, an expression can
/// carry constructs the spacing table cannot say, so this path adds two
/// rules:
///
/// - a Joint punct glues the token that follows it (`==`, `!=`, `&&`,
///   `::`, `->` — multi-character operators stay one unit), and
/// - the token after `.` glues too (`a.b`, `x.0`, `a..b`): proc-macro2
///   marks `.` Alone before an ident, but canonical Rust always hugs it.
///
/// Residual limits, documented rather than hidden: token streams carry no
/// adjacency to non-punct tokens, so a unary `&`/`*`/closure-`|` prefix or
/// a postfix `?` following another token (`-> &b`, `|x| x`, `x?`) keeps one
/// token space — and syn's AST re-emission gives closure `||` heads Alone
/// spacing outright, so they render `| |` even though the same expression's
/// `->` stays glued (`Token![->]` emits its `-` Joint). These are exactly
/// the spots where rustfmt would need the parse tree to tell a unary prefix
/// from a binary operator. [`canonical_type_string`] deliberately does
/// NONE of this: type output is a pinned surface and keeps the pure
/// spacing-table behavior.
pub fn canonical_token_string(tokens: TokenStream) -> String {
    let mut out = String::new();
    write_canonical_tokens(&mut out, tokens, true);
    out
}

fn write_canonical_tokens(out: &mut String, tokens: TokenStream, glue_after_joint_punct: bool) {
    // With `glue_after_joint_punct`, a Joint punct glues the token that
    // follows it, so that next token skips the generic spacing decision
    // entirely (multi-character operators stay one unit), and so does `.`:
    // field/method access and ranges are always hugged in canonical Rust,
    // even though proc-macro2 marks `.` Alone before an ident. In that same
    // expression path, the token AFTER an operator run's trailing punct
    // keeps its normal space (`&& c`, not `&&c`): the table's
    // reference/pointer/lifetime prefix rules are disabled there — they only
    // make sense at the start of a unary prefix, never directly after
    // another punct.
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
                // Invisible (precedence) groups render inline.
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
                // `.` glues in the expression path even though it is Alone
                // before an ident: `a.b` / `x.0` / `a..b` are always hugged.
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

/// The first rendered character of a non-group token, for the spacing
/// decision against the already-written output.
fn token_first_char(token: &TokenTree) -> Option<char> {
    match token {
        TokenTree::Ident(ident) => ident.to_string().chars().next(),
        TokenTree::Punct(punct) => Some(punct.as_char()),
        TokenTree::Literal(literal) => literal.to_string().chars().next(),
        TokenTree::Group(_) => None,
    }
}

/// Whether a single space separates the already-written output from the next
/// token character, per canonical Rust spelling (types and metadata
/// expressions share the rules). With `prefix_glue = false` — the expression
/// path right after an operator punct — the reference/pointer/lifetime
/// prefix rules are disabled, so an operator's trailing punct keeps the
/// space before the next token (`&& c`, not `&&c`).
fn push_token_spacing(out: &mut String, next: char, prefix_glue: bool) {
    let needs_space = match (out.chars().next_back(), next) {
        // Start of the rendered type: nothing to separate from.
        (None, _) => false,
        // Close/separator punctuation hugs the preceding token
        // (`Vec<T>`, `>>`, `[u8; 4]`).
        (_, ')') | (_, ']') | (_, '>') | (_, ',') | (_, ';') | (_, '.') => false,
        // Opening delimiters and generic opens hug the following token, on
        // both sides (`Fn(A)`, `(A, B)`, `Vec<T>`, `Option<[u8; 4]>`).
        (_, '(') | (_, '[') | (_, '<') => false,
        (Some('('), _) | (Some('['), _) | (Some('<'), _) => false,
        // `::` stays glued on both sides (`std::vec::Vec`).
        (_, ':') | (Some(':'), _) => false,
        // Reference/pointer/lifetime prefixes hug the following token
        // (`&str`, `&'a str`, `*const T`) — but only at the start of a
        // prefix, never as the trailing half of an operator run.
        (Some('&'), _) | (Some('*'), _) | (Some('\''), _) if prefix_glue => false,
        // Everything else — ident boundaries, `+`, `=`, tokens after `,`
        // (`HashMap<K, V>`, `dyn A + Send`, `Item = u32`) — takes a space.
        _ => true,
    };
    if needs_space {
        out.push(' ');
    }
}

/// Checks if a type is Option<T> and returns (is_option, inner_type).
/// If not an Option, returns the original type as inner_type.
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

/// Extracts `T` from `Vec<T>` (after any `Option<..>` wrapper has been
/// removed by the caller). Returns `None` for non-`Vec` types.
///
/// Milestone M3 (backlog #3): `component(list(item = ...))` fields must be
/// `Vec<T>` (or `Option<Vec<T>>`); the expansion errors early otherwise.
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

/// Resolves the generated value-holder type of a nested form from its base
/// type, by the shared naming convention: `Address` (or
/// `some_lib::Address`) -> `AddressFormValueHolder` as a single-segment
/// path. The ident must resolve in the module where the derive runs — the
/// same convention prototyping relies on for its scaffold imports.
///
/// Returns `None` when the base type is not a simple path (tuples, arrays,
/// references), which the caller turns into a derive error.
pub fn nested_form_holder_type(base: &Type) -> Option<Type> {
    let Type::Path(type_path) = base else {
        return None;
    };
    // Generic nested forms (`Address<T>`) resolve to a bare holder ident that
    // cannot carry the generics — unsupported in v1, surfaced by the emitted
    // code failing to name-resolve (hazard 10's documented failure mode).
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
        // The exact shape the proc-macro printer mangles: every token pair
        // gains a space (`Vec < Option < HashMap < String, u32 > > >`).
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
        // The non-type schema strings (defaults, conversions, async
        // validator paths) run through the same printer; joint puncts keep
        // multi-character operators one unit instead of spacing them apart.
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
        // Field/method access and ranges: `.` is Alone before an ident
        // (proc-macro2 marks Joint only punct-after-punct), but canonical
        // Rust always hugs it.
        assert_eq!(canonical_expr("value . field"), "value.field");
        assert_eq!(canonical_expr("items . 0"), "items.0");
        assert_eq!(
            canonical_expr("crate :: defaults :: country () . to_string ()"),
            "crate::defaults::country().to_string()"
        );
        assert_eq!(canonical_expr("0 .. 9"), "0..9");
        // `->` is not a standalone expression, so a closure carries the
        // joint run. The run glues (syn's Token![->] emits its `-` Joint),
        // but the closure head does NOT: syn re-emits the two pipes with
        // Alone spacing, so each keeps one token space — the documented
        // closure-head residual.
        assert_eq!(canonical_expr("|| -> i32 { 1 }"), "| | -> i32 { 1 }");
    }
}
