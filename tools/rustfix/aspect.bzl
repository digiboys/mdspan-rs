"""
aspect to apply clippy fixes
"""

load("@rules_rust//rust:defs.bzl", "rust_clippy_aspect")

def _rustfix_aspect_impl(target, ctx):
    phony = ctx.actions.declare_file(ctx.label.name + ".rustfix.phony")

    diagnostics_files = []
    clippy_output = getattr(target[OutputGroupInfo], "clippy_output", depset())
    for f in clippy_output.to_list():
        if f.path.endswith(".clippy.diagnostics"):
            diagnostics_files.append(f)

    args = {
        "command": "touch {phony}".format(phony = phony.path),
    }
    if diagnostics_files:
        args = {
            "command": (
                "{rustfix} {diags} && ".format(
                    rustfix = ctx.executable._rustfix.path,
                    diags = " ".join([f.path for f in diagnostics_files]),
                ) + args["command"]
            ),
            "use_default_shell_env": True,
            "execution_requirements": {
                "no-sandbox": "1",
                "no-cache": "1",
            },
        }

    ctx.actions.run_shell(
        arguments = [f.path for f in diagnostics_files],
        inputs = diagnostics_files + ctx.rule.files.srcs + (
            [ctx.executable._rustfix] if diagnostics_files else []
        ),
        outputs = [phony],
        **args
    )

    return [OutputGroupInfo(rustfix = depset([phony]))]

rustfix_aspect = aspect(
    implementation = _rustfix_aspect_impl,
    attrs = {
        "_rustfix": attr.label(
            default = "//tools/rustfix",
            executable = True,
            cfg = "exec",
        ),
    },
    requires = [rust_clippy_aspect],
    attr_aspects = ["deps", "proc_macro_deps"],
    required_aspect_providers = [OutputGroupInfo],
)
