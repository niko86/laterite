//! `lat excel <in> <out>` — AGS4 ↔ XLSX (laterite-ags4-excel). Direction is inferred
//! from the output extension (`.xlsx` ⇒ export, `.ags` ⇒ import); `--export` /
//! `--import` force it when the extension is ambiguous.

use std::process::exit;

use crate::cli::ExcelArgs;

/// `Some(true)` = export (AGS4 → Excel), `Some(false)` = import, `None` =
/// ambiguous. Explicit `--export` / `--import` win; otherwise the output
/// extension decides (`.xlsx` ⇒ export, `.ags` ⇒ import).
fn direction(args: &ExcelArgs) -> Option<bool> {
    if args.export {
        return Some(true);
    }
    if args.import {
        return Some(false);
    }
    match args
        .output
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("xlsx") => Some(true),
        Some("ags") => Some(false),
        _ => None,
    }
}

pub fn run(args: &ExcelArgs) -> ! {
    let Some(export) = direction(args) else {
        eprintln!(
            "error: can't infer direction from output {} — pass --export (→ .xlsx) or --import (→ .ags)",
            args.output.display()
        );
        exit(5);
    };

    let result = if export {
        laterite_ags4_excel::ags4_to_excel(&args.input, &args.output, None)
    } else {
        laterite_ags4_excel::excel_to_ags4(&args.input, &args.output, !args.no_format_numeric)
    };

    match result {
        Ok(s) => {
            eprintln!(
                "{} {} → {} ({} sheet(s), {} row(s))",
                if export { "exported" } else { "imported" },
                args.input.display(),
                args.output.display(),
                s.sheets_written,
                s.rows_written
            );
            for w in &s.warnings {
                eprintln!("  warning: {w}");
            }
            exit(0);
        }
        Err(e) => {
            eprintln!("error: {e}");
            exit(e.exit_code());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn args(output: &str, export: bool, import: bool) -> ExcelArgs {
        ExcelArgs {
            input: PathBuf::from("in.ags"),
            output: PathBuf::from(output),
            export,
            import,
            no_format_numeric: false,
        }
    }

    #[test]
    fn direction_inferred_from_the_output_extension() {
        assert_eq!(direction(&args("out.xlsx", false, false)), Some(true));
        assert_eq!(direction(&args("out.XLSX", false, false)), Some(true)); // case-insensitive
        assert_eq!(direction(&args("out.ags", false, false)), Some(false));
        assert_eq!(direction(&args("out.dat", false, false)), None); // ambiguous → exit 5
        assert_eq!(direction(&args("noext", false, false)), None);
    }

    #[test]
    fn explicit_flags_override_the_extension() {
        assert_eq!(direction(&args("out.ags", true, false)), Some(true)); // --export wins
        assert_eq!(direction(&args("out.xlsx", false, true)), Some(false)); // --import wins
    }
}
