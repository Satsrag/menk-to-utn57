# menk-to-utn57

[English](#english) | [中文](#中文) | [Live WASM Demo](https://satsrag.github.io/menk-to-utn57/)

---

<a id="english"></a>
## English

**menk-to-utn57** is a lightweight, pure-Rust library for Traditional Mongolian encoding conversion:

- Extracts positioned [UTN #57](https://www.unicode.org/notes/tn57/) written units (`PositionedWrittenUnit`) and single-character Mongolian Latin (`get_standard_latin`) from **Menk** PUA words (`U+E264..=U+E34F`).
- Resolves a Menk word + optional Latin spelling (`menk_word_and_latin_to_unicode`), or UTN #57 written units + pure Unicode Mongolian letters (`written_units_and_letters_to_unicode`), automatically deducing Chachlag `MVS` (`U+180E`) and returning **all** matching minimal-FVS standard Unicode Mongolian [`Solution`](src/resolve.rs)s (verified in-process by [`mongol-norm`](https://crates.io/crates/mongol-norm), `wasm32` compatible).

### Installation

```toml
[dependencies]
menk-to-utn57 = { git = "https://github.com/Satsrag/menk-to-utn57" }
```

### Usage

#### 1. Menk word + optional Latin → standard Unicode (`menk_word_and_latin_to_unicode`)

Pass a Menk PUA word (`U+E264..=U+E34F`) and `latin: Option<&str>`:
- `None`: automatically transcribes `menk_word` via `get_standard_latin` first, then resolves to Unicode.
- `Some(latin)`: uses your proofread single-character Mongolian Latin spelling (no `MVS` or `FVS`; case-insensitive, `ö`/`ü` accepted as aliases for `ɵ`/`ʉ`).

Returns `Result<Vec<Solution>, ResolveError>` sorted by `fvs_count` ascending:

```rust
use menk_to_utn57::{
    get_standard_latin, menk_word_and_latin_to_unicode, menk_word_and_latin_to_unicode_with_k,
    KLetter,
};

// 1) Pass `None` to auto-transcribe Latin via `get_standard_latin` (e.g. Chachlag "baina")
let baina_menk = "\u{E2C1}\u{E26C}\u{E281}\u{E2B6}\u{E26A}";
assert_eq!(get_standard_latin(baina_menk).unwrap(), "baina");
let baina = menk_word_and_latin_to_unicode(baina_menk, None).unwrap();
assert_eq!(baina.len(), 1);
assert_eq!(baina[0].unicode, "ᠪᠠᠢᠨ\u{180E}ᠠ");
assert_eq!(baina[0].fvs_count, 0);
assert!(!baina[0].used_greedy_fallback);

// 2) Pass `Some(latin)` with proofread Latin ("dʉ" or alias "dü") -> minimal FVS1 + FVS2
let du = menk_word_and_latin_to_unicode("\u{E310}\u{E2A7}", Some("dʉ")).unwrap();
assert_eq!(du[0].unicode, "ᠳ\u{180B}ᠦ\u{180C}");
assert_eq!(du[0].fvs_count, 2);

// 3) Ambiguous glyph split "zʉil" returns ALL valid decompositions (sorted by fvs_count)
let zuil = menk_word_and_latin_to_unicode("\u{E318}\u{E2A9}\u{E27E}\u{E2F9}", None).unwrap();
assert_eq!(zuil.len(), 2);
assert_eq!(zuil[0].unicode, "ᠵᠦᠢᠯ");          // 0 FVS
assert_eq!(zuil[1].unicode, "ᠵᠦ\u{180D}ᠢᠯ");  // 1 FVS

// 4) Choose 'ᠺ' (Inner Mongolia, default) vs 'ᠻ' (Mongolia) for 'k'
let kino_menk = "\u{E333}\u{E27F}\u{E2B9}\u{E286}";
assert_eq!(
    menk_word_and_latin_to_unicode(kino_menk, None).unwrap()[0].unicode,
    "ᠺᠢᠨᠣ\u{180B}"
);
assert_eq!(
    menk_word_and_latin_to_unicode_with_k(kino_menk, Some("kino"), KLetter::Kha).unwrap()[0]
        .unicode,
    "ᠻᠢᠨᠣ\u{180B}"
);
```

Each `Solution` contains:
- `unicode: String` — resolved UTN #57 Unicode Mongolian word;
- `fvs_count: usize` — number of FVS characters used;
- `letters: Vec<ResolvedLetter>` — per-letter `(letter, position, written, fvs)` breakdown;
- `used_greedy_fallback: bool` — `false` when proven globally minimal within the 65,536-check exact budget; `true` if an extreme word fell back to greedy stripping (shape is still valid, flagged for optional manual review).

<details>
<summary>Single-character Latin inventory (34 letters → 35 Mongolian letters <code>U+1820..=U+1842</code>)</summary>

| Latin | Letter | Latin | Letter | Latin | Letter | Latin | Letter | Latin | Letter |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `a` | `ᠠ` | `ë` | `ᠧ` | `m` | `ᠮ` | `c` | `ᠴ` | `č` | `ᠼ` |
| `e` | `ᠡ` | `n` | `ᠨ` | `l` | `ᠯ` | `z` | `ᠵ` | `ž` | `ᠽ` |
| `i` | `ᠢ` | `ŋ` | `ᠩ` | `s` | `ᠰ` | `y` | `ᠶ` | `ħ` | `ᠾ` |
| `o` | `ᠣ` | `b` | `ᠪ` | `š` | `ᠱ` | `r` | `ᠷ` | `ř` | `ᠿ` |
| `u` | `ᠤ` | `p` | `ᠫ` | `t` | `ᠲ` | `w` | `ᠸ` | `ł` | `ᡀ` |
| `ɵ` / `ö` | `ᠥ` | `h` | `ᠬ` | `d` | `ᠳ` | `f` | `ᠹ` | `ẑ` | `ᡁ` |
| `ʉ` / `ü` | `ᠦ` | `g` | `ᠭ` | | | `k` | `ᠺ` / `ᠻ` | `ĉ` | `ᡂ` |

</details>

#### 2. UTN #57 written units + pure Mongolian letters → standard Unicode (`written_units_and_letters_to_unicode`)

If you already have `&[PositionedWrittenUnit]` and pure Unicode Mongolian letters (`U+1820..=U+1842`, without `MVS` / `FVS`), call `written_units_and_letters_to_unicode` directly (completely decoupled from Menk):

```rust
use menk_to_utn57::{
    written_units_and_letters_to_unicode, Fvs, PositionedWrittenUnit, UnitPosition, WrittenUnit,
};

let units = [
    PositionedWrittenUnit::new(WrittenUnit::N, UnitPosition::Init),
    PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
    PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
    PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
    PositionedWrittenUnit::new(WrittenUnit::L, UnitPosition::Fina),
];

let solutions = written_units_and_letters_to_unicode(&units, "ᠨᠠᠠᠯ").unwrap();
assert_eq!(solutions.len(), 2);
assert_eq!(solutions[0].unicode, "ᠨᠠᠠ\u{180B}ᠯ");
assert_eq!(solutions[0].letters[2].fvs, Some(Fvs::Fvs1));
assert_eq!(solutions[1].unicode, "ᠨᠠ\u{180B}ᠠᠯ");
```

---

<a id="中文"></a>
## 中文

**menk-to-utn57** 是一个轻量级纯 Rust 传统蒙古文编码转换库：

- 将**蒙科立**（Menk PUA，`U+E264..=U+E34F`）词串提取为带位置的 [UTN #57](https://www.unicode.org/notes/tn57/) 书写单元（`PositionedWrittenUnit`）和单字符拉丁转写（`get_standard_latin`）。
- 结合**蒙科立词串 + 可选拉丁转写**（`menk_word_and_latin_to_unicode`），或**UTN #57 书写单元 + 纯蒙古文基础字母串**（`written_units_and_letters_to_unicode`），自动推导词尾分写 `Chachlag` 的 `MVS`（`U+180E`），枚举**全部**匹配的最小 FVS 标准 Unicode 蒙古文解法（[`Solution`](src/resolve.rs)），全程经 [`mongol-norm`](https://crates.io/crates/mongol-norm) 进程内校验（支持 `wasm32`）。

### 安装

```toml
[dependencies]
menk-to-utn57 = { git = "https://github.com/Satsrag/menk-to-utn57" }
```

### 使用方法

#### 1. 蒙科立词串 + 可选拉丁转写 → 标准 Unicode（`menk_word_and_latin_to_unicode`）

传入蒙科立词串（`U+E264..=U+E34F`）与 `latin: Option<&str>`：
- 传 `None`：默认自动调用 `get_standard_latin` 从蒙科立词串转写拉丁后再进行转换；
- 传 `Some(latin)`：使用校对后的单字符蒙古文拉丁转写（不含 `MVS` / `FVS`，兼容大小写及 `ö`/`ü` 别名）。

返回按 `fvs_count` 升序排列的 `Result<Vec<Solution>, ResolveError>`：

```rust
use menk_to_utn57::{
    get_standard_latin, menk_word_and_latin_to_unicode, menk_word_and_latin_to_unicode_with_k,
    KLetter,
};

// 1) 不传拉丁（None）：自动通过 `get_standard_latin` 转写拉丁并推导 MVS (U+180E)
let baina_menk = "\u{E2C1}\u{E26C}\u{E281}\u{E2B6}\u{E26A}";
assert_eq!(get_standard_latin(baina_menk).unwrap(), "baina");
let baina = menk_word_and_latin_to_unicode(baina_menk, None).unwrap();
assert_eq!(baina.len(), 1);
assert_eq!(baina[0].unicode, "ᠪᠠᠢᠨ\u{180E}ᠠ");
assert_eq!(baina[0].fvs_count, 0);
assert!(!baina[0].used_greedy_fallback);

// 2) 传入校对拉丁（Some）：如 "dʉ"（也兼容 "dü"），自动补全最小 FVS1 + FVS2
let du = menk_word_and_latin_to_unicode("\u{E310}\u{E2A7}", Some("dʉ")).unwrap();
assert_eq!(du[0].unicode, "ᠳ\u{180B}ᠦ\u{180C}");
assert_eq!(du[0].fvs_count, 2);

// 3) 存在多种笔画切分的词 "zʉil"：返回全部合法解法（最少 FVS 在前）
let zuil = menk_word_and_latin_to_unicode("\u{E318}\u{E2A9}\u{E27E}\u{E2F9}", None).unwrap();
assert_eq!(zuil.len(), 2);
assert_eq!(zuil[0].unicode, "ᠵᠦᠢᠯ");          // 0 FVS
assert_eq!(zuil[1].unicode, "ᠵᠦ\u{180D}ᠢᠯ");  // 1 FVS

// 4) 'k' 默认映射内蒙古标准 'ᠺ'，也可通过 `_with_k` 指定蒙古国标准 'ᠻ'
let kino_menk = "\u{E333}\u{E27F}\u{E2B9}\u{E286}";
assert_eq!(
    menk_word_and_latin_to_unicode(kino_menk, None).unwrap()[0].unicode,
    "ᠺᠢᠨᠣ\u{180B}"
);
assert_eq!(
    menk_word_and_latin_to_unicode_with_k(kino_menk, Some("kino"), KLetter::Kha).unwrap()[0]
        .unicode,
    "ᠻᠢᠨᠣ\u{180B}"
);
```

每条 `Solution` 包含：
- `unicode: String`：求解出的标准 UTN #57 Unicode 蒙古文单词；
- `fvs_count: usize`：插入的 FVS 数量；
- `letters: Vec<ResolvedLetter>`：逐字母的 `(letter, position, written, fvs)` 明细；
- `used_greedy_fallback: bool`：`false` 表示在 65,536 次精确搜索预算内严格求得全局最少 FVS；`true` 表示极端长词回退到了贪心剥离（字形仍严格正确，可供后续人工复核）。

<details>
<summary>单字符拉丁字母对照表（34 个单字符 → 35 个蒙古文字母 <code>U+1820..=U+1842</code>）</summary>

| 拉丁 | 蒙古文字母 | 拉丁 | 蒙古文字母 | 拉丁 | 蒙古文字母 | 拉丁 | 蒙古文字母 | 拉丁 | 蒙古文字母 |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `a` | `ᠠ` | `ë` | `ᠧ` | `m` | `ᠮ` | `c` | `ᠴ` | `č` | `ᠼ` |
| `e` | `ᠡ` | `n` | `ᠨ` | `l` | `ᠯ` | `z` | `ᠵ` | `ž` | `ᠽ` |
| `i` | `ᠢ` | `ŋ` | `ᠩ` | `s` | `ᠰ` | `y` | `ᠶ` | `ħ` | `ᠾ` |
| `o` | `ᠣ` | `b` | `ᠪ` | `š` | `ᠱ` | `r` | `ᠷ` | `ř` | `ᠿ` |
| `u` | `ᠤ` | `p` | `ᠫ` | `t` | `ᠲ` | `w` | `ᠸ` | `ł` | `ᡀ` |
| `ɵ` / `ö` | `ᠥ` | `h` | `ᠬ` | `d` | `ᠳ` | `f` | `ᠹ` | `ẑ` | `ᡁ` |
| `ʉ` / `ü` | `ᠦ` | `g` | `ᠭ` | | | `k` | `ᠺ` / `ᠻ` | `ĉ` | `ᡂ` |

</details>

#### 2. UTN #57 书写单元 + 纯蒙古文字母 → 标准 Unicode（`written_units_and_letters_to_unicode`）

如果手里已有 `&[PositionedWrittenUnit]` 字形序列与纯蒙古文基础字母串（`U+1820..=U+1842`，不含 `MVS` / `FVS`），可直接调用 `written_units_and_letters_to_unicode`（与蒙科立完全解耦）：

```rust
use menk_to_utn57::{
    written_units_and_letters_to_unicode, Fvs, PositionedWrittenUnit, UnitPosition, WrittenUnit,
};

let units = [
    PositionedWrittenUnit::new(WrittenUnit::N, UnitPosition::Init),
    PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
    PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
    PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
    PositionedWrittenUnit::new(WrittenUnit::L, UnitPosition::Fina),
];

let solutions = written_units_and_letters_to_unicode(&units, "ᠨᠠᠠᠯ").unwrap();
assert_eq!(solutions.len(), 2);
assert_eq!(solutions[0].unicode, "ᠨᠠᠠ\u{180B}ᠯ");
assert_eq!(solutions[0].letters[2].fvs, Some(Fvs::Fvs1));
assert_eq!(solutions[1].unicode, "ᠨᠠ\u{180B}ᠠᠯ");
```

## License / 许可证

Licensed under the [Apache License, Version 2.0](LICENSE). / 本项目采用 [Apache-2.0](LICENSE) 许可证开源。
