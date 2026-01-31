use koruma_derive_core::ValidatorAttr;
use proc_macro2::TokenStream;
use quote::quote;

/// Convert a ValidatorAttr to a TokenStream for generating koruma attribute.
/// This produces tokens like `ValidatorPath::<Type>(arg1 = val1, arg2 = val2)`.
pub fn validator_attr_to_tokens(validator: &ValidatorAttr) -> TokenStream {
    let path = &validator.validator;

    let type_params = if validator.infer_type {
        quote! { ::<_> }
    } else if let Some(explicit_ty) = &validator.explicit_type {
        quote! { ::<#explicit_ty> }
    } else {
        quote! {}
    };

    let args = if validator.args.is_empty() {
        quote! {}
    } else {
        let arg_tokens: Vec<_> = validator
            .args
            .iter()
            .map(|(name, expr)| quote! { #name = #expr })
            .collect();
        quote! { (#(#arg_tokens),*) }
    };

    quote! { #path #type_params #args }
}
