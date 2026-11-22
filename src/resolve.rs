//! Resolve all valid minimal-FVS UTN #57 Unicode Mongolian solutions for a word from UTN #57 shapes and pure Unicode Mongolian letters.
//!
//! 根据单词的 UTN #57 位置书写单元（`PositionedWrittenUnit`）与纯蒙古文 Unicode 基础字母串（`U+1820..=U+1842`，不含 `FVS` / `MVS`），
//! 从目标字形自动推导词尾分写 `Chachlag` 的 `MVS`（`U+180E` + `ᠠ`/`ᠡ`），枚举全部合法的逐字母子字形切分路径，并为每条路径计算出严格最少 `FVS` 的标准 Unicode 解法。

use crate::generated::utn57_variants::utn57_pinning_variants;
use mongol_norm::{Fvs, Locale, Position, PositionedWrittenUnit, Shaper, WrittenUnit};
use std::error::Error;
use std::fmt::{self, Display};
use std::sync::OnceLock;

/// Per-letter shape and FVS assignment inside a [`Solution`].
///
/// 单个匹配解法（[`Solution`]）中，单个蒙古文字母对应的位置、子字形与 FVS 明细。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResolvedLetter {
    /// Unicode Mongolian base letter (`U+1820..=U+1842`).
    /// 蒙古文 Unicode 基础字母（`U+1820..=U+1842`）。
    pub letter: char,
    /// Positional slot of this letter (`Isol`, `Init`, `Medi`, `Fina`).
    /// 该字母所处的位置槽位（`Isol`、`Init`、`Medi`、`Fina`）。
    pub position: Position,
    /// Expanded UTN #57 written-unit slice assigned to this letter (e.g. `&[A]` vs `&[A, A]`).
    /// 当前解法分配给该字母的展开后 UTN #57 书写单元序列（如 `&[A]` 或 `&[A, A]`）。
    pub written: &'static [WrittenUnit],
    /// Minimal FVS attached to this letter (`None` when the contextual default already produces `written`).
    /// 当前字母最终保留的最小 FVS（若默认上下文规则已能产出 `written` 则为 `None`）。
    pub fvs: Option<Fvs>,
}

/// One valid minimal-FVS word solution matching the target UTN #57 glyph shape.
///
/// 匹配目标 UTN #57 字形的一条完整解法（包含最终最少 FVS 的 Unicode 字符串及逐字母明细）。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Solution {
    /// Canonical Unicode Mongolian word with minimal `FVS` (and Chachlag `MVS` `U+180E` when applicable).
    /// 最终生成的带最少 `FVS`（及词尾分写 `Chachlag` 所需的 `MVS` `U+180E`）的蒙古文 Unicode 字符串。
    pub unicode: String,
    /// Total number of `FVS` characters inserted in [`Self::unicode`].
    /// [`Self::unicode`] 中实际附加的 `FVS` 总数。
    pub fvs_count: usize,
    /// Per-letter breakdown in 1-to-1 correspondence with the input `letters`.
    /// 与输入 `letters` 按顺序一一对应的逐字母子字形与 `FVS` 明细。
    pub letters: Vec<ResolvedLetter>,
    /// Whether FVS minimization fell back to greedy right-to-left stripping after exhausting the
    /// exact subset-search budget (`false` = proven globally minimal FVS; `true` = valid minimal
    /// rendering produced via greedy fallback, flagged for optional manual review).
    /// 是否在精确子集搜索预算耗尽后回退到了从右向左贪心剥离（`false` = 精确搜索保证全局最少 FVS；`true` = 走了贪心回退，字形仍严格正确，可供后续人工复核是否有多余 FVS）。
    pub used_greedy_fallback: bool,
}

/// Error returned when resolving UTN #57 [`PositionedWrittenUnit`]s + pure Mongolian letters into Unicode Mongolian.
///
/// 将 UTN #57 [`PositionedWrittenUnit`] 序列与纯蒙古文字母串求解为标准蒙古文 Unicode 序列时返回的错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolveError {
    /// Input `letters` string contains a character outside `U+1820..=U+1842` at 0-based char `index`.
    /// 输入的 `letters` 在第 `index` 个字符（从 0 计数）处包含不在基础字母范围 `U+1820..=U+1842` 内的字符。
    InvalidLetter {
        /// 0-based character index in `letters.chars()`.
        /// `letters.chars()` 中的字符下标（从 0 开始）。
        index: usize,
        /// Offending character.
        /// 触发错误的字符。
        character: char,
    },
    /// Target positioned written-unit sequence was rejected by [`mongol_norm::Shaper`].
    /// 目标位置书写单元序列未通过 [`mongol_norm::Shaper`] 校验。
    TargetNormalization(mongol_norm::Error),
    /// No FVS assignment for the given Mongolian letters matches the target UTN #57 glyph shape.
    /// 在给定的蒙古文字母序列下，没有任何 FVS 组合能够生成与目标一致的 UTN #57 字形。
    ShapeMismatch,
}

