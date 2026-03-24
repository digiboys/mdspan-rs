use std::{collections::BTreeMap, path::PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use regex::Regex;
use serde::Deserialize;

#[derive(Parser)]
#[command(about = "Check assembly output against a set of patterns")]
struct Args {
    /// Path to the .s assembly file (reads from stdin if omitted)
    #[arg(long)]
    asm: Option<PathBuf>,

    /// Path to the TOML config file
    #[arg(long)]
    config: PathBuf,

    /// Print passing checks as well as failing ones
    #[arg(long, default_value_t = false)]
    verbose: bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(default)]
    check: Vec<Check>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Check {
    #[serde(default)]
    functions: Vec<String>,
    #[serde(default)]
    exclude: Vec<String>,
    #[serde(default)]
    must_contain: Vec<String>,
    #[serde(default)]
    must_not_contain: Vec<String>,
}

struct CompiledCheck {
    source: Check,
    functions_re: Vec<Regex>,
    exclude_re: Vec<Regex>,
    must_contain_re: Vec<Regex>,
    must_not_contain_re: Vec<Regex>,
}

fn compile_regexes(patterns: &[String], flag: &str) -> Result<Vec<Regex>> {
    patterns
        .iter()
        .map(|p| Regex::new(p).with_context(|| format!("invalid {flag} regex: {p:?}")))
        .collect()
}

impl CompiledCheck {
    fn compile(check: Check) -> Result<Self> {
        if check.functions.is_empty() {
            bail!("check must specify functions to search");
        }
        if check.must_contain.is_empty() && check.must_not_contain.is_empty() {
            bail!(
                "check for {:?} must have at least one must_contain or must_not_contain",
                check.functions
            );
        }

        let functions_re = compile_regexes(&check.functions, "functions")?;
        let exclude_re = compile_regexes(&check.exclude, "exclude")?;
        let must_contain_re = compile_regexes(&check.must_contain, "must_contain")?;
        let must_not_contain_re = compile_regexes(&check.must_not_contain, "must_not_contain")?;

        Ok(Self {
            source: check,
            functions_re,
            exclude_re,
            must_contain_re,
            must_not_contain_re,
        })
    }

    fn apply<'a>(&self, functions: &'a BTreeMap<&str, Vec<&str>>) -> Vec<(&'a str, String)> {
        let mut failures = vec![];

        for (name, lines) in functions {
            if !self.functions_re.iter().any(|r| r.is_match(name)) {
                continue;
            }

            if self.exclude_re.iter().any(|r| r.is_match(name)) {
                continue;
            }

            for re in &self.must_contain_re {
                if !lines.iter().any(|l| re.is_match(l)) {
                    failures.push((
                        *name,
                        format!("must_contain {:?} — no matching line found", re.as_str()),
                    ));
                }
            }

            for re in &self.must_not_contain_re {
                for line in lines {
                    if re.is_match(line) {
                        failures.push((
                            *name,
                            format!("must_not_contain {:?} matched: {}", re.as_str(), line.trim()),
                        ));
                    }
                }
            }
        }

        failures
    }
}

fn parse_functions<'a>(asm: &'a str) -> BTreeMap<&'a str, Vec<&'a str>> {
    let mut chunks: Vec<Vec<&str>> = vec![];

    for line in asm.lines() {
        let is_label = line.ends_with(':') && !line.starts_with(|c: char| c.is_whitespace());

        if is_label {
            chunks.push(vec![line]);
        } else if let Some(chunk) = chunks.last_mut() {
            chunk.push(line);
        }
    }

    for i in (1..chunks.len()).rev() {
        if chunks[i][0].trim_end_matches(':').starts_with("Lloh") {
            let lloh = chunks.remove(i);
            chunks[i - 1].extend(lloh);
        }
    }

    chunks
        .into_iter()
        .map(|mut chunk| {
            let name = chunk.remove(0).trim_end_matches(':');
            (name, chunk)
        })
        .collect()
}

