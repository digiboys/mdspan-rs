#!/usr/bin/env bash

set -euo pipefail

workspace_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
output_base="${workspace_dir}/bazel-rust-analyzer"

label="${1:-//...}"
if [[ "$label" = "{label}" ]]; then
    label="//..."
fi

bazelisk \
  "--output_base=${output_base}" \
  build \
  --output_groups=clippy_stdout \
  --config=clippy_diag \
  --keep_going \
  -- "$label" || true