impl Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLetter { index, character } => write!(
                f,
                "invalid Mongolian letter {:?} (U+{:04X}) at char index {} (expected U+1820..=U+1842)",
                character, *character as u32, index
            ),
            Self::TargetNormalization(err) => {
                write!(f, "failed to normalize target UTN #57 units: {err}")
            }
            Self::ShapeMismatch => {
                write!(
                    f,
                    "Mongolian letters cannot produce the target UTN #57 shape"
                )
            }
        }
    }
}

impl Error for ResolveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TargetNormalization(err) => Some(err),
            Self::InvalidLetter { .. } | Self::ShapeMismatch => None,
        }
    }
}

impl From<mongol_norm::Error> for ResolveError {
    fn from(err: mongol_norm::Error) -> Self {
        Self::TargetNormalization(err)
    }
}

pub use mongol_norm::is_mongolian_letter;

/// Shared process-wide [`Shaper`] instance (`Shaper` is `Send + Sync`).
///
/// 进程级共享的 [`Shaper`] 实例（`Shaper` 实现了 `Send + Sync`，只需初始化一次）。
fn default_shaper() -> &'static Shaper {
    static SHAPER: OnceLock<Shaper> = OnceLock::new();
    SHAPER.get_or_init(|| Shaper::new(Locale::Mng))
}

/// Compute all valid minimal-FVS [`Solution`]s for a word from its UTN #57 [`PositionedWrittenUnit`]s
/// and pure Unicode Mongolian letters (`U+1820..=U+1842`, without `MVS` or `FVS`).
///
/// - Enumerates **every** valid per-letter written-unit decomposition that shapes to the target glyph
///   sequence (including trailing/standalone Chachlag `MVS` `U+180E` + `'ᠠ'`/`'ᠡ'`) without pruning
///   alternative FVS decompositions.
/// - Solves each decomposition for its strictly minimal `FVS` assignment while preserving each letter's
///   assigned written units.
/// - Returns a non-empty `Vec<Solution>` sorted by `fvs_count` ascending (fewest `FVS` first), or
///   [`ResolveError::ShapeMismatch`] if no decomposition matches.
/// - Accepts both Inner Mongolia `'ᠺ'` (`U+183A`, `K2`) and Mongolia `'ᠻ'` (`U+183B`, `K`) directly
///   in `letters`, aligning them automatically with `K` / `K2` glyph units.
///
/// 根据单词的 UTN #57 [`PositionedWrittenUnit`] 字形序列与纯蒙古文基础字母串（`U+1820..=U+1842`，不含 `MVS` / `FVS`）计算全部匹配解法：
/// - 枚举能够拼出目标字形（含词尾或独立分写 `Chachlag` 的 `MVS` `U+180E` + `'ᠠ'`/`'ᠡ'`）的**全部**合法逐字母子字形切分路径（不做 0-FVS 剪枝）；
/// - 对每条切分路径求解保持逐字母子字形不变的严格最少 `FVS` 组合；
/// - 返回按 `fvs_count` 升序排列（最少 `FVS` 在前）的非空 `Vec<Solution>`；若无任何合法解法则返回 [`ResolveError::ShapeMismatch`]；
/// - `letters` 中可直接使用内蒙古标准 `'ᠺ'`（`U+183A`）或蒙古国标准 `'ᠻ'`（`U+183B`），自动与字形中的 `K` / `K2` 对齐。
pub fn utn57_and_letters_to_unicode(
    units: &[PositionedWrittenUnit],
    letters: &str,
) -> Result<Vec<Solution>, ResolveError> {
    let letter_vec = parse_mongolian_letters(letters)?;
    if units.is_empty() {
        if !letter_vec.is_empty() {
            return Err(ResolveError::ShapeMismatch);
        }
        return Ok(vec![Solution {
            unicode: String::new(),
            fvs_count: 0,
            letters: Vec::new(),
            used_greedy_fallback: false,
        }]);
    }
    if letter_vec.is_empty() {
        return Err(ResolveError::ShapeMismatch);
    }

    let shaper = default_shaper();

    // Align `WrittenUnit::K` / `WrittenUnit::K2` in `units` left-to-right with `'ᠺ'` (`U+183A`, `K2`)
    // and `'ᠻ'` (`U+183B`, `K`) in `letters`, since both share identical visual glyphs in UTN #57.
    // 由于 `'ᠺ'`（`U+183A`，`K2`）与 `'ᠻ'`（`U+183B`，`K`）在 UTN #57 中视觉字形相同，按 `letters` 中的顺序对齐目标单元中的 `K` / `K2`。
    let mut k_targets = letter_vec.iter().filter_map(|&ch| match ch {
        '\u{183A}' => Some(WrittenUnit::K2), // 'ᠺ' U+183A MONGOLIAN LETTER KA
        '\u{183B}' => Some(WrittenUnit::K),  // 'ᠻ' U+183B MONGOLIAN LETTER KHA
        _ => None,
    });
    let adjusted_units: Vec<PositionedWrittenUnit> = units
        .iter()
        .map(|item| {
            if matches!(item.unit, WrittenUnit::K | WrittenUnit::K2) {
                if let Some(target_k) = k_targets.next() {
                    return PositionedWrittenUnit::new(target_k, item.position);
                }
            }
            *item
        })
        .collect();

    let target_shape = shaper.canonical_positioned_units(&adjusted_units)?;
    let target = parse_word_target(&target_shape).ok_or(ResolveError::ShapeMismatch)?;

    let (stem_letters, chachlag_slot) = if target.has_chachlag {
        let last = *letter_vec.last().expect("letter_vec is non-empty");
        let slot = resolve_chachlag_slot(last).ok_or(ResolveError::ShapeMismatch)?;
        let stem = &letter_vec[..letter_vec.len() - 1];
        if stem.is_empty() != target.stem_units.is_empty() {
            return Err(ResolveError::ShapeMismatch);
        }
        (stem, Some(slot))
    } else {
        (letter_vec.as_slice(), None)
    };

    // 1. Enumerate all valid `(letter -> expanded_written)` decompositions.
    // 1. 枚举全部合法的 `(字母 -> 子字形)` 切分路径。
    let raw_paths = enumerate_paths(shaper, target.stem_units, stem_letters, chachlag_slot);
    if raw_paths.is_empty() {
        return Err(ResolveError::ShapeMismatch);
    }

    // 2. Solve each valid decomposition for its exact minimal-FVS rendering while preserving per-letter written units.
    // 2. 对每条合法切分路径求解保持逐字母子字形不变的精确最少 FVS 编码。
    let is_unique_path = raw_paths.len() == 1;
    let mut solutions: Vec<Solution> = raw_paths
        .into_iter()
        .filter_map(|path| {
            minimize_path_fvs(
                shaper,
                path,
                target.has_chachlag,
                &target_shape,
                is_unique_path,
            )
        })
        .collect();

    if solutions.is_empty() {
        return Err(ResolveError::ShapeMismatch);
    }

    // 3. Stable sort by `fvs_count` ascending so 0-FVS / fewest-FVS solutions come first.
    // 3. 按 `fvs_count` 升序稳定排序，保证 0-FVS 及最少 FVS 的解法排在最前。
    solutions.sort_by_key(|s| s.fvs_count);
    Ok(solutions)
}

