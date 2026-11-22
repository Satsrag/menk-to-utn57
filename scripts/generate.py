#!/usr/bin/env python3
"""Generate Rust Menk shape extraction and MenkShape -> UTN #57 mapping tables.
从 data/ 目录下的 JSON 数据表生成 Rust 孟克字形提取与 MenkShape -> UTN #57 映射代码。

Inputs / 输入:
  - data/menk_char_shapes.json
  - data/menk_char_positions.json
  - data/menk_shape_map.json
  - data/utn57_letter_variants.json
Outputs / 输出:
  - src/generated/menk_shapes.rs
  - src/generated/utn57_map.rs
  - src/generated/utn57_variants.rs
"""

from __future__ import annotations

import json
import shutil
import subprocess
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DATA_DIR = ROOT / "data"
CHAR_SHAPES_PATH = DATA_DIR / "menk_char_shapes.json"
POSITIONS_PATH = DATA_DIR / "menk_char_positions.json"
SHAPE_MAP_PATH = DATA_DIR / "menk_shape_map.json"
LETTER_VARIANTS_PATH = DATA_DIR / "utn57_letter_variants.json"
OUT_DIR = ROOT / "src" / "generated"

ALL_UTN57_UNITS = {
    "A", "Aa", "B", "B2", "C", "Ch", "Cr", "D", "Dd", "F", "G", "Gx",
    "H", "Hr", "Hx", "I", "Ix", "J", "K", "K2", "L", "M", "N", "O",
    "P", "R", "Rh", "S", "Sh", "Sz", "T", "U", "Ue", "Ux", "W", "Y",
    "Z", "Zr",
}

POS_TO_VARIANT = {
    "isol": "Isol",
    "init": "Init",
    "medi": "Medi",
    "fina": "Fina",
}

FVS_TO_EXPR = {
    None: "None",
    1: "Some(Fvs::Fvs1)",
    2: "Some(Fvs::Fvs2)",
    3: "Some(Fvs::Fvs3)",
    4: "Some(Fvs::Fvs4)",
}


def expand_written_units(units: list[str], slot: str) -> list[str]:
    """Expand single-unit duplicates (`Dd`, `Cr`, medial `H`/`Hx`) just as `mongol_norm::Shaper::shape` does.
    按 `mongol_norm::Shaper::shape` 的规则展开单单元同形异码（`Dd`、`Cr`、词中 `H`/`Hx`）。
    """
    expanded: list[str] = []
    for unit in units:
        if unit == "Dd":
            expanded.extend(["O", "A"])
        elif unit == "H" and slot == "medi":
            expanded.extend(["A", "A"])
        elif unit == "Hx" and slot == "medi":
            expanded.extend(["N", "N"])
        elif unit == "Cr":
            expanded.extend(["O", "O"])
        else:
            expanded.append(unit)
    return expanded


def format_char_predicate(
    fn_name: str,
    doc_en: str,
    doc_zh: str,
    codepoints: list[int],
    cp_latin: dict[int, str],
) -> list[str]:
    """Format a `const fn(char) -> bool` match predicate with readable comments.
    生成带可读码位与拉丁转写注释的 `const fn(char) -> bool` 匹配函数。
    """
    out = [
        f"/// {doc_en}",
        "///",
        f"/// {doc_zh}",
        f"pub const fn {fn_name}(ch: char) -> bool {{",
        "    matches!(",
        "        ch,",
    ]
    for idx, cp in enumerate(codepoints):
        ch = chr(cp)
        prefix = "        " if idx == 0 else "        | "
        out.append(f"{prefix}'{ch}' // U+{cp:04X} ({cp_latin[cp]})")
    out.extend([
        "    )",
        "}",
        "",
    ])
    return out


