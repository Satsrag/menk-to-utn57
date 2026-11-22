//! Browser WebAssembly bindings for `menk-to-utn57`.
//!
//! `menk-to-utn57` 的浏览器 WebAssembly 绑定。

use menk_to_utn57::{
    get_standard_latin, menk_word_and_latin_to_unicode_with_k, menk_word_to_utn57, Fvs, KLetter,
    Position, ResolveError, Solution, UnitPosition,
};
use std::fmt::Write as _;
use wasm_bindgen::prelude::*;

fn push_json_str(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn position_name(pos: Position) -> &'static str {
    match pos {
        Position::Isol => "Isol",
        Position::Init => "Init",
        Position::Medi => "Medi",
        Position::Fina => "Fina",
    }
}

fn unit_position_name(pos: UnitPosition) -> &'static str {
    match pos {
        UnitPosition::Isol => "isol",
        UnitPosition::Init => "init",
        UnitPosition::Medi => "medi",
        UnitPosition::Fina => "fina",
        UnitPosition::Control => "control",
    }
}

fn fvs_name(fvs: Fvs) -> &'static str {
    match fvs {
        Fvs::Fvs1 => "FVS1",
        Fvs::Fvs2 => "FVS2",
        Fvs::Fvs3 => "FVS3",
        Fvs::Fvs4 => "FVS4",
    }
}

fn push_solutions_json(out: &mut String, solutions: &[Solution]) {
    out.push('[');
    for (s_idx, sol) in solutions.iter().enumerate() {
        if s_idx > 0 {
            out.push(',');
        }
        out.push_str("{\"unicode\":");
        push_json_str(out, &sol.unicode);
        let _ = write!(
            out,
            ",\"fvs_count\":{},\"used_greedy_fallback\":{},\"letters\":[",
            sol.fvs_count, sol.used_greedy_fallback
        );
        for (l_idx, letter) in sol.letters.iter().enumerate() {
            if l_idx > 0 {
                out.push(',');
            }
            out.push_str("{\"letter\":");
            let mut buf = [0u8; 4];
            push_json_str(out, letter.letter.encode_utf8(&mut buf));
            out.push_str(",\"position\":");
            push_json_str(out, position_name(letter.position));
            out.push_str(",\"written\":[");
            for (u_idx, unit) in letter.written.iter().enumerate() {
                if u_idx > 0 {
                    out.push(',');
                }
                push_json_str(out, unit.as_str());
            }
            out.push_str("],\"fvs\":");
            match letter.fvs {
                Some(f) => push_json_str(out, fvs_name(f)),
                None => out.push_str("null"),
            }
            out.push('}');
        }
        out.push_str("]}");
    }
    out.push(']');
}

fn error_json(err: &ResolveError) -> String {
    let mut out = String::from("{\"ok\":false,\"error\":{");
    match err {
        ResolveError::InvalidMenkChar(e) => {
            out.push_str("\"kind\":\"InvalidMenkChar\",");
            let _ = write!(out, "\"index\":{},\"char\":", e.index);
            let mut buf = [0u8; 4];
            push_json_str(&mut out, e.character.encode_utf8(&mut buf));
            let _ = write!(out, ",\"codepoint\":\"U+{:04X}\"", e.character as u32);
        }
        ResolveError::InvalidLatin { index, character } => {
            out.push_str("\"kind\":\"InvalidLatin\",");
            let _ = write!(out, "\"index\":{index},\"char\":");
            let mut buf = [0u8; 4];
            push_json_str(&mut out, character.encode_utf8(&mut buf));
            let _ = write!(out, ",\"codepoint\":\"U+{:04X}\"", *character as u32);
        }
        ResolveError::InvalidLetter { index, character } => {
            out.push_str("\"kind\":\"InvalidLetter\",");
            let _ = write!(out, "\"index\":{index},\"char\":");
            let mut buf = [0u8; 4];
            push_json_str(&mut out, character.encode_utf8(&mut buf));
            let _ = write!(out, ",\"codepoint\":\"U+{:04X}\"", *character as u32);
        }
        ResolveError::TargetNormalization(_) => {
            out.push_str("\"kind\":\"TargetNormalization\"");
        }
        ResolveError::ShapeMismatch => {
            out.push_str("\"kind\":\"ShapeMismatch\"");
        }
    }
    out.push_str(",\"message\":");
    push_json_str(&mut out, &err.to_string());
    out.push_str("}}");
    out
}

/// Transcribe a Menk PUA word (`U+E264..=U+E34F`) into its default single-character Latin spelling.
#[wasm_bindgen]
pub fn infer_standard_latin(menk_word: &str) -> Result<String, JsValue> {
    get_standard_latin(menk_word).map_err(|err| JsValue::from_str(&err.to_string()))
}

/// Resolve a Menk PUA word with an optional custom Latin override into a JSON payload containing
/// extracted UTN #57 written units, inferred Latin, and all minimal-FVS [`Solution`]s (or a
/// structured error object).
#[wasm_bindgen]
pub fn resolve_menk_to_unicode(menk_word: &str, latin: Option<String>, use_kha: bool) -> String {
    let k_letter = if use_kha { KLetter::Kha } else { KLetter::Ka };

    let units = match menk_word_to_utn57(menk_word) {
        Ok(u) => u,
        Err(e) => return error_json(&ResolveError::InvalidMenkChar(e)),
    };
    let inferred_latin = match get_standard_latin(menk_word) {
        Ok(l) => l,
        Err(e) => return error_json(&ResolveError::InvalidMenkChar(e)),
    };

    let res = menk_word_and_latin_to_unicode_with_k(menk_word, latin.as_deref(), k_letter);
    let solutions = match res {
        Ok(s) => s,
        Err(e) => return error_json(&e),
    };

    let used_latin = latin.as_deref().unwrap_or(&inferred_latin);

    let mut out = String::from("{\"ok\":true,\"inferred_latin\":");
    push_json_str(&mut out, &inferred_latin);
    out.push_str(",\"used_latin\":");
    push_json_str(&mut out, used_latin);
    out.push_str(",\"units\":[");
    for (idx, u) in units.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        let label = format!("{}:{}", u.unit.as_str(), unit_position_name(u.position));
        push_json_str(&mut out, &label);
    }
    out.push_str("],\"solutions\":");
    push_solutions_json(&mut out, &solutions);
    out.push('}');
    out
}