/// Validate that every character in `letters` is a pure Traditional Mongolian letter (`U+1820..=U+1842`).
///
/// 校验 `letters` 中的每个字符均属于纯蒙古文基础字母（`U+1820..=U+1842`）；出现任何其他字符（含 `MVS` / `FVS`）即报错。
fn parse_mongolian_letters(letters: &str) -> Result<Vec<char>, ResolveError> {
    let mut out = Vec::with_capacity(letters.len() / 3);
    for (index, character) in letters.chars().enumerate() {
        if !is_mongolian_letter(character) {
            return Err(ResolveError::InvalidLetter { index, character });
        }
        out.push(character);
    }
    Ok(out)
}

/// Parsed single-word target shape: one contiguous stem chain plus optional trailing Chachlag (`MVS + Aa`).
///
/// 解析后的单词目标字形：一段连续词干 + 可选的词尾（或独立）分写 `Chachlag`（`MVS + Aa`）。
struct WordTarget<'a> {
    stem_units: &'a [WrittenUnit],
    has_chachlag: bool,
}

/// Parse canonical single-word `shape` (`Shaper::shape` output).
/// Only trailing/standalone Chachlag (`MVS + Aa`) is accepted as a control sequence;
/// suffixes (`MVS + non-Aa`), mid-word `MVS`, `ZWNJ`, `ZWJ`, etc. return `None`.
///
/// 解析单个单词的规范字形：仅允许词尾或独立的分写 `Chachlag`（`MVS + Aa`）；
/// 格助词（`MVS + 非 Aa`）、词中 `MVS` 及 `ZWNJ` / `ZWJ` 等均视为非单词返回 `None`。
fn parse_word_target(shape: &[WrittenUnit]) -> Option<WordTarget<'_>> {
    let (stem_units, has_chachlag) = match shape {
        [stem @ .., WrittenUnit::Mvs, WrittenUnit::Aa] => (stem, true),
        _ => (shape, false),
    };
    if (!has_chachlag && stem_units.is_empty()) || stem_units.iter().any(|u| u.is_structural()) {
        return None;
    }
    Some(WordTarget {
        stem_units,
        has_chachlag,
    })
}

