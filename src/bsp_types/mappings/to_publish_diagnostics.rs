// Parts copied from rust-analyzer

use std::collections::HashMap;

use cargo_metadata::diagnostic::{
    Diagnostic as MetadataDiagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
};
use itertools::Itertools;
use paths::AbsPath;

use crate::bsp_types::notifications::{Diagnostic, PublishDiagnosticsParams};
use crate::bsp_types::{BuildTargetIdentifier, TextDocumentIdentifier};

pub fn create_diagnostics(
    msg: &MetadataDiagnostic,
    origin_id: Option<String>,
    build_target: &BuildTargetIdentifier,
    workspace_root: &AbsPath,
) -> Vec<PublishDiagnosticsParams> {
    let diagnostics = map_cargo_diagnostic_to_bsp(msg, workspace_root);
    diagnostics
        .into_iter()
        .map(|(url, diagnostic)| PublishDiagnosticsParams {
            text_document: TextDocumentIdentifier {
                uri: url.to_string(),
            },
            build_target: build_target.clone(),
            origin_id: origin_id.clone(),
            diagnostics: diagnostic,
            reset: false,
        })
        .collect()
}

enum MappedRustChildDiagnostic {
    RelatedDiagnostic(lsp_types::DiagnosticRelatedInformation),
    MessageLine(String),
}

