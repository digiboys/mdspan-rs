load("@bazel_skylib//rules:common_settings.bzl", "BuildSettingInfo")
load(
    "@rules_rust//rust:rust_common.bzl",
    "CrateInfo",
    "DepInfo",
    _RUST_COMMON_PROVIDERS = "COMMON_PROVIDERS",
)
load(":emit_asm.bzl", "emit_asm_action")

def _emit_asm_aspect_impl(target, ctx):
    crate_info = target[CrateInfo]
    dep_info = target[DepInfo]
    toolchain = ctx.toolchains["@rules_rust//rust:toolchain_type"]

    if crate_info.is_test and getattr(ctx.rule.attr, "crate", None):
        lib = ctx.rule.attr.crate
        if DepInfo in lib:
            dep_info = lib[DepInfo]

    rustc_flags = list(ctx.rule.attr.rustc_flags)

    if ctx.attr._asm_opt[BuildSettingInfo].value:
        rustc_flags.extend([
            "-C",
            "opt-level={}".format(ctx.attr._asm_opt[BuildSettingInfo].value),
        ])

    asm_file = emit_asm_action(
        ctx,
        crate_info,
        dep_info,
        toolchain,
        rustc_flags,
    )

    return [
        OutputGroupInfo(
            asm_files = depset([asm_file]),
        ),
    ]

emit_asm_aspect = aspect(
    implementation = _emit_asm_aspect_impl,
    attrs = {
        "_asm_opt": attr.label(
            default = Label("//tools/emit_asm:opt"),
            doc = "rustc opt-level",
        ),
        "_rustfilt": attr.label(
            default = "@bindeps//:rustfilt__rustfilt",
            executable = True,
            cfg = "exec",
        ),
    },
    toolchains = ["@rules_rust//rust:toolchain_type"],
    required_providers = _RUST_COMMON_PROVIDERS,
)