/// Build the fixed Chachlag [`LetterSlot`] for `'ᠠ'` (`U+1820`) or `'ᠡ'` (`U+1821`) matching `Aa:isol`.
/// Any other letter returns `None`.
///
/// 为分写字尾 `'ᠠ'`（`U+1820`）或 `'ᠡ'`（`U+1821`）构造对应 `Aa:isol` 的固定 [`LetterSlot`]；其他字母返回 `None`。
fn resolve_chachlag_slot(letter: char) -> Option<LetterSlot> {
    if !matches!(letter, 'ᠠ' | 'ᠡ') {
        return None;
    }
    let var = utn57_pinning_variants(letter, Position::Isol)
        .iter()
        .find(|v| v.expanded_written == [WrittenUnit::Aa])?;
    Some(LetterSlot {
        letter,
        position: Position::Isol,
        expanded_written: var.expanded_written,
        pin_fvs: var.fvs,
        kept_fvs: None,
    })
}

/// Positional slot (`Isol`, `Init`, `Medi`, `Fina`) of letter `index` inside a single contiguous chain of length `count`.
///
/// 计算长度为 `count` 的单条连写链中第 `index` 个字母的固定位置（`Isol`、`Init`、`Medi`、`Fina`）。
const fn letter_position(index: usize, count: usize) -> Position {
    match (index, count) {
        (0, 1) => Position::Isol,
        (0, _) => Position::Init,
        _ if index + 1 == count => Position::Fina,
        _ => Position::Medi,
    }
}

/// Expand single-unit UTN #57 duplicates (`Dd`, `Cr`, medial `H`/`Hx`) on `Shaper::shape_detailed`
/// token output so each letter's shaped units can be compared 1-to-1 against `expanded_written`.
/// (See [`matches_chain_prefix`] for coupling notes to upstream `mongol-norm` rules & decoupling plans.)
///
/// 将 `Shaper::shape_detailed` 中单个字母产出的同形异码单元（`Dd`、`Cr`、词中 `H`/`Hx`）展开，以便与 `expanded_written` 精确比对。
/// （与上游 `mongol-norm` 拆解/合并规则的耦合说明及后续解耦方案见 [`matches_chain_prefix`] 注释。）
fn expand_token_written(raw: &[WrittenUnit], pos: Position) -> Vec<WrittenUnit> {
    let mut out = Vec::with_capacity(raw.len() + 1);
    for &u in raw {
        match (u, pos) {
            (WrittenUnit::Dd, _) => out.extend_from_slice(&[WrittenUnit::O, WrittenUnit::A]),
            (WrittenUnit::Cr, _) => out.extend_from_slice(&[WrittenUnit::O, WrittenUnit::O]),
            (WrittenUnit::H, Position::Medi) => {
                out.extend_from_slice(&[WrittenUnit::A, WrittenUnit::A]);
            }
            (WrittenUnit::Hx, Position::Medi) => {
                out.extend_from_slice(&[WrittenUnit::N, WrittenUnit::N]);
            }
            _ => out.push(u),
        }
    }
    out
}

/// Check whether `raw_units` in a sub-chain matches `target_units` after UTN #57 duplicate
/// unification via [`Shaper::canonical_units`].
///
/// 检查子链的 `raw_units` 经 [`Shaper::canonical_units`] 统一同形异码后是否与 `target_units` 完全一致。
fn matches_chain_units(
    shaper: &Shaper,
    raw_units: &[WrittenUnit],
    target_units: &[WrittenUnit],
) -> bool {
    raw_units == target_units || shaper.canonical_units(raw_units) == target_units
}