fn main() -> Result<()> {
    let args = Args::parse();

    let config_str =
        std::fs::read_to_string(&args.config).with_context(|| format!("failed to read {}", args.config.display()))?;
    let config: Config = toml::from_str(&config_str).context("failed to parse config TOML")?;

    let asm = match args.asm {
        Some(path) => std::fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?,
        None => std::io::read_to_string(std::io::stdin()).context("failed to read assembly from stdin")?,
    };

    let functions = parse_functions(&asm);

    if args.verbose {
        eprintln!("Parsed {} functions", functions.len());
    }

    let checks = config
        .check
        .into_iter()
        .map(CompiledCheck::compile)
        .collect::<Result<Vec<_>>>()?;

    let mut any_failure = false;

    for (i, check) in checks.iter().enumerate() {
        let failures = check.apply(&functions);

        if failures.is_empty() {
            if args.verbose {
                println!("PASS  check[{i}] functions={:?}", check.source.functions);
            }
        } else {
            any_failure = true;
            for (fn_name, msg) in &failures {
                println!(
                    "FAIL  check[{i}] functions={:?}  fn={fn_name}  {msg}",
                    check.source.functions
                );
            }
        }
    }

    if any_failure {
        bail!("one or more checks failed");
    }

    println!("All checks passed.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_ASM: &str = r#"
.type _std::rt::lang_start,@function
_std::rt::lang_start:
    .cfi_startproc
    sub sp, sp, #32
    bl  _std::rt::lang_start_internal
    ret
    .cfi_endproc

.type _mod_layout::tests::test_no_panic,@function
_mod_layout::tests::test_no_panic:
    .cfi_startproc
    sub sp, sp, #64
    mov w0, #42
    ret
    .cfi_endproc

.type _mod_layout::tests::test_panics,@function
_mod_layout::tests::test_panics:
    .cfi_startproc
    sub sp, sp, #64
    bl  _core::panicking::panic_fmt
    .cfi_endproc

.type _mod_layout::stride,@function
_mod_layout::stride:
    .cfi_startproc
    mov w0, #1
    ret
    .cfi_endproc

_mod_layout::tests::test_layout_right_mapping_stride_out_of_bounds:
    .cfi_startproc
    sub     sp, sp, #64
    .cfi_def_cfa_offset 64
    stp     x29, x30, [sp, #48]
    add     x29, sp, #48
    .cfi_def_cfa w29, 16
    .cfi_offset w30, -8
    .cfi_offset w29, -16
Lloh6:
    adrp    x8, l___unnamed_2@PAGE
Lloh7:
    add     x8, x8, l___unnamed_2@PAGEOFF
    mov     w9, #1
    stp     x8, x9, [sp]
    mov     w8, #8
    stp     xzr, xzr, [sp, #24]
    str     x8, [sp, #16]
Lloh8:
    adrp    x1, l___unnamed_4@PAGE
Lloh9:
    add     x1, x1, l___unnamed_4@PAGEOFF
    mov     x0, sp
    bl      _core::panicking::panic_fmt
    .loh AdrpAdd    Lloh8, Lloh9
    .loh AdrpAdd    Lloh6, Lloh7
    .cfi_endproc

    .p2align        2
"#;

    #[test]
    fn test_parse_functions() {
        let fns = parse_functions(SAMPLE_ASM);
        assert!(fns.contains_key("_std::rt::lang_start"));
        assert!(fns.contains_key("_mod_layout::tests::test_no_panic"));
        assert!(fns.contains_key("_mod_layout::tests::test_panics"));
        assert!(fns.contains_key("_mod_layout::stride"));
        assert!(fns.contains_key("_mod_layout::tests::test_layout_right_mapping_stride_out_of_bounds"));
    }

    #[test]
    fn test_must_not_contain_passes() {
        let fns = parse_functions(SAMPLE_ASM);

        let check = CompiledCheck::compile(Check {
            functions: vec!["_mod_layout::tests::test_no_panic".to_string()],
            exclude: vec![],
            must_contain: vec![],
            must_not_contain: vec!["panic_fmt".to_string()],
        })
        .unwrap();
        assert!(check.apply(&fns).is_empty());
    }

    #[test]
    fn test_must_not_contain_fails() {
        let fns = parse_functions(SAMPLE_ASM);

        let check = CompiledCheck::compile(Check {
            functions: vec!["_mod_layout::tests::test_panics".to_string()],
            exclude: vec![],
            must_contain: vec![],
            must_not_contain: vec!["panic_fmt".to_string()],
        })
        .unwrap();
        assert!(!check.apply(&fns).is_empty());

        let check = CompiledCheck::compile(Check {
            functions: vec!["_mod_layout::tests::test_layout_right_mapping_stride_out_of_bounds".to_string()],
            exclude: vec![],
            must_contain: vec![],
            must_not_contain: vec!["bl[[:space:]]+_core::panicking::panic_fmt".to_string()],
        })
        .unwrap();
        assert!(!check.apply(&fns).is_empty());
    }

    #[test]
    fn test_exclude() {
        let fns = parse_functions(SAMPLE_ASM);
        let check = CompiledCheck::compile(Check {
            functions: vec!["_mod_layout::tests::.*".to_string()],
            exclude: vec![
                "_mod_layout::tests::test_panics".to_string(),
                "_mod_layout::tests::test_layout_right_mapping_stride_out_of_bounds".to_string(),
            ],
            must_contain: vec![],
            must_not_contain: vec!["panic_fmt".to_string()],
        })
        .unwrap();
        assert!(check.apply(&fns).is_empty());
    }

    #[test]
    fn test_must_contain_passes() {
        let fns = parse_functions(SAMPLE_ASM);

        let check = CompiledCheck::compile(Check {
            functions: vec!["_mod_layout::stride".to_string()],
            exclude: vec![],
            must_contain: vec!["ret".to_string()],
            must_not_contain: vec![],
        })
        .unwrap();
        assert!(check.apply(&fns).is_empty());

        let check = CompiledCheck::compile(Check {
            functions: vec!["_mod_layout::tests::test_layout_right_mapping_stride_out_of_bounds".to_string()],
            exclude: vec![],
            must_contain: vec!["bl[[:space:]]+_core::panicking::panic_fmt".to_string()],
            must_not_contain: vec![],
        })
        .unwrap();
        assert!(check.apply(&fns).is_empty());
    }

    #[test]
    fn test_must_contain_fails() {
        let fns = parse_functions(SAMPLE_ASM);
        let check = CompiledCheck::compile(Check {
            functions: vec!["_mod_layout::stride".to_string()],
            exclude: vec![],
            must_contain: vec!["panic_fmt".to_string()],
            must_not_contain: vec![],
        })
        .unwrap();
        assert!(!check.apply(&fns).is_empty());
    }

    #[test]
    fn test_missing_must() {
        assert!(
            CompiledCheck::compile(Check {
                functions: vec!["_mod_layout::.*".to_string()],
                exclude: vec![],
                must_contain: vec![],
                must_not_contain: vec![],
            })
            .is_err()
        );
    }
}
