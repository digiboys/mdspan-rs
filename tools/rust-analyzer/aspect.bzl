"""
aspect used by //tools/rust-analyzer:rust-analyzer-check.bash
which outputs clippy lints to stdout in json format
"""

load("@rules_rust//rust:defs.bzl", "rust_clippy_aspect")

def _clippy_stdout_aspect_impl(target, ctx):
    phony_files = []

    clippy_output = getattr(target[OutputGroupInfo], "clippy_output", depset())
    for f in clippy_output.to_list():
        if f.path.endswith(".clippy.diagnostics"):
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
)
