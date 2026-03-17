"""
fetch a starpls binary to use with an IDE
"""

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

_VERSION = "0.1.22"

_PLATFORMS = [
    ("linux", "amd64", "sha256-cRHNU4m6xIDwXuIK28Y4M366oG80D0eFAUl7ZREviX8="),
    ("darwin", "arm64", "sha256-FTIMQJ9V9QrAUiIaKe7SaaMnjtZ958WyRi9Yv64Ho/o="),
]

def _starpls_host_repo_impl(rctx):
    rctx.file(
        "BUILD.bazel",
        """\
exports_files(["starpls"])
""",
    )

    rctx.symlink(rctx.path(rctx.attr.binary), "starpls")

_starpls_host_repo = repository_rule(
    implementation = _starpls_host_repo_impl,
    attrs = {
        "binary": attr.label(),
    },
)

def _starpls_extension_impl(mctx):
    for os, cpu, integrity in _PLATFORMS:
        http_archive(
            name = "starpls_{}_{}".format(os, cpu),
            build_file_content = """\
filegroup(
    name = "all_files",
    srcs = ["starpls"],
    visibility = ["//visibility:public"],
)
            """,
            url = "https://github.com/withered-magic/starpls/releases/download/v{version}/starpls-{os}-{cpu}.tar.gz".format(
                cpu = cpu,
                os = os,
                version = _VERSION,
            ),
            integrity = integrity,
        )

    binary = "@starpls_{}//:starpls".format(
        "linux_amd64" if mctx.os.name == "linux" else "darwin_arm64",
    )

    _starpls_host_repo(
        name = "starpls",
        binary = binary,
    )

    return mctx.extension_metadata(
        root_module_direct_deps = [],
        root_module_direct_dev_deps = ["starpls"],
        reproducible = True,
    )

starpls = module_extension(
    implementation = _starpls_extension_impl,
)
