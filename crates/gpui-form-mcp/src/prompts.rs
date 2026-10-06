use std::{any::TypeId, collections::BTreeSet};

use serde_json::{Map, Value};

use crate::*;

/// MCP prompt template names generated for one form descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpFormPromptNames {
    /// Prompt for drafting a complete direct-submit argument object.
    pub fill: String,
    /// Prompt for repairing invalid arguments or an edit-session patch.
    pub repair: String,
    /// Prompt for choosing direct submit versus edit-session submit.
    pub submit: String,
}

impl McpFormPromptNames {
    /// Return all generated prompt names in fill, repair, submit order.
    pub fn all(&self) -> [&str; 3] {
        [
            self.fill.as_str(),
            self.repair.as_str(),
            self.submit.as_str(),
        ]
    }
}

#[derive(Clone, Copy)]
pub(crate) enum McpFormPromptKind {
    Fill,
    Repair,
    Submit,
}

pub(crate) struct McpFormPromptSpec {
    name: String,
    definition: PromptDefinition,
    descriptor: McpFormDescriptor,
    kind: McpFormPromptKind,
}

/// Return generated MCP prompt template names for `descriptor`.
pub fn form_prompt_names(descriptor: McpFormDescriptor) -> McpFormPromptNames {
    let tool_name = descriptor.tool_name();
    McpFormPromptNames {
        fill: format!("fill_{tool_name}_form"),
        repair: format!("repair_{tool_name}_form"),
        submit: format!("submit_{tool_name}_form"),
    }
}

/// Build the JSON value served by a form's descriptor resource.
///
/// The descriptor includes resource links, form/tool metadata, field labels,
/// descriptions, examples, required/default/storage metadata, component MCP
/// Register generated fill, repair, and submit prompt templates for one form.
///
/// Prompt templates are opt-in and reference the generated descriptor, schema,
/// and examples resources. This helper registers those resources first if they
/// are not already present.
pub fn register_form_prompt_templates<Form>(server: &mut McpServer) -> Result<(), McpToolError>
where
    Form: McpForm,
{
    register_form_prompt_templates_for_descriptor(server, Form::descriptor())
}

pub(crate) fn register_form_prompt_templates_for_descriptor(
    server: &mut McpServer,
    descriptor: McpFormDescriptor,
) -> Result<(), McpToolError> {
    let resource_specs = form_resource_specs(descriptor)?;
    let prompt_specs = form_prompt_specs(descriptor)?;
    register_form_resource_specs_if_missing(server, resource_specs)?;
    ensure_prompt_specs_available(server, &prompt_specs)?;
    register_form_prompt_specs(server, prompt_specs)
}

pub(crate) fn form_prompt_specs(
    descriptor: McpFormDescriptor,
) -> Result<Vec<McpFormPromptSpec>, McpToolError> {
    let names = form_prompt_names(descriptor);
    let title = descriptor.title();
    Ok(vec![
        form_prompt_spec(
            names.fill,
            rust_i18n::t!("prompts.fill_title", title = title).to_string(),
            rust_i18n::t!("prompts.fill_description", title = title).to_string(),
            vec![optional_prompt_argument(
                "goal",
                rust_i18n::t!("prompts.arg_goal_description").as_ref(),
            )],
            descriptor,
            McpFormPromptKind::Fill,
        )?,
        form_prompt_spec(
            names.repair,
            rust_i18n::t!("prompts.repair_title", title = title).to_string(),
            rust_i18n::t!("prompts.repair_description", title = title).to_string(),
            vec![
                optional_prompt_argument(
                    "validation_issues",
                    rust_i18n::t!("prompts.arg_validation_issues_description").as_ref(),
                ),
                optional_prompt_argument(
                    "current_values",
                    rust_i18n::t!("prompts.arg_current_values_description").as_ref(),
                ),
            ],
            descriptor,
            McpFormPromptKind::Repair,
        )?,
        form_prompt_spec(
            names.submit,
            rust_i18n::t!("prompts.submit_title", title = title).to_string(),
            rust_i18n::t!("prompts.submit_description", title = title).to_string(),
            vec![optional_prompt_argument(
                "workflow",
                rust_i18n::t!("prompts.arg_workflow_description").as_ref(),
            )],
            descriptor,
            McpFormPromptKind::Submit,
        )?,
    ])
}

