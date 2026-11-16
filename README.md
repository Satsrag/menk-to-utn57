# menk-to-utn57

[English](#english) | [中文](#中文)

---

<a id="english"></a>
## English

**menk-to-utn57** is a lightweight, pure-Rust library that extracts glyph shapes from Menk PUA Mongolian text (`U+E264..=U+E34F`) and converts them into positioned [UTN #57](https://www.unicode.org/notes/tn57/) written units ([`PositionedWrittenUnit`](https://docs.rs/mongol-norm/latest/mongol_norm/struct.PositionedWrittenUnit.html)).

All UTN #57 types (`WrittenUnit`, `UnitPosition`, `PositionedWrittenUnit`, `Position`) come directly from [`mongol-norm`](https://crates.io/crates/mongol-norm) and are re-exported so callers can normalize converted units to canonical Unicode Mongolian without adding a separate dependency. Everything runs in-process with zero runtime I/O (`wasm32-unknown-unknown` compatible).

### Data Flow

```text
Menk PUA word (U+E264..=U+E34F)
  → decompose each char into MenkShape sequence (data/menk_char_shapes.json)
  → split into connected sub-chains via only_init / only_fina & Aa rules (data/menk_char_positions.json)
  → map each (MenkShape, Isol|Init|Medi|Fina) slot to PositionedWrittenUnit (data/menk_shape_map.json)
  → (optional) normalize to canonical Unicode Mongolian via mongol_norm::Shaper
```

### Installation

```toml
[dependencies]
menk-to-utn57 = { git = "https://github.com/Satsrag/menk-to-utn57" }
```

### Usage

#### 1. Convert a Menk word to UTN #57 units & canonical Unicode Mongolian

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

// Normalize to canonical Unicode Mongolian string via re-exported `mongol_norm`
let shaper = Shaper::new(Locale::Mng);
let unicode_word = shaper.normalize_positioned_written_units(&units).unwrap();
assert!(!unicode_word.is_empty());
```

#### 2. Convert mixed text (Menk runs + spaces/punctuation passthrough)

`menk_word_to_utn57` operates on a single Menk letter run (`U+E264..=U+E34F`). For full text containing spaces or punctuation, split Menk runs and pass non-Menk spans through unchanged:

```rust
use menk_to_utn57::mongol_norm::{Locale, Shaper};
use menk_to_utn57::menk_word_to_utn57;

fn convert_menk_text(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let shaper = Shaper::new(Locale::Mng);
    let mut out = String::with_capacity(input.len());
    let mut run = String::new();

    for ch in input.chars() {
        if ('\u{E264}'..='\u{E34F}').contains(&ch) {
            run.push(ch);
        } else {
            if !run.is_empty() {
                let units = menk_word_to_utn57(&run)?;
                out.push_str(&shaper.normalize_positioned_written_units(&units)?);
                run.clear();
            }
            out.push(ch);
        }
    }
    if !run.is_empty() {
        let units = menk_word_to_utn57(&run)?;
        out.push_str(&shaper.normalize_positioned_written_units(&units)?);
    }
    Ok(out)
}
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
│   ├── menk_char_shapes.json      # 236 Menk chars (U+E264..=U+E34F) -> MenkShape sequences
│   └── menk_shape_map.json        # 44 MenkShapes -> 4-slot UTN #57 PositionedWrittenUnits
├── docs/
│   └── menk_positions.html        # Self-contained bilingual visual calibration table
├── scripts/
│   ├── generate.py                # Generates src/generated/*.rs from data/*.json
│   └── generate_positions_html.py # Generates docs/menk_positions.html
├── src/
│   ├── conversion.rs              # Shape extraction & sub-chain UTN #57 conversion
│   ├── generated/
│   │   ├── menk_shapes.rs         # Generated MenkShape enum & char predicates
│   │   └── utn57_map.rs           # Generated MenkShape::to_utn57 lookup
│   └── lib.rs                     # Public API & mongol_norm re-exports
└── tests/
    └── conversion_tests.rs        # Integration & corpus normalization tests
```

### Development

```bash
# Regenerate src/generated/*.rs after editing data/*.json
python3 scripts/generate.py

# Regenerate docs/menk_positions.html
python3 scripts/generate_positions_html.py

# Run linter and tests (including doc-tests)
cargo clippy --all-targets -- -D warnings
cargo test
```

---

<a id="中文"></a>
## 中文

**menk-to-utn57** 是一个轻量级纯 Rust 库，用于从孟克私有区蒙古文编码（`U+E264..=U+E34F`）提取基础字形（`MenkShape`），并将其转换为带位置的 [UTN #57](https://www.unicode.org/notes/tn57/) 书写单元（[`PositionedWrittenUnit`](https://docs.rs/mongol-norm/latest/mongol_norm/struct.PositionedWrittenUnit.html)）。

库内所有 UTN #57 类型（`WrittenUnit`、`UnitPosition`、`PositionedWrittenUnit`、`Position`）均直接复用并重导出自 [`mongol-norm`](https://crates.io/crates/mongol-norm)，调用方无需重复声明依赖即可直接归一化为规范 Unicode 蒙古文；全程在进程内完成，无运行时文件或网络 I/O（支持 `wasm32-unknown-unknown`）。

### 数据流

```text
孟克编码词串 (U+E264..=U+E34F)
  → 拆分为 MenkShape 基础字形序列 (data/menk_char_shapes.json)
  → 按 only_init / only_fina 及 Aa 规则切分连写子链 (data/menk_char_positions.json)
  → 按子链位置槽位 (Isol|Init|Medi|Fina) 映射为 PositionedWrittenUnit (data/menk_shape_map.json)
  →（可选）通过 mongol_norm::Shaper 归一化为规范 Unicode 蒙古文
```

### 安装

```toml
[dependencies]
menk-to-utn57 = { git = "https://github.com/Satsrag/menk-to-utn57" }
```

### 使用示例

#### 1. 将孟克词串转换为 UTN #57 位置书写单元并归一化为 Unicode 蒙古文

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

// 使用重导出的 `mongol_norm` 归一化为规范 Unicode 蒙古文字符串
let shaper = Shaper::new(Locale::Mng);
let unicode_word = shaper.normalize_positioned_written_units(&units).unwrap();
assert!(!unicode_word.is_empty());
```

#### 2. 转换包含空格与标点的完整文本

`menk_word_to_utn57` 处理单个孟克字母连续片段（`U+E264..=U+E34F`）。处理混排完整文本时，只需切出孟克编码片段转换，其余字符原样保留：

```rust
use menk_to_utn57::mongol_norm::{Locale, Shaper};
use menk_to_utn57::menk_word_to_utn57;

fn convert_menk_text(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let shaper = Shaper::new(Locale::Mng);
    let mut out = String::with_capacity(input.len());
    let mut run = String::new();

    for ch in input.chars() {
        if ('\u{E264}'..='\u{E34F}').contains(&ch) {
            run.push(ch);
        } else {
            if !run.is_empty() {
                let units = menk_word_to_utn57(&run)?;
                out.push_str(&shaper.normalize_positioned_written_units(&units)?);
                run.clear();
            }
            out.push(ch);
        }
    }
    if !run.is_empty() {
        let units = menk_word_to_utn57(&run)?;
        out.push_str(&shaper.normalize_positioned_written_units(&units)?);
    }
    Ok(out)
}
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
│   ├── menk_char_shapes.json      # 236 个孟克字符 (U+E264..=U+E34F) -> MenkShape 拆分表
│   └── menk_shape_map.json        # 44 种 MenkShape -> 四位置 UTN #57 书写单元映射表
├── docs/
│   └── menk_positions.html        # 内嵌子集字体的中英双语可视化位置核对表
├── scripts/
│   ├── generate.py                # 从 data/*.json 生成 src/generated/*.rs
│   └── generate_positions_html.py # 生成 docs/menk_positions.html
├── src/
│   ├── conversion.rs              # 字形提取与子链切分 UTN #57 转换逻辑
│   ├── generated/
│   │   ├── menk_shapes.rs         # 自动生成的 MenkShape 枚举与字符位置判定函数
│   │   └── utn57_map.rs           # 自动生成的 MenkShape::to_utn57 映射表
│   └── lib.rs                     # 公共导出与 mongol_norm 重导出
└── tests/
    └── conversion_tests.rs        # 集成测试与本地词库全量校验
```

### 开发与测试

```bash
# 修改 data/*.json 后重新生成 Rust 映射代码
python3 scripts/generate.py

# 重新生成可视化核对页面 docs/menk_positions.html
python3 scripts/generate_positions_html.py

# 运行静态检查与测试（含 README 文档测试）
cargo clippy --all-targets -- -D warnings
cargo test
```

## License / 许可证

Licensed under the [Apache License, Version 2.0](LICENSE). / 本项目采用 [Apache-2.0](LICENSE) 许可证开源。
