use proc_macro2::{Delimiter, TokenStream, TokenTree};
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
pub fn canonical_type_string(ty: &Type) -> String {
    let mut out = String::new();
    write_type_tokens(&mut out, ty.to_token_stream());
    out
}

fn write_type_tokens(out: &mut String, tokens: TokenStream) {
    for token in tokens {
        match token {
            TokenTree::Group(group) => match group.delimiter() {
                // Invisible (precedence) groups render inline.
                Delimiter::None => write_type_tokens(out, group.stream()),
                delimiter => {
                    let (open, close) = match delimiter {
                        Delimiter::Parenthesis => ('(', ')'),
                        Delimiter::Bracket => ('[', ']'),
                        _ => ('{', '}'),
                    };
                    push_type_spacing(out, open);
                    out.push(open);
                    write_type_tokens(out, group.stream());
                    push_type_spacing(out, close);
                    out.push(close);
                },
            },
            TokenTree::Ident(ident) => {
                let text = ident.to_string();
                if let Some(first) = text.chars().next() {
                    push_type_spacing(out, first);
                }
                out.push_str(&text);
            },
            TokenTree::Punct(punct) => {
                let ch = punct.as_char();
                push_type_spacing(out, ch);
                out.push(ch);
            },
            TokenTree::Literal(literal) => {
                let text = literal.to_string();
                if let Some(first) = text.chars().next() {
                    push_type_spacing(out, first);
                }
                out.push_str(&text);
            },
        }
    }
}

/// Whether a single space separates the already-written output from the next
/// token character, per canonical type spelling.
fn push_type_spacing(out: &mut String, next: char) {
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
        // (`&str`, `&'a str`, `*const T`).
        (Some('&'), _) | (Some('*'), _) | (Some('\''), _) => false,
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
    use super::canonical_type_string;

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
}
