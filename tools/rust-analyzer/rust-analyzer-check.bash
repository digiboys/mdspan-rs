#!/usr/bin/env bash

set -euo pipefail

workspace_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"

label="${1:-//...}"
if [[ "$label" = "{label}" ]]; then
    label="//..."
fi

bazelisk \
  "--output_base=${workspace_dir}/bazel-rust-analyzer" \
  build \
  --aspects=//tools/rust-analyzer:aspect.bzl%clippy_stdout_aspect \
  --@rules_rust//rust/settings:error_format=json \
  --@rules_rust//rust/settings:capture_clippy_output=true \
  --output_groups=clippy_stdout \
  --config=clippy_settings \
  --keep_going \
  -- "$label" || true
