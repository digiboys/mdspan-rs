visibility("//tools/emit_asm/...")

def _sysroot(toolchain):
    """Derive --sysroot from the rustc executable path.

    rules_rust lays out its toolchains as:
        external/<repo>/bin/rustc
        external/<repo>/lib/rustlib/<triple>/lib/*.rlib

    The sysroot is everything before /bin/rustc.
    """
    path = toolchain.rustc.path
    idx = path.rfind("/bin/")
    if idx < 0:
        fail("asm: cannot derive --sysroot from rustc path: " + path)
    return path[:idx]

def emit_asm_action(ctx, crate_info, dep_info, toolchain, rustc_flags):
    """Declare a rustc --emit=asm action and return output files."""

    mangled_asm_file = ctx.actions.declare_file(
        "{}__asm.mangled.s".format(crate_info.name),
    )
    asm_file = ctx.actions.declare_file(
        "{}__asm.s".format(crate_info.name),
    )

    args = ctx.actions.args()

    args.add("--emit=asm")
    args.add("--crate-name", crate_info.name)
    args.add("--edition", crate_info.edition or "2015")

    # rust_test wraps its crate in a test harness; --test is the correct flag,
    # not --crate-type bin (which would omit the harness and fail to compile).
    if crate_info.is_test:
        args.add("--test")
    else:
        args.add("--crate-type", crate_info.type)

    # codegen-units=1 is critical: without it rustc emits one .s per CGU
    # (mycrate.cgu-0.s, mycrate.cgu-1.s, ...) and we would need to declare
    # all of them as outputs without knowing their count ahead of time.
    args.add("-C", "codegen-units=1")

    # Strip DWARF; it adds hundreds of lines of noise with no readability gain.
    args.add("-C", "debuginfo=0")

    args.add("--sysroot", _sysroot(toolchain))

    dep_outputs = []
    seen_externs = {}
    dep_dirs = {}

    for dep_crate in dep_info.transitive_crates.to_list():
        if dep_crate.name not in seen_externs:
            seen_externs[dep_crate.name] = True
            args.add("--extern", "{name}={path}".format(
                name = dep_crate.name,
                path = dep_crate.output.path,
            ))
            dep_outputs.append(dep_crate.output)
            dep_dirs[dep_crate.output.dirname] = True

    for d in dep_dirs.keys():
        args.add("-Ldependency=" + d)

    args.add_all(crate_info.cfgs, format_each = "--cfg=%s")
    args.add_all(rustc_flags)
    args.add(crate_info.root)
    args.add("-o", mangled_asm_file)

    ctx.actions.run_shell(
        command = """\
{rustc} --remap-path-prefix=$(pwd)= \"$@\"
        """.format(rustc = toolchain.rustc.path),
        tools = [toolchain.rustc],
        arguments = [args],
        inputs = depset(
            direct = (
                [crate_info.root] +
                crate_info.srcs.to_list() +
                dep_outputs
            ),
            transitive = [
                crate_info.compile_data,
                toolchain.rust_std,
                toolchain.rustc_lib,
                dep_info.transitive_proc_macro_data,
            ],
        ),
        outputs = [mangled_asm_file],
        env = dict(crate_info.rustc_env),
        use_default_shell_env = True,
        mnemonic = "RustEmitAsm",
        progress_message = "emitting asm for {}".format(
            crate_info.root.short_path,
        ),
    )

    ctx.actions.run_shell(
        inputs = depset(
            direct = [mangled_asm_file],
            transitive = [
                ctx.attr._rustfilt.files,
                crate_info.srcs,
            ],
        ),
        outputs = [asm_file],
        command = """\
cat {infile} | {rustfilt} > {outfile}
        """.format(
            infile = mangled_asm_file.path,
            outfile = asm_file.path,
            rustfilt = ctx.executable._rustfilt.path,
        ),
        use_default_shell_env = True,
        mnemonic = "RustDemangleAsm",
        progress_message = "demangling asm for {}".format(
            crate_info.root.short_path,
        ),
    )

    return asm_file