/// Converts a Rust root diagnostic to LSP form
///
/// This flattens the Rust diagnostic by:
///
/// 1. Creating a LSP diagnostic with the root message and primary span.
/// 2. Adding any labelled secondary spans to `relatedInformation`
/// 3. Categorising child diagnostics as either `SuggestedFix`es,
///    `relatedInformation` or additional message lines.
///
/// If the diagnostic has no primary span this will return `None`
fn map_cargo_diagnostic_to_bsp(
    diagnostic: &MetadataDiagnostic,
    workspace_root: &AbsPath,
) -> HashMap<lsp_types::Url, Vec<Diagnostic>> {
    let primary_spans: Vec<&DiagnosticSpan> =
        diagnostic.spans.iter().filter(|s| s.is_primary).collect();
    if primary_spans.is_empty() {
        return HashMap::new();
    }

    let severity = diagnostic_severity(diagnostic.level);
    let source = String::from("cargo");

    let mut code = diagnostic.code.as_ref().map(|c| c.code.clone());
    let mut code_description = None;
    if let Some(code_val) = &code {
        // See if this is an RFC #2103 scoped lint (e.g. from Clippy)
        let scoped_code: Vec<&str> = code_val.split("::").collect();
        if scoped_code.len() == 2 {
            if scoped_code[0].eq("clippy") {
                code_description = clippy_code_description(code.as_deref());
            }
            code = Some(String::from(scoped_code[1]));
        } else {
            code_description = rustc_code_description(code.as_deref());
        }
    }

    let tags = diagnostic_tags(&diagnostic.code);

    // Indicates whether primary span label needs to be added to the diagnostics
    // message.
    let mut needs_primary_span_label = true;
    let mut subdiagnostics = vec![];

    for secondary_span in diagnostic.spans.iter().filter(|s| !s.is_primary) {
        let related = diagnostic_related_information(workspace_root, secondary_span);
        if let Some(related) = related {
            subdiagnostics.push(related);
        }
    }

    let mut message = diagnostic.message.clone();
    for child in &diagnostic.children {
        let child = map_rust_child_diagnostic(workspace_root, child);
        match child {
            MappedRustChildDiagnostic::RelatedDiagnostic(diagnostic) => {
                subdiagnostics.push(diagnostic);
            }
            MappedRustChildDiagnostic::MessageLine(message_line) => {
                message.push_str(&format!("\n{}", message_line));

                // These secondary messages usually duplicate the content of the
                // primary span label.
                needs_primary_span_label = false;
            }
        }
    }

    let mut diagnostics: HashMap<lsp_types::Url, Vec<Diagnostic>> = HashMap::new();

    for primary_span in &primary_spans {
        let primary_location = primary_location(workspace_root, primary_span);
        if needs_primary_span_label {
            if let Some(primary_span_label) = &primary_span.label {
                message.push_str(&format!("\n{}", primary_span_label));
            }
        }

        let mut related_info_macro_calls = vec![];

        // If error occurs from macro expansion, add related info pointing to
        // where the error originated
        // Also, we would generate an additional diagnostic, so that exact place of macro
        // will be highlighted in the error origin place.
        let span_stack = std::iter::successors(Some(*primary_span), |span| {
            Some(&span.expansion.as_ref()?.span)
        });
        for (i, span) in span_stack.enumerate() {
            if is_dummy_macro_file(&span.file_name) {
                continue;
            }

            // First span is the original diagnostic, others are macro call locations that
            // generated that code.
            let is_in_macro_call = i != 0;

            let secondary_location = location(workspace_root, span);
            if secondary_location == primary_location {
                continue;
            }
            related_info_macro_calls.push(lsp_types::DiagnosticRelatedInformation {
                location: secondary_location.clone(),
                message: if is_in_macro_call {
                    "Error originated from macro call here".to_string()
                } else {
                    "Actual error occurred here".to_string()
                },
            });
            // For the additional in-macro diagnostic we add the inverse message pointing to the error location in code.
            let information_for_additional_diagnostic =
                vec![lsp_types::DiagnosticRelatedInformation {
                    location: primary_location.clone(),
                    message: "Exact error occurred here".to_string(),
                }];

            let diagnostic = lsp_types::Diagnostic {
                range: secondary_location.range,
                // downgrade to hint if we're pointing at the macro
                severity: Some(lsp_types::DiagnosticSeverity::HINT),
                code: code.clone().map(lsp_types::NumberOrString::String),
                code_description: code_description.clone(),
                source: Some(source.clone()),
                message: message.clone(),
                related_information: Some(information_for_additional_diagnostic),
                tags: tags.as_ref().cloned(),
                data: Some(serde_json::json!({ "rendered": diagnostic.rendered })),
            };
            add_diagnostic(secondary_location.uri, diagnostic, &mut diagnostics);
        }

        // Emit the primary diagnostic.
        let diagnostic = lsp_types::Diagnostic {
            range: primary_location.range,
            severity,
            code: code.clone().map(lsp_types::NumberOrString::String),
            code_description: code_description.clone(),
            source: Some(source.clone()),
            message: message.clone(),
            related_information: {
                let info = related_info_macro_calls
                    .iter()
                    .cloned()
                    .chain(subdiagnostics.iter().cloned())
                    .collect::<Vec<_>>();
                if info.is_empty() {
                    None
                } else {
                    Some(info)
                }
            },
            tags: tags.as_ref().cloned(),
            data: Some(serde_json::json!({ "rendered": diagnostic.rendered })),
        };
        add_diagnostic(primary_location.uri.clone(), diagnostic, &mut diagnostics);

        // Emit hint-level diagnostics for all `related_information` entries such as "help"s.
        let back_ref = lsp_types::DiagnosticRelatedInformation {
            location: primary_location,
            message: "original diagnostic".to_string(),
        };
        for sub in &subdiagnostics {
            let diagnostic = lsp_types::Diagnostic {
                range: sub.location.range,
                severity: Some(lsp_types::DiagnosticSeverity::HINT),
                code: code.clone().map(lsp_types::NumberOrString::String),
                code_description: code_description.clone(),
                source: Some(source.clone()),
                message: sub.message.clone(),
                related_information: Some(vec![back_ref.clone()]),
                tags: None, // don't apply modifiers again
                data: None,
            };
            add_diagnostic(sub.location.uri.clone(), diagnostic, &mut diagnostics);
        }
    }
    diagnostics
}

fn add_diagnostic(
    url: lsp_types::Url,
    diagnostic: Diagnostic,
    diagnostics: &mut HashMap<lsp_types::Url, Vec<Diagnostic>>,
) {
    if let std::collections::hash_map::Entry::Vacant(e) = diagnostics.entry(url.clone()) {
        e.insert(vec![diagnostic]);
    } else {
        diagnostics.get_mut(&url).unwrap().push(diagnostic);
    }
}