pub(crate) fn form_prompt_spec(
    name: String,
    title: String,
    description: String,
    arguments: Vec<McpPromptArgument>,
    descriptor: McpFormDescriptor,
    kind: McpFormPromptKind,
) -> Result<McpFormPromptSpec, McpToolError> {
    let definition = component_shape_mcp::prompt_definition(
        name.clone(),
        Some(title),
        Some(description),
        Some(arguments),
    )?;
    Ok(McpFormPromptSpec {
        name,
        definition,
        descriptor,
        kind,
    })
}

pub(crate) fn optional_prompt_argument(name: &'static str, description: &str) -> McpPromptArgument {
    McpPromptArgument::new(name)
        .with_description(description)
        .with_required(false)
}

pub(crate) fn register_form_prompt_specs(
    server: &mut McpServer,
    specs: Vec<McpFormPromptSpec>,
) -> Result<(), McpToolError> {
    for spec in specs {
        let descriptor = spec.descriptor;
        let kind = spec.kind;
        server.add_prompt(spec.definition, move |arguments| {
            form_prompt_result(descriptor, kind, arguments)
        })?;
    }
    Ok(())
}

pub(crate) fn ensure_prompt_specs_available(
    server: &McpServer,
    specs: &[McpFormPromptSpec],
) -> Result<(), McpToolError> {
    for spec in specs {
        if server.contains_prompt(&spec.name) {
            return Err(McpToolError::duplicate_prompt(spec.name.clone()));
        }
    }
    Ok(())
}

pub(crate) fn form_prompt_result(
    descriptor: McpFormDescriptor,
    kind: McpFormPromptKind,
    arguments: Option<Map<String, Value>>,
) -> McpPromptResult {
    let title = descriptor.title();
    let description = match kind {
        McpFormPromptKind::Fill => {
            rust_i18n::t!("prompts.fill_result_description", title = title).to_string()
        },
        McpFormPromptKind::Repair => {
            rust_i18n::t!("prompts.repair_result_description", title = title).to_string()
        },
        McpFormPromptKind::Submit => {
            rust_i18n::t!("prompts.submit_result_description", title = title).to_string()
        },
    };
    component_shape_mcp::text_prompt_result(
        Some(description),
        form_prompt_text(descriptor, kind, arguments),
    )
}

pub(crate) fn form_prompt_text(
    descriptor: McpFormDescriptor,
    kind: McpFormPromptKind,
    arguments: Option<Map<String, Value>>,
) -> String {
    let resources = form_resource_uris(descriptor);
    let names = editor_tool_names(descriptor);
    let mut text = match kind {
        McpFormPromptKind::Fill => rust_i18n::t!(
            "prompts.fill_body",
            form_name = descriptor.form_name(),
            tool_name = descriptor.tool_name(),
            descriptor_uri = resources.descriptor,
            schema_uri = resources.schema,
            examples_uri = resources.examples,
        )
        .to_string(),
        McpFormPromptKind::Repair => rust_i18n::t!(
            "prompts.repair_body",
            form_name = descriptor.form_name(),
            tool_name = descriptor.tool_name(),
            descriptor_uri = resources.descriptor,
            schema_uri = resources.schema,
            patch_tool = names.patch,
        )
        .to_string(),
        McpFormPromptKind::Submit => rust_i18n::t!(
            "prompts.submit_body",
            form_name = descriptor.form_name(),
            tool_name = descriptor.tool_name(),
            open_tool = names.open,
            patch_tool = names.patch,
            validate_tool = names.validate,
            submit_tool = names.submit,
            descriptor_uri = resources.descriptor,
            schema_uri = resources.schema,
        )
        .to_string(),
    };

    if let Some(arguments) = arguments
        && !arguments.is_empty()
    {
        text.push_str(&format!(
            "\n\n{}\n",
            rust_i18n::t!("prompts.caller_context")
        ));
        text.push_str(
            &serde_json::to_string_pretty(&Value::Object(arguments)).unwrap_or_else(|error| {
                format!("{{\"error\":\"failed to render prompt arguments: {error}\"}}")
            }),
        );
    }

    text
}

