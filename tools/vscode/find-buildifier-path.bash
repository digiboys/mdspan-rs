#!/usr/bin/env bash

set -euo pipefail

workspace_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"

exec "$workspace_dir/tools/vscode/find-bazel-binary-path.bash" \
    @buildifier_prebuilt//:buildifier
