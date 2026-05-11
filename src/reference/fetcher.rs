use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};

pub const UNITY_CS_REFERENCE_URL: &str =
    "https://github.com/Unity-Technologies/UnityCsReference.git";
const LICENSE_ENV_VAR: &str = "UNITY_CLI_ACCEPT_LICENSE";
const GITHUB_TOKEN_ENV_VARS: &[&str] = &["GITHUB_TOKEN", "GH_TOKEN"];

pub fn build_clone_args(url: &str, branch: &str, dest: &Path, depth: u32) -> Vec<String> {
    vec![
        "--depth".to_string(),
        depth.to_string(),
        "--single-branch".to_string(),
        "--branch".to_string(),
        branch.to_string(),
        url.to_string(),
        dest.display().to_string(),
    ]
}

pub fn require_license_accepted(flag: bool) -> Result<()> {
    if flag {
        return Ok(());
    }
    if let Ok(value) = std::env::var(LICENSE_ENV_VAR) {
        if !value.trim().is_empty() && value != "0" {
            return Ok(());
        }
    }
    Err(anyhow!(
        "UnityCsReference is distributed under the Unity Companion License. Pass --accept-license or set {}=1 to confirm consent before fetching.",
        LICENSE_ENV_VAR
    ))
}

pub fn ensure_git_available() -> Result<()> {
    Command::new("git")
        .arg("--version")
        .output()
        .context("git binary not found in PATH; install git or use a future zip fallback")?;
    Ok(())
}

fn github_token() -> Option<String> {
    for key in GITHUB_TOKEN_ENV_VARS {
        if let Ok(v) = std::env::var(key) {
            if !v.trim().is_empty() {
                return Some(v);
            }
        }
    }
    None
}

pub fn run_clone(
    url: &str,
    branch: &str,
    dest: &Path,
    depth: u32,
    accept_license: bool,
) -> Result<()> {
    require_license_accepted(accept_license)?;
    if ensure_git_available().is_ok() {
        return run_clone_via_git(url, branch, dest, depth);
    }
    // git binary unavailable: fall back to archive download.
    fetch_via_zip(branch, dest)
}

fn run_clone_via_git(url: &str, branch: &str, dest: &Path, depth: u32) -> Result<()> {
    let mut cmd = Command::new("git");
    if let Some(token) = github_token() {
        cmd.arg("-c")
            .arg(format!("http.extraHeader=Authorization: token {token}"));
    }
    cmd.arg("clone");
    for arg in build_clone_args(url, branch, dest, depth) {
        cmd.arg(arg);
    }
    let status = cmd
        .status()
        .with_context(|| format!("failed to spawn git clone for {url}"))?;
    if !status.success() {
        return Err(anyhow!("git clone exited with status {status}"));
    }
    Ok(())
}

/// Branch -> GitHub archive zip URL.
pub fn archive_url_for_branch(branch: &str) -> String {
    format!(
        "https://github.com/Unity-Technologies/UnityCsReference/archive/refs/heads/{branch}.zip"
    )
}

pub fn fetch_via_zip(branch: &str, dest: &Path) -> Result<()> {
    let url = archive_url_for_branch(branch);
    let agent = ureq::Agent::new_with_defaults();
    let mut request = agent.get(&url);
    if let Some(token) = github_token() {
        request = request.header("Authorization", format!("token {token}"));
    }
    let response = request
        .call()
        .with_context(|| format!("failed to GET {url}"))?;
    let mut body = response.into_body();
    let mut buffer = Vec::new();
    body.as_reader()
        .read_to_end(&mut buffer)
        .with_context(|| format!("failed to read archive body for {url}"))?;
    extract_zip_to(&buffer, dest)
}

pub fn extract_zip_to(archive_bytes: &[u8], dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)
        .with_context(|| format!("failed to create destination {}", dest.display()))?;
    let cursor = Cursor::new(archive_bytes);
    let mut archive =
        zip::ZipArchive::new(cursor).context("failed to open zip archive from buffer")?;
    let prefix = detect_top_level_prefix(&mut archive)?;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .with_context(|| format!("failed to read zip entry {i}"))?;
        let raw_name = file.name().to_string();
        let stripped = match strip_prefix(&raw_name, &prefix) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };
        if stripped.contains("..") {
            return Err(anyhow!(
                "zip entry escapes destination via parent segments: {raw_name}"
            ));
        }
        let target_path: PathBuf = dest.join(&stripped);
        if raw_name.ends_with('/') {
            fs::create_dir_all(&target_path)
                .with_context(|| format!("failed to create directory {}", target_path.display()))?;
            continue;
        }
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create parent {}", parent.display()))?;
        }
        let mut out = fs::File::create(&target_path)
            .with_context(|| format!("failed to open {} for writing", target_path.display()))?;
        std::io::copy(&mut file, &mut out)
            .with_context(|| format!("failed to write {}", target_path.display()))?;
    }
    Ok(())
}

fn detect_top_level_prefix(archive: &mut zip::ZipArchive<Cursor<&[u8]>>) -> Result<String> {
    if archive.is_empty() {
        return Err(anyhow!("zip archive is empty"));
    }
    let first = archive.by_index(0).context("zip archive has no entries")?;
    let name = first.name();
    if let Some(idx) = name.find('/') {
        Ok(name[..=idx].to_string())
    } else {
        Ok(String::new())
    }
}

