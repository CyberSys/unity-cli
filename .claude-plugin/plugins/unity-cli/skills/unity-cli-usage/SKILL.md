---
name: unity-cli-usage
description: Bootstrap the unity-cli toolchain for Unity Editor automation. Use when verifying the unity-cli binary, choosing between typed subcommands and raw tool calls, switching active Unity instances, or troubleshooting host/port and install-mode issues. Do not use once a more specific Unity workflow skill applies; defer to `unity-scene-create`, `unity-csharp-edit`, `unity-editor-tools`, or another domain skill instead.
allowed-tools: Bash(unity-cli:*), Read, Grep, Glob
user-invocable: false
metadata:
  author: akiojin
  version: 0.3.0
  category: foundation
  triggers:
    - bootstrap
    - install
    - connect
    - ping
    - instance
  siblings:
    - unity-scene-create
    - unity-csharp-edit
    - unity-editor-tools
---

# unity-cli Usage

Bootstrap the unity-cli toolchain so other Unity skills can run reliably. This is a foundation skill that loads automatically when no other unity-* skill matches a connection or install question.

## Use When

- The user asks how to verify or install `unity-cli`.
- The user needs help with `system ping`, `instances list`, or `instances set-active`.
- The user is unsure whether a typed subcommand or `raw` is appropriate.
- A workflow is blocked on host/port selection, install mode, or connection troubleshooting.

## Do Not Use When

- A more specific skill clearly matches the task. For scene authoring, use `unity-scene-create`. For C# edits, use `unity-csharp-edit`. For Editor state inspection, use `unity-editor-tools`.
- The request only inspects or edits project files without invoking Unity.

## Preferred Flow

1. Detect the binary: prefer an installed `unity-cli` on `PATH`; fall back to `cargo run --` from the repo.
2. Verify reachability with `unity-cli system ping`.
3. When multiple editors may run, call `unity-cli instances list` and pick the target with `unity-cli instances set-active <host:port>`.
4. Prefer typed subcommands (`system`, `scene`, `instances`); use `raw` only when no typed command exists.
5. Use `--output json` for chained automation.

```bash
if ! command -v unity-cli >/dev/null 2>&1; then
  if [ -f Cargo.toml ] && grep -q '^name = "unity-cli"' Cargo.toml; then
    echo "unity-cli not installed globally. Use: cargo run -- <args>"
  else
    echo "Install: cargo install --path . (or download a release binary)"
    exit 1
  fi
fi
unity-cli --version
unity-cli system ping
```

## Examples

- "Check whether unity-cli can reach my Unity Editor." → run `unity-cli system ping`.
- "Switch to the Unity instance running on port 6401." → `unity-cli instances list --ports 6400,6401` then `unity-cli instances set-active localhost:6401`.
- "Should this workflow use `scene create` or a raw tool call?" → prefer typed `unity-cli scene create` when one exists; otherwise hand off to `raw`.

## References

- [runtime-checklist.md](references/runtime-checklist.md): binary selection, instance selection, command routing, CI environment notes.
