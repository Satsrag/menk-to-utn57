//! Menk PUA (`U+E264..=U+E34F`) shape extraction, UTN #57 positioned written-unit conversion,
//! and pure-letter Unicode Mongolian (`U+1820..=U+1842`) + minimal-FVS resolution.
//!
//! 孟克私有区编码（`U+E264..=U+E34F`）基础字形提取、UTN #57 位置书写单元（`PositionedWrittenUnit`）转换，
//! 以及结合纯蒙古文基础字母串（`U+1820..=U+1842`）求解标准 UTN #57 Unicode 与最小 FVS 序列。
//!
//! All UTN #57 types ([`WrittenUnit`], [`UnitPosition`], [`PositionedWrittenUnit`], [`Fvs`]) come
//! directly from [`mongol_norm`], which is re-exported so callers can shape or normalize results
//! without declaring an extra dependency.
//!
//! 所有 UTN #57 类型（[`WrittenUnit`]、[`UnitPosition`]、[`PositionedWrittenUnit`]、[`Fvs`]）均直接复用并重导出自
//! [`mongol_norm`]，调用方无需额外引入依赖即可直接塑形或归一化为规范 Unicode 蒙古文。

pub mod generated {
    pub mod menk_shapes;
    pub mod utn57_map;
    pub mod utn57_variants;
}
pub mod conversion;
pub mod resolve;

pub use conversion::{
    get_standard_letters, get_standard_letters_with_k, menk_shapes, menk_word_to_utn57,
    parse_menk_shapes,
};
pub use generated::menk_shapes::{
    get_char_shape, get_shape, is_menk_char_only_fina, is_menk_char_only_init, menk_char_letter,
    menk_char_shapes, MenkShape,
};
pub use generated::utn57_variants::{utn57_letter_variants, LetterVariant};
pub use mongol_norm;
pub use mongol_norm::{
    is_mongolian_letter, Fvs, Position, PositionedWrittenUnit, UnitPosition, WrittenUnit,
};
pub use resolve::{utn57_and_letters_to_unicode, ResolveError, ResolvedLetter, Solution};

use std::error::Error;
use std::fmt;

/// Regional Unicode letter choice when extracting approximate Mongolian letters from Menk `'k'` glyphs.
///
/// 从孟克 `'k'` 字形提取粗略蒙古文 Unicode 基础字母时的地区用字配置（区分内蒙古与蒙古国用字习惯）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum KLetter {
    /// `U+183A` (`'ᠺ'`, Mongolian Letter KA, `WrittenUnit::K2`), standard in Inner Mongolia (default).
    /// `U+183A`（`'ᠺ'`，`WrittenUnit::K2`），内蒙古标准用法（默认）。
    #[default]
    Ka,
    /// `U+183B` (`'ᠻ'`, Mongolian Letter KHA, `WrittenUnit::K`), used in Mongolia.
    /// `U+183B`（`'ᠻ'`，`WrittenUnit::K`），蒙古国用法。
    Kha,
}

impl KLetter {
    /// Return the Unicode Mongolian character (`'ᠺ'` or `'ᠻ'`).
    ///
    /// 返回对应的蒙古文 Unicode 字母（`'ᠺ'` 或 `'ᠻ'`）。
    pub const fn as_char(self) -> char {
        match self {
            Self::Ka => 'ᠺ',  // U+183A MONGOLIAN LETTER KA (Inner Mongolia)
            Self::Kha => 'ᠻ', // U+183B MONGOLIAN LETTER KHA (Mongolia)
        }
    }

    /// Return the UTN #57 [`WrittenUnit`] produced by this `'k'` letter.
    ///
    /// 返回该 `'k'` 字母对应的 UTN #57 [`WrittenUnit`]。
    pub const fn written_unit(self) -> WrittenUnit {
        match self {
            Self::Ka => WrittenUnit::K2,
            Self::Kha => WrittenUnit::K,
        }
    }
}

/// Error returned when an input character is outside the Menk inventory (`U+E264..=U+E34F`).
///
/// 当输入字符不在孟克字母/字形编码范围（`U+E264..=U+E34F`）内时返回的错误。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenkShapeError {
    /// Character index (0-based in `.chars()`) where the unsupported character occurred.
    /// 非法字符所在的字符下标（按 `.chars()` 从 0 计数）。
    pub index: usize,
    /// The unsupported character.
    /// 不受支持的字符。
    pub character: char,
}

impl fmt::Display for MenkShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unsupported Menk character U+{:04X} at char index {}",
            self.character as u32, self.index
        )
    }
}

impl Error for MenkShapeError {}

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme_doctests {}