/// Register prompt templates for every inventory-discovered generated MCP form.
///
/// This is opt-in. It can be chained with `McpServer::builder(...).register(...)`
/// after `gpui_form::mcp::register`, or called directly on a mutable server.
pub fn register_prompt_templates(server: &mut McpServer) -> Result<(), McpToolError> {
    let options = McpFormRegistrationOptions::all();
    let resource_specs = form_resource_specs_for_options(options)?;
    let prompt_specs = form_prompt_specs_for_options(options)?;
    register_form_resource_specs_if_missing(server, resource_specs)?;
    ensure_prompt_specs_available(server, &prompt_specs)?;
    register_form_prompt_specs(server, prompt_specs)
}

/// Register prompt templates for every context-backed form matching `Context`.
///
/// Use this with [`register_context_submitters`] when context-backed forms
/// should also expose reusable fill, repair, and submit prompt templates.
pub fn register_context_submitter_prompt_templates<Context>(
    server: &mut McpServer,
) -> Result<(), McpToolError>
where
    Context: Clone + Send + Sync + 'static,
{
    let resource_specs = context_submit_resource_specs::<Context>()?;
    let prompt_specs = context_submit_prompt_specs::<Context>()?;
    register_form_resource_specs_if_missing(server, resource_specs)?;
    ensure_prompt_specs_available(server, &prompt_specs)?;
    register_form_prompt_specs(server, prompt_specs)
}

pub(crate) fn form_prompt_specs_for_options(
    options: McpFormRegistrationOptions,
) -> Result<Vec<McpFormPromptSpec>, McpToolError> {
    if !options.registers_tools() {
        return Ok(Vec::new());
    }

    let mut seen_tool_names = BTreeSet::new();
    let mut specs = Vec::new();
    if options.submit_handlers {
        for registration in registry::submit_handler_registrations() {
            push_descriptor_prompt_specs(
                &mut seen_tool_names,
                &mut specs,
                registration.descriptor(),
            )?;
        }
    }
    if options.editors {
        for registration in registry::editor_registrations() {
            push_descriptor_prompt_specs(
                &mut seen_tool_names,
                &mut specs,
                registration.descriptor(),
            )?;
        }
    }
    Ok(specs)
}

pub(crate) fn context_submit_prompt_specs<Context>() -> Result<Vec<McpFormPromptSpec>, McpToolError>
where
    Context: Clone + Send + Sync + 'static,
{
    let mut seen_tool_names = BTreeSet::new();
    let mut specs = Vec::new();
    let context_type_id = TypeId::of::<Context>();
    for registration in registry::context_submit_registrations() {
        if registration.context_type_id() == context_type_id {
            push_descriptor_prompt_specs(
                &mut seen_tool_names,
                &mut specs,
                registration.descriptor(),
            )?;
        }
    }
    Ok(specs)
}

pub(crate) fn push_descriptor_prompt_specs(
    seen_tool_names: &mut BTreeSet<String>,
    specs: &mut Vec<McpFormPromptSpec>,
    descriptor: McpFormDescriptor,
) -> Result<(), McpToolError> {
    if !seen_tool_names.insert(descriptor.tool_name()) {
        return Ok(());
    }
    specs.extend(form_prompt_specs(descriptor)?);
    Ok(())
}