def generate() -> None:
    """Read JSON tables in `data/` and write Rust source files into `src/generated/`.
    读取 `data/` 目录下的 JSON 表并生成 `src/generated/` 下的 Rust 源码文件。
    """
    char_shapes_raw: dict[str, dict[str, object]] = json.loads(
        CHAR_SHAPES_PATH.read_text(encoding="utf-8")
    )
    pos_cfg: dict[str, list[str]] = json.loads(POSITIONS_PATH.read_text(encoding="utf-8"))
    shape_map: dict[str, dict[str, str]] = json.loads(SHAPE_MAP_PATH.read_text(encoding="utf-8"))
    letter_variants_raw: dict[str, dict[str, object]] = json.loads(
        LETTER_VARIANTS_PATH.read_text(encoding="utf-8")
    )

    expected_letter_cps = list(range(0x1820, 0x1843))
    parsed_letter_cps = sorted(int(k.removeprefix("U+"), 16) for k in letter_variants_raw)
    if parsed_letter_cps != expected_letter_cps:
        raise ValueError("data/utn57_letter_variants.json must cover U+1820..=U+1842")

    latin_to_unicode: dict[str, str] = {}
    for hex_key, item in letter_variants_raw.items():
        cp = int(hex_key.removeprefix("U+"), 16)
        uch = str(item["char"])
        if ord(uch) != cp:
            raise ValueError(f"Code point mismatch in {hex_key}")
        lat = str(item["latin"])
        if len(lat) != 1:
            raise ValueError(f"Expected single-character Latin in {hex_key}, got {lat!r}")
        if lat == "k":
            if cp not in (0x183A, 0x183B):
                raise ValueError(f"Unexpected 'k' letter {hex_key}")
        else:
            if lat in latin_to_unicode:
                raise ValueError(f"Duplicate Latin key {lat!r} in utn57_letter_variants.json")
            latin_to_unicode[lat] = uch

    only_init_cps = sorted(int(s.removeprefix("U+"), 16) for s in pos_cfg["only_init"])
    only_fina_cps = sorted(int(s.removeprefix("U+"), 16) for s in pos_cfg["only_fina"])

    for name, entry in shape_map.items():
        if len(entry["char"]) != 1:
            raise ValueError(f"{name}.char must be a single character")

    # Group characters by shape sequence: single-shape entries first, then multi-shape entries.
    # 按字形序列归组：单字形在前，多字形组合在后。
    cp_latin: dict[int, str] = {}
    seq_groups: dict[tuple[str, ...], list[int]] = defaultdict(list)
    for hex_key, item in char_shapes_raw.items():
        cp = int(hex_key.removeprefix("U+"), 16)
        ch = str(item["char"])
        if ord(ch) != cp:
            raise ValueError(f"Code point mismatch for {hex_key}: got U+{ord(ch):04X}")
        latin = str(item["latin"])
        if len(latin) != 1 or (latin != "k" and latin not in latin_to_unicode):
            raise ValueError(f"Unknown or multi-char Latin in {hex_key}: {latin!r}")
        shapes = tuple(str(s) for s in item["shapes"])  # type: ignore[union-attr]
        for s in shapes:
            if s not in shape_map:
                raise ValueError(f"Unknown MenkShape '{s}' in {hex_key}")
        cp_latin[cp] = latin
        seq_groups[shapes].append(cp)

    expected_cps = list(range(0xE264, 0xE350))
    if sorted(cp_latin.keys()) != expected_cps:
        raise ValueError("data/menk_char_shapes.json must cover U+E264..=U+E34F")

    single_shapes = {seq[0] for seq in seq_groups if len(seq) == 1}
    if single_shapes != set(shape_map.keys()):
        missing = set(shape_map.keys()) - single_shapes
        extra = single_shapes - set(shape_map.keys())
        raise ValueError(f"Mismatch between single shapes and shape_map: missing={missing}, extra={extra}")

    shape_order = {name: idx for idx, name in enumerate(shape_map)}
    ordered_groups = sorted(
        seq_groups.items(),
        key=lambda kv: (0 if len(kv[0]) == 1 else 1, [shape_order[s] for s in kv[0]], kv[1][0]),
    )

    # 1. Generate src/generated/menk_shapes.rs
    # 1. 生成 src/generated/menk_shapes.rs
    lines: list[str] = [
        "// Generated by scripts/generate.py from data/*.json — DO NOT EDIT.",
        "// 由 scripts/generate.py 根据 data/*.json 自动生成，请勿手动修改。",
        "",
        "/// Single-character Menk glyph shape (`44` distinct shapes in `U+E264..=U+E34F`).",
        "///",
        "/// 孟克编码单字符基础字形（共 `44` 种，位于 `U+E264..=U+E34F`）。",
        "#[allow(non_camel_case_types)]",
        "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]",
        "pub enum MenkShape {",
    ]
    for name, entry in shape_map.items():
        ch = entry["char"]
        lines.append(f"    /// `{name}`: `'{ch}'` (`U+{ord(ch):04X}`).")
        lines.append(f"    {name},")
    lines.append("}")
    lines.append("")

    lines.append("impl MenkShape {")
    lines.append(f"    /// All {len(shape_map)} single-character Menk shapes.")
    lines.append("    ///")
    lines.append(f"    /// 全部 {len(shape_map)} 种孟克单字符基础字形。")
    lines.append("    pub const ALL: &'static [Self] = &[")
    for name in shape_map:
        lines.append(f"        Self::{name},")
    lines.append("    ];")
    lines.append("")

    lines.append("    /// Key name in `data/menk_shape_map.json`.")
    lines.append("    ///")
    lines.append("    /// 在 `data/menk_shape_map.json` 中对应的键名。")
    lines.append("    pub const fn name(self) -> &'static str {")
    lines.append("        match self {")
    for name in shape_map:
        lines.append(f'            Self::{name} => "{name}",')
    lines.append("        }")
    lines.append("    }")
    lines.append("")

    lines.append("    /// Representative Menk PUA shape character.")
    lines.append("    ///")
    lines.append("    /// 该基础字形对应的代表孟克字符。")
    lines.append("    pub const fn shape_char(self) -> char {")
    lines.append("        match self {")
    for name, entry in shape_map.items():
        ch = entry["char"]
        lines.append(f"            Self::{name} => '{ch}', // U+{ord(ch):04X}")
    lines.append("        }")
    lines.append("    }")
    lines.append("")

    lines.append("    /// Parse a representative Menk PUA shape character into [`MenkShape`].")
    lines.append("    ///")
    lines.append("    /// 将代表孟克字形字符解析为 [`MenkShape`]。")
    lines.append("    pub const fn from_shape_char(ch: char) -> Option<Self> {")
    lines.append("        match ch {")
    for name, entry in shape_map.items():
        ch = entry["char"]
        lines.append(f"            '{ch}' => Some(Self::{name}), // U+{ord(ch):04X}")
    lines.append("            _ => None,")
    lines.append("        }")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    lines.append("/// Decompose a Menk PUA character (`U+E264..=U+E34F`) into its [`MenkShape`] sequence.")
    lines.append("///")
    lines.append("/// 将孟克字符（`U+E264..=U+E34F`）拆分为对应的 [`MenkShape`] 基础字形序列。")
    lines.append("pub const fn menk_char_shapes(ch: char) -> Option<&'static [MenkShape]> {")
    lines.append("    use MenkShape::*;")
    lines.append("    match ch {")
    for shapes, cps in ordered_groups:
        seq = ", ".join(shapes)
        if len(cps) == 1:
            cp = cps[0]
            lines.append(f"        '{chr(cp)}' => Some(&[{seq}]), // U+{cp:04X} ({cp_latin[cp]})")
        else:
            for idx, cp in enumerate(cps):
                prefix = "        " if idx == 0 else "        | "
                if idx + 1 < len(cps):
                    lines.append(f"{prefix}'{chr(cp)}' // U+{cp:04X} ({cp_latin[cp]})")
                else:
                    lines.append(
                        f"{prefix}'{chr(cp)}' => Some(&[{seq}]), // U+{cp:04X} ({cp_latin[cp]})"
                    )
    lines.append("        _ => None,")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    lines.extend(
        format_char_predicate(
            "is_menk_char_only_init",
            "True when `ch` is strictly initial (starts a new sub-chain; from `data/menk_char_positions.json`).",
            "判断 `ch` 是否仅用于词首（在前方切分子链；数据来自 `data/menk_char_positions.json`）。",
            only_init_cps,
            cp_latin,
        )
    )
    lines.extend(
        format_char_predicate(
            "is_menk_char_only_fina",
            "True when `ch` is strictly final (closes the current sub-chain; from `data/menk_char_positions.json`).",
            "判断 `ch` 是否仅用于词尾（在后方切分子链；数据来自 `data/menk_char_positions.json`）。",
            only_fina_cps,
            cp_latin,
        )
    )

    lines.append("/// Return the representative shape character string of a single character (`ch` unchanged if outside Menk).")
    lines.append("///")
    lines.append("/// 获取单个字符对应的代表字形字符串（非孟克字符原样返回）。")
    lines.append("pub fn get_char_shape(ch: char) -> String {")
    lines.append("    if let Some(shapes) = menk_char_shapes(ch) {")
    lines.append("        shapes.iter().map(|s| s.shape_char()).collect()")
    lines.append("    } else {")
    lines.append("        ch.to_string()")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    lines.append("/// Convert Menk characters in `text` to their representative shape characters, passing other characters through.")
    lines.append("///")
    lines.append("/// 将文本中的孟克字符转换为代表字形字符，其余字符原样保留。")
    lines.append("pub fn get_shape(text: &str) -> String {")
    lines.append("    let mut out = String::with_capacity(text.len());")
    lines.append("    for ch in text.chars() {")
    lines.append("        if let Some(shapes) = menk_char_shapes(ch) {")
    lines.append("            for &s in shapes {")
    lines.append("                out.push(s.shape_char());")
    lines.append("            }")
    lines.append("        } else {")
    lines.append("            out.push(ch);")
    lines.append("        }")
    lines.append("    }")
    lines.append("    out")
    lines.append("}")
    lines.append("")

    # Group Menk characters by phoneme in codepoint order for `menk_char_latin` and `menk_char_letter`.
    # 按基础字母音位分组（保持码位顺序）以生成 `menk_char_latin` 与 `menk_char_letter`。
    latin_groups: dict[str, list[int]] = defaultdict(list)
    for cp in expected_cps:
        latin_groups[cp_latin[cp]].append(cp)

    lines.append("/// Return the single-character Mongolian Latin letter of a Menk PUA character (`U+E264..=U+E34F`),")
    lines.append("/// matching `MenkCode::getStandardLatin` with unambiguous single-character notation (`ë`, `ŋ`, `ħ`, `ř`, `ł`, `ẑ`, `ĉ`).")
    lines.append("///")
    lines.append("/// 获取单个蒙科立字符（`U+E264..=U+E34F`）对应的单字符蒙古文拉丁转写（与 PHP `MenkCode::getStandardLatin` 对应，采用无歧义单字符记法）。")
    lines.append("pub const fn menk_char_latin(ch: char) -> Option<char> {")
    lines.append("    match ch {")
    for latin_val, cps in latin_groups.items():
        rhs_expr = f"Some('{latin_val}')"
        rhs_comment = f"'{latin_val}' (U+{ord(latin_val):04X})"
        if len(cps) == 1:
            cp = cps[0]
            lines.append(f"        '{chr(cp)}' => {rhs_expr}, // U+{cp:04X} -> {rhs_comment}")
        else:
            for idx, cp in enumerate(cps):
                prefix = "        " if idx == 0 else "        | "
                if idx + 1 < len(cps):
                    lines.append(f"{prefix}'{chr(cp)}' // U+{cp:04X}")
                else:
                    lines.append(f"{prefix}'{chr(cp)}' => {rhs_expr}, // U+{cp:04X} -> {rhs_comment}")
    lines.append("        _ => None,")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    lines.append("/// Return the approximate Unicode Mongolian letter (`U+1820..=U+1842`) of a Menk PUA character (`U+E264..=U+E34F`).")
    lines.append("///")
    lines.append("/// 获取单个蒙科立字符（`U+E264..=U+E34F`）对应的粗略蒙古文 Unicode 基础字母（`U+1820..=U+1842`，不含 FVS / MVS）。")
    lines.append("pub const fn menk_char_letter(ch: char, k_letter: crate::KLetter) -> Option<char> {")
    lines.append("    match ch {")
    for latin_val, cps in latin_groups.items():
        if latin_val == "k":
            rhs_expr = "Some(k_letter.as_char())"
            rhs_comment = "U+183A / U+183B"
        else:
            uch = latin_to_unicode[latin_val]
            rhs_expr = f"Some('{uch}')"
            rhs_comment = f"U+{ord(uch):04X}"
        if len(cps) == 1:
            cp = cps[0]
            lines.append(f"        '{chr(cp)}' => {rhs_expr}, // U+{cp:04X} -> {rhs_comment}")
        else:
            for idx, cp in enumerate(cps):
                prefix = "        " if idx == 0 else "        | "
                if idx + 1 < len(cps):
                    lines.append(f"{prefix}'{chr(cp)}' // U+{cp:04X}")
                else:
                    lines.append(f"{prefix}'{chr(cp)}' => {rhs_expr}, // U+{cp:04X} -> {rhs_comment}")
    lines.append("        _ => None,")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    (OUT_DIR / "menk_shapes.rs").write_text("\n".join(lines), encoding="utf-8")

    # 2. Generate src/generated/utn57_map.rs using mongol_norm types directly
    # 2. 生成直接使用 mongol_norm 类型的 src/generated/utn57_map.rs
    map_lines: list[str] = [
        "// Generated by scripts/generate.py from data/menk_shape_map.json — DO NOT EDIT.",
        "// 由 scripts/generate.py 根据 data/menk_shape_map.json 自动生成，请勿手动修改。",
        "",
        "use super::menk_shapes::MenkShape;",
        "use mongol_norm::{Position, PositionedWrittenUnit, UnitPosition, WrittenUnit};",
        "",
        "impl MenkShape {",
        "    /// Map `(self, position)` to a UTN #57 [`PositionedWrittenUnit`] via `data/menk_shape_map.json`.",
        "    ///",
        "    /// 根据 `data/menk_shape_map.json` 的四位置槽位表将 `(self, position)` 映射为 UTN #57 [`PositionedWrittenUnit`]。",
        "    pub const fn to_utn57(self, position: Position) -> PositionedWrittenUnit {",
        "        match (self, position) {",
    ]

    for name, entry in shape_map.items():
        for slot in ("isol", "init", "medi", "fina"):
            raw = entry[slot]
            unit_s, pos_s = raw.split(":")
            if unit_s not in ALL_UTN57_UNITS:
                raise ValueError(f"Unknown WrittenUnit '{unit_s}' in {name}.{slot}")
            pos_var = POS_TO_VARIANT[pos_s]
            slot_var = POS_TO_VARIANT[slot]
            map_lines.append(
                f"            (Self::{name}, Position::{slot_var}) => "
                f"PositionedWrittenUnit::new(WrittenUnit::{unit_s}, UnitPosition::{pos_var}),"
            )

    map_lines.extend([
        "        }",
        "    }",
        "}",
        "",
    ])

    (OUT_DIR / "utn57_map.rs").write_text("\n".join(map_lines), encoding="utf-8")

    # 3. Generate src/generated/utn57_variants.rs from data/utn57_letter_variants.json
    # 3. 根据 data/utn57_letter_variants.json 生成 src/generated/utn57_variants.rs

    var_lines: list[str] = [
        "// Generated by scripts/generate.py from data/utn57_letter_variants.json — DO NOT EDIT.",
        "// 由 scripts/generate.py 根据 data/utn57_letter_variants.json 自动生成，请勿手动修改。",
        "",
        "use mongol_norm::{Fvs, Position, WrittenUnit};",
        "",
        "/// One positional FVS variant of a Unicode Mongolian letter (`U+1820..=U+1842`) in UTN #57.",
        "///",
        "/// UTN #57 中单个蒙古文 Unicode 字母（`U+1820..=U+1842`）在特定位置下的自由变体（FVS）条目。",
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]",
        "pub struct LetterVariant {",
        "    /// Free Variation Selector (`None` when UTN #57 defines no explicit FVS for this form).",
        "    /// 自由变体选择符（若标准未对该字形定义显式 FVS 则为 `None`）。",
        "    pub fvs: Option<Fvs>,",
        "    /// Raw UTN #57 written units (`MNG.json`).",
        "    /// 原始 UTN #57 书写单元序列（对应 `MNG.json`）。",
        "    pub written: &'static [WrittenUnit],",
        "    /// Written units with single-unit duplicates (`Dd`, `Cr`, medial `H`/`Hx`) expanded to match [`mongol_norm::Shaper::shape`].",
        "    /// 展开单单元同形异码（`Dd`、`Cr`、词中 `H`/`Hx`）后的书写单元序列，与 [`mongol_norm::Shaper::shape`] 对齐。",
        "    pub expanded_written: &'static [WrittenUnit],",
        "    /// Whether this variant is the context-free default at this position.",
        "    /// 是否为该位置下的无上下文默认变体。",
        "    pub default: bool,",
        "}",
        "",
    ]

    def fmt_variant(item: dict[str, object], slot: str) -> str:
        fvs_val = item["fvs"]
        if fvs_val not in FVS_TO_EXPR:
            raise ValueError(f"Invalid FVS value: {fvs_val}")
        raw_units = [str(u) for u in item["written"]]  # type: ignore[union-attr]
        for u in raw_units:
            if u not in ALL_UTN57_UNITS:
                raise ValueError(f"Unknown WrittenUnit '{u}' in utn57_letter_variants.json")
        exp_units = expand_written_units(raw_units, slot)
        raw_expr = ", ".join(f"WrittenUnit::{u}" for u in raw_units)
        exp_expr = ", ".join(f"WrittenUnit::{u}" for u in exp_units)
        def_expr = "true" if item["default"] else "false"
        return (
            f"LetterVariant {{ fvs: {FVS_TO_EXPR[fvs_val]}, "  # type: ignore[index]
            f"written: &[{raw_expr}], expanded_written: &[{exp_expr}], default: {def_expr} }}"
        )

    var_lines.extend([
        "/// Look up all UTN #57 positional FVS variants for a Unicode Mongolian letter (`U+1820..=U+1842`).",
        "///",
        "/// 查询蒙古文 Unicode 字母（`U+1820..=U+1842`）在指定位置下的全部 UTN #57 FVS 变体条目。",
        "pub const fn utn57_letter_variants(letter: char, position: Position) -> &'static [LetterVariant] {",
        "    match (letter, position) {",
    ])

    total_variants = 0
    for hex_key, entry in letter_variants_raw.items():
        cp = int(hex_key.removeprefix("U+"), 16)
        ch = str(entry["char"])
        if ord(ch) != cp:
            raise ValueError(f"Code point mismatch in {hex_key}")
        latin_s = str(entry["latin"])
        alias_s = str(entry["alias"])
        variants_by_pos: dict[str, list[dict[str, object]]] = entry["variants"]  # type: ignore[assignment]
        for slot in ("isol", "init", "medi", "fina"):
            slot_var = POS_TO_VARIANT[slot]
            items = variants_by_pos[slot]
            total_variants += len(items)
            var_lines.append(
                f"        ('{ch}', Position::{slot_var}) => &[ // U+{cp:04X} ({latin_s}, {alias_s})"
            )
            for item in items:
                var_lines.append(f"            {fmt_variant(item, slot)},")
            var_lines.append("        ],")

    var_lines.extend([
        "        _ => &[],",
        "    }",
        "}",
        "",
        "/// Deduplicated candidate variants per `(letter, position)`, preferring explicit `Some(Fvs)` so",
        "/// every distinct `expanded_written` shape can be pinned before redundant-FVS stripping.",
        "///",
        "/// 按 `expanded_written` 去重后的候选变体表（优先保留带显式 `Some(Fvs)` 的条目以便锁定字形，随后再剥离冗余 FVS）。",
        "pub(crate) const fn utn57_pinning_variants(letter: char, position: Position) -> &'static [LetterVariant] {",
        "    match (letter, position) {",
    ])

    for hex_key, entry in letter_variants_raw.items():
        cp = int(hex_key.removeprefix("U+"), 16)
        ch = str(entry["char"])
        latin_s = str(entry["latin"])
        alias_s = str(entry["alias"])
        variants_by_pos = entry["variants"]  # type: ignore[assignment]
        for slot in ("isol", "init", "medi", "fina"):
            slot_var = POS_TO_VARIANT[slot]
            items = variants_by_pos[slot]
            # Order explicit FVS variants before `fvs: null` so pinning always gets `Some(Fvs)` when available,
            # while keeping default variants earlier for deterministic tie-breaking.
            # 将带显式 FVS 的条目排在 `fvs: null` 之前以确保可锁定字形，同时优先排默认变体以保持稳定消歧顺序。
            ordered = sorted(
                items,
                key=lambda x: (1 if x["fvs"] is None else 0, 0 if x["default"] else 1, x["fvs"] or 0),
            )
            seen_exp: set[tuple[str, ...]] = set()
            deduped: list[dict[str, object]] = []
            for item in ordered:
                exp = tuple(expand_written_units([str(u) for u in item["written"]], slot))  # type: ignore[union-attr]
                if exp not in seen_exp:
                    seen_exp.add(exp)
                    deduped.append(item)
            deduped.sort(key=lambda x: (0 if x["default"] else 1, x["fvs"] or 0))
            var_lines.append(
                f"        ('{ch}', Position::{slot_var}) => &[ // U+{cp:04X} ({latin_s}, {alias_s})"
            )
            for item in deduped:
                var_lines.append(f"            {fmt_variant(item, slot)},")
            var_lines.append("        ],")

    var_lines.extend([
        "        _ => &[],",
        "    }",
        "}",
        "",
    ])

    (OUT_DIR / "utn57_variants.rs").write_text("\n".join(var_lines), encoding="utf-8")

    if shutil.which("rustfmt"):
        subprocess.run(
            [
                "rustfmt",
                "--edition",
                "2021",
                str(OUT_DIR / "menk_shapes.rs"),
                str(OUT_DIR / "utn57_map.rs"),
                str(OUT_DIR / "utn57_variants.rs"),
            ],
            check=True,
        )

    print(
        f"Generated {len(ordered_groups)} shape groups ({len(cp_latin)} chars), "
        f"{len(shape_map)} shape mappings, "
        f"{len(only_init_cps)} only_init & {len(only_fina_cps)} only_fina, "
        f"{len(letter_variants_raw)} UTN #57 letters ({total_variants} variants) into {OUT_DIR}"
    )


if __name__ == "__main__":
    generate()
