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
  --extra_toolchains=//toolchain:local_rust_stable_clippy_no_fail \
  --aspects=//tools/rust-analyzer:aspect.bzl%clippy_stdout_aspect \
  --@rules_rust//rust/settings:error_format=json \
  --@rules_rust//rust/settings:capture_clippy_output=true \
  --output_groups=clippy_stdout \
  --config=clippy_settings \
  --keep_going \
  -- "$label" || true
