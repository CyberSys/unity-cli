use serde_json::json;

use super::model::{Severity, Violation};
use super::runner::LintOutcome;

#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    Text,
    Json,
}

pub fn render(outcome: &LintOutcome, format: ReportFormat, severity: Severity) -> String {
    match format {
        ReportFormat::Text => render_text(outcome, severity),
        ReportFormat::Json => render_json(outcome),
    }
}

fn render_text(outcome: &LintOutcome, severity: Severity) -> String {
    let mut out = String::new();
    for v in &outcome.violations {
        let label = match severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        out.push_str(&format!(
            "{}: {} [{}] {}\n",
            v.path.display(),
            label,
            v.rule,
            v.message
        ));
    }
    out.push_str(&format!(
        "\n{} skills checked, {} violations\n",
        outcome.skills.len(),
        outcome.violations.len()
    ));
    out
}

fn render_json(outcome: &LintOutcome) -> String {
    let arr: Vec<_> = outcome
        .violations
        .iter()
        .map(|v: &Violation| {
            json!({
                "path": v.path,
                "rule": v.rule,
                "severity": v.severity,
                "skill": v.skill,
                "message": v.message,
            })
        })
        .collect();
    serde_json::to_string_pretty(&arr).unwrap_or_else(|_| "[]".to_string())
}
