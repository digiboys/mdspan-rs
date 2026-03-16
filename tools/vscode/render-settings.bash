#!/usr/bin/env bash

set -euo pipefail

workspace_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
template_path="$workspace_dir/.vscode/settings.template.json"
output_path="$workspace_dir/.vscode/settings.json"
tmp_path="$(mktemp "$workspace_dir/.vscode/settings.json.XXXXXX")"

escape_sed() {
    printf '%s\n' "$1" | sed 's/[&|]/\\&/g'
}

cleanup() {
    rm -f "$tmp_path"
}

trap cleanup EXIT

cd "$workspace_dir"

rust_analyzer_path="$("$workspace_dir/tools/vscode/find-rust-analyzer-path.bash")"
rustfmt_path="$("$workspace_dir/tools/vscode/find-rustfmt-path.bash")"
buildifier_path="$("$workspace_dir/tools/vscode/find-buildifier-path.bash")"

escaped_workspace_dir="$(escape_sed "$workspace_dir")"
escaped_rust_analyzer_path="$(escape_sed "$rust_analyzer_path")"
escaped_rustfmt_path="$(escape_sed "$rustfmt_path")"
escaped_buildifier_path="$(escape_sed "$buildifier_path")"

sed \
    -e "s|__WORKSPACE_ROOT__|$escaped_workspace_dir|g" \
    -e "s|__RUST_ANALYZER_PATH__|$escaped_rust_analyzer_path|g" \
    -e "s|__RUSTFMT_PATH__|$escaped_rustfmt_path|g" \
    -e "s|__BUILDIFIER_PATH__|$escaped_buildifier_path|g" \
    "$template_path" >"$tmp_path"
mv "$tmp_path" "$output_path"
chmod 644 "$output_path"

printf 'Wrote %s\n' "$output_path"
