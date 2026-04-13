# Sync Behavior

## Targets

- Project sync updates `./.codex/skills`.
- Global sync updates `${CODEX_HOME:-~/.codex}/skills`.
- Choose the smallest target that satisfies the request.

## Safe Sequence

1. Run `scripts/sync-gh-skills.sh --check` if current state is unknown.
2. Run the sync with the intended target and optional `--ref`.
3. Re-run `--check`.
4. Report which skills changed and where backups were written.

## Failure Modes

- A stale global install can mask project changes.
- Backups live under `.backup/gh-sync-<timestamp>/`.
- Restart the host tool if updated skills do not appear immediately.
