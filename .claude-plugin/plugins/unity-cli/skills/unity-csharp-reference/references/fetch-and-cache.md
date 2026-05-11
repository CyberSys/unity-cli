# Fetch and Cache

Operational guide for the `unity-cli reference` cache commands.

## Cache Layout

```
~/.unity/cache/UnityCsReference/
  <unity-version>/
    .git/
    Runtime/
    Editor/
    Modules/
    .unity-cli-meta.json   # { version, branch, commit_sha, fetched_at, source_url }
```

Override the base path with `UNITY_CLI_CACHE_ROOT` (defaults to `~/.unity/cache`). Each Unity version lives in its own directory and is independent of `~/.unity/tools/` (which stores managed binaries).

## Branch Mapping

The CLI maps the active Unity version (read from `ProjectSettings/ProjectVersion.txt`) to a UnityCsReference branch using a static table:

| Unity major.minor | Branch       |
|-------------------|--------------|
| `2020.3`          | `2020.3/staging` |
| `2021.3`          | `2021.3/staging` |
| `2022.3`          | `2022.3/staging` |
| `2023.1`          | `2023.1/staging` |
| `2023.2`          | `2023.2/staging` |
| `6000.0`          | `6000.0/staging` |

Unmapped versions require `--branch <name>` so the CLI can fetch without guessing.

## Commands

### Fetch

```bash
# Auto-detect Unity version from the current project
unity-cli reference fetch --accept-license

# Explicit version + branch
unity-cli reference fetch --version 2023.2.20f1 --branch 2023.2/staging --accept-license

# Refetch an existing snapshot
unity-cli reference fetch --version 2023.2.20f1 --force --accept-license
```

Fetch uses `git clone --depth 1 --single-branch --branch <branch>` so the on-disk footprint stays close to the source tree size. The `GITHUB_TOKEN` / `GH_TOKEN` environment variable is injected as `http.extraHeader=Authorization: token ...` to relieve rate limits when set.

### Status

```bash
unity-cli reference status --output json
```

Returns `{ ok: true, versions: [ { version, branch, fetchedAt, sizeBytes, path } ... ] }`. Use this to confirm a fetch completed and to monitor disk usage.

### Clean

```bash
# Show what would be removed (LRU by mtime)
unity-cli reference clean --keep 1 --dry-run

# Actually remove old snapshots
unity-cli reference clean --keep 1
```

`clean` retains the newest `--keep` snapshots (mtime descending) and removes the rest. The CLI prints the removed paths so they can be re-cached on demand.

## Troubleshooting

- `git binary not found`: install `git` or point `PATH` at a working installation.
- `Unity Companion License`: pass `--accept-license` or export `UNITY_CLI_ACCEPT_LICENSE=1`.
- `Unity version ... not in the static branch map`: pass `--branch <name>` explicitly; consider opening an issue to extend the static table.