fn diagnostic_severity(level: DiagnosticLevel) -> Option<lsp_types::DiagnosticSeverity> {
    let res = match level {
        DiagnosticLevel::Ice => lsp_types::DiagnosticSeverity::ERROR,
        DiagnosticLevel::Error => lsp_types::DiagnosticSeverity::ERROR,
        DiagnosticLevel::Warning => lsp_types::DiagnosticSeverity::WARNING,
        DiagnosticLevel::FailureNote => lsp_types::DiagnosticSeverity::INFORMATION,
        DiagnosticLevel::Note => lsp_types::DiagnosticSeverity::INFORMATION,
        DiagnosticLevel::Help => lsp_types::DiagnosticSeverity::HINT,
        _ => return None,
    };
    Some(res)
}

fn rustc_code_description(code: Option<&str>) -> Option<lsp_types::CodeDescription> {
    code.filter(|code| {
        let mut chars = code.chars();
        chars.next().map_or(false, |c| c == 'E')
            && chars.by_ref().take(4).all(|c| c.is_ascii_digit())
            && chars.next().is_none()
    })
    .and_then(|code| {
        lsp_types::Url::parse(&format!(
            "https://doc.rust-lang.org/error-index.html#{}",
            code
        ))
        .ok()
        .map(|href| lsp_types::CodeDescription { href })
    })
}

fn clippy_code_description(code: Option<&str>) -> Option<lsp_types::CodeDescription> {
    code.and_then(|code| {
        lsp_types::Url::parse(&format!(
            "https://rust-lang.github.io/rust-clippy/master/index.html#{}",
            code
        ))
        .ok()
        .map(|href| lsp_types::CodeDescription { href })
    })
}

fn diagnostic_tags(code: &Option<DiagnosticCode>) -> Option<Vec<lsp_types::DiagnosticTag>> {
    let mut tags = vec![];
    if let Some(code) = code {
        let code = code.code.as_str();
        if matches!(
            code,
            "dead_code"
                | "unknown_lints"
                | "unreachable_code"
                | "unused_attributes"
                | "unused_imports"
                | "unused_macros"
                | "unused_variables"
        ) {
            tags.push(lsp_types::DiagnosticTag::UNNECESSARY);
        }

        if matches!(code, "deprecated") {
            tags.push(lsp_types::DiagnosticTag::DEPRECATED);
        }
    }
    if tags.is_empty() {
        None
    } else {
        Some(tags)
    }
}

/// Returns a `Url` object from a given path, will lowercase drive letters if present.
/// This will only happen when processing windows paths.
///
/// When processing non-windows path, this is essentially the same as `Url::from_file_path`.
fn url_from_abs_path(path: &AbsPath) -> lsp_types::Url {
    let url = lsp_types::Url::from_file_path(path).unwrap();
    match path.as_ref().components().next() {
        Some(std::path::Component::Prefix(prefix))
            if matches!(
                prefix.kind(),
                std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_)
            ) =>
        {
            // Need to lowercase driver letter
        }
        _ => return url,
    }

    let driver_letter_range = {
        let (scheme, drive_letter, _rest) = match url.as_str().splitn(3, ':').collect_tuple() {
            Some(it) => it,
            None => return url,
        };
        let start = scheme.len() + ':'.len_utf8();
        start..(start + drive_letter.len())
    };

    // Note: lowercasing the `path` itself doesn't help, the `Url::parse`
    // machinery *also* canonicalizes the drive letter. So, just massage the
    // string in place.
    let mut url: String = url.into();
    url[driver_letter_range].make_ascii_lowercase();
    lsp_types::Url::parse(&url).unwrap()
}

