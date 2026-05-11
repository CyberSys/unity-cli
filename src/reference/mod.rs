#![allow(dead_code)]

pub mod cache;
pub mod fetcher;
pub mod search;
pub mod version;

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use walkdir::WalkDir;

pub fn maybe_execute_reference_tool(tool_name: &str, params: &Value) -> Option<Result<Value>> {
    match tool_name {
        "reference_fetch" => Some(execute_fetch(params)),
        "reference_status" => Some(execute_status(params)),
        "reference_search" => Some(execute_search(params)),
        "reference_grep" => Some(execute_grep(params)),
        "reference_view" => Some(execute_view(params)),
        "reference_clean" => Some(execute_clean(params)),
        _ => None,
    }
}

fn resolve_version(params: &Value) -> Result<String> {
    if let Some(v) = params.get("version").and_then(Value::as_str) {
        if !v.is_empty() {
            return Ok(v.to_string());
        }
    }
    let project_root = params
        .get("projectRoot")
        .and_then(Value::as_str)
        .map(Path::new)
        .unwrap_or_else(|| Path::new("."));
    Ok(version::detect_from_project(project_root)?.version)
}

fn resolve_version_and_branch(params: &Value) -> Result<(String, String)> {
    let explicit_version = params
        .get("version")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let explicit_branch = params
        .get("branch")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    if let (Some(v), Some(b)) = (explicit_version.clone(), explicit_branch.clone()) {
        return Ok((v, b));
    }
    let project_root = params
        .get("projectRoot")
        .and_then(Value::as_str)
        .map(Path::new)
        .unwrap_or_else(|| Path::new("."));
    let detected = version::detect_from_project(project_root)?;
    Ok((
        explicit_version.unwrap_or(detected.version),
        explicit_branch.unwrap_or(detected.branch),
    ))
}

fn execute_fetch(params: &Value) -> Result<Value> {
    let (version, branch) = resolve_version_and_branch(params)?;
    let accept_license = params
        .get("acceptLicense")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let force = params
        .get("force")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let dest = cache::version_dir(&version)?;
    if dest.exists() && !force {
        return Ok(json!({
            "ok": true,
            "skipped": true,
            "reason": "destination already exists; pass force=true to refetch",
            "version": version,
            "branch": branch,
            "path": dest.display().to_string(),
        }));
    }
    if dest.exists() && force {
        std::fs::remove_dir_all(&dest)
            .with_context(|| format!("failed to remove {}", dest.display()))?;
    }
    fetcher::run_clone(
        fetcher::UNITY_CS_REFERENCE_URL,
        &branch,
        &dest,
        1,
        accept_license,
    )?;
    let meta = cache::CacheMeta {
        version: version.clone(),
        branch: branch.clone(),
        commit_sha: None,
        fetched_at: now_unix_seconds_string(),
        source_url: fetcher::UNITY_CS_REFERENCE_URL.to_string(),
    };
    cache::write_meta(&meta)?;
    Ok(json!({
        "ok": true,
        "version": version,
        "branch": branch,
        "path": dest.display().to_string(),
        "fetchedAt": meta.fetched_at,
    }))
}

fn execute_status(_params: &Value) -> Result<Value> {
    let versions = cache::list_versions()?;
    let mut entries = Vec::new();
    for v in versions {
        let dir = cache::version_dir(&v)?;
        let size_bytes = dir_size(&dir).unwrap_or(0);
        let meta = cache::read_meta(&v).ok();
        entries.push(json!({
            "version": v,
            "branch": meta.as_ref().map(|m| m.branch.clone()).unwrap_or_default(),
            "fetchedAt": meta.as_ref().map(|m| m.fetched_at.clone()).unwrap_or_default(),
            "sizeBytes": size_bytes,
            "path": dir.display().to_string(),
        }));
    }
    Ok(json!({
        "ok": true,
        "versions": entries,
    }))
}

fn execute_search(params: &Value) -> Result<Value> {
    let pattern = params
        .get("pattern")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("search requires `pattern`"))?;
    let version = resolve_version(params)?;
    let dir = cache::version_dir(&version)?;
    let file_glob = params.get("path").and_then(Value::as_str);
    let context = params.get("context").and_then(Value::as_u64).unwrap_or(0) as u32;
    let max_results = params
        .get("maxResults")
        .and_then(Value::as_u64)
        .map(|n| n as usize);
    let mut hits = search::run_grep(&dir, pattern, file_glob, context)?;
    if let Some(n) = max_results {
        hits.truncate(n);
    }
    Ok(json!({
        "ok": true,
        "version": version,
        "hits": hits,
    }))
}