/// Check whether `cur_units` can still be extended to match `target_units` (leaving the final target
/// unit unconstrained so chain-ending contractions like `O + Aa -> B2:fina` are not pruned early).
///
/// 校验 `cur_units` 是否仍可能扩展为 `target_units`（保留末尾 1 个单元不做硬比较，避免提前剪掉 `O + Aa -> B2:fina` 等链尾合并）。
///
/// # Known Limitation / 当前存在的问题
///
/// - `target_len.saturating_sub(1)` and `cur_units.len() <= target_len` implicitly assume `mongol-norm`
///   only contracts a 2-unit pair at the chain tail (at most rewriting the last 1 unit, never mid-chain
///   and never >=3-to-1); [`expand_token_written`] (and `expand_written_units` in `scripts/generate.py`)
///   likewise mirrors `mongol-norm`'s current single-unit expansions.
///   此处的 `target_len.saturating_sub(1)` 与 `cur_units.len() <= target_len` 隐式假设了 `mongol-norm`
///   只做链尾二合一合并（最多改写末尾 1 个单元，不做词中合并或三合一）；[`expand_token_written`]
///   （及 `scripts/generate.py` 的 `expand_written_units`）也同样镜像了当前单单元展开规则。
/// - Prefix pruning cannot simply be removed (~1.006e8 combinations across `word.txt`), so if upstream
///   `mongol-norm` changes contraction arity or adds mid-chain merges, this check would prune valid paths.
///   因不能去掉前缀剪枝（`word.txt` 无剪枝会膨胀至 1 亿+ 组合），一旦上游 `mongol-norm` 修改合并规则或新增词中合并，
///   此处会漏解。
///
/// # When & How to Resolve / 什么时候解决
///
/// - **When / 时机**：Resolve after upstream `mongol-norm` settles its split/merge design
///   (`orca_session_id:antigravity_91a3c774_964c_4042_9243_63750066dd96`), or immediately if `mongol-norm`
///   introduces mid-chain or >=3-unit contractions.
///   待上游 `mongol-norm` 确定最终拆解/合并机制后统一改造（讨论见
///   `orca_session_id:antigravity_91a3c774_964c_4042_9243_63750066dd96`），或上游新增词中/多元合并时立即调整。
/// - **How / 方案**：
///   1. *Preferred / 首选*：Use upstream `mongol-norm` prefix-check & per-token canonical expansion APIs,
///      deleting `saturating_sub(1)` and [`expand_token_written`].
///      改调上游 `mongol-norm` 提供的子链前缀兼容判断与单字母规范展开接口，彻底删掉 `saturating_sub(1)` 与
///      [`expand_token_written`]。
///   2. *Fallback / 备选*：Pad non-final prefixes with `WrittenUnit::Nirugu` for [`Shaper::canonical_units`]
///      and dynamically probe the lookback window once in `OnceLock` at startup.
///      若上游不暴露该接口，则在本地给前缀垫 `Nirugu` 调 [`Shaper::canonical_units`]，并在 `OnceLock` 启动时自动探测回看窗口。
fn matches_chain_prefix(cur_units: &[WrittenUnit], target_units: &[WrittenUnit]) -> bool {
    let target_len = target_units.len();
    let check_len = cur_units.len().min(target_len.saturating_sub(1));
    cur_units.len() <= target_len && cur_units[..check_len] == target_units[..check_len]
}

/// Internal per-letter state during path enumeration and FVS minimization.
///
/// 路径枚举与 FVS 最小化过程中单个蒙古文字母的位置、子字形与 FVS 状态。
#[derive(Clone, Copy, Debug)]
struct LetterSlot {
    letter: char,
    position: Position,
    expanded_written: &'static [WrittenUnit],
    pin_fvs: Option<Fvs>,
    kept_fvs: Option<Fvs>,
}

impl LetterSlot {
    const fn into_resolved(self) -> ResolvedLetter {
        ResolvedLetter {
            letter: self.letter,
            position: self.position,
            written: self.expanded_written,
            fvs: self.kept_fvs,
        }
    }
}

/// Backtracking enumerator that maps a single contiguous stem (`stem_letters` -> `stem_units`)
/// plus an optional fixed Chachlag tail (`MVS + ᠠ/ᠡ`).
///
/// 将单条连续词干（`stem_letters` -> `stem_units`）及可选词尾分写 `Chachlag`（`MVS + ᠠ/ᠡ`）做回溯切分的枚举器。
struct PathEnumerator<'a> {
    shaper: &'a Shaper,
    stem_units: &'a [WrittenUnit],
    stem_letters: &'a [char],
    chachlag_slot: Option<LetterSlot>,
    cur_units: Vec<WrittenUnit>,
    cur_slots: Vec<LetterSlot>,
    out_paths: Vec<Vec<LetterSlot>>,
}

/// Enumerate all valid `(letter -> expanded_written)` decompositions for a single word.
///
/// 枚举单个单词（单条词干 + 可选 `Chachlag` 字尾）的全部合法切分路径。
fn enumerate_paths(
    shaper: &Shaper,
    stem_units: &[WrittenUnit],
    stem_letters: &[char],
    chachlag_slot: Option<LetterSlot>,
) -> Vec<Vec<LetterSlot>> {
    if let Some(chachlag) = chachlag_slot {
        if stem_letters.is_empty() {
            return vec![vec![chachlag]];
        }
    }

    let mut enumerator = PathEnumerator::new(shaper, stem_units, stem_letters, chachlag_slot);
    enumerator.step(0);
    enumerator.out_paths
}

impl<'a> PathEnumerator<'a> {
    fn new(
        shaper: &'a Shaper,
        stem_units: &'a [WrittenUnit],
        stem_letters: &'a [char],
        chachlag_slot: Option<LetterSlot>,
    ) -> Self {
        Self {
            shaper,
            stem_units,
            stem_letters,
            chachlag_slot,
            cur_units: Vec::with_capacity(stem_units.len() + 1),
            cur_slots: Vec::with_capacity(stem_letters.len()),
            out_paths: Vec::new(),
        }
    }

