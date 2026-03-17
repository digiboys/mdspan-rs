load("@platforms//host:constraints.bzl", "HOST_CONSTRAINTS")

STABLE_TOOL_REPOS = [
    "rust_linux_x86_64__x86_64-unknown-linux-gnu__stable_tools",
    "rust_macos_aarch64__aarch64-apple-darwin__stable_tools",
]

_SOURCE_TOOLS = STABLE_TOOL_REPOS[int("@platforms//os:osx" in HOST_CONSTRAINTS)]

def _clippy_no_fail_toolchain_tools_impl(rctx):
    source_build = rctx.path(rctx.attr.source_build_file)
    source_root = source_build.dirname

    strip_root_prefix = lambda path: str(path)[len(str(source_root)) + 1:]

    result = rctx.execute(["find", str(source_root), "-not", "-type", "d"])
    if result.return_code != 0:
        fail("find failed: " + result.stderr)

    for line in result.stdout.splitlines():
        src = rctx.path(line)
        rel = strip_root_prefix(src)

        if rel not in ("BUILD.bazel", "WORKSPACE.bazel", "bin/clippy-driver"):
            rctx.symlink(src, rel)

    rctx.file(
        "bin/clippy-driver",
        """\
#!/usr/bin/env bash
{real_clippy_driver} "$@" || true
""".format(
            real_clippy_driver = "{}/bin/clippy-driver".format(source_root),
        ),
        executable = True,
    )

    rctx.file(
        "BUILD.bazel",
        rctx.read(source_build),
    )

clippy_no_fail_toolchain_tools = repository_rule(
    implementation = _clippy_no_fail_toolchain_tools_impl,
    attrs = {
        "source_build_file": attr.label(
            default = Label("@{}//:BUILD.bazel".format(_SOURCE_TOOLS)),
            allow_single_file = True,
        ),
    },
)