fn execute_grep(params: &Value) -> Result<Value> {
    let pattern = params
        .get("pattern")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("grep requires `pattern`"))?;
    let version = resolve_version(params)?;
    let dir = cache::version_dir(&version)?;
    let file_glob = params.get("fileGlob").and_then(Value::as_str);
    let context = params.get("context").and_then(Value::as_u64).unwrap_or(0) as u32;
    let hits = search::run_grep(&dir, pattern, file_glob, context)?;
    Ok(json!({
        "ok": true,
        "version": version,
        "hits": hits,
    }))
}

fn execute_view(params: &Value) -> Result<Value> {
    let path = params
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("view requires `path`"))?;
    let version = resolve_version(params)?;
    let dir = cache::version_dir(&version)?;
    let start_line = params
        .get("startLine")
        .and_then(Value::as_u64)
        .map(|n| n as u32);
    let max_lines = params
        .get("maxLines")
        .and_then(Value::as_u64)
        .map(|n| n as u32);
    let out = search::run_view(&dir, path, start_line, max_lines)?;
    Ok(json!({
        "ok": true,
        "version": version,
        "view": out,
    }))
}

fn execute_clean(params: &Value) -> Result<Value> {
    let keep = params.get("keep").and_then(Value::as_u64).unwrap_or(1) as usize;
    let dry_run = params
        .get("dryRun")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let removed = cache::gc(keep, dry_run)?;
    Ok(json!({
        "ok": true,
        "keep": keep,
        "dryRun": dry_run,
        "removed": removed
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>(),
    }))
}

