# menk-to-utn57

[English](#english) | [中文](#中文)

---

<a id="english"></a>
## English

**menk-to-utn57** is a lightweight, pure-Rust library with two decoupled layers:

1. **Menk shape extraction (`conversion.rs`)**: Extracts glyph shapes from Menk PUA Mongolian text (`U+E264..=U+E34F`) into positioned [UTN #57](https://www.unicode.org/notes/tn57/) written units ([`PositionedWrittenUnit`](https://docs.rs/mongol-norm/latest/mongol_norm/struct.PositionedWrittenUnit.html)), plus `get_standard_letters` / `get_standard_letters_with_k` to bootstrap approximate pure Unicode Mongolian letter strings (`U+1820..=U+1842`) before human proofreading.
2. **UTN #57 shape + pure Unicode letter solver (`resolve.rs`)**: Completely independent of Menk — takes a word's `&[PositionedWrittenUnit]` shape and its pure Unicode Mongolian letter string (`U+1820..=U+1842`, without `MVS` or `FVS`), deduces Chachlag `MVS` (`U+180E`) from the shape, enumerates **all** valid per-letter written-unit decompositions, and computes the **strictly minimal Free Variation Selectors (`FVS1..FVS4`)** for each decomposition (`Vec<Solution>`), verified directly against `mongol_norm::Shaper::shape`.

All UTN #57 types (`WrittenUnit`, `UnitPosition`, `PositionedWrittenUnit`, `Position`, `Fvs`) come directly from [`mongol-norm`](https://crates.io/crates/mongol-norm) and are re-exported so callers can shape or normalize results without adding a separate dependency. Everything runs in-process with zero runtime I/O (`wasm32-unknown-unknown` compatible).

### Data Flow

```text
Menk PUA word (U+E264..=U+E34F)
  → decompose each char into MenkShape sequence (data/menk_char_shapes.json)
  → split into connected sub-chains via only_init / only_fina & Aa rules (data/menk_char_positions.json)
  → map each (MenkShape, Isol|Init|Medi|Fina) slot to PositionedWrittenUnit (data/menk_shape_map.json)
  ├─→ shape-only canonical Unicode Mongolian via mongol_norm::Shaper
  └─→ UTN #57 PositionedWrittenUnit slice + pure Unicode Mongolian letters (U+1820..=U+1842)
      + UTN #57 letter & FVS variant table (data/utn57_letter_variants.json)
      → all matching minimal-FVS solutions with per-letter breakdown (utn57_and_letters_to_unicode)
```

### Installation

```toml
[dependencies]
menk-to-utn57 = { git = "https://github.com/Satsrag/menk-to-utn57" }
```

### Usage

#### 1. Enumerate all matching minimal-FVS Unicode Mongolian solutions (`Vec<Solution>`)

Passing `&[PositionedWrittenUnit]` and pure Traditional Mongolian letters (`U+1820..=U+1842`, without `MVS` or `FVS`) into `utn57_and_letters_to_unicode`:
- Enumerates **every** valid per-letter written-unit decomposition matching the target shape (including Chachlag `MVS` `U+180E` + `'ᠠ'`/`'ᠡ'`), without pruning alternative FVS decompositions.
- Computes the strictly minimal `FVS` assignment for each decomposition and returns `Vec<Solution>` sorted by `fvs_count` ascending (each `Solution` includes `unicode`, `fvs_count`, per-letter `letters: Vec<ResolvedLetter>`, and `used_greedy_fallback: bool` indicating whether the exact subset-search budget of 65,536 checks was exhausted and fell back to greedy stripping for optional manual review).
- Accepts both Inner Mongolia `'ᠺ'` (`U+183A`) and Mongolia `'ᠻ'` (`U+183B`) directly in `letters`:

```rust
use menk_to_utn57::{
    get_standard_letters, get_standard_letters_with_k, menk_word_to_utn57,
    utn57_and_letters_to_unicode, Fvs, KLetter, Position, PositionedWrittenUnit, UnitPosition,
    WrittenUnit,
};

// 1) Chachlag word "ᠪᠠᠢᠨᠠ" deduces MVS (U+180E) from shape and returns `["ᠪᠠᠢᠨ᠎ᠠ"]`:
let baina_menk = "\u{E2C1}\u{E26C}\u{E281}\u{E2B6}\u{E26A}";
let baina_units = menk_word_to_utn57(baina_menk).unwrap();
assert_eq!(get_standard_letters(baina_menk).unwrap(), "ᠪᠠᠢᠨᠠ");
let baina_solutions = utn57_and_letters_to_unicode(&baina_units, "ᠪᠠᠢᠨᠠ").unwrap();
assert_eq!(baina_solutions.len(), 1);
assert_eq!(baina_solutions[0].unicode, "ᠪᠠᠢᠨ\u{180E}ᠠ");
assert_eq!(baina_solutions[0].fvs_count, 0);

// 2) Unique FVS decomposition "ᠳᠦ" returns `["ᠳ᠋ᠦ᠌"]` with minimal FVS1 + FVS2:
let du_units = menk_word_to_utn57("\u{E310}\u{E2A7}").unwrap();
let du_solutions = utn57_and_letters_to_unicode(&du_units, "ᠳᠦ").unwrap();
assert_eq!(du_solutions.len(), 1);
assert_eq!(du_solutions[0].unicode, "ᠳ\u{180B}ᠦ\u{180C}");
assert_eq!(du_solutions[0].fvs_count, 2);

// 3) Multiple valid decompositions ("ᠨᠠᠠᠯ" on `[N, A, A, A, L]`) return ALL matching `Solution`s:
let naal_units = [
    PositionedWrittenUnit::new(WrittenUnit::N, UnitPosition::Init),
    PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
    PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
    PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
    PositionedWrittenUnit::new(WrittenUnit::L, UnitPosition::Fina),
];
let naal_solutions = utn57_and_letters_to_unicode(&naal_units, "ᠨᠠᠠᠯ").unwrap();
assert_eq!(naal_solutions.len(), 2);

// Solution 0: first `'ᠠ'` = `[A]` (None), second `'ᠠ'` = `[A, A]` (Some(Fvs1)) -> "ᠨᠠᠠ᠋ᠯ"
assert_eq!(naal_solutions[0].unicode, "ᠨᠠᠠ\u{180B}ᠯ");
assert_eq!(naal_solutions[0].letters[1].position, Position::Medi);
assert_eq!(naal_solutions[0].letters[1].written, &[WrittenUnit::A]);
assert_eq!(naal_solutions[0].letters[2].written, &[WrittenUnit::A, WrittenUnit::A]);
assert_eq!(naal_solutions[0].letters[2].fvs, Some(Fvs::Fvs1));

// Solution 1: first `'ᠠ'` = `[A, A]` (Some(Fvs1)), second `'ᠠ'` = `[A]` (None) -> "ᠨᠠ᠋ᠠᠯ"
assert_eq!(naal_solutions[1].unicode, "ᠨᠠ\u{180B}ᠠᠯ");

// 4) Both `'ᠺ'` (U+183A, Inner Mongolia) and `'ᠻ'` (U+183B, Mongolia) work directly in `letters`:
let kino_menk = "\u{E333}\u{E27F}\u{E2B9}\u{E286}";
let kino_units = menk_word_to_utn57(kino_menk).unwrap();
assert_eq!(get_standard_letters(kino_menk).unwrap(), "ᠺᠢᠨᠣ");
assert_eq!(
    get_standard_letters_with_k(kino_menk, KLetter::Kha).unwrap(),
    "ᠻᠢᠨᠣ"
);
assert_eq!(
    utn57_and_letters_to_unicode(&kino_units, "ᠺᠢᠨᠣ").unwrap()[0].unicode,
    "ᠺᠢᠨᠣ\u{180B}"
);
assert_eq!(
    utn57_and_letters_to_unicode(&kino_units, "ᠻᠢᠨᠣ").unwrap()[0].unicode,
    "ᠻᠢᠨᠣ\u{180B}"
);
```

#### 2. Convert a Menk word to UTN #57 units & shape-only canonical Unicode

```rust
use menk_to_utn57::mongol_norm::{Locale, Shaper};
use menk_to_utn57::{menk_word_to_utn57, PositionedWrittenUnit, UnitPosition, WrittenUnit};

// Menk PUA word "ba" (U+E2C1 + U+E26B)
let units = menk_word_to_utn57("\u{E2C1}\u{E26B}").unwrap();
assert_eq!(
    units,
    vec![
        PositionedWrittenUnit::new(WrittenUnit::B, UnitPosition::Init),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::Aa, UnitPosition::Fina),
    ]
);

// Normalize to shape-equivalent canonical Unicode Mongolian via re-exported `mongol_norm`
let shaper = Shaper::new(Locale::Mng);
let unicode_word = shaper.normalize_positioned_written_units(&units).unwrap();
assert!(!unicode_word.is_empty());
```

#### 3. Extract `MenkShape` sequences

```rust
use menk_to_utn57::{get_shape, menk_shapes, MenkShape};

let shapes = menk_shapes("\u{E2FD}\u{E26C}\u{E325}\u{E26A}").unwrap();
assert_eq!(
    shapes,
    vec![MenkShape::S, MenkShape::A_MEDI, MenkShape::K, MenkShape::Aa]
);

// Representative Menk shape string (non-Menk chars pass through unchanged)
let shape_str = get_shape("\u{E2FD}\u{E26C}\u{E325}\u{E26A}");
assert_eq!(shape_str.chars().count(), 4);
```

### Sub-chain Splitting & Visual Calibration

Menk presentation glyphs share shapes across medial and final positions for bows and round bellies (`B`, `G`, `K`, `O`), while crowns (`A_INIT`, `N_INIT`, `H_INIT`, `Hx_INIT`, `M_INIT`, `L_INIT`) and tailed finals (`S:fina`, `Sh:fina`, `T:fina`, `D:fina`, etc.) are position-specific:

1. **`only_init` (39 code points)**: Flushes any preceding sub-chain *before* the character.
2. **`Aa` joining rule**: Connects `MenkShape::Aa` onto `A_INIT | A_MEDI | I | O` as `Aa:fina` (including `I + Aa` = `G:fina` and `O + Aa` = `B2:fina`); otherwise starts an isolated Chachlag sub-chain (`Aa:isol`).
3. **`only_fina` (48 code points)**: Flushes the current sub-chain *after* the character.

Open [`docs/menk_positions.html`](docs/menk_positions.html) directly in any browser to inspect all 236 Menk glyphs (with an embedded subsetted `menk.ttf` font, bilingual English/Chinese notes, search/filtering, and interactive JSON export).

### Repository Layout

```text
.
├── Cargo.toml
├── LICENSE
├── README.md
├── data/
│   ├── menk_char_positions.json   # 39 only_init & 48 only_fina code points
│   ├── menk_char_shapes.json      # 236 Menk chars (U+E264..=U+E34F) -> MenkShape sequences & metadata
│   ├── menk_shape_map.json        # 44 MenkShapes -> 4-slot UTN #57 PositionedWrittenUnits
│   └── utn57_letter_variants.json # 35 Unicode Mongolian letters -> 216 UTN #57 positional & FVS variants
├── docs/
│   └── menk_positions.html        # Self-contained bilingual visual calibration table
├── scripts/
│   ├── generate.py                # Generates src/generated/*.rs from data/*.json
│   └── generate_positions_html.py # Generates docs/menk_positions.html
├── src/
│   ├── conversion.rs              # Menk -> MenkShape, PositionedWrittenUnit & get_standard_letters
│   ├── generated/
│   │   ├── menk_shapes.rs         # Generated MenkShape enum, letter & position lookups
│   │   ├── utn57_map.rs           # Generated MenkShape::to_utn57 lookup
│   │   └── utn57_variants.rs      # Generated UTN #57 letter & FVS variant lookup
│   ├── lib.rs                     # Public API, KLetter & mongol_norm re-exports
│   └── resolve.rs                 # Menk-independent UTN #57 shape + Unicode letters -> Vec<Solution> solver
└── tests/
    ├── conversion_tests.rs        # Menk shape & UTN #57 unit conversion tests
    └── resolve_tests.rs           # Exhaustive multi-solution FVS solver & variant table tests
```

### Development

```bash
# Regenerate src/generated/*.rs after editing data/*.json
python3 scripts/generate.py

# Regenerate docs/menk_positions.html
python3 scripts/generate_positions_html.py

# Run linter and tests (including doc-tests)
cargo clippy --all-targets -- -D warnings
cargo test --release
cargo test --doc
```

---

<a id="中文"></a>
## 中文

**menk-to-utn57** 是一个轻量级纯 Rust 库，采用两层解耦架构：

1. **孟克字形提取层（`conversion.rs`）**：从孟克私有区蒙古文编码（`U+E264..=U+E34F`）提取基础字形（`MenkShape`）并转换为带位置的 [UTN #57](https://www.unicode.org/notes/tn57/) 书写单元（[`PositionedWrittenUnit`](https://docs.rs/mongol-norm/latest/mongol_norm/struct.PositionedWrittenUnit.html)）；同时提供 `get_standard_letters` / `get_standard_letters_with_k` 在人工校对前生成粗略纯蒙古文 Unicode 基础字母串（`U+1820..=U+1842`）用于测试与预填。
2. **UTN #57 字形 + 纯蒙古文字母全解求解层（`resolve.rs`）**：与孟克编码完全解耦——接收单个词的 `&[PositionedWrittenUnit]` 字形序列与纯蒙古文基础字母串（`U+1820..=U+1842`，不含 `MVS` / `FVS`），从字形自动推导词尾分写 `Chachlag` 的 `MVS`（`U+180E`），枚举**全部**合法的逐字母子字形切分路径，并为每条路径计算出**严格最少数量的自由变体选择符（`FVS1..FVS4`）**（返回 `Vec<Solution>`），全程经 `mongol_norm::Shaper::shape` 精确校验。

库内所有 UTN #57 类型（`WrittenUnit`、`UnitPosition`、`PositionedWrittenUnit`、`Position`、`Fvs`）均直接复用并重导出自 [`mongol-norm`](https://crates.io/crates/mongol-norm)，调用方无需重复声明依赖；全程在进程内完成，无运行时文件或网络 I/O（支持 `wasm32-unknown-unknown`）。

### 数据流

```text
孟克编码词串 (U+E264..=U+E34F)
  → 拆分为 MenkShape 基础字形序列 (data/menk_char_shapes.json)
  → 按 only_init / only_fina 及 Aa 规则切分连写子链 (data/menk_char_positions.json)
  → 按子链位置槽位 (Isol|Init|Medi|Fina) 映射为 PositionedWrittenUnit (data/menk_shape_map.json)
  ├─→ 仅按字形归一化为规范 Unicode 蒙古文 (mongol_norm::Shaper)
  └─→ UTN #57 PositionedWrittenUnit 序列 + 纯蒙古文 Unicode 基础字母串 (U+1820..=U+1842)
      + UTN #57 字母与 FVS 变体表 (data/utn57_letter_variants.json)
      → 枚举全部匹配的最小 FVS 解法与逐字母明细 (utn57_and_letters_to_unicode)
```

### 安装

```toml
[dependencies]
menk-to-utn57 = { git = "https://github.com/Satsrag/menk-to-utn57" }
```

### 使用示例

#### 1. 根据 UTN #57 字形与纯蒙古文字母串枚举全部匹配解法（`Vec<Solution>`）

将单词的 `&[PositionedWrittenUnit]` 字形序列与纯蒙古文基础字母串（`U+1820..=U+1842`，不含 `MVS` / `FVS`）传入 `utn57_and_letters_to_unicode`：
- 枚举能够拼出目标字形（含词尾或独立分写 `Chachlag` 的 `MVS` `U+180E` + `'ᠠ'`/`'ᠡ'`）的**全部**合法逐字母子字形切分路径（不做 0-FVS 剪枝）。
- 为每条切分路径计算严格最少的 `FVS` 组合，按 `fvs_count` 升序返回 `Vec<Solution>`（每个 `Solution` 包含最终字符串 `unicode`、`fvs_count`、逐字母明细 `letters: Vec<ResolvedLetter>`，以及 `used_greedy_fallback: bool` 标记是否在耗尽 65,536 次精确搜索预算后回退至贪心剥离，便于后续人工复核）。
- `letters` 中可直接使用内蒙古标准 `'ᠺ'`（`U+183A`）或蒙古国标准 `'ᠻ'`（`U+183B`）：

```rust
use menk_to_utn57::{
    get_standard_letters, get_standard_letters_with_k, menk_word_to_utn57,
    utn57_and_letters_to_unicode, Fvs, KLetter, Position, PositionedWrittenUnit, UnitPosition,
    WrittenUnit,
};

// 1) 分写字尾词 "ᠪᠠᠢᠨᠠ" 从字形自动推导 MVS (U+180E)，返回 `["ᠪᠠᠢᠨ᠎ᠠ"]`：
let baina_menk = "\u{E2C1}\u{E26C}\u{E281}\u{E2B6}\u{E26A}";
let baina_units = menk_word_to_utn57(baina_menk).unwrap();
assert_eq!(get_standard_letters(baina_menk).unwrap(), "ᠪᠠᠢᠨᠠ");
let baina_solutions = utn57_and_letters_to_unicode(&baina_units, "ᠪᠠᠢᠨᠠ").unwrap();
assert_eq!(baina_solutions.len(), 1);
assert_eq!(baina_solutions[0].unicode, "ᠪᠠᠢᠨ\u{180E}ᠠ");
assert_eq!(baina_solutions[0].fvs_count, 0);

// 2) 唯一 FVS 切分 "ᠳᠦ" 自动补充最小 FVS1 + FVS2 返回 `["ᠳ᠋ᠦ᠌"]`：
let du_units = menk_word_to_utn57("\u{E310}\u{E2A7}").unwrap();
let du_solutions = utn57_and_letters_to_unicode(&du_units, "ᠳᠦ").unwrap();
assert_eq!(du_solutions.len(), 1);
assert_eq!(du_solutions[0].unicode, "ᠳ\u{180B}ᠦ\u{180C}");
assert_eq!(du_solutions[0].fvs_count, 2);

// 3) 多切分路径（目标 `[N, A, A, A, L]` + 纯字母 `"ᠨᠠᠠᠯ"`）一次返回全部匹配的 `Solution`：
let naal_units = [
    PositionedWrittenUnit::new(WrittenUnit::N, UnitPosition::Init),
    PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
    PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
    PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
    PositionedWrittenUnit::new(WrittenUnit::L, UnitPosition::Fina),
];
let naal_solutions = utn57_and_letters_to_unicode(&naal_units, "ᠨᠠᠠᠯ").unwrap();
assert_eq!(naal_solutions.len(), 2);

// 解法 0：第 1 个 `'ᠠ'` = `[A]` (None)，第 2 个 `'ᠠ'` = `[A, A]` (Some(Fvs1)) -> "ᠨᠠᠠ᠋ᠯ"
assert_eq!(naal_solutions[0].unicode, "ᠨᠠᠠ\u{180B}ᠯ");
assert_eq!(naal_solutions[0].letters[1].position, Position::Medi);
assert_eq!(naal_solutions[0].letters[1].written, &[WrittenUnit::A]);
assert_eq!(naal_solutions[0].letters[2].written, &[WrittenUnit::A, WrittenUnit::A]);
assert_eq!(naal_solutions[0].letters[2].fvs, Some(Fvs::Fvs1));

// 解法 1：第 1 个 `'ᠠ'` = `[A, A]` (Some(Fvs1))，第 2 个 `'ᠠ'` = `[A]` (None) -> "ᠨᠠ᠋ᠠᠯ"
assert_eq!(naal_solutions[1].unicode, "ᠨᠠ\u{180B}ᠠᠯ");

// 4) 内蒙古标准 `'ᠺ'`（U+183A）与蒙古国标准 `'ᠻ'`（U+183B）均可直接传入求解：
let kino_menk = "\u{E333}\u{E27F}\u{E2B9}\u{E286}";
let kino_units = menk_word_to_utn57(kino_menk).unwrap();
assert_eq!(get_standard_letters(kino_menk).unwrap(), "ᠺᠢᠨᠣ");
assert_eq!(
    get_standard_letters_with_k(kino_menk, KLetter::Kha).unwrap(),
    "ᠻᠢᠨᠣ"
);
assert_eq!(
    utn57_and_letters_to_unicode(&kino_units, "ᠺᠢᠨᠣ").unwrap()[0].unicode,
    "ᠺᠢᠨᠣ\u{180B}"
);
assert_eq!(
    utn57_and_letters_to_unicode(&kino_units, "ᠻᠢᠨᠣ").unwrap()[0].unicode,
    "ᠻᠢᠨᠣ\u{180B}"
);
```

#### 2. 将孟克词串转换为 UTN #57 位置书写单元（仅按字形归一化）

```rust
use menk_to_utn57::mongol_norm::{Locale, Shaper};
use menk_to_utn57::{menk_word_to_utn57, PositionedWrittenUnit, UnitPosition, WrittenUnit};

// 孟克编码词 "ba" (U+E2C1 + U+E26B)
let units = menk_word_to_utn57("\u{E2C1}\u{E26B}").unwrap();
assert_eq!(
    units,
    vec![
        PositionedWrittenUnit::new(WrittenUnit::B, UnitPosition::Init),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::Aa, UnitPosition::Fina),
    ]
);

// 使用重导出的 `mongol_norm` 仅按字形归一化为规范 Unicode 蒙古文字符串
let shaper = Shaper::new(Locale::Mng);
let unicode_word = shaper.normalize_positioned_written_units(&units).unwrap();
assert!(!unicode_word.is_empty());
```

#### 3. 提取 `MenkShape` 基础字形序列

```rust
use menk_to_utn57::{get_shape, menk_shapes, MenkShape};

let shapes = menk_shapes("\u{E2FD}\u{E26C}\u{E325}\u{E26A}").unwrap();
assert_eq!(
    shapes,
    vec![MenkShape::S, MenkShape::A_MEDI, MenkShape::K, MenkShape::Aa]
);

// 提取代表字形字符串（非孟克字符原样透传）
let shape_str = get_shape("\u{E2FD}\u{E26C}\u{E325}\u{E26A}");
assert_eq!(shape_str.chars().count(), 4);
```

### 子链切分规则与可视化核对表

孟克显现编码中，弓形与圆肚形（`B`、`G`、`K`、`O`）在词中与词尾可同形混用，而带冠词首（`A_INIT`、`N_INIT`、`H_INIT`、`Hx_INIT`、`M_INIT`、`L_INIT`）和带尾词尾（`S:fina`、`Sh:fina`、`T:fina`、`D:fina` 等）具有确定边界：

1. **`only_init`（39 个码位）**：遇到该字符时在其**前方**切分子链。
2. **`Aa` 连写规则**：`MenkShape::Aa` 仅在紧跟 `A_INIT | A_MEDI | I | O` 时连写为 `Aa:fina`（含 `I + Aa` = `G:fina`、`O + Aa` = `B2:fina`）；其余上下文断开为独立分写字尾 `Aa:isol`（Chachlag）。
3. **`only_fina`（48 个码位）**：遇到该字符时在其**后方**切分子链。

直接在浏览器打开 [`docs/menk_positions.html`](docs/menk_positions.html) 即可查看全部 236 个孟克码位的竖排字形、中英双语核对说明（支持 `双语 / English / 简体中文` 切换）、检索筛选以及交互式 JSON 导出。

### 目录结构

```text
.
├── Cargo.toml
├── LICENSE
├── README.md
├── data/
│   ├── menk_char_positions.json   # 39 个 only_init 与 48 个 only_fina 码位配置
│   ├── menk_char_shapes.json      # 236 个孟克字符 (U+E264..=U+E34F) -> MenkShape 拆分表与元数据
│   ├── menk_shape_map.json        # 44 种 MenkShape -> 四位置 UTN #57 书写单元映射表
│   └── utn57_letter_variants.json # 35 个蒙古文字母 -> 216 条 UTN #57 位置与 FVS 变体表
├── docs/
│   └── menk_positions.html        # 内嵌子集字体的中英双语可视化位置核对表
├── scripts/
│   ├── generate.py                # 从 data/*.json 生成 src/generated/*.rs
│   └── generate_positions_html.py # 生成 docs/menk_positions.html
├── src/
│   ├── conversion.rs              # 孟克 -> MenkShape、PositionedWrittenUnit 与 get_standard_letters
│   ├── generated/
│   │   ├── menk_shapes.rs         # 自动生成的 MenkShape 枚举、基础字母与字符位置查询函数
│   │   ├── utn57_map.rs           # 自动生成的 MenkShape::to_utn57 映射表
│   │   └── utn57_variants.rs      # 自动生成的 UTN #57 字母与 FVS 变体查询表
│   ├── lib.rs                     # 公共导出、KLetter 与 mongol_norm 重导出
│   └── resolve.rs                 # 与孟克解耦的 UTN #57 字形 + 纯蒙古文字母 -> Vec<Solution> 全解求解器
└── tests/
    ├── conversion_tests.rs        # 孟克字形与 UTN #57 单元转换集成测试
    └── resolve_tests.rs           # UTN #57 + 纯蒙古文字母全解 FVS 求解与变体表集成测试
```

### 开发与测试

```bash
# 修改 data/*.json 后重新生成 Rust 映射代码
python3 scripts/generate.py

# 重新生成可视化核对页面 docs/menk_positions.html
python3 scripts/generate_positions_html.py

# 运行静态检查与测试（含 README 文档测试）
cargo clippy --all-targets -- -D warnings
cargo test --release
cargo test --doc
```

## License / 许可证

Licensed under the [Apache License, Version 2.0](LICENSE). / 本项目采用 [Apache-2.0](LICENSE) 许可证开源。