    /// Enumerate valid pinning variants for `stem_letters[letter_index]` at its fixed positional slot.
    ///
    /// 按 `stem_letters[letter_index]` 在单链中的固定位置枚举合法变体并线性向后回溯。
    fn step(&mut self, letter_index: usize) {
        let letter = self.stem_letters[letter_index];
        let is_last = letter_index + 1 == self.stem_letters.len();
        let position = letter_position(letter_index, self.stem_letters.len());

        for var in utn57_pinning_variants(letter, position) {
            let prev_len = self.cur_units.len();
            self.cur_units.extend_from_slice(var.expanded_written);

            let matched = if is_last {
                matches_chain_units(self.shaper, &self.cur_units, self.stem_units)
            } else {
                matches_chain_prefix(&self.cur_units, self.stem_units)
            };

            if matched {
                self.cur_slots.push(LetterSlot {
                    letter,
                    position,
                    expanded_written: var.expanded_written,
                    pin_fvs: var.fvs,
                    kept_fvs: None,
                });
                if is_last {
                    let mut path = self.cur_slots.clone();
                    if let Some(chachlag) = self.chachlag_slot {
                        path.push(chachlag);
                    }
                    self.out_paths.push(path);
                } else {
                    self.step(letter_index + 1);
                }
                self.cur_slots.pop();
            }
            self.cur_units.truncate(prev_len);
        }
    }
}

/// Render `slots` using `kept_fvs` on each [`LetterSlot`], inserting `MVS` (`U+180E`) before the
/// last slot when `has_chachlag` is `true`.
///
/// 按每个 [`LetterSlot`] 上的 `kept_fvs` 渲染出蒙古文 Unicode 单词；当 `has_chachlag` 为 `true` 时在末位字母前插入 `MVS`（`U+180E`）。
fn render_slots(slots: &[LetterSlot], has_chachlag: bool) -> String {
    let mut out = String::with_capacity(slots.len() * 6 + 3);
    let mvs_idx = has_chachlag.then(|| slots.len() - 1);
    for (i, slot) in slots.iter().enumerate() {
        if mvs_idx == Some(i) {
            out.push('᠎'); // U+180E MONGOLIAN VOWEL SEPARATOR
        }
        out.push(slot.letter);
        if let Some(fvs) = slot.kept_fvs {
            out.push(fvs.as_char());
        }
    }
    out
}

/// Verify that `candidate` shapes to `target_shape` AND (when multiple decompositions exist) every
/// letter token produces its assigned `expected_expanded` slice so stripping FVS on Path B cannot
/// silently collapse into Path A.
///
/// 校验 `candidate` 整词塑形等于 `target_shape`；当存在多条切分路径时进一步核对每个字母产出的展开子字形与 `expected_expanded` 完全一致，防止剥离 FVS 时串入其他切分路径。
fn matches_path_shape(
    shaper: &Shaper,
    candidate: &str,
    target_shape: &[WrittenUnit],
    expected_expanded: &[&[WrittenUnit]],
    is_unique_path: bool,
) -> bool {
    if shaper.shape(candidate).ok().as_deref() != Some(target_shape) {
        return false;
    }
    if is_unique_path {
        return true;
    }
    let Ok(details) = shaper.shape_detailed(candidate) else {
        return false;
    };
    let mut idx = 0usize;
    for tok in details {
        if tok.alias.is_none() {
            continue;
        }
        let Some(&exp) = expected_expanded.get(idx) else {
            return false;
        };
        if expand_token_written(&tok.written, tok.position).as_slice() != exp {
            return false;
        }
        idx += 1;
    }
    idx == expected_expanded.len()
}

/// Convert finalized `slots` into a public [`Solution`].
///
/// 将确定了 `kept_fvs` 的 `slots` 封装为公开的 [`Solution`]。
fn build_solution(
    unicode: String,
    fvs_count: usize,
    slots: Vec<LetterSlot>,
    used_greedy_fallback: bool,
) -> Solution {
    let letters = slots.into_iter().map(LetterSlot::into_resolved).collect();
    Solution {
        unicode,
        fvs_count,
        letters,
        used_greedy_fallback,
    }
}

/// Maximum candidate evaluations allowed for exact minimum-cardinality FVS subset search
/// (`65_536 = 2^16`, ~100 ms worst-case in release builds).
///
/// Guarantees exhaustive exact search for every path with `<= 16` FVS-capable slots, and still
/// resolves longer words exactly when few FVS are needed (e.g. `k = 1..3`). Only paths that
/// exhaust this budget fall back to greedy right-to-left fixpoint stripping with
/// [`Solution::used_greedy_fallback`] set to `true`.
///
/// 精确最少 FVS 子集枚举允许的最大候选校验次数（`65_536 = 2^16`，release 下最坏约 100ms）：
/// 保证可加 FVS 槽位 `<= 16` 的路径 100% 穷举完；更长的词若实际只需少量 FVS（如 `k = 1..3`）同样可精确求得；
/// 仅在耗尽预算时才回退至贪心剥离，并将 [`Solution::used_greedy_fallback`] 置为 `true` 以便后续人工复核。
const EXACT_FVS_SEARCH_MAX_CHECKS: usize = 65_536;

