# Contributing to the unity-cli skills plugin

This plugin ships workflow-oriented skills for the Rust `unity-cli` to both
Claude Code and Codex CLI. All skills under
`.claude-plugin/plugins/unity-cli/skills/unity-*` follow **Skill Contract v1**
(SPEC: [#160](https://github.com/akiojin/unity-cli/issues/160)). The contract
is enforced by `unity-cli skills lint` and gated in CI.

## Quick start

```bash
# 1. Create a new skill directory
mkdir -p .claude-plugin/plugins/unity-cli/skills/unity-<domain>-<action>/references

# 2. Copy the runtime checklist (required for every skill)
cp .claude-plugin/plugins/unity-cli/skills/unity-cli-usage/references/runtime-checklist.md \
   .claude-plugin/plugins/unity-cli/skills/unity-<domain>-<action>/references/

# 3. Author SKILL.md following the template below

# 4. Refresh the symlinks for Claude Code and Codex
ln -sf "../../.claude-plugin/plugins/unity-cli/skills/unity-<domain>-<action>" \
       ".claude/skills/unity-<domain>-<action>"
ln -sf "../../.claude-plugin/plugins/unity-cli/skills/unity-<domain>-<action>" \
       ".agents/skills/unity-<domain>-<action>"

# 5. Validate
cargo run -- skills lint --severity error
```

## Naming

- Skill name: `unity-<domain>-<action>` (kebab-case, lowercase, ≤ 64 chars).
- Directory name and frontmatter `name` must match.
- Reserved words `claude` and `anthropic` are forbidden.

## Required frontmatter

```yaml
---
name: unity-<domain>-<action>
description: <verb> <object> with unity-cli. Use when <user phrasing>. Do not use for <other case>; use `<sibling>` instead.
allowed-tools: Bash(unity-cli:*), Read, Grep, Glob
metadata:
  author: <name>
  version: 0.3.0          # SemVer; bump on contract changes
  category: scenes        # foundation | scenes | assets | code | editor | input | testing | prefabs | ui
  triggers:               # singular, lowercase, source of truth for R08/R22
    - scene
    - bootstrap
  siblings:               # cross-listed bidirectionally with R12
    - unity-gameobject-edit
    - unity-prefab-workflow
---
```

Optional fields:

- `user-invocable: false` — only allowed on `category: foundation` skills.
- `metadata` may include other custom keys; the linter ignores them.

## Description writing rules

Anthropic best practices apply (see `docs/skills.md` for the full mapping):

| Rule | What | Why |
|---|---|---|
| Hard limit | ≤ 1024 characters | Anthropic platform limit |
| Soft limit | At least one `metadata.triggers` entry inside the first 250 characters | Listing truncation; front-load the main trigger |
| Person | Third-person / imperative (`Manage`, `Drive`, `Configure`) | First and second-person patterns confuse model selection |
| Use when / Do not use | Both must appear | Disambiguates against siblings |
| Sibling mention | At least one `metadata.siblings` entry must appear in the description text | Anchors the boundary so users see the alternative |

Bad description:

```yaml
description: I can help you manage Unity prefabs.   # ❌ first-person, no Use when, no sibling
```

Good description:

```yaml
description: Manage Unity prefab assets with unity-cli. Use when the user asks to create a prefab from a scene object, open a prefab in edit mode, or instantiate a prefab. Do not use for general scene editing; use `unity-gameobject-edit` or `unity-scene-create` instead.
```

## SKILL.md body structure

The body must contain these `## H2` headings in this order:

1. `## Use When`
2. `## Do Not Use When`
3. `## Preferred Flow`
4. `## Examples`
5. `## References`

Constraints:

- ≤ 500 lines
- `## References` must contain at least one markdown link
- Reference files must be 1 level deep (no nested links between references)
- Reference files > 100 lines must include `## Table of Contents` in the first 15 lines
- No time-sensitive language (`as of`, `until next release`, `currently`, `現時点では`)

## Required references

Every skill must include:

- `references/runtime-checklist.md` — connection / instance / install-mode prerequisites. Copy from `unity-cli-usage` and customize only when necessary.

Skill-specific references go in the same directory as siblings.

## Allowed tools

```yaml
allowed-tools: Bash(unity-cli:*), Read, Grep, Glob
```

Only `unity-csharp-edit` may also include `Edit, Write`. The bare `Bash` token
is rejected; use the prefix-bound `Bash(unity-cli:*)`.

## Validating before commit

```bash
cargo run -- skills lint --severity error
```

The linter applies all 22 rules (R01..R22) and exits non-zero on any
violation. CI runs the same command — see `.github/workflows/ci.yml` and
`.github/workflows/release.yml`.

## Pull request checklist

- [ ] `cargo run -- skills lint --severity error` exits 0
- [ ] `cargo test --all-targets` is green
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `.claude/skills/<name>` symlink updated
- [ ] `.agents/skills/<name>` symlink updated
- [ ] `metadata.version` bumped if behavior or contract changed
- [ ] `metadata.siblings` cross-listed bidirectionally
- [ ] Linked Issue references the contract SPEC ([#160](https://github.com/akiojin/unity-cli/issues/160)) when modifying skill structure