/// Converts line_offset and column_offset from 1-based to 0-based.
fn position(
    span: &DiagnosticSpan,
    line_offset: usize,
    column_offset: usize,
) -> lsp_types::Position {
    let line_index = line_offset - span.line_start;

    let mut true_column_offset = column_offset;
    if let Some(line) = span.text.get(line_index) {
        if line.text.chars().count() == line.text.len() {
            // all one byte utf-8 char
            return lsp_types::Position {
                line: (line_offset as u32).saturating_sub(1),
                character: (column_offset as u32).saturating_sub(1),
            };
        }
        let mut char_offset = 0;
        for c in line.text.chars() {
            char_offset += 1;
            if char_offset > column_offset {
                break;
            }
            true_column_offset += char::len_utf16(c) - 1;
        }
    }

    lsp_types::Position {
        line: (line_offset as u32).saturating_sub(1),
        character: (true_column_offset as u32).saturating_sub(1),
    }
}

/// Converts a cargo span to a LSP location
fn location(workspace_root: &AbsPath, span: &DiagnosticSpan) -> lsp_types::Location {
    let file_name = workspace_root.join(&span.file_name);
    let uri = url_from_abs_path(&file_name);

    let range = {
        lsp_types::Range::new(
            position(span, span.line_start, span.column_start),
            position(span, span.line_end, span.column_end),
        )
    };
    lsp_types::Location::new(uri, range)
}

/// Converts a non-primary cargo span to a LSP related information.
/// If the span is unlabelled this will return `None`.
fn diagnostic_related_information(
    workspace_root: &AbsPath,
    span: &DiagnosticSpan,
) -> Option<lsp_types::DiagnosticRelatedInformation> {
    let message = span.label.clone()?;
    let location = location(workspace_root, span);
    Some(lsp_types::DiagnosticRelatedInformation { location, message })
}

fn map_rust_child_diagnostic(
    workspace_root: &AbsPath,
    diagnostic: &MetadataDiagnostic,
) -> MappedRustChildDiagnostic {
    let spans: Vec<&DiagnosticSpan> = diagnostic.spans.iter().filter(|s| s.is_primary).collect();
    if spans.is_empty() {
        // We use spanless children as a way to print multi-line messages.
        return MappedRustChildDiagnostic::MessageLine(diagnostic.message.clone());
    }

    let mut suggested_replacements = vec![];
    for &span in &spans {
        if let Some(suggested_replacement) = &span.suggested_replacement {
            if !suggested_replacement.is_empty() {
                suggested_replacements.push(suggested_replacement);
            }
        }
    }

    // We render suggestion diagnostics by appending the suggested replacement.
    let mut message = diagnostic.message.clone();
    if !suggested_replacements.is_empty() {
        message.push_str(": ");
        let suggestions = suggested_replacements
            .iter()
            .map(|suggestion| format!("`{}", suggestion))
            .join(", ");
        message.push_str(&suggestions);
    }

    MappedRustChildDiagnostic::RelatedDiagnostic(lsp_types::DiagnosticRelatedInformation {
        location: location(workspace_root, spans[0]),
        message,
    })
}

/// Extracts a suitable "primary" location from a rustc diagnostic.
///
/// This takes locations pointing into the standard library, or generally outside the current
/// workspace into account and tries to avoid those, in case macros are involved.
fn primary_location(workspace_root: &AbsPath, span: &DiagnosticSpan) -> lsp_types::Location {
    let span_stack = std::iter::successors(Some(span), |span| Some(&span.expansion.as_ref()?.span));
    for span in span_stack.clone() {
        let abs_path = workspace_root.join(&span.file_name);
        if !is_dummy_macro_file(&span.file_name) && abs_path.starts_with(workspace_root) {
            return location(workspace_root, span);
        }
    }

    // Fall back to the outermost macro invocation if no suitable span comes up.
    let last_span = span_stack.last().unwrap();
    location(workspace_root, last_span)
}

/// Checks whether a file name is from macro invocation and does not refer to an actual file.
fn is_dummy_macro_file(file_name: &str) -> bool {
    file_name.starts_with('<') && file_name.ends_with('>')
}
