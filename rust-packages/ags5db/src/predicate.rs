//! `--where field<op>value` parsing.
//!
//! Mirrors Python `_cli_predicate.parse_where`. The 2-char ops (`!=`, `<=`,
//! `>=`) are checked before 1-char (`<`, `>`, `=`) so `samp_top>=5` parses
//! as `(samp_top, >=, 5)`, not `(samp_top>, =, 5)`.
//!
//! Values are coerced int -> float -> string so `loca_id=BH01` and
//! `samp_top>=5.0` both work.

use ags5_core::error::CliError;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Predicate {
    pub field: String,
    pub op: Op,
    pub value: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Lt,
    Le,
    Eq,
    Ne,
    Gt,
    Ge,
}

impl Op {
    pub fn as_sql(self) -> &'static str {
        match self {
            Op::Lt => "<",
            Op::Le => "<=",
            Op::Eq => "=",
            Op::Ne => "!=",
            Op::Gt => ">",
            Op::Ge => ">=",
        }
    }
}

/// Find the first operator in `s` (2-char ops have priority).
fn find_op(s: &str) -> Option<(usize, usize, Op)> {
    // Walk byte-by-byte. The fields and ASCII ops are 1-byte; UTF-8 multi-byte
    // chars in values get scanned past harmlessly since we only match ASCII.
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Two-char ops first.
        if i + 1 < bytes.len() {
            match &bytes[i..i + 2] {
                b"!=" => return Some((i, i + 2, Op::Ne)),
                b"<=" => return Some((i, i + 2, Op::Le)),
                b">=" => return Some((i, i + 2, Op::Ge)),
                _ => {}
            }
        }
        match bytes[i] {
            b'<' => return Some((i, i + 1, Op::Lt)),
            b'>' => return Some((i, i + 1, Op::Gt)),
            b'=' => return Some((i, i + 1, Op::Eq)),
            _ => {}
        }
        i += 1;
    }
    None
}

pub fn parse(raw: &str) -> Result<Predicate, CliError> {
    let (s, e, op) = find_op(raw).ok_or_else(|| CliError::Predicate {
        arg: raw.to_string(),
        reason: "expected 'field<op>value' where <op> is one of =, !=, <, <=, >, >=".into(),
    })?;
    let field = raw[..s].trim();
    let value_str = raw[e..].trim();
    if field.is_empty() {
        return Err(CliError::Predicate {
            arg: raw.to_string(),
            reason: format!("empty field name before {:?}", op.as_sql()),
        });
    }
    if value_str.is_empty() {
        return Err(CliError::Predicate {
            arg: raw.to_string(),
            reason: format!("empty value after {:?}", op.as_sql()),
        });
    }

    // Coerce int -> float -> string. AGS IDs like "BH01" pass through as text.
    let value = if let Ok(i) = value_str.parse::<i64>() {
        Value::from(i)
    } else if let Ok(f) = value_str.parse::<f64>() {
        Value::from(f)
    } else {
        Value::from(value_str.to_string())
    };

    Ok(Predicate {
        field: field.to_string(),
        op,
        value,
    })
}

pub fn parse_many(raw: &[String]) -> Result<Vec<Predicate>, CliError> {
    raw.iter().map(|s| parse(s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_char_ops_beat_one_char() {
        let p = parse("samp_top>=5").unwrap();
        assert_eq!(p.field, "samp_top");
        assert_eq!(p.op, Op::Ge);
        assert_eq!(p.value, Value::from(5));
    }

    #[test]
    fn string_value_passes_through() {
        let p = parse("loca_id=BH01").unwrap();
        assert_eq!(p.value, Value::from("BH01"));
    }

    #[test]
    fn float_value_coerced() {
        let p = parse("depth<=1.5").unwrap();
        assert_eq!(p.value, Value::from(1.5));
    }

    #[test]
    fn missing_op_errors() {
        assert!(parse("noopvalue").is_err());
    }
}
