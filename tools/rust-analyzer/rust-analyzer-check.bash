#!/usr/bin/env bash

set -euo pipefail

workspace_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
output_base="${workspace_dir}/bazel-rust-analyzer"

label="${1:-//...}"
if [[ "$label" = "{label}" ]]; then
    label="//..."
fi

stamp_file="$(mktemp)"
trap 'rm -f "$stamp_file"' EXIT

build_status=0
bazelisk \
  "--output_base=${output_base}" \
  build \
  --aspects=//tools/rust-analyzer:aspect.bzl%clippy_stdout_aspect \
  --@rules_rust//rust/settings:error_format=json \
  --@rules_rust//rust/settings:capture_clippy_output=true \
  --output_groups=clippy_stdout \
  --config=clippy_settings \
  --keep_going \
  -- "$label" || build_status=$?

if (( build_status != 0 )); then
    while IFS= read -r -d '' diagnostics_file; do
        if grep -q '^{"$message_type":"diagnostic",' "$diagnostics_file"; then
            cat "$diagnostics_file"
        fi
    done < <(find "$output_base" -type f -name '*.clippy.diagnostics' -newer "$stamp_file" -print0 | sort -z)
fi
