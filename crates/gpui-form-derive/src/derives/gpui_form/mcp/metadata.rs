use super::*;

pub(super) fn tool_metadata_tokens(
    facade_crate: &Path,
    original_input: &DeriveInput,
    options: Option<&McpToolOptions>,
) -> syn::Result<TokenStream> {
    let mcp_crate: Path = syn::parse_quote!(#facade_crate::mcp);
    let span = original_input.ident.span();
    if options.and_then(|options| options.read_only) == Some(true)
        && options.and_then(|options| options.destructive) == Some(true)
    {
        return Err(syn::Error::new(
            span,
            "MCP tool annotation hints cannot be both read-only and destructive",
        ));
    }

    if let Some(name) = options.and_then(|options| options.name.as_deref()) {
        component_shape::validate_mcp_tool_name(name)
            .map_err(|error| syn::Error::new(span, error.to_string()))?;
    }
    if let Some(title) = options.and_then(|options| options.title.as_deref()) {
        component_shape::validate_mcp_tool_metadata_text("title", title)
            .map_err(|error| syn::Error::new(span, error.to_string()))?;
    }

    let description = options
        .and_then(|options| options.description.as_deref())
        .map(str::to_string)
        .or_else(|| doc_description(&original_input.attrs));
    if let Some(description) = description.as_deref() {
        component_shape::validate_mcp_tool_metadata_text("description", description)
            .map_err(|error| syn::Error::new(span, error.to_string()))?;
    }

    let mut tokens = quote! {
        #mcp_crate::McpToolMetadata::new()
    };

    if let Some(name) = options.and_then(|options| options.name.as_deref()) {
        let name = LitStr::new(name, span);
        tokens = quote! { #tokens.with_name(#name) };
    }
    if let Some(title) = options.and_then(|options| options.title.as_deref()) {
        let title = LitStr::new(title, span);
        tokens = quote! { #tokens.with_title(#title) };
    }
    if let Some(description) = description {
        let description = LitStr::new(&description, span);
        tokens = quote! { #tokens.with_description(#description) };
    }
    if let Some(read_only) = options.and_then(|options| options.read_only) {
        tokens = quote! { #tokens.with_read_only_hint(#read_only) };
    }
    if let Some(destructive) = options.and_then(|options| options.destructive) {
        tokens = quote! { #tokens.with_destructive_hint(#destructive) };
    }
    if let Some(idempotent) = options.and_then(|options| options.idempotent) {
        tokens = quote! { #tokens.with_idempotent_hint(#idempotent) };
    }
    if let Some(open_world) = options.and_then(|options| options.open_world) {
        tokens = quote! { #tokens.with_open_world_hint(#open_world) };
    }

    if let Some(options) = options
        && !options.icons.is_empty()
    {
        let icons = options
            .icons
            .iter()
            .map(|icon| icon_tokens(&mcp_crate, icon, span))
            .collect::<Vec<_>>();
        tokens = quote! {{
            const MCP_TOOL_ICONS: &[#mcp_crate::McpToolIcon] = &[#(#icons),*];
            (#tokens).with_icons(MCP_TOOL_ICONS)
        }};
    }

    Ok(tokens)
}

fn icon_tokens(
    mcp_crate: &Path,
    icon: &McpIconOptions,
    span: Span,
) -> TokenStream {
    let src = LitStr::new(&icon.src, span);
    let mut tokens = quote! { #mcp_crate::McpToolIcon::new(#src) };
    if let Some(mime_type) = icon.mime_type.as_deref() {
        let mime_type = LitStr::new(mime_type, span);
        tokens = quote! { #tokens.with_mime_type(#mime_type) };
    }
    if !icon.sizes.is_empty() {
        let sizes = icon
            .sizes
            .iter()
            .map(|size| LitStr::new(size, span))
            .collect::<Vec<_>>();
        tokens = quote! { #tokens.with_sizes(&[#(#sizes),*]) };
    }
    if let Some(theme) = icon.theme {
        let theme = match theme {
            McpIconThemeOption::Light => quote! { #mcp_crate::McpIconTheme::Light },
            McpIconThemeOption::Dark => quote! { #mcp_crate::McpIconTheme::Dark },
        };
        tokens = quote! { #tokens.with_theme(#theme) };
    }
    tokens
}

fn doc_description(attrs: &[syn::Attribute]) -> Option<String> {
    let mut lines = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| match &attr.meta {
            syn::Meta::NameValue(meta) => match &meta.value {
                syn::Expr::Lit(expr) => match &expr.lit {
                    syn::Lit::Str(value) => Some(value.value().trim().to_string()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();

    let first = lines.iter().position(|line| !line.is_empty())?;
    let last = lines.iter().rposition(|line| !line.is_empty())?;
    lines.drain(..first);
    lines.truncate(last - first + 1);

    Some(lines.join("\n"))
}