/// Outcome of a bounded-budget exact subset search at a given FVS count `k`.
///
/// 带预算限制的精确子集搜索结果。
enum ExactSearchOutcome {
    /// Found a valid rendering using `k` FVS within the budget.
    Found(String),
    /// Exhausted all combinations at this `k` without finding a match.
    NotFound,
    /// Ran out of candidate-check budget before finishing this `k`.
    BudgetExhausted,
}

/// Helper that finds the minimal-FVS rendering for a single decomposition path.
///
/// 针对单条已确定 `(字母 -> 子字形)` 的切分路径求解最少 FVS 的辅助结构体。
struct FvsMinimizer<'a> {
    shaper: &'a Shaper,
    slots: Vec<LetterSlot>,
    has_chachlag: bool,
    target_shape: &'a [WrittenUnit],
    expected_expanded: Vec<&'static [WrittenUnit]>,
    fvs_indices: Vec<usize>,
    is_unique_path: bool,
    remaining_budget: usize,
}

impl<'a> FvsMinimizer<'a> {
    fn new(
        shaper: &'a Shaper,
        slots: Vec<LetterSlot>,
        has_chachlag: bool,
        target_shape: &'a [WrittenUnit],
        is_unique_path: bool,
    ) -> Self {
        Self::with_budget(
            shaper,
            slots,
            has_chachlag,
            target_shape,
            is_unique_path,
            EXACT_FVS_SEARCH_MAX_CHECKS,
        )
    }

    fn with_budget(
        shaper: &'a Shaper,
        mut slots: Vec<LetterSlot>,
        has_chachlag: bool,
        target_shape: &'a [WrittenUnit],
        is_unique_path: bool,
        exact_search_budget: usize,
    ) -> Self {
        let expected_expanded = slots.iter().map(|s| s.expanded_written).collect();
        let mut fvs_indices = Vec::new();
        for (i, slot) in slots.iter_mut().enumerate() {
            slot.kept_fvs = None;
            if slot.pin_fvs.is_some() {
                fvs_indices.push(i);
            }
        }
        Self {
            shaper,
            slots,
            has_chachlag,
            target_shape,
            expected_expanded,
            fvs_indices,
            is_unique_path,
            remaining_budget: exact_search_budget,
        }
    }

    fn matches(&self, candidate: &str) -> bool {
        matches_path_shape(
            self.shaper,
            candidate,
            self.target_shape,
            &self.expected_expanded,
            self.is_unique_path,
        )
    }

    fn set_all_fvs(&mut self, enabled: bool) {
        for &slot_idx in &self.fvs_indices {
            self.slots[slot_idx].kept_fvs = if enabled {
                self.slots[slot_idx].pin_fvs
            } else {
                None
            };
        }
    }

    /// Recursively choose `remaining` slots from `fvs_indices[start..]` to enable `pin_fvs`,
    /// stopping early if a match is found or `remaining_budget` reaches 0.
    ///
    /// 从 `fvs_indices[start..]` 中选出 `remaining` 个槽位保留 `pin_fvs`，命中或预算耗尽时立即返回。
    fn find_with_k_fvs(&mut self, start: usize, remaining: usize) -> ExactSearchOutcome {
        if remaining == 0 {
            if self.remaining_budget == 0 {
                return ExactSearchOutcome::BudgetExhausted;
            }
            self.remaining_budget -= 1;
            let cand = render_slots(&self.slots, self.has_chachlag);
            return if self.matches(&cand) {
                ExactSearchOutcome::Found(cand)
            } else {
                ExactSearchOutcome::NotFound
            };
        }

        let max_start = self.fvs_indices.len() - remaining;
        for i in start..=max_start {
            let slot_idx = self.fvs_indices[i];
            self.slots[slot_idx].kept_fvs = self.slots[slot_idx].pin_fvs;
            match self.find_with_k_fvs(i + 1, remaining - 1) {
                ExactSearchOutcome::NotFound => self.slots[slot_idx].kept_fvs = None,
                outcome => return outcome,
            }
        }
        ExactSearchOutcome::NotFound
    }

