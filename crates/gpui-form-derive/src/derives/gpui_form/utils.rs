use quote::ToTokens as _;
use syn::Ident;
use syn::{GenericArgument, PathArguments, Type};

pub fn extract_type_ident(ty: &Type) -> Ident {
    match ty {
        Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last().unwrap_or_else(|| {
                panic!(
                    "Expected at least one segment in type path: {:?}",
                    type_path.to_token_stream()
                )
            });

            if last_segment.ident == "Option"
                && let PathArguments::AngleBracketed(args) = &last_segment.arguments
                && let Some(GenericArgument::Type(inner_type)) = args.args.first()
            {
                return extract_type_ident(inner_type);
            }
            last_segment.ident.clone()
        },
        _ => panic!(
            "Unsupported type for component field: not a Type::Path. Got: {:?}",
            ty.to_token_stream()
        ),
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
