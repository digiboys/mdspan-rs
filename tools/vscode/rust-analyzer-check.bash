#!/usr/bin/env bash

set -euo pipefail

workspace_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"

label="${1:-}"

if [ -z "$label" ] || [ "$label" = "{label}" ]; then
    label="//..."
fi

startup_options=(
    "--output_base=$workspace_dir/bazel-rust-analyzer"
)

common_args=(
    "--@rules_rust//rust/settings:rustc_output_diagnostics=true"
    "--@rules_rust//rust/settings:clippy_output_diagnostics=true"
    "--@rules_rust//rust/settings:capture_clippy_output=true"
    "--@rules_rust//rust/settings:error_format=json"
    "--output_groups=+clippy_output"
)

build_output="$(mktemp)"
cleanup() {
    rm -f "$build_output"
}
trap cleanup EXIT

build_status=0
if ! bazel "${startup_options[@]}" \
    build "${common_args[@]}" "$label" >"$build_output" 2>&1; then
    build_status=$?
fi

grep '^{.*}$' "$build_output" || true

if [ "$build_status" -eq 0 ]; then
    execution_root="$(bazel "${startup_options[@]}" info execution_root 2>/dev/null)"

    mapfile -t outputs < <(
        bazel "${startup_options[@]}" \
            cquery "${common_args[@]}" --output=files "$label" 2>/dev/null |
            grep '\.clippy\.diagnostics$' || true
    )

    for output in "${outputs[@]}"; do
        if [ -f "$execution_root/$output" ]; then
            cat "$execution_root/$output"
        fi
    done
fi

if grep -q '^{.*}$' "$build_output"; then
    exit 0
fi

exit "$build_status"
