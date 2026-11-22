//! Conversion from Menk PUA words to `MenkShape` sequences and UTN #57 `PositionedWrittenUnit`s.
//!
//! 孟克编码词串到 `MenkShape` 序列及 UTN #57 `PositionedWrittenUnit` 的转换逻辑。

use crate::{
    is_menk_char_only_fina, is_menk_char_only_init, menk_char_latin, menk_char_letter,
    menk_char_shapes, KLetter, MenkShape, MenkShapeError,
};
use mongol_norm::{Position, PositionedWrittenUnit};

/// Compute the positional slot (`Isol`, `Init`, `Medi`, `Fina`) of element `index` in a sub-chain of length `count`.
///
/// 计算长度为 `count` 的子链中第 `index` 个元素所处的位置槽位（`Isol`、`Init`、`Medi`、`Fina`）。
const fn slot_position(index: usize, count: usize) -> Position {
    match (index, count) {
        (0, 1) => Position::Isol,
        (0, _) => Position::Init,
        _ if index + 1 == count => Position::Fina,
        _ => Position::Medi,
    }
}

/// Push the non-empty `current` sub-chain into `chains` and reset `current`.
///
/// 若当前子链非空，则将其推入 `chains` 并清空 `current`。
fn flush_chain(chains: &mut Vec<Vec<MenkShape>>, current: &mut Vec<MenkShape>) {
    if !current.is_empty() {
        chains.push(std::mem::take(current));
    }
}

/// Transcribe a Menk PUA word (`U+E264..=U+E34F`) into its single-character Mongolian Latin
/// spelling (Rust port of PHP `MenkCode::getStandardLatin`, using unambiguous single-character
/// notation `ë`, `ŋ`, `ħ`, `ř`, `ł`, `ẑ`, `ĉ`).
///
/// 将蒙科立词串（`U+E264..=U+E34F`）转写为单字符蒙古文拉丁拼写（移植自 PHP `MenkCode::getStandardLatin`，
/// 采用无歧义单字符记法 `ë`、`ŋ`、`ħ`、`ř`、`ł`、`ẑ`、`ĉ`）。
pub fn get_standard_latin(word: &str) -> Result<String, MenkShapeError> {
    let mut out = String::with_capacity(word.len());
    for (index, character) in word.chars().enumerate() {
        let ch = menk_char_latin(character).ok_or(MenkShapeError { index, character })?;
        out.push(ch);
    }
    Ok(out)
}

/// Approximate the pure Unicode Mongolian letter sequence (`U+1820..=U+1842`, without `MVS` or `FVS`)
/// of a Menk PUA word (`U+E264..=U+E34F`), using [`KLetter::Ka`] (`'ᠺ'` `U+183A`) for `'k'`.
///
/// 获取孟克编码词串（`U+E264..=U+E34F`）对应的粗略纯蒙古文 Unicode 基础字母串（`U+1820..=U+1842`，不含 `MVS` / `FVS`，
/// 默认将 `'k'` 映射为内蒙古标准 [`KLetter::Ka`] `'ᠺ'` `U+183A`，供测试与人工校对前预填使用）。
pub fn get_standard_letters(word: &str) -> Result<String, MenkShapeError> {
    get_standard_letters_with_k(word, KLetter::default())
}

/// Approximate the pure Unicode Mongolian letter sequence (`U+1820..=U+1842`, without `MVS` or `FVS`)
/// of a Menk PUA word (`U+E264..=U+E34F`) with an explicit [`KLetter`] choice (`'ᠺ'` vs `'ᠻ'`).
///
/// 按指定的 [`KLetter`]（`'ᠺ'` `U+183A` 或 `'ᠻ'` `U+183B`）获取孟克编码词串对应的粗略纯蒙古文 Unicode 基础字母串（不含 `MVS` / `FVS`）。
pub fn get_standard_letters_with_k(
    word: &str,
    k_letter: KLetter,
) -> Result<String, MenkShapeError> {
    let mut out = String::with_capacity(word.len());
    for (index, character) in word.chars().enumerate() {
        let ch =
            menk_char_letter(character, k_letter).ok_or(MenkShapeError { index, character })?;
        out.push(ch);
    }
    Ok(out)
}

