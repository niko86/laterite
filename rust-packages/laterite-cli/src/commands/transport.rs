//! `lat pack / unpack / lock / unlock` — the zstd + age passphrase transport
//! envelope (core's `transport` feature). Passphrases never come from a flag
//! (argv leaks into `ps` + shell history): `--password-file` →
//! `$LAT_TRANSPORT_PASSWORD` → an interactive, non-echoed TTY prompt.

use std::process::exit;

use laterite_ags4_core::transport::{self, SCRYPT_LOG_N};

use crate::cli::{LockArgs, PackArgs, PasswordArgs, UnlockArgs, UnpackArgs};

pub fn run_pack(args: &PackArgs) -> ! {
    match transport::pack(&args.input, &args.output, args.level) {
        Ok(s) => {
            eprintln!(
                "packed {} → {} ({:.1}× smaller, {:.2}s)",
                args.input.display(),
                args.output.display(),
                s.ratio,
                s.elapsed_s
            );
            exit(0);
        }
        Err(e) => {
            eprintln!("error: {e}");
            exit(e.exit_code());
        }
    }
}

pub fn run_unpack(args: &UnpackArgs) -> ! {
    match transport::unpack(&args.input, &args.output) {
        Ok(s) => {
            eprintln!(
                "unpacked {} → {} ({} bytes, {:.2}s)",
                args.input.display(),
                args.output.display(),
                s.bytes,
                s.elapsed_s
            );
            exit(0);
        }
        Err(e) => {
            eprintln!("error: {e}");
            exit(e.exit_code());
        }
    }
}

pub fn run_lock(args: &LockArgs) -> ! {
    let pw = match resolve_password(&args.password, "Passphrase to lock with: ") {
        Ok(p) => p,
        Err(code) => exit(code),
    };
    let log_n = args.log_n.unwrap_or(SCRYPT_LOG_N);
    match transport::lock(&args.input, &args.output, &pw, args.level, log_n) {
        Ok(s) => {
            eprintln!(
                "locked {} → {} ({:.1}× smaller, {:.2}s)",
                args.input.display(),
                args.output.display(),
                s.ratio,
                s.elapsed_s
            );
            exit(0);
        }
        Err(e) => {
            eprintln!("error: {e}");
            exit(e.exit_code());
        }
    }
}

pub fn run_unlock(args: &UnlockArgs) -> ! {
    let pw = match resolve_password(&args.password, "Passphrase to unlock: ") {
        Ok(p) => p,
        Err(code) => exit(code),
    };
    match transport::unlock(&args.input, &args.output, &pw) {
        Ok(s) => {
            eprintln!(
                "unlocked {} → {} ({} bytes, {:.2}s)",
                args.input.display(),
                args.output.display(),
                s.bytes,
                s.elapsed_s
            );
            exit(0);
        }
        Err(e) => {
            eprintln!("error: {e}");
            exit(e.exit_code());
        }
    }
}

/// Passphrase resolution: `--password-file` → `$LAT_TRANSPORT_PASSWORD` → a TTY
/// prompt (never echoed). Returns `Err(exit_code)` on a read failure. A trailing
/// newline in the password file is stripped so `echo pw > f` works.
fn resolve_password(args: &PasswordArgs, prompt: &str) -> Result<String, i32> {
    if let Some(p) = args.password_file.as_deref() {
        let s = std::fs::read_to_string(p).map_err(|e| {
            eprintln!("error: --password-file {}: {e}", p.display());
            3
        })?;
        return Ok(s.trim_end_matches(['\r', '\n']).to_string());
    }
    if let Ok(p) = std::env::var("LAT_TRANSPORT_PASSWORD") {
        if !p.is_empty() {
            return Ok(p);
        }
    }
    rpassword::prompt_password(prompt).map_err(|e| {
        eprintln!("error: reading passphrase: {e}");
        5
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::PasswordArgs;

    #[test]
    fn password_file_wins_and_strips_the_trailing_newline() {
        // `--password-file` is checked first (before the env var), and one
        // trailing newline is stripped so `echo pw > f` round-trips.
        let path = std::env::temp_dir().join(format!("lat-pw-test-{}.txt", std::process::id()));
        std::fs::write(&path, "hunter2\n").unwrap();
        let args = PasswordArgs {
            password_file: Some(path.clone()),
        };
        let got = resolve_password(&args, "unused");
        std::fs::remove_file(&path).ok();
        assert_eq!(got.unwrap(), "hunter2");
    }
}