    /// Greedily strip redundant FVS right-to-left until no further removal keeps the path valid.
    ///
    /// 从右向左反复剥离冗余 FVS，直到再去掉任意一个都会改变字形或串路径为止。
    fn strip_redundant_fvs_greedy(&mut self) {
        loop {
            let mut changed = false;
            for &slot_idx in self.fvs_indices.iter().rev() {
                if let Some(prev_fvs) = self.slots[slot_idx].kept_fvs.take() {
                    let cand = render_slots(&self.slots, self.has_chachlag);
                    if self.matches(&cand) {
                        changed = true;
                    } else {
                        self.slots[slot_idx].kept_fvs = Some(prev_fvs);
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }

    fn solve(mut self) -> Option<Solution> {
        // 1. Test 0 FVS for this path.
        // 1. 优先测试当前路径在不加任何 FVS 时是否直接命中。
        let bare = render_slots(&self.slots, self.has_chachlag);
        if self.matches(&bare) {
            return Some(build_solution(bare, 0, self.slots, false));
        }

        if self.fvs_indices.is_empty() {
            return None;
        }

        // Verify all-pinned candidate works in full word context.
        // 校验全锁定 FVS 在整词上下文中有效。
        self.set_all_fvs(true);
        let full = render_slots(&self.slots, self.has_chachlag);
        if !self.matches(&full) {
            return None;
        }

        let total = self.fvs_indices.len();

        // 2. Bounded-budget exact minimum-cardinality subset search (`k = 1, 2, ...`).
        // 2. 在校验预算内按子集大小 `k = 1, 2, ...` 递增做精确最少 FVS 搜索。
        self.set_all_fvs(false);
        for k in 1..total {
            match self.find_with_k_fvs(0, k) {
                ExactSearchOutcome::Found(unicode) => {
                    return Some(build_solution(unicode, k, self.slots, false));
                }
                ExactSearchOutcome::NotFound => continue,
                ExactSearchOutcome::BudgetExhausted => {
                    // 3. Budget exhausted: fall back to greedy right-to-left fixpoint stripping
                    //    and mark `used_greedy_fallback = true` for downstream manual review.
                    // 3. 预算耗尽：回退到从右向左贪心不动点剥离，并标记 `used_greedy_fallback = true` 供后续人工复核。
                    self.set_all_fvs(true);
                    self.strip_redundant_fvs_greedy();
                    let unicode = render_slots(&self.slots, self.has_chachlag);
                    let fvs_count = self.slots.iter().filter(|s| s.kept_fvs.is_some()).count();
                    return Some(build_solution(unicode, fvs_count, self.slots, true));
                }
            }
        }

        self.set_all_fvs(true);
        Some(build_solution(full, total, self.slots, false))
    }
}

/// Find the minimal-FVS [`Solution`] for a single decomposition path `slots`.
///
/// 对单条确定了 `(字母 -> 子字形)` 分配的路径 `slots` 求解严格最少的 FVS 组合并构造 [`Solution`]。
fn minimize_path_fvs(
    shaper: &Shaper,
    slots: Vec<LetterSlot>,
    has_chachlag: bool,
    target_shape: &[WrittenUnit],
    is_unique_path: bool,
) -> Option<Solution> {
    FvsMinimizer::new(shaper, slots, has_chachlag, target_shape, is_unique_path).solve()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fvs_minimizer_reports_greedy_fallback_when_budget_is_exhausted() {
        // Construct `ᠪᠠ᠋ᠨ᠋ᠠ᠋ᠯ` (3 FVS slots where all 3 FVS are needed: medial `A+A`, medial `A` on `ᠨ`, medial `A+A`).
        // 构造一个需要多个 FVS 的路径，验证预算耗尽时自动回退到贪心并标记 `used_greedy_fallback = true`。
        let shaper = default_shaper();
        let expected_unicode = "ᠪᠠ\u{180B}ᠨ\u{180B}ᠠ\u{180B}ᠯ";
        let target_shape = shaper.shape(expected_unicode).unwrap();
        let letters: Vec<char> = "ᠪᠠᠨᠠᠯ".chars().collect();
        let paths = enumerate_paths(shaper, &target_shape, &letters, None);
        let path = paths
            .into_iter()
            .find(|p| p.iter().filter(|s| s.pin_fvs.is_some()).count() >= 3)
            .expect("expected path with >= 3 FVS-capable slots");

        let exact = FvsMinimizer::with_budget(
            shaper,
            path.clone(),
            false,
            &target_shape,
            false,
            EXACT_FVS_SEARCH_MAX_CHECKS,
        )
        .solve()
        .unwrap();
        assert!(!exact.used_greedy_fallback);
        assert_eq!(shaper.shape(&exact.unicode).unwrap(), target_shape);

        let greedy = FvsMinimizer::with_budget(shaper, path, false, &target_shape, false, 1)
            .solve()
            .unwrap();
        assert!(greedy.used_greedy_fallback);
        assert_eq!(shaper.shape(&greedy.unicode).unwrap(), target_shape);
        assert_eq!(greedy.fvs_count, exact.fvs_count);
    }
}
