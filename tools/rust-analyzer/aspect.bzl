"""
aspect used by //tools/rust-analyzer:rust-analyzer-check.bash
which outputs clippy lints to stdout in json format
"""

load("@rules_rust//rust:defs.bzl", "rust_clippy_aspect")
load("@rules_rust//rust:rust_common.bzl", _RUST_COMMON_PROVIDERS = "COMMON_PROVIDERS")

def _clippy_stdout_aspect_impl(target, ctx):
    phony_files = []

    clippy_output = target[OutputGroupInfo].clippy_output
    for f in clippy_output.to_list():
        if not f.path.endswith(".clippy.diagnostics"):
            fail("clippy_output files should end with '.clippy.diagnostics'")

        phony = ctx.actions.declare_file(f.basename + ".clippy.phony")
        ctx.actions.run_shell(
            inputs = [f],
            outputs = [phony],
            command = """\
(gre '{pattern}' {diag} && cat {diag}) || touch {phony}
        """.format(
                diag = f.path,
                pattern = '^{"$message_type":"diagnostic",',
                phony = phony.path,
            ),
        )
        phony_files.append(phony)

    return [OutputGroupInfo(clippy_stdout = depset(phony_files))]

clippy_stdout_aspect = aspect(
    implementation = _clippy_stdout_aspect_impl,
    requires = [rust_clippy_aspect],
    attr_aspects = ["deps", "proc_macro_deps"],
    required_aspect_providers = [OutputGroupInfo],
    required_providers = _RUST_COMMON_PROVIDERS,
)
