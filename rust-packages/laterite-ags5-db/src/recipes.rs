//! Recipe catalogue parser.
//!
//! The canonical `recipes.md` lives in this crate's `data/` directory
//! and is embedded into the Rust binary via `include_str!` so the
//! read-side CLI has no runtime filesystem dependency.
//!
//! Parse strategy: split on `^## ` headers. The first chunk is the
//! preamble (skipped). Each subsequent chunk's first line is the recipe
//! name; we keep only those that match the `[a-z][a-z0-9-]+` slug shape
//! so the "Pattern-by-question-shape index" section is skipped.

use serde::Serialize;

// S3a: data/ moved to ../laterite-ags4-core/data/.
const RECIPES_MD: &str = include_str!("../../laterite-ags4-core/data/recipes.md");

#[derive(Debug, Clone, Serialize)]
pub struct Recipe {
    pub name: String,
    pub shape: String,
    pub body: String,
}

pub fn load_all() -> Vec<Recipe> {
    let mut out = Vec::new();
    let mut chunks = split_on_h2(RECIPES_MD).into_iter();
    // Drop the preamble (text before the first `## ` header).
    chunks.next();
    for chunk in chunks {
        let mut lines = chunk.lines();
        let name = match lines.next() {
            Some(l) => l.trim().to_string(),
            None => continue,
        };
        if !is_slug(&name) {
            continue;
        }
        let body: String = chunk
            .split_once('\n')
            .map(|x| x.1)
            .unwrap_or("")
            .trim()
            .to_string();
        let shape = extract_shape(&body);
        out.push(Recipe { name, shape, body });
    }
    out
}

fn split_on_h2(s: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();
    for line in s.split_inclusive('\n') {
        if let Some(rest) = line.strip_prefix("## ") {
            parts.push(std::mem::take(&mut current));
            // Strip the leading "## " from the header so the first line of
            // each subsequent chunk is just the recipe name.
            current.push_str(rest);
        } else {
            current.push_str(line);
        }
    }
    parts.push(current);
    parts
}

fn is_slug(s: &str) -> bool {
    let mut chars = s.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return false,
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    let mut len = 1;
    for c in chars {
        if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return false;
        }
        len += 1;
    }
    len >= 2
}

fn extract_shape(body: &str) -> String {
    for line in body.lines() {
        if let Some(rest) = line.trim().strip_prefix("**Shape:**") {
            return rest.trim().to_string();
        }
    }
    String::new()
}

pub fn substitute_group(body: &str, group_code: &str) -> String {
    body.replace("<test_group>", &group_code.to_lowercase())
        .replace("<GROUP>", &group_code.to_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_recipes() {
        let recipes = load_all();
        let names: Vec<&str> = recipes.iter().map(|r| r.name.as_str()).collect();
        // These four are documented as load-bearing in the contractor spec.
        // If the upstream recipes.md is restructured and any of them drop
        // out the parity tests will tell us — this assertion is just to
        // catch a totally-empty parse (e.g. include_str! pointing at the
        // wrong file).
        assert!(names.contains(&"depth-band-join"), "names={:?}", names);
        assert!(!recipes.iter().any(|r| r.name.contains(' ')));
    }
}
