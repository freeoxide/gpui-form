use syn::{GenericArgument, PathArguments, Type};

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