fn now_unix_seconds_string() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn dir_size(path: &Path) -> Result<u64> {
    let mut total = 0u64;
    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                total = total.saturating_add(meta.len());
            }
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = env::var(key).ok();
            env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.previous {
                env::set_var(self.key, value);
            } else {
                env::remove_var(self.key);
            }
        }
    }

    fn unique_temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_nanos();
        env::temp_dir().join(format!("unity-cli-reference-mod-{label}-{nanos}"))
    }

    #[test]
    fn maybe_execute_returns_none_for_unknown_tool() {
        assert!(maybe_execute_reference_tool("nonexistent", &json!({})).is_none());
    }

    #[test]
    fn execute_clean_dry_run_on_empty_cache_returns_empty_removed() {
        let _guard = crate::test_env::env_lock()
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let root = unique_temp_path("clean");
        let _env = EnvVarGuard::set("UNITY_CLI_CACHE_ROOT", root.to_str().unwrap());
        let value =
            maybe_execute_reference_tool("reference_clean", &json!({"dryRun": true, "keep": 1}))
                .unwrap()
                .unwrap();
        assert_eq!(value["ok"], true);
        assert_eq!(value["dryRun"], true);
        assert!(value["removed"].is_array());
        assert_eq!(value["removed"].as_array().unwrap().len(), 0);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn execute_status_returns_versions_list() {
        let _guard = crate::test_env::env_lock()
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let root = unique_temp_path("status");
        let _env = EnvVarGuard::set("UNITY_CLI_CACHE_ROOT", root.to_str().unwrap());
        let base = root.join("UnityCsReference");
        std::fs::create_dir_all(base.join("2023.2.20f1")).unwrap();
        let value = maybe_execute_reference_tool("reference_status", &json!({}))
            .unwrap()
            .unwrap();
        assert_eq!(value["ok"], true);
        let versions = value["versions"].as_array().unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0]["version"], "2023.2.20f1");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_version_uses_explicit_param() {
        let v = resolve_version(&json!({"version": "2023.2.20f1"})).unwrap();
        assert_eq!(v, "2023.2.20f1");
    }

    #[test]
    fn resolve_version_falls_back_to_project_detection() {
        let tmp = unique_temp_path("resolve-version");
        let settings = tmp.join("ProjectSettings");
        std::fs::create_dir_all(&settings).unwrap();
        std::fs::write(
            settings.join("ProjectVersion.txt"),
            "m_EditorVersion: 2023.2.20f1\n",
        )
        .unwrap();
        let v = resolve_version(&json!({"projectRoot": tmp.to_str().unwrap()})).unwrap();
        assert_eq!(v, "2023.2.20f1");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn resolve_version_and_branch_uses_explicit_overrides() {
        let (v, b) = resolve_version_and_branch(
            &json!({"version": "2025.1.0f1", "branch": "custom/branch"}),
        )
        .unwrap();
        assert_eq!(v, "2025.1.0f1");
        assert_eq!(b, "custom/branch");
    }

    #[test]
    fn resolve_version_and_branch_detects_from_project_when_missing() {
        let tmp = unique_temp_path("resolve-vb");
        let settings = tmp.join("ProjectSettings");
        std::fs::create_dir_all(&settings).unwrap();
        std::fs::write(
            settings.join("ProjectVersion.txt"),
            "m_EditorVersion: 2023.2.20f1\n",
        )
        .unwrap();
        let (v, b) =
            resolve_version_and_branch(&json!({"projectRoot": tmp.to_str().unwrap()})).unwrap();
        assert_eq!(v, "2023.2.20f1");
        assert_eq!(b, "2023.2/staging");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn now_unix_seconds_string_is_decimal() {
        let s = now_unix_seconds_string();
        assert!(!s.is_empty());
        assert!(s.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn dir_size_empty_returns_zero() {
        let tmp = unique_temp_path("dirsize-empty");
        std::fs::create_dir_all(&tmp).unwrap();
        assert_eq!(dir_size(&tmp).unwrap(), 0);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn dir_size_sums_file_lengths() {
        let tmp = unique_temp_path("dirsize-files");
        let sub = tmp.join("a");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(tmp.join("a.txt"), b"hello").unwrap();
        std::fs::write(sub.join("b.txt"), b"world!!").unwrap();
        assert_eq!(dir_size(&tmp).unwrap(), 5 + 7);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if file_type.is_dir() {
                copy_dir_recursive(&src_path, &dst_path)?;
            } else if file_type.is_file() {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    fn setup_cache_with_fixture(label: &str) -> (PathBuf, &'static str) {
        let root = unique_temp_path(label);
        let version = "fixture-version";
        let dest = root.join("UnityCsReference").join(version);
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reference-cache");
        copy_dir_recursive(&fixture, &dest).unwrap();
        (root, version)
    }

    #[test]
    fn execute_grep_via_dispatcher_returns_hits() {
        let _guard = crate::test_env::env_lock()
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let (root, version) = setup_cache_with_fixture("grep-disp");
        let _env = EnvVarGuard::set("UNITY_CLI_CACHE_ROOT", root.to_str().unwrap());
        let value = maybe_execute_reference_tool(
            "reference_grep",
            &json!({"pattern": "class Animator", "version": version}),
        )
        .unwrap()
        .unwrap();
        assert_eq!(value["ok"], true);
        let hits = value["hits"].as_array().unwrap();
        assert!(!hits.is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn execute_view_via_dispatcher_returns_lines() {
        let _guard = crate::test_env::env_lock()
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let (root, version) = setup_cache_with_fixture("view-disp");
        let _env = EnvVarGuard::set("UNITY_CLI_CACHE_ROOT", root.to_str().unwrap());
        let value = maybe_execute_reference_tool(
            "reference_view",
            &json!({
                "path": "Runtime/Export/Animation/Animator.bindings.cs",
                "version": version,
                "startLine": 1,
                "maxLines": 3,
            }),
        )
        .unwrap()
        .unwrap();
        assert_eq!(value["ok"], true);
        assert_eq!(value["view"]["lines"].as_array().unwrap().len(), 3);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn execute_search_via_dispatcher_truncates_max_results() {
        let _guard = crate::test_env::env_lock()
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let (root, version) = setup_cache_with_fixture("search-disp");
        let _env = EnvVarGuard::set("UNITY_CLI_CACHE_ROOT", root.to_str().unwrap());
        let value = maybe_execute_reference_tool(
            "reference_search",
            &json!({
                "pattern": "class",
                "version": version,
                "maxResults": 1,
            }),
        )
        .unwrap()
        .unwrap();
        assert_eq!(value["ok"], true);
        assert_eq!(value["hits"].as_array().unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn execute_status_includes_size_bytes() {
        let _guard = crate::test_env::env_lock()
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let (root, version) = setup_cache_with_fixture("status-size");
        let _env = EnvVarGuard::set("UNITY_CLI_CACHE_ROOT", root.to_str().unwrap());
        let value = maybe_execute_reference_tool("reference_status", &json!({}))
            .unwrap()
            .unwrap();
        let versions = value["versions"].as_array().unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0]["version"], version);
        assert!(versions[0]["sizeBytes"].as_u64().unwrap() > 0);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn execute_fetch_skips_when_destination_exists_without_force() {
        let _guard = crate::test_env::env_lock()
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let (root, version) = setup_cache_with_fixture("fetch-skip");
        let _env = EnvVarGuard::set("UNITY_CLI_CACHE_ROOT", root.to_str().unwrap());
        let value = maybe_execute_reference_tool(
            "reference_fetch",
            &json!({
                "version": version,
                "branch": "fixture/branch",
                "acceptLicense": true,
            }),
        )
        .unwrap()
        .unwrap();
        assert_eq!(value["ok"], true);
        assert_eq!(value["skipped"], true);
        let _ = std::fs::remove_dir_all(&root);
    }
}
