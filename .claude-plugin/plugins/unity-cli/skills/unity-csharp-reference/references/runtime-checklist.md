# Runtime Checklist

Run these checks before the first `reference fetch` in a new environment or worktree.

## Prerequisites

- `git` is on `PATH`. Verify with `git --version`.
- The target project has `ProjectSettings/ProjectVersion.txt` so Unity version detection can run automatically. Without a project, pass `--version <X.Y.Zfn>` and `--branch <branch>` explicitly.
- Network connectivity to `github.com` is available. For rate-limited environments, export `GITHUB_TOKEN` or `GH_TOKEN` before fetching.
- Disk budget: budget roughly 400-600 MB per cached Unity version. Use `unity-cli reference status --output json` to inspect current usage.

## License Acceptance

UnityCsReference is distributed under the Unity Companion License. The CLI refuses to fetch without explicit consent:

- Pass `--accept-license` to `reference fetch`, or
- Export `UNITY_CLI_ACCEPT_LICENSE=1` for non-interactive sessions.

Local caches are for personal reference only. Do not redistribute the cached source.

## First-Run Verification

```bash
unity-cli reference fetch --accept-license
unity-cli reference status --output json
```

Expected: `status` reports a single cached entry with `version`, `branch`, `fetchedAt`, `sizeBytes`, and `path`. Re-running `fetch` without `--force` skips with a notice.

## Recovery

- Fetch failed half-way: rerun with `--force` to wipe the version directory and clone again.
- Cache directory unreadable: remove `~/.unity/cache/UnityCsReference/<version>` and re-fetch.
- Disk pressure: `unity-cli reference clean --keep 1` keeps the newest snapshot and removes the rest.
