#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/release/collect-closing-issues.sh --range <git-range>

Collect auto-close issue lines for the release PR body.

Rules:
- Prefer each merged PR's `## Closing Issues` section when present
- Otherwise fall back to closing keywords in the PR body
- Also include closing keywords from commit bodies in the release range
- Exclude SPEC issues (`gwt-spec` label or `gwt-spec:` title prefix)

Output:
- `Closes #N` lines, one per issue, sorted and deduplicated
- `None` when no closable non-SPEC issues are found
EOF
}

RANGE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --range)
      RANGE="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

[[ -n "${RANGE}" ]] || {
  echo "ERROR: --range is required" >&2
  usage >&2
  exit 1
}

extract_keyword_issue_numbers() {
  grep -Eio '(close[sd]?|fix(e[sd])?|resolve[sd]?)[^#]*#[0-9]+' \
    | grep -Eo '#[0-9]+' \
    | tr -d '#'
}

extract_closing_issues_section() {
  awk '
    BEGIN { in_section = 0 }
    /^##[[:space:]]+Closing Issues[[:space:]]*$/ { in_section = 1; next }
    /^##[[:space:]]+/ {
      if (in_section) {
        exit
      }
      next
    }
    in_section { print }
  '
}

body_declares_closing_section() {
  grep -Eq '^##[[:space:]]+Closing Issues[[:space:]]*$'
}

is_spec_issue() {
  local issue_number="$1"
  local issue_json

  issue_json="$(gh issue view "${issue_number}" --json labels,title)"
  jq -e '
    ([.labels[]?.name] | index("gwt-spec")) != null
    or (.title | startswith("gwt-spec:"))
  ' >/dev/null <<<"${issue_json}"
}

merged_pr_numbers="$(
  git log --merges --pretty=%s "${RANGE}" \
    | sed -En 's/^Merge pull request #([0-9]+).*$/\1/p' \
    | sort -u
)"

raw_issue_numbers="$(
  {
    for pr_number in ${merged_pr_numbers}; do
      pr_body="$(gh pr view "${pr_number}" --json body --jq '.body')"
      if body_declares_closing_section <<<"${pr_body}"; then
        extract_closing_issues_section <<<"${pr_body}" | extract_keyword_issue_numbers || true
      else
        printf '%s\n' "${pr_body}" | extract_keyword_issue_numbers || true
      fi
    done

    git log --format=%B "${RANGE}" | extract_keyword_issue_numbers || true
  } | awk 'NF' | sort -nu
)"

closing_lines="$(
  for issue_number in ${raw_issue_numbers}; do
    if is_spec_issue "${issue_number}"; then
      continue
    fi
    printf 'Closes #%s\n' "${issue_number}"
  done | sort -u
)"

if [[ -z "${closing_lines}" ]]; then
  echo "None"
else
  printf '%s\n' "${closing_lines}"
fi
