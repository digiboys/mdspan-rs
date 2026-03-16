#!/usr/bin/env bash

set -euo pipefail

if [ "$#" -ne 1 ]; then
    printf 'usage: %s <bazel-label>\n' "$0" >&2
    exit 2
fi

workspace_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
label="$1"

cd "$workspace_dir"

bazel build "$label" >&2

execution_root="$(bazel info execution_root 2>/dev/null)"
output_base="$(bazel info output_base 2>/dev/null)"
binary_relpath="$(
    bazel cquery --output=files "$label" 2>/dev/null |
        sed -n '1p'
)"

if [[ "$binary_relpath" == external/* ]] && [ -x "$output_base/$binary_relpath" ]; then
    printf '%s\n' "$output_base/$binary_relpath"
    exit 0
fi

printf '%s\n' "$execution_root/$binary_relpath"