/// Extract the [`MenkShape`] sequence of a Menk PUA word (`U+E264..=U+E34F`).
///
/// 提取孟克编码词串（`U+E264..=U+E34F`）对应的 [`MenkShape`] 基础字形序列。
pub fn menk_shapes(word: &str) -> Result<Vec<MenkShape>, MenkShapeError> {
    let mut out = Vec::with_capacity(word.len());
    for (index, character) in word.chars().enumerate() {
        let shapes = menk_char_shapes(character).ok_or(MenkShapeError { index, character })?;
        out.extend_from_slice(shapes);
    }
    Ok(out)
}

/// Parse a PUA shape string returned by [`crate::get_shape`] into `Vec<MenkShape>`.
///
/// 将 [`crate::get_shape`] 生成的字形字符串解析为 `Vec<MenkShape>`。
pub fn parse_menk_shapes(shape_str: &str) -> Result<Vec<MenkShape>, MenkShapeError> {
    let mut out = Vec::with_capacity(shape_str.len());
    for (index, character) in shape_str.chars().enumerate() {
        let shape =
            MenkShape::from_shape_char(character).ok_or(MenkShapeError { index, character })?;
        out.push(shape);
    }
    Ok(out)
}

/// Convert a Menk PUA word (`U+E264..=U+E34F`) into UTN #57 [`PositionedWrittenUnit`]s.
///
/// 将孟克编码词串（`U+E264..=U+E34F`）转换为 UTN #57 [`PositionedWrittenUnit`] 序列。
///
/// Sub-chain boundaries are determined by:
/// 子链切分规则如下：
/// 1. [`is_menk_char_only_init`]: flushes any preceding sub-chain before a strictly initial glyph.
///    遇到仅用于词首（`only_init`）的字符时，在其前方切分子链。
/// 2. [`MenkShape::Aa`]: joins onto `A_INIT | A_MEDI | I | O` as `Aa:fina`; otherwise starts an
///    isolated Chachlag sub-chain (`Aa:isol`).
///    遇到 `Aa` 时，若前接 `A_INIT | A_MEDI | I | O` 则连写为 `Aa:fina`，否则独立成链为分离字尾 `Aa:isol`。
/// 3. [`is_menk_char_only_fina`]: flushes the current sub-chain after a strictly final glyph.
///    遇到仅用于词尾（`only_fina`）的字符时，在其后方切分子链。
pub fn menk_word_to_utn57(word: &str) -> Result<Vec<PositionedWrittenUnit>, MenkShapeError> {
    let mut chains: Vec<Vec<MenkShape>> = Vec::new();
    let mut current: Vec<MenkShape> = Vec::new();
    let mut total_shapes = 0usize;

    for (index, character) in word.chars().enumerate() {
        let shapes = menk_char_shapes(character).ok_or(MenkShapeError { index, character })?;
        total_shapes += shapes.len();

        if is_menk_char_only_init(character) {
            flush_chain(&mut chains, &mut current);
        }
        for &shape in shapes {
            // Connect `Aa` only after open stem/tooth/belly units (`A_INIT`, `A_MEDI`, `I`, `O`);
            // otherwise detach it as an isolated Chachlag tail (`Aa:isol`).
            // 仅在 `A_INIT`、`A_MEDI`、`I`、`O` 之后将 `Aa` 连写为词尾，其余情况断开为独立字尾 `Aa:isol`。
            if shape == MenkShape::Aa
                && !matches!(
                    current.last(),
                    Some(MenkShape::A_INIT | MenkShape::A_MEDI | MenkShape::I | MenkShape::O)
                )
            {
                flush_chain(&mut chains, &mut current);
            }
            current.push(shape);
        }
        if is_menk_char_only_fina(character) {
            flush_chain(&mut chains, &mut current);
        }
    }
    flush_chain(&mut chains, &mut current);

    let mut out = Vec::with_capacity(total_shapes);
    for chain in &chains {
        let len = chain.len();
        for (i, &shape) in chain.iter().enumerate() {
            out.push(shape.to_utn57(slot_position(i, len)));
        }
    }
    Ok(out)
}