fn strip_prefix<'a>(name: &'a str, prefix: &str) -> Option<&'a str> {
    if prefix.is_empty() {
        Some(name)
    } else {
        name.strip_prefix(prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;
    use std::sync::Mutex;

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
        fn unset(key: &'static str) -> Self {
            let previous = env::var(key).ok();
            env::remove_var(key);
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

    fn env_lock() -> &'static Mutex<()> {
        crate::test_env::env_lock()
    }

    #[test]
    fn build_clone_args_emits_shallow_single_branch() {
        let dest = PathBuf::from("/tmp/unity-cs-reference/2023.2.20f1");
        let args = build_clone_args(UNITY_CS_REFERENCE_URL, "2023.2/staging", &dest, 1);
        assert_eq!(
            args,
            vec![
                "--depth".to_string(),
                "1".to_string(),
                "--single-branch".to_string(),
                "--branch".to_string(),
                "2023.2/staging".to_string(),
                UNITY_CS_REFERENCE_URL.to_string(),
                dest.display().to_string(),
            ]
        );
    }

    #[test]
    fn license_required_when_flag_false_and_env_unset() {
        let _guard = env_lock().lock().unwrap_or_else(|p| p.into_inner());
        let _env = EnvVarGuard::unset("UNITY_CLI_ACCEPT_LICENSE");
        let err = require_license_accepted(false).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("Unity Companion License"));
        assert!(msg.contains("--accept-license"));
    }

    #[test]
    fn license_ok_when_flag_true() {
        let _guard = env_lock().lock().unwrap_or_else(|p| p.into_inner());
        let _env = EnvVarGuard::unset("UNITY_CLI_ACCEPT_LICENSE");
        require_license_accepted(true).expect("license OK when flag set");
    }

    #[test]
    fn license_ok_when_env_set() {
        let _guard = env_lock().lock().unwrap_or_else(|p| p.into_inner());
        let _env = EnvVarGuard::set("UNITY_CLI_ACCEPT_LICENSE", "1");
        require_license_accepted(false).expect("license OK when env set");
    }

    #[test]
    fn license_rejects_zero_value() {
        let _guard = env_lock().lock().unwrap_or_else(|p| p.into_inner());
        let _env = EnvVarGuard::set("UNITY_CLI_ACCEPT_LICENSE", "0");
        let err = require_license_accepted(false).unwrap_err();
        assert!(format!("{err:#}").contains("Unity Companion License"));
    }

    #[test]
    fn github_token_returns_none_when_env_unset() {
        let _guard = env_lock().lock().unwrap_or_else(|p| p.into_inner());
        let _g1 = EnvVarGuard::unset("GITHUB_TOKEN");
        let _g2 = EnvVarGuard::unset("GH_TOKEN");
        assert!(github_token().is_none());
    }

    #[test]
    fn github_token_skips_empty_and_picks_first_non_empty() {
        let _guard = env_lock().lock().unwrap_or_else(|p| p.into_inner());
        let _g1 = EnvVarGuard::set("GITHUB_TOKEN", "");
        let _g2 = EnvVarGuard::set("GH_TOKEN", "ghp_test_value");
        assert_eq!(github_token().as_deref(), Some("ghp_test_value"));
    }

    #[test]
    fn ensure_git_available_succeeds_in_test_env() {
        ensure_git_available().expect("git is expected on dev/CI environment");
    }

    #[test]
    fn run_clone_rejects_when_license_not_accepted() {
        let _guard = env_lock().lock().unwrap_or_else(|p| p.into_inner());
        let _env = EnvVarGuard::unset("UNITY_CLI_ACCEPT_LICENSE");
        let dest = PathBuf::from("/tmp/unity-cli-reference-clone-license-guard");
        let err = run_clone(UNITY_CS_REFERENCE_URL, "2023.2/staging", &dest, 1, false).unwrap_err();
        assert!(format!("{err:#}").contains("Unity Companion License"));
    }

    #[test]
    fn archive_url_uses_unity_cs_reference_org() {
        let url = archive_url_for_branch("2023.2/staging");
        assert!(url.starts_with(
            "https://github.com/Unity-Technologies/UnityCsReference/archive/refs/heads/"
        ));
        assert!(url.ends_with("/2023.2/staging.zip"));
    }

    fn build_sample_zip() -> Vec<u8> {
        use std::io::Write;
        use zip::write::SimpleFileOptions;
        let mut buf = Vec::new();
        {
            let mut writer = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            let options =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            writer
                .start_file("UnityCsReference-fixture/Editor/Foo.cs", options)
                .unwrap();
            writer.write_all(b"public class Foo {}\n").unwrap();
            writer
                .start_file("UnityCsReference-fixture/Runtime/Bar/Bar.cs", options)
                .unwrap();
            writer.write_all(b"public class Bar {}\n").unwrap();
            writer.finish().unwrap();
        }
        buf
    }

    #[test]
    fn extract_zip_to_strips_top_level_prefix_and_writes_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let buf = build_sample_zip();
        extract_zip_to(&buf, tmp.path()).unwrap();
        let foo = tmp.path().join("Editor/Foo.cs");
        let bar = tmp.path().join("Runtime/Bar/Bar.cs");
        assert!(foo.exists(), "Editor/Foo.cs should exist");
        assert!(bar.exists(), "Runtime/Bar/Bar.cs should exist");
        let contents = std::fs::read_to_string(&foo).unwrap();
        assert!(contents.contains("class Foo"));
    }

    #[test]
    fn extract_zip_to_rejects_entries_with_parent_segments() {
        use std::io::Write;
        use zip::write::SimpleFileOptions;
        let mut buf = Vec::new();
        {
            let mut writer = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            let options =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            writer
                .start_file("UnityCsReference-fixture/../escape.cs", options)
                .unwrap();
            writer.write_all(b"bad").unwrap();
            writer.finish().unwrap();
        }
        let tmp = tempfile::TempDir::new().unwrap();
        let err = extract_zip_to(&buf, tmp.path()).unwrap_err();
        assert!(format!("{err:#}").contains(".."));
    }
}
