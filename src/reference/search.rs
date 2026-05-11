use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GrepHit {
    pub path: String,
    pub line: u32,
    pub text: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ViewOutput {
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub lines: Vec<String>,
}

pub fn run_grep(
    root: &Path,
    pattern: &str,
    file_glob: Option<&str>,
    context: u32,
) -> Result<Vec<GrepHit>> {
    let regex = Regex::new(pattern).with_context(|| format!("invalid regex: {pattern}"))?;
    let glob_re = file_glob.map(glob_to_regex).transpose()?;
    let mut hits = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if let Some(re) = &glob_re {
            if !re.is_match(&file_name) {
                continue;
            }
        }
        let rel = path.strip_prefix(root).unwrap_or(path);
        let rel_str = rel.display().to_string();
        let contents = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let lines: Vec<&str> = contents.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            if regex.is_match(line) {
                let line_no = (idx + 1) as u32;
                let ctx = context as usize;
                let before_start = idx.saturating_sub(ctx);
                let after_end = (idx + 1 + ctx).min(lines.len());
                let context_before: Vec<String> = lines[before_start..idx]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                let context_after: Vec<String> = lines[idx + 1..after_end]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                hits.push(GrepHit {
                    path: rel_str.clone(),
                    line: line_no,
                    text: (*line).to_string(),
                    context_before,
                    context_after,
                });
            }
        }
    }
    hits.sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));
    Ok(hits)
}

pub fn run_view(
    root: &Path,
    rel_path: &str,
    start_line: Option<u32>,
    max_lines: Option<u32>,
) -> Result<ViewOutput> {
    if rel_path
        .split(['/', std::path::MAIN_SEPARATOR])
        .any(|seg| seg == "..")
    {
        return Err(anyhow!("path must not contain '..' segments: {rel_path}"));
    }
    let path = root.join(rel_path);
    let contents =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let all_lines: Vec<&str> = contents.lines().collect();
    let start = start_line.unwrap_or(1).max(1) as usize;
    if start > all_lines.len() && !all_lines.is_empty() {
        return Err(anyhow!(
            "start_line {start} exceeds file length {}",
            all_lines.len()
        ));
    }
    let max = max_lines.map(|m| m as usize).unwrap_or(usize::MAX);
    let begin_idx = start.saturating_sub(1);
    let end_idx = begin_idx.saturating_add(max).min(all_lines.len());
    let slice: Vec<String> = all_lines[begin_idx..end_idx]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let end_line = if slice.is_empty() {
        start as u32
    } else {
        (begin_idx + slice.len()) as u32
    };
    Ok(ViewOutput {
        path: rel_path.to_string(),
        start_line: start as u32,
        end_line,
        lines: slice,
    })
}

fn glob_to_regex(glob: &str) -> Result<Regex> {
    let mut out = String::from("^");
    for ch in glob.chars() {
        match ch {
            '*' => out.push_str("[^/]*"),
            '?' => out.push('.'),
            '.' | '+' | '(' | ')' | '|' | '^' | '$' | '{' | '}' | '[' | ']' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out.push('$');
    Regex::new(&out).with_context(|| format!("invalid file_glob: {glob}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reference-cache")
    }

    #[test]
    fn grep_finds_class_animator_with_line_numbers() {
        let hits = run_grep(&fixture_root(), "class Animator", None, 0).unwrap();
        assert!(!hits.is_empty());
        for hit in &hits {
            assert!(hit.text.contains("class Animator"));
            assert!(hit.line >= 1);
        }
        assert!(hits.iter().any(|h| h.path.contains("Animator.bindings.cs")));
    }

    #[test]
    fn grep_filters_by_filename_glob() {
        let hits = run_grep(&fixture_root(), "class", Some("*.cs"), 0).unwrap();
        assert!(!hits.is_empty());
        for hit in &hits {
            assert!(hit.path.ends_with(".cs"));
        }
    }

    #[test]
    fn grep_returns_context_lines() {
        let hits = run_grep(&fixture_root(), "Play\\(string stateName\\)$", None, 1).unwrap();
        let hit = hits
            .into_iter()
            .find(|h| h.text.contains("Play(string stateName)"))
            .expect("should find Play binding");
        assert_eq!(hit.context_before.len(), 1);
        assert_eq!(hit.context_after.len(), 1);
    }

    #[test]
    fn view_returns_requested_line_range() {
        let out = run_view(
            &fixture_root(),
            "Runtime/Export/Animation/Animator.bindings.cs",
            Some(3),
            Some(2),
        )
        .unwrap();
        assert_eq!(out.start_line, 3);
        assert_eq!(out.lines.len(), 2);
        assert_eq!(out.end_line, 4);
    }

    #[test]
    fn view_rejects_parent_traversal() {
        let err = run_view(&fixture_root(), "../escape.cs", None, None).unwrap_err();
        assert!(format!("{err:#}").contains(".."));
    }
}
