//! Menk PUA (`U+E264..=U+E34F`) shape extraction and UTN #57 positioned written-unit conversion.
//!
//! 孟克私有区编码（`U+E264..=U+E34F`）基础字形提取与 UTN #57 位置书写单元（`PositionedWrittenUnit`）转换库。
//!
//! All UTN #57 types ([`WrittenUnit`], [`UnitPosition`], [`PositionedWrittenUnit`]) come directly
//! from [`mongol_norm`], which is re-exported so callers can normalize converted units to canonical
//! Unicode Mongolian without declaring an extra dependency.
//!
//! 所有 UTN #57 类型（[`WrittenUnit`]、[`UnitPosition`]、[`PositionedWrittenUnit`]）均直接复用并重导出自
//! [`mongol_norm`]，调用方无需额外引入依赖即可直接归一化为规范 Unicode 蒙古文。

pub mod generated {
    pub mod menk_shapes;
    pub mod utn57_map;
}
pub mod conversion;

pub use conversion::{menk_shapes, menk_word_to_utn57, parse_menk_shapes};
pub use generated::menk_shapes::{
    get_char_shape, get_shape, is_menk_char_only_fina, is_menk_char_only_init, menk_char_shapes,
    MenkShape,
};
pub use mongol_norm;
pub use mongol_norm::{Position, PositionedWrittenUnit, UnitPosition, WrittenUnit};

use std::error::Error;
use std::fmt;

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
