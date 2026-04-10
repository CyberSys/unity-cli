---
name: gh-skills-sync
description: Sync the gh-* automation skills from the upstream skills repository into Codex and project skill directories. Use when the user asks to refresh, check, or pin `gh-fix-ci`, `gh-fix-issue`, `gh-pr`, or `gh-pr-check` in this repo or the global Codex install. Do not use for updating unity-cli workflow skills; edit those skill folders directly.
allowed-tools: Bash, Read, Grep, Glob
metadata:
  author: akiojin
  version: 0.2.0
  category: maintenance
---

# gh-skills-sync

Use this skill when asked to update or synchronize GitHub-related skills:

- `gh-fix-ci`
- `gh-fix-issue`
- `gh-pr`
- `gh-pr-check`

Read `references/sync-behavior.md` when you need details about source selection, backup behavior, or project-vs-global targeting.

## Use When

- The user wants to refresh the local copy of the `gh-*` automation skills.
- The task is to check whether project and global copies are in sync.
- The user wants to pin synchronization to a specific upstream ref.

## Do Not Use When

- The task is about editing `unity-cli` skills in `.claude-plugin/plugins/unity-cli/skills`.
- The request is to use a `gh-*` skill rather than sync it.

## Recommended Flow

1. Run `--check` first when the user only asked for status or when the current state is unclear.
2. Decide whether to target project skills, global skills, or both.
3. Run the sync command with any required `--ref`.
4. Re-run `--check` after changes and report what moved.

## Command

Run from repository root:

```bash
scripts/sync-gh-skills.sh
```

## Typical Options

```bash
# check only (no write)
scripts/sync-gh-skills.sh --check

# update project skills only
scripts/sync-gh-skills.sh --target project

# pin a specific ref
scripts/sync-gh-skills.sh --ref main
```

## Examples

- "Check whether our local `gh-*` skills are behind upstream."
- "Update only the project copy of the GitHub automation skills."
- "Sync the `gh-*` skills from a pinned upstream ref."

## Behavior

- Default source repo: `akiojin/skills`
- Sync targets:
  - Global: `${CODEX_HOME:-~/.codex}/skills`
  - Project: `./.codex/skills`
- Existing destination skill directories are backed up under `.backup/gh-sync-<timestamp>/`.
- Fetch method is automatic:
  - Prefer `skill-installer` if available.
  - Fallback to `git sparse-checkout` so it also works in Claude Code environments.

## Common Issues

- The wrong target was updated: use `--target project` or the appropriate target selector before syncing.
- The user only wanted a dry check: start with `--check`.
- Updated skills do not appear immediately: restart Codex or Claude Code after the sync.
- Use `scripts/sync-gh-skills.sh --check` after any update to verify consistency.
