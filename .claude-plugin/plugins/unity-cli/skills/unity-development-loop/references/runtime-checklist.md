# Runtime Checklist

## Binary Selection

- Prefer an installed `unity-cli` binary when it exists on `PATH`.
- If this repository is the current workspace and no global binary is installed, use `cargo run -- <args>`.
- Verify the binary with `unity-cli --version` before debugging a runtime loop.

## Instance Selection

- Use `unity-cli system ping` when a single active Unity Editor is expected.
- Use `unity-cli instances list` when multiple editors may be running.
- Use `unity-cli instances set-active <host:port>` only after confirming the target is `up`.

## Command Routing

- Use `unity-csharp-edit` for Unity-side code changes inside the loop.
- Use `unity-playmode-testing` or `unity-ui-automation` for narrow runtime checks.
- Use `unity-editor-tools` for console, profiler, and editor-state diagnostics.

## Evidence Defaults

- Prefer logs or state reads for non-visual behavior.
- Prefer a screenshot for final visual confirmation.
- Prefer short video only when timing or motion matters.
- Prefer profiler capture only for explicit performance questions.
