use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::reference::index;
use crate::reference::search;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Hunk {
    pub before: Vec<String>,
    pub after: Vec<String>,
    #[serde(rename = "beforeStart")]
    pub before_start: u32,
    #[serde(rename = "afterStart")]
    pub after_start: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SymbolDiff {
    pub symbol: String,
    pub kind: String,
    #[serde(
        rename = "beforePath",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub before_path: Option<String>,
    #[serde(
        rename = "beforeLine",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub before_line: Option<u32>,
    #[serde(rename = "afterPath", skip_serializing_if = "Option::is_none", default)]
    pub after_path: Option<String>,
    #[serde(rename = "afterLine", skip_serializing_if = "Option::is_none", default)]
    pub after_line: Option<u32>,
    pub hunks: Vec<Hunk>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SymbolSummary {
    pub symbol: String,
    pub kind: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct PathDiff {
    pub added: Vec<SymbolSummary>,
    pub removed: Vec<SymbolSummary>,
    pub changed: Vec<SymbolDiff>,
    pub truncated: bool,
}

const DEFAULT_VIEW_WINDOW: u32 = 30;
const DEFAULT_MAX_SYMBOLS: usize = 50;

pub fn compute_line_diff(before: &[String], after: &[String]) -> Vec<Hunk> {
    if before == after {
        return vec![];
    }
    let before_refs: Vec<&str> = before.iter().map(|s| s.as_str()).collect();
    let after_refs: Vec<&str> = after.iter().map(|s| s.as_str()).collect();
    let diff = similar::TextDiff::from_slices(&before_refs, &after_refs);
    let mut hunks: Vec<Hunk> = Vec::new();
    let mut current: Option<(Vec<String>, Vec<String>, u32, u32)> = None;
    let mut before_line = 1u32;
    let mut after_line = 1u32;

    for change in diff.iter_all_changes() {
        let value = change.value().to_string();
        match change.tag() {
            similar::ChangeTag::Equal => {
                if let Some((b, a, bs, as_)) = current.take() {
                    hunks.push(Hunk {
                        before: b,
                        after: a,
                        before_start: bs,
                        after_start: as_,
                    });
                }
                before_line += 1;
                after_line += 1;
            }
            similar::ChangeTag::Delete => {
                let entry = current
                    .get_or_insert_with(|| (Vec::new(), Vec::new(), before_line, after_line));
                entry.0.push(value);
                before_line += 1;
            }
            similar::ChangeTag::Insert => {
                let entry = current
                    .get_or_insert_with(|| (Vec::new(), Vec::new(), before_line, after_line));
                entry.1.push(value);
                after_line += 1;
            }
        }
    }
    if let Some((b, a, bs, as_)) = current.take() {
        hunks.push(Hunk {
            before: b,
            after: a,
            before_start: bs,
            after_start: as_,
        });
    }
    hunks
}

pub fn compute_symbol_diff(
    from_dir: &Path,
    to_dir: &Path,
    symbol_fqn: &str,
) -> Result<Option<SymbolDiff>> {
    let from_index = index::build_or_update_index(from_dir)
        .with_context(|| format!("failed to index {}", from_dir.display()))?;
    let to_index = index::build_or_update_index(to_dir)
        .with_context(|| format!("failed to index {}", to_dir.display()))?;
    let (name, namespace) = split_fqn(symbol_fqn);
    let before_hit = find_first(&from_index, name, namespace.as_deref());
    let after_hit = find_first(&to_index, name, namespace.as_deref());

    if before_hit.is_none() && after_hit.is_none() {
        return Ok(None);
    }
    let before_lines = match &before_hit {
        Some(hit) => read_excerpt(from_dir, &hit.path, hit.line)?,
        None => Vec::new(),
    };
    let after_lines = match &after_hit {
        Some(hit) => read_excerpt(to_dir, &hit.path, hit.line)?,
        None => Vec::new(),
    };
    let kind = before_hit
        .as_ref()
        .or(after_hit.as_ref())
        .map(|h| h.kind.clone())
        .unwrap_or_default();
    let hunks = compute_line_diff(&before_lines, &after_lines);
    Ok(Some(SymbolDiff {
        symbol: symbol_fqn.to_string(),
        kind,
        before_path: before_hit.as_ref().map(|h| h.path.clone()),
        before_line: before_hit.as_ref().map(|h| h.line),
        after_path: after_hit.as_ref().map(|h| h.path.clone()),
        after_line: after_hit.as_ref().map(|h| h.line),
        hunks,
    }))
}

pub fn compute_path_diff(
    from_dir: &Path,
    to_dir: &Path,
    path_filter: Option<&str>,
    max_symbols: Option<usize>,
) -> Result<PathDiff> {
    let from_index = index::build_or_update_index(from_dir)
        .with_context(|| format!("failed to index {}", from_dir.display()))?;
    let to_index = index::build_or_update_index(to_dir)
        .with_context(|| format!("failed to index {}", to_dir.display()))?;
    let max = max_symbols.unwrap_or(DEFAULT_MAX_SYMBOLS);

    let mut before_map: std::collections::BTreeMap<String, &index::ReferenceSymbolEntry> =
        std::collections::BTreeMap::new();
    let mut after_map: std::collections::BTreeMap<String, &index::ReferenceSymbolEntry> =
        std::collections::BTreeMap::new();
    let prefix = path_filter.unwrap_or("");

    for file in from_index.files.values() {
        for sym in &file.symbols {
            if !sym.path.starts_with(prefix) {
                continue;
            }
            before_map.entry(symbol_key(sym)).or_insert(sym);
        }
    }
    for file in to_index.files.values() {
        for sym in &file.symbols {
            if !sym.path.starts_with(prefix) {
                continue;
            }
            after_map.entry(symbol_key(sym)).or_insert(sym);
        }
    }

    let mut added: Vec<SymbolSummary> = Vec::new();
    let mut removed: Vec<SymbolSummary> = Vec::new();
    let mut changed: Vec<SymbolDiff> = Vec::new();
    let mut total = 0usize;
    let mut truncated = false;

    for (key, sym) in &after_map {
        if !before_map.contains_key(key) {
            if total >= max {
                truncated = true;
                break;
            }
            added.push(SymbolSummary {
                symbol: key.clone(),
                kind: sym.kind.clone(),
                path: sym.path.clone(),
            });
            total += 1;
        }
    }
    if !truncated {
        for (key, sym) in &before_map {
            if !after_map.contains_key(key) {
                if total >= max {
                    truncated = true;
                    break;
                }
                removed.push(SymbolSummary {
                    symbol: key.clone(),
                    kind: sym.kind.clone(),
                    path: sym.path.clone(),
                });
                total += 1;
            }
        }
    }
    if !truncated {
        for (key, before_sym) in &before_map {
            if let Some(after_sym) = after_map.get(key) {
                if total >= max {
                    truncated = true;
                    break;
                }
                let before_lines = read_excerpt(from_dir, &before_sym.path, before_sym.line)?;
                let after_lines = read_excerpt(to_dir, &after_sym.path, after_sym.line)?;
                let hunks = compute_line_diff(&before_lines, &after_lines);
                if hunks.is_empty() {
                    continue;
                }
                changed.push(SymbolDiff {
                    symbol: key.clone(),
                    kind: before_sym.kind.clone(),
                    before_path: Some(before_sym.path.clone()),
                    before_line: Some(before_sym.line),
                    after_path: Some(after_sym.path.clone()),
                    after_line: Some(after_sym.line),
                    hunks,
                });
                total += 1;
            }
        }
    }

    Ok(PathDiff {
        added,
        removed,
        changed,
        truncated,
    })
}

fn split_fqn(fqn: &str) -> (&str, Option<String>) {
    if let Some(idx) = fqn.rfind('.') {
        let (ns, name) = fqn.split_at(idx);
        (&name[1..], Some(ns.to_string()))
    } else {
        (fqn, None)
    }
}

fn find_first(
    index: &index::ReferenceSymbolIndex,
    name: &str,
    namespace: Option<&str>,
) -> Option<index::ReferenceSymbolEntry> {
    index::find_symbol(index, name, None, namespace)
        .into_iter()
        .next()
}

fn symbol_key(sym: &index::ReferenceSymbolEntry) -> String {
    sym.fqn.clone().unwrap_or_else(|| sym.name.clone())
}

fn read_excerpt(root: &Path, rel_path: &str, line: u32) -> Result<Vec<String>> {
    let start = line.saturating_sub(1).max(1);
    let view = search::run_view(root, rel_path, Some(start), Some(DEFAULT_VIEW_WINDOW))
        .with_context(|| format!("failed to read excerpt for {rel_path} at line {line}"))?;
    Ok(view.lines)
}

pub fn ensure_cache_dir(dir: &Path, label: &str) -> Result<()> {
    if !dir.exists() {
        return Err(anyhow!(
            "reference cache for {} ({}) does not exist; run `unity-cli reference fetch --version {}` first",
            label,
            dir.display(),
            label
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reference-cache-v2")
    }

    fn v1() -> PathBuf {
        fixture_root().join("v1")
    }

    fn v2() -> PathBuf {
        fixture_root().join("v2")
    }

    #[test]
    fn compute_line_diff_empty_for_identical_lines() {
        let lines = vec!["a".to_string(), "b".to_string()];
        assert!(compute_line_diff(&lines, &lines).is_empty());
    }

    #[test]
    fn compute_line_diff_replacement_isolates_changed_lines() {
        let before = vec!["a".to_string(), "b".to_string()];
        let after = vec!["a".to_string(), "c".to_string()];
        let hunks = compute_line_diff(&before, &after);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].before, vec!["b".to_string()]);
        assert_eq!(hunks[0].after, vec!["c".to_string()]);
        assert_eq!(hunks[0].before_start, 2);
        assert_eq!(hunks[0].after_start, 2);
    }

    #[test]
    fn compute_line_diff_for_inserts_only_returns_addition_hunk() {
        let before = vec!["a".to_string(), "b".to_string()];
        let after = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let hunks = compute_line_diff(&before, &after);
        assert_eq!(hunks.len(), 1);
        assert!(hunks[0].before.is_empty());
        assert_eq!(hunks[0].after, vec!["c".to_string()]);
        assert_eq!(hunks[0].after_start, 3);
        assert_eq!(hunks[0].before_start, 3);
    }

    #[test]
    fn compute_line_diff_for_deletes_only_returns_removal_hunk() {
        let before = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let after = vec!["a".to_string(), "c".to_string()];
        let hunks = compute_line_diff(&before, &after);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].before, vec!["b".to_string()]);
        assert!(hunks[0].after.is_empty());
        assert_eq!(hunks[0].before_start, 2);
        assert_eq!(hunks[0].after_start, 2);
    }

    #[test]
    fn compute_line_diff_for_separate_changes_produces_multiple_hunks() {
        let before = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];
        let after = vec![
            "X".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "Y".to_string(),
        ];
        let hunks = compute_line_diff(&before, &after);
        assert!(
            hunks.len() >= 2,
            "expected at least 2 hunks for two separate changes: {hunks:?}"
        );
        // First hunk: a -> X at line 1
        assert_eq!(hunks.first().unwrap().before_start, 1);
        // Last hunk: e -> Y at line 5
        assert_eq!(hunks.last().unwrap().before_start, 5);
    }

    #[test]
    fn split_fqn_separates_namespace_and_name() {
        assert_eq!(
            split_fqn("UnityEngine.Animator"),
            ("Animator", Some("UnityEngine".to_string()))
        );
        assert_eq!(split_fqn("Animator"), ("Animator", None));
        assert_eq!(
            split_fqn("UnityEngine.Animation.Animator"),
            ("Animator", Some("UnityEngine.Animation".to_string()))
        );
    }

    #[test]
    fn compute_symbol_diff_detects_changed_animator() {
        let diff = compute_symbol_diff(&v1(), &v2(), "UnityEngine.Animator")
            .unwrap()
            .expect("Animator should be present in both versions");
        assert_eq!(diff.symbol, "UnityEngine.Animator");
        assert_eq!(diff.kind, "class");
        assert!(diff.before_path.is_some());
        assert!(diff.after_path.is_some());
        assert!(
            !diff.hunks.is_empty(),
            "Animator changed in v2 should yield hunks"
        );
    }

    #[test]
    fn compute_symbol_diff_returns_none_for_missing_in_both() {
        let result = compute_symbol_diff(&v1(), &v2(), "UnityEngine.DefinitelyNotHere").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn compute_symbol_diff_returns_diff_when_only_one_side_has_symbol() {
        let added_only = compute_symbol_diff(&v1(), &v2(), "UnityEngine.Awaitable")
            .unwrap()
            .expect("Awaitable should be present in v2");
        assert!(added_only.before_path.is_none());
        assert!(added_only.after_path.is_some());
        let removed_only = compute_symbol_diff(&v1(), &v2(), "UnityEngine.LegacyAnimator")
            .unwrap()
            .expect("LegacyAnimator should be present in v1");
        assert!(removed_only.before_path.is_some());
        assert!(removed_only.after_path.is_none());
    }

    #[test]
    fn compute_path_diff_categorises_runtime_export() {
        let diff = compute_path_diff(&v1(), &v2(), Some("Runtime/Export"), None).unwrap();
        assert!(
            diff.added.iter().any(|s| s.symbol.ends_with("Awaitable")),
            "Awaitable should be added: {:?}",
            diff.added
        );
        assert!(
            diff.removed
                .iter()
                .any(|s| s.symbol.ends_with("LegacyAnimator")),
            "LegacyAnimator should be removed: {:?}",
            diff.removed
        );
        assert!(
            diff.changed
                .iter()
                .any(|s| s.symbol.ends_with("Animator") && !s.symbol.ends_with("LegacyAnimator")),
            "Animator should be changed: {:?}",
            diff.changed
        );
        assert!(!diff.truncated);
    }

    #[test]
    fn compute_path_diff_can_be_truncated() {
        let diff = compute_path_diff(&v1(), &v2(), None, Some(1)).unwrap();
        let total = diff.added.len() + diff.removed.len() + diff.changed.len();
        assert_eq!(total, 1);
        assert!(diff.truncated);
    }

    #[test]
    fn ensure_cache_dir_errors_on_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let err = ensure_cache_dir(&tmp.path().join("missing"), "2023.2.20f1").unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("does not exist"));
        assert!(msg.contains("2023.2.20f1"));
    }
}
