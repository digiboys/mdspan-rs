#!/usr/bin/env bash

workspace_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"

bazelisk \
    run \
    @rules_rust//tools/rust_analyzer:discover_bazel_rust_project -- \
    --bazel=bazelisk \
    --bazel_startup_option=--output_base="$workspace_dir/bazel-rust-analyzer" \
    --bazel_arg=--watchfs \
    ${1:+"$1"} 2>/dev/null
