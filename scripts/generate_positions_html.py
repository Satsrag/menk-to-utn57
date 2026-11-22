#!/usr/bin/env python3
"""Generate self-contained bilingual `docs/menk_positions.html` for reviewing Menk `U+E264..=U+E34F`
`only_init` and `only_fina` flags.

生成内嵌子集字体的中英双语可视化核对表 `docs/menk_positions.html`（覆盖 `U+E264..=U+E34F` 共 236 项）。
"""

from __future__ import annotations

import base64
import io
import json
import re
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DATA_DIR = ROOT / "data"
CHAR_SHAPES_JSON = DATA_DIR / "menk_char_shapes.json"
POSITIONS_JSON = DATA_DIR / "menk_char_positions.json"
SHAPE_MAP_JSON = DATA_DIR / "menk_shape_map.json"
OUT_HTML = ROOT / "docs" / "menk_positions.html"

FONT_PATH = ROOT.parent / "fonts" / "menk.ttf"
WORD_PATH = ROOT / "word.txt"

_pos_cfg = json.loads(POSITIONS_JSON.read_text(encoding="utf-8"))
ONLY_INIT: set[int] = {int(s.removeprefix("U+"), 16) for s in _pos_cfg["only_init"]}
ONLY_FINA: set[int] = {int(s.removeprefix("U+"), 16) for s in _pos_cfg["only_fina"]}

# Bilingual reference notes (`review_flag`, `note_en`, `note_zh`)
# 中英双语核对说明（`是否重点核对`, `英文说明`, `中文说明`）
REVIEW_NOTES: dict[int, tuple[bool, str, str]] = {
    0xE269: (
        True,
        "Connected left-sweeping tail Aa:fina (lsb=0, strictly final)",
        "连写左撇尾 Aa:fina（无内置间距 lsb=0，确定词尾）",
    ),
    0xE26A: (
        True,
        "Detached Chachlag 'a' tail (built-in MVS gap lsb=206, strictly final)",
        "分离式 Chachlag a（内置 MVS 间距 lsb=206，确定词尾）",
    ),
    0xE274: (
        True,
        "Detached Chachlag 'e' tail (built-in MVS gap lsb=206, strictly final)",
        "分离式 Chachlag e（内置 MVS 间距 lsb=206，确定词尾）",
    ),
    0xE279: (
        True,
        "Standalone 'i' (A_INIT+G): crown is strictly init; bottom G bow can continue as medi",
        "单字 i (A_INIT+G)：顶部带冠仅 init，底部 G 弓形与 medi 通用",
    ),
    0xE27B: (
        True,
        "Final 'i' bow slot; shares glyph with G:medi when followed without space",
        "码表 i 尾槽位，与 G:medi 共享弓形，无空格后续时可按 medi 连写",
    ),
    0xE282: (
        True,
        "Isol/final 'i' (G bow); interchangeable with medial G bow",
        "独立/词尾 i (G)，与 G 弓形通用",
    ),
    0xE283: (
        True,
        "Standalone 'o' (A_INIT+O): crown is strictly init; bottom O belly matches O:medi",
        "单字 o (A_INIT+O)：顶部带冠仅 init，底部圆肚 O 与 medi 同形",
    ),
    0xE285: (
        True,
        "Final round 'o' (U:fina / B); interchangeable with B:medi bow when unspaced",
        "词尾大肚 o (U:fina / B)，无空格后续时与 B:medi 弓形通用",
    ),
    0xE286: (
        True,
        "Final 'o' belly (O:fina); identical ink to O:medi",
        "词尾圆肚 o (O:fina)，与 O:medi 同形",
    ),
    0xE287: (
        True,
        "Post-bow 'o' belly; identical ink to O:medi",
        "弓形辅音内 o，与 medi 同形",
    ),
    0xE28B: (
        True,
        "Standalone 'u' (A_INIT+O): crown is strictly init; bottom O belly matches O:medi",
        "单字 u (A_INIT+O)：顶部带冠仅 init，底部圆肚 O 与 medi 同形",
    ),
    0xE28D: (
        True,
        "Final round 'u' (U:fina / B); interchangeable with B:medi bow when unspaced",
        "词尾大肚 u (U:fina / B)，无空格后续时与 B:medi 弓形通用",
    ),
    0xE28E: (
        True,
        "Final 'u' belly (O:fina); identical ink to O:medi",
        "词尾圆肚 u (O:fina)，与 O:medi 同形",
    ),
    0xE28F: (
        True,
        "Post-bow 'u' belly; identical ink to O:medi",
        "弓形辅音内 u，与 medi 同形",
    ),
    0xE294: (
        True,
        "Standalone 'ɵ' (A_INIT+B): crown is strictly init; bottom B bow matches B:medi",
        "单字 ɵ (A_INIT+B)：顶部带冠仅 init，底部 B 与 medi 弓形通用",
    ),
    0xE296: (
        True,
        "Final round 'ɵ' (U:fina / B); interchangeable with B:medi bow",
        "词尾大肚 ɵ (U:fina / B)，与 B:medi 弓形通用",
    ),
    0xE298: (
        True,
        "Post-bow final 'ɵ' (Ue:fina); long tail, strictly final",
        "弓形辅音后 ɵ 尾 (Ue:fina)，底带长尾确定 final",
    ),
    0xE299: (
        True,
        "Final 'ɵ' belly (O:fina); identical ink to O:medi",
        "词尾圆肚 ɵ (O:fina)，与 O:medi 同形",
    ),
    0xE29A: (
        True,
        "Post-bow 'ɵ' belly; identical ink to O:medi",
        "弓形辅音内 ɵ，与 medi 同形",
    ),
    0xE2A1: (
        True,
        "Standalone 'ʉ' (A_INIT+B): crown is strictly init; bottom B bow matches B:medi",
        "单字 ʉ (A_INIT+B)：顶部带冠仅 init，底部 B 与 medi 弓形通用",
    ),
    0xE2A3: (
        True,
        "Final round 'ʉ' (U:fina / B); interchangeable with B:medi bow",
        "词尾大肚 ʉ (U:fina / B)，与 B:medi 弓形通用",
    ),
    0xE2A5: (
        True,
        "Post-bow final 'ʉ' (Ue:fina); long tail, strictly final",
        "弓形辅音后 ʉ 尾 (Ue:fina)，底带长尾确定 final",
    ),
    0xE2A6: (
        True,
        "Final 'ʉ' belly (O:fina); identical ink to O:medi",
        "词尾圆肚 ʉ (O:fina)，与 O:medi 同形",
    ),
    0xE2A7: (
        True,
        "Post-bow 'ʉ' belly; shared between medi and fina",
        "弓形辅音内 ʉ，与 medi 通用",
    ),
    0xE2B6: (
        True,
        "Dotted final 'n' before Chachlag (N:fina); strictly final",
        "Chachlag 前带点 n 尾 (N:fina)，确定词尾",
    ),
    0xE2C1: (
        True,
        "Initial 'b' (before a/e/i); shares glyph with B:medi",
        "b 首槽位（接 a/e/i），与 B:medi 同形",
    ),
    0xE2C2: (
        True,
        "Initial 'b' (before o/u); shares glyph with B:medi",
        "b 首槽位（接 o/u），与 B:medi 同形",
    ),
    0xE2C4: (
        True,
        "Final 'b' variant B2:fina (O+Aa); connected top, closed bottom, strictly final",
        "词尾异体 b 合字 B2:fina (O+Aa)，上连下闭，仅 final",
    ),
    0xE2C8: (
        True,
        "Initial 'p' (before a/e/i); shares glyph with P:medi",
        "p 首槽位（接 a/e/i），与 P:medi 同形",
    ),
    0xE2C9: (
        True,
        "Initial 'p' (before o/u); shares glyph with P:medi",
        "p 首槽位（接 o/u），与 P:medi 同形",
    ),
    0xE2D0: (
        True,
        "Feminine initial 'h' (before e/i); shares glyph with G:medi",
        "阴性 h 首槽位（接 e/i），与 G:medi 同形",
    ),
    0xE2D4: (
        True,
        "Feminine initial 'h' (before ɵ/ʉ); shares glyph with G:medi",
        "阴性 h 首槽位（接 ɵ/ʉ），与 G:medi 同形",
    ),
    0xE2FD: (
        True,
        "Initial 's'; shares glyph with S:medi",
        "s 首槽位，与 S:medi 同形",
    ),
    0xE2FE: (
        True,
        "Initial 's' variant; shares glyph with S:medi",
        "s 首槽位变体，与 S:medi 同形",
    ),
    0xE2FF: (
        True,
        "Final tailed 's' (S:fina); strictly final",
        "s 词尾长尾形 (S:fina)，仅 final",
    ),
    0xE303: (
        True,
        "Initial 'š'; shares glyph with Sh:medi",
        "š 首槽位，与 Sh:medi 同形",
    ),
    0xE305: (
        True,
        "Final tailed 'š' (Sh:fina); strictly final",
        "š 词尾长尾形 (Sh:fina)，仅 final",
    ),
    0xE308: (
        True,
        "Initial 't'; shares glyph with T:medi",
        "t 首槽位，与 T:medi 同形",
    ),
    0xE30A: (
        True,
        "Final 't' (T:fina); strictly final",
        "t 词尾形 (T:fina)，仅 final",
    ),
    0xE30E: (
        True,
        "Initial 'd' (T form); shares glyph with T:medi",
        "d 首槽位 (T 形)，与 T:medi 同形",
    ),
    0xE310: (
        True,
        "Initial 'd' (D form); shares glyph with D:medi",
        "d 首槽位 (D 形)，与 D:medi 同形",
    ),
    0xE312: (
        True,
        "Final tailed 'd' (D:fina); strictly final",
        "d 词尾长尾形 (D:fina)，仅 final",
    ),
    0xE315: (
        True,
        "Initial 'c'; shares glyph with Ch:medi",
        "c 首槽位，与 Ch:medi 同形",
    ),
    0xE316: (
        True,
        "Final 'c' (Ch:fina); strictly final",
        "c 词尾形 (Ch:fina)，仅 final",
    ),
    0xE318: (
        True,
        "Initial 'z' (I form); shares glyph with I:medi",
        "z 首槽位 (I 形)，与 I:medi 同形",
    ),
    0xE31B: (
        True,
        "Final tailed 'z' (J:fina); strictly final",
        "z 词尾长尾形 (J:fina)，仅 final",
    ),
    0xE31C: (
        True,
        "Final 'z' bow (I:fina / G); interchangeable with medial G bow",
        "z 词尾弓形 (I:fina / G)，与 G 弓形通用",
    ),
    0xE31E: (
        True,
        "Initial 'y'; shares glyph with Y:medi",
        "y 首槽位，与 Y:medi 同形",
    ),
    0xE31F: (
        True,
        "Final 'y' bow (I:fina / G); interchangeable with G bow (Chachlag split handled by Aa rule)",
        "y 词尾弓形 (I:fina / G)，与 G 弓形通用（后接 Aa 时由 Aa 规则切为 Chachlag）",
    ),
    0xE322: (
        True,
        "Initial 'r'; shares glyph with R:medi",
        "r 首槽位，与 R:medi 同形",
    ),
    0xE325: (
        True,
        "Final 'r' bow (R:fina / K); interchangeable with K bow (Chachlag split handled by Aa rule)",
        "r 词尾刺弓形 (R:fina / K)，与 K 弓形通用（后接 Aa 时由 Aa 规则切为 Chachlag）",
    ),
    0xE329: (
        True,
        "Initial 'w'; shares glyph with W:medi",
        "w 首槽位，与 W:medi 同形",
    ),
    0xE32A: (
        True,
        "Final tailed 'w' (W:fina); strictly final",
        "w 词尾长尾形 (W:fina)，仅 final",
    ),
    0xE32B: (
        True,
        "Final round 'w' (U:fina / B); interchangeable with B bow (Chachlag split handled by Aa rule)",
        "w 词尾大肚形 (U:fina / B)，与 B 弓形通用（后接 Aa 时由 Aa 规则切为 Chachlag）",
    ),
    0xE33F: (
        True,
        "Initial 'č'; shares glyph with C:medi",
        "č 首槽位，与 C:medi 同形",
    ),
    0xE340: (
        True,
        "Final 'č' (C:fina); strictly final",
        "č 词尾形 (C:fina)，仅 final",
    ),
    0xE342: (
        True,
        "Initial 'ž'; shares glyph with Z:medi",
        "ž 首槽位，与 Z:medi 同形",
    ),
    0xE343: (
        True,
        "Final 'ž' (Z:fina); strictly final",
        "ž 词尾形 (Z:fina)，仅 final",
    ),
    0xE348: (
        True,
        "Initial 'ř'; shares glyph with Rh:medi",
        "ř 首槽位，与 Rh:medi 同形",
    ),
    0xE34A: (
        True,
        "Final tailed 'ř' (Rh:fina); strictly final",
        "ř 词尾长尾形 (Rh:fina)，仅 final",
    ),
}


def load_existing_html_assets() -> tuple[str | None, dict[int, dict[str, int]]]:
    """Extract embedded Base64 font and corpus counts from existing HTML if local font/corpus is absent.
    当本地不存在原始字体或词表时，从已生成的 HTML 中复用内嵌字体与词频数据。
    """
    if not OUT_HTML.exists():
        return None, {}
    text = OUT_HTML.read_text(encoding="utf-8")
    m_font = re.search(r"data:font/ttf;base64,([A-Za-z0-9+/=]+)", text)
    font_b64 = m_font.group(1) if m_font else None
    counts_by_cp: dict[int, dict[str, int]] = {}
    m_data = re.search(r"const RAW_DATA = (\[.*?\]);", text, re.S)
    if m_data:
        for item in json.loads(m_data.group(1)):
            counts_by_cp[int(item["cp"])] = item.get("counts", {})
    return font_b64, counts_by_cp


def build_subset_font_base64(fallback_b64: str | None) -> str:
    """Build Base64-encoded TTF subset covering `U+E264..=U+E34F`.
    构建仅包含 `U+E264..=U+E34F` 的子集 TTF 字体 Base64 字符串。
    """
    if not FONT_PATH.exists():
        if fallback_b64:
            return fallback_b64
        raise FileNotFoundError(f"Font not found at {FONT_PATH} and no existing HTML fallback")
    from fontTools import subset
    from fontTools.ttLib import TTFont

    font = TTFont(str(FONT_PATH))
    options = subset.Options()
    options.layout_features = ["*"]
    subsetter = subset.Subsetter(options=options)
    subsetter.populate(unicodes=list(range(0xE264, 0xE350)) + [0x20])
    subsetter.subset(font)
    buf = io.BytesIO()
    font.save(buf)
    return base64.b64encode(buf.getvalue()).decode("ascii")


def load_word_stats(fallback_counts: dict[int, dict[str, int]]) -> dict[int, dict[str, int]]:
    """Compute positional corpus frequencies from local `word.txt` when available.
    在本地存在 `word.txt` 时统计各码位在不同位置的词频。
    """
    if not WORD_PATH.exists():
        return fallback_counts
    chachlag_tails = {0xE269, 0xE26A, 0xE274}
    stats: dict[int, list[int]] = defaultdict(lambda: [0, 0, 0, 0, 0])
    for line in WORD_PATH.read_text(encoding="utf-8").splitlines():
        w = line.strip()
        for run in re.findall(r"[\ue264-\ue34f]+", w):
            n = len(run)
            for i, ch in enumerate(run):
                cp = ord(ch)
                if n == 1:
                    stats[cp][0] += 1
                elif i == 0:
                    stats[cp][1] += 1
                elif i == n - 1:
                    stats[cp][4] += 1
                elif i == n - 2 and ord(run[i + 1]) in chachlag_tails:
                    stats[cp][3] += 1
                else:
                    stats[cp][2] += 1
    return {
        cp: {
            "isol": c[0],
            "init": c[1],
            "medi": c[2],
            "chachlag": c[3],
            "fina": c[4],
        }
        for cp, c in stats.items()
    }


def build_utn57_preview(shapes: list[str], shape_map: dict[str, dict[str, str]]) -> str:
    """Summarize UTN #57 target units for display under the MenkShape sequence.
    汇总该字形序列对应的 UTN #57 目标单元说明。
    """
    if len(shapes) == 1:
        entry = shape_map[shapes[0]]
        vals = [entry[s] for s in ("isol", "init", "medi", "fina")]
        if len(set(vals)) == 1:
            return vals[0]
        return f"init:{entry['init']} · medi:{entry['medi']} · fina:{entry['fina']}"
    return " + ".join(shape_map[s]["medi"] for s in shapes[:-1]) + " + " + shape_map[shapes[-1]]["fina"]


def default_note(
    cp: int, only_init: bool, only_fina: bool, utn_hint: str
) -> tuple[bool, str, str]:
    """Return `(review, note_en, note_zh)` for code point `cp`.
    返回码位 `cp` 对应的 `(重点核对标记, 英文说明, 中文说明)`。
    """
    if cp in REVIEW_NOTES:
        return REVIEW_NOTES[cp]
    if only_init and only_fina:
        return (
            False,
            f"Standalone isolated glyph ({utn_hint})",
            f"独立单字字形 ({utn_hint})",
        )
    if only_init:
        return (
            False,
            f"Strictly initial crown/head glyph ({utn_hint})",
            f"确定词首冠/头形 ({utn_hint})",
        )
    if only_fina:
        return (
            False,
            f"Strictly final closing/tail glyph ({utn_hint})",
            f"确定词尾收笔形 ({utn_hint})",
        )
    return (
        False,
        f"Medial or position-shared glyph ({utn_hint})",
        f"词中或位置通用字形 ({utn_hint})",
    )


def main() -> None:
    fallback_b64, fallback_counts = load_existing_html_assets()
    font_b64 = build_subset_font_base64(fallback_b64)
    char_shapes = json.loads(CHAR_SHAPES_JSON.read_text(encoding="utf-8"))
    shape_map = json.loads(SHAPE_MAP_JSON.read_text(encoding="utf-8"))
    stats = load_word_stats(fallback_counts)

    rows = []
    groups_order = []
    seen_groups = set()

    for cp in range(0xE264, 0xE350):
        hex_key = f"U+{cp:04X}"
        item = char_shapes[hex_key]
        lat = item["latin"]
        sh = item["shapes"]
        if lat not in seen_groups:
            seen_groups.add(lat)
            groups_order.append(lat)
        o_init = cp in ONLY_INIT
        o_fina = cp in ONLY_FINA
        utn_hint = build_utn57_preview(sh, shape_map)
        review, note_en, note_zh = default_note(cp, o_init, o_fina, utn_hint)
        c = stats.get(cp, {"isol": 0, "init": 0, "medi": 0, "chachlag": 0, "fina": 0})
        rows.append(
            {
                "cp": cp,
                "hex": hex_key,
                "char": chr(cp),
                "latin": lat,
                "shapes": sh,
                "utn57": utn_hint,
                "onlyInit": o_init,
                "onlyFina": o_fina,
                "review": review,
                "noteEn": note_en,
                "noteZh": note_zh,
                "counts": c,
            }
        )

    data_json = json.dumps(rows, ensure_ascii=False)
    groups_json = json.dumps(groups_order, ensure_ascii=False)

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Menk Position Calibration Table / 孟克编码首尾位置核对表 (U+E264–U+E34F)</title>
<style>
  @font-face {{
    font-family: 'MenkSubset';
    src: url('data:font/ttf;base64,{font_b64}') format('truetype');
    font-weight: normal;
    font-style: normal;
  }}

  :root {{
    --bg: #f6f8fa;
    --card: #ffffff;
    --border: #d0d7de;
    --border-soft: #eaeef2;
    --text: #1f2328;
    --muted: #59636e;
    --init-bg: #dafbe1;
    --init-fg: #1a7f37;
    --fina-bg: #ddf4ff;
    --fina-fg: #0969da;
    --isol-bg: #fbefff;
    --isol-fg: #8250df;
    --mod-bg: #fff8c5;
    --mod-fg: #9a6700;
    --rev-bg: #ffebe9;
    --rev-fg: #cf222e;
  }}

  * {{ box-sizing: border-box; }}

  body {{
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC", "Hiragino Sans GB", sans-serif;
    background: var(--bg);
    color: var(--text);
    font-size: 14px;
    line-height: 1.45;
  }}

  body[data-lang="en"] .zh-only {{ display: none !important; }}
  body[data-lang="zh"] .en-only {{ display: none !important; }}

  .topbar {{
    position: sticky;
    top: 0;
    z-index: 20;
    background: rgba(255, 255, 255, 0.96);
    backdrop-filter: blur(6px);
    border-bottom: 1px solid var(--border);
    padding: 12px 16px;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.04);
  }}

  .topbar-inner {{
    max-width: 1360px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }}

  .title-row {{
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }}

  h1 {{
    margin: 0;
    font-size: 16.5px;
    font-weight: 600;
  }}

  .subtitle {{
    font-size: 12px;
    color: var(--muted);
  }}

  .actions {{
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }}

  button, select {{
    appearance: none;
    border: 1px solid var(--border);
    background: var(--card);
    color: var(--text);
    padding: 5px 10px;
    border-radius: 6px;
    font-size: 12.5px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.12s, border-color 0.12s;
  }}

  button:hover, select:hover {{
    background: #f3f4f6;
    border-color: #afb8c1;
  }}

  button.primary {{
    background: #1f883d;
    border-color: #1f883d;
    color: #fff;
  }}

  button.primary:hover {{
    background: #1a7f37;
  }}

  button.danger {{
    color: #cf222e;
  }}

  .controls-row {{
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }}

  .filter-pills {{
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
  }}

  .pill {{
    padding: 4px 9px;
    border-radius: 999px;
    font-size: 12px;
  }}

  .pill.active {{
    background: #1f2328;
    border-color: #1f2328;
    color: #fff;
  }}

  .search-input {{
    padding: 5px 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: 12.5px;
    min-width: 220px;
  }}

  .group-bar {{
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    align-items: center;
  }}

  .group-bar span {{
    font-size: 12px;
    color: var(--muted);
    margin-right: 2px;
  }}

  .group-link {{
    text-decoration: none;
    color: var(--text);
    background: #eff2f5;
    padding: 2px 7px;
    border-radius: 4px;
    font-size: 12px;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }}

  .group-link:hover {{
    background: #d0d7de;
  }}

  .container {{
    max-width: 1360px;
    margin: 14px auto 48px;
    padding: 0 12px;
  }}

  .legend {{
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 14px;
    margin-bottom: 12px;
    font-size: 12.5px;
    color: var(--muted);
    display: flex;
    flex-wrap: wrap;
    gap: 14px;
  }}

  .legend strong {{ color: var(--text); }}

  .table-wrap {{
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow-x: auto;
  }}

  table {{
    width: 100%;
    border-collapse: collapse;
    min-width: 860px;
  }}

  th, td {{
    border-bottom: 1px solid var(--border-soft);
    padding: 6px 10px;
    text-align: left;
    vertical-align: middle;
  }}

  th {{
    background: #f6f8fa;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
    border-bottom: 1px solid var(--border);
    white-space: nowrap;
  }}

  tr.group-header td {{
    background: #eef2f6;
    font-weight: 600;
    font-size: 13px;
    color: #24292f;
    padding: 6px 12px;
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
  }}

  tr.data-row:hover {{
    background: #f9fbfd;
  }}

  tr.data-row.modified {{
    background: #fffdf0;
  }}

  .col-hex {{
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-weight: 600;
    font-size: 13px;
    white-space: nowrap;
    width: 84px;
  }}

  .col-glyph {{
    width: 78px;
    text-align: center;
  }}

  .glyph-box {{
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 62px;
    min-height: 70px;
    padding: 6px;
    background: #fcfcfd;
    border: 1px dashed #d8dee4;
    border-radius: 6px;
  }}

  .menk-glyph {{
    font-family: 'MenkSubset', sans-serif;
    font-size: 50px;
    line-height: 1;
    writing-mode: vertical-lr;
    -webkit-writing-mode: vertical-lr;
    text-orientation: sideways;
    -webkit-text-orientation: sideways;
    color: #111;
    user-select: all;
  }}

  .col-shape {{
    white-space: nowrap;
    width: 190px;
  }}

  .shape-code {{
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 12.5px;
    font-weight: 600;
    color: #0550ae;
  }}

  .utn-sub {{
    font-size: 11.5px;
    color: var(--muted);
    margin-top: 2px;
  }}

  .col-chk {{
    width: 116px;
    text-align: center;
    cursor: pointer;
    user-select: none;
    transition: background 0.1s;
  }}

  .col-chk:hover {{
    background: #f0f4f8;
  }}

  .col-chk.checked-init {{
    background: var(--init-bg);
  }}

  .col-chk.checked-fina {{
    background: var(--fina-bg);
  }}

  .chk-label {{
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
  }}

  .chk-label input {{
    width: 17px;
    height: 17px;
    cursor: pointer;
    margin: 0;
  }}

  .col-status {{
    width: 136px;
    white-space: nowrap;
  }}

  .badge {{
    display: inline-block;
    padding: 2px 7px;
    border-radius: 999px;
    font-size: 11.5px;
    font-weight: 600;
  }}

  .badge-init {{ background: var(--init-bg); color: var(--init-fg); }}
  .badge-fina {{ background: var(--fina-bg); color: var(--fina-fg); }}
  .badge-isol {{ background: var(--isol-bg); color: var(--isol-fg); }}
  .badge-medi {{ background: #eff2f5; color: var(--muted); }}
  .badge-mod  {{ background: var(--mod-bg); color: var(--mod-fg); margin-left: 4px; }}
  .badge-rev  {{ background: var(--rev-bg); color: var(--rev-fg); margin-right: 5px; }}

  .stats-line {{
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11.5px;
    color: var(--muted);
    margin-bottom: 2px;
  }}

  .stats-line b {{
    color: var(--text);
  }}

  .note-text {{
    font-size: 12.5px;
    color: #333;
  }}

  .note-en {{
    color: #1f2328;
  }}

  .note-zh {{
    color: #59636e;
    margin-top: 1px;
  }}

  .toast {{
    position: fixed;
    bottom: 20px;
    right: 20px;
    background: #1f2328;
    color: #fff;
    padding: 9px 14px;
    border-radius: 8px;
    font-size: 13px;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.2s;
    z-index: 50;
  }}

  .toast.show {{ opacity: 1; }}
</style>
</head>
<body data-lang="bi">

<div class="topbar">
  <div class="topbar-inner">
    <div class="title-row">
      <div>
        <h1>
          <span class="en-only">Menk Position Calibration Table (U+E264–U+E34F, 236 chars)</span>
          <span class="en-only zh-only"> / </span>
          <span class="zh-only">孟克编码首尾位置核对表（U+E264–U+E34F，共 236 项）</span>
        </h1>
        <div class="subtitle">
          <span class="en-only">Embedded subsetted menk.ttf · Auto-saved in localStorage · Copy diff or export JSON</span>
          <span class="en-only zh-only"> · </span>
          <span class="zh-only">内嵌 menk.ttf 子集字体 · 勾选自动保存在浏览器本地 · 一键复制差异或导出 JSON</span>
        </div>
      </div>
      <div class="actions">
        <select id="langSelect" aria-label="Language / 语言">
          <option value="bi">Bilingual / 双语</option>
          <option value="en">English</option>
          <option value="zh">简体中文</option>
        </select>
        <button class="primary" id="btnCopyDiff">
          <span class="en-only">Copy Diff</span><span class="en-only zh-only"> / </span><span class="zh-only">复制修改差异</span>
        </button>
        <button id="btnCopyJson">
          <span class="en-only">Copy JSON</span><span class="en-only zh-only"> / </span><span class="zh-only">复制 JSON</span>
        </button>
        <button id="btnDownloadJson">
          <span class="en-only">Export JSON</span><span class="en-only zh-only"> / </span><span class="zh-only">导出 JSON</span>
        </button>
        <button id="btnImportJson">
          <span class="en-only">Import JSON</span><span class="en-only zh-only"> / </span><span class="zh-only">导入 JSON</span>
        </button>
        <button class="danger" id="btnReset">
          <span class="en-only">Reset</span><span class="en-only zh-only"> / </span><span class="zh-only">重置默认</span>
        </button>
        <input type="file" id="fileInput" accept=".json" style="display:none">
      </div>
    </div>

    <div class="controls-row">
      <div class="filter-pills" id="filterPills">
        <button class="pill active" data-filter="all" id="pillAll"></button>
        <button class="pill" data-filter="init" id="pillInit"></button>
        <button class="pill" data-filter="fina" id="pillFina"></button>
        <button class="pill" data-filter="isol" id="pillIsol"></button>
        <button class="pill" data-filter="medi" id="pillMedi"></button>
        <button class="pill" data-filter="review" id="pillReview"></button>
        <button class="pill" data-filter="modified" id="pillMod"></button>
      </div>
      <input type="search" class="search-input" id="searchInput" placeholder="Search / 搜索 U+E27B, G, Chachlag...">
    </div>

    <div class="group-bar" id="groupBar">
      <span><span class="en-only">Jump to</span><span class="en-only zh-only"> / </span><span class="zh-only">快速跳转</span>:</span>
    </div>
  </div>
</div>

<div class="container">
  <div class="legend">
    <div>
      <strong>
        <span class="en-only">Sub-chain Splitting Rules:</span>
        <span class="en-only zh-only"> </span>
        <span class="zh-only">子链切分语义：</span>
      </strong>
    </div>
    <div>
      <span class="badge badge-init">only_init</span>
      <span class="en-only">Splits sub-chain <strong>before</strong> this char</span>
      <span class="en-only zh-only"> / </span>
      <span class="zh-only">在<strong>其前</strong>切分子链（不可上连）</span>
    </div>
    <div>
      <span class="badge badge-fina">only_fina</span>
      <span class="en-only">Splits sub-chain <strong>after</strong> this char</span>
      <span class="en-only zh-only"> / </span>
      <span class="zh-only">在<strong>其后</strong>切分子链（不可下连）</span>
    </div>
    <div>
      <span class="badge badge-isol">isol (init+final)</span>
      <span class="en-only">Both checked — standalone sub-chain</span>
      <span class="en-only zh-only"> / </span>
      <span class="zh-only">同时勾选 — 前后均切分，独立成链</span>
    </div>
    <div>
      <span class="badge badge-medi">medi / shared</span>
      <span class="en-only">Neither checked — joins by context</span>
      <span class="en-only zh-only"> / </span>
      <span class="zh-only">均不勾选 — 按上下文连写</span>
    </div>
  </div>

  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th><span class="en-only">Code</span><span class="en-only zh-only"> / </span><span class="zh-only">编码</span></th>
          <th style="text-align:center"><span class="en-only">Glyph</span><span class="en-only zh-only"> / </span><span class="zh-only">字形</span></th>
          <th><span class="en-only">Latin &amp; MenkShape</span><span class="en-only zh-only"> / </span><span class="zh-only">拉丁组与字形</span></th>
          <th style="text-align:center"><span class="en-only">Only Init</span><span class="en-only zh-only"> / </span><span class="zh-only">仅 init</span></th>
          <th style="text-align:center"><span class="en-only">Only Final</span><span class="en-only zh-only"> / </span><span class="zh-only">仅 final</span></th>
          <th><span class="en-only">Role</span><span class="en-only zh-only"> / </span><span class="zh-only">当前判定</span></th>
          <th><span class="en-only">Corpus Stats &amp; Rationale</span><span class="en-only zh-only"> / </span><span class="zh-only">词库频次与依据说明</span></th>
        </tr>
      </thead>
      <tbody id="tableBody"></tbody>
    </table>
  </div>
</div>

<div class="toast" id="toast"></div>

<script>
// Embedded character table & group order / 内嵌码位数据与分组顺序
const RAW_DATA = {data_json};
const GROUPS = {groups_json};
const STORAGE_KEY = "menk_to_utn57_positions_v3";
const LANG_KEY = "menk_to_utn57_lang_v1";

// Current checkbox state: hex -> {{ onlyInit: bool, onlyFina: bool }}
// 当前复选框状态映射
let state = {{}};
let currentLang = localStorage.getItem(LANG_KEY) || "bi";
document.body.dataset.lang = currentLang;
document.getElementById("langSelect").value = currentLang;

function biText(en, zh) {{
  if (currentLang === "en") return en;
  if (currentLang === "zh") return zh;
  return `${{en}} / ${{zh}}`;
}}

function initDefaultState() {{
  state = {{}};
  for (const row of RAW_DATA) {{
    state[row.hex] = {{ onlyInit: row.onlyInit, onlyFina: row.onlyFina }};
  }}
}}

function loadState() {{
  initDefaultState();
  try {{
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {{
      const parsed = JSON.parse(saved);
      for (const row of RAW_DATA) {{
        if (parsed[row.hex]) {{
          state[row.hex] = {{
            onlyInit: !!parsed[row.hex].onlyInit,
            onlyFina: !!parsed[row.hex].onlyFina,
          }};
        }}
      }}
    }}
  }} catch (e) {{
    console.warn("Failed to load localStorage:", e);
  }}
}}

function saveState() {{
  try {{
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  }} catch (e) {{
    console.warn("Failed to save localStorage:", e);
  }}
}}

function isModified(row) {{
  const cur = state[row.hex];
  return cur.onlyInit !== row.onlyInit || cur.onlyFina !== row.onlyFina;
}}

function statusBadgeHtml(row) {{
  const cur = state[row.hex];
  let b = "";
  if (cur.onlyInit && cur.onlyFina) {{
    b = `<span class="badge badge-isol">${{biText("isol", "独立 isol")}}</span>`;
  }} else if (cur.onlyInit) {{
    b = `<span class="badge badge-init">${{biText("only init", "仅 init")}}</span>`;
  }} else if (cur.onlyFina) {{
    b = `<span class="badge badge-fina">${{biText("only final", "仅 final")}}</span>`;
  }} else {{
    b = `<span class="badge badge-medi">${{biText("medi · shared", "medi·通用")}}</span>`;
  }}
  if (isModified(row)) {{
    b += `<span class="badge badge-mod">${{biText("edited", "已改")}}</span>`;
  }}
  return b;
}}

let currentFilter = "all";
let currentQuery = "";

function matchesFilter(row) {{
  const cur = state[row.hex];
  if (currentFilter === "init" && !cur.onlyInit) return false;
  if (currentFilter === "fina" && !cur.onlyFina) return false;
  if (currentFilter === "isol" && !(cur.onlyInit && cur.onlyFina)) return false;
  if (currentFilter === "medi" && (cur.onlyInit || cur.onlyFina)) return false;
  if (currentFilter === "review" && !row.review) return false;
  if (currentFilter === "modified" && !isModified(row)) return false;

  if (currentQuery) {{
    const q = currentQuery.toLowerCase();
    const hay = `${{row.hex}} ${{row.latin}} ${{row.shapes.join("+")}} ${{row.utn57}} ${{row.noteEn}} ${{row.noteZh}}`.toLowerCase();
    if (!hay.includes(q)) return false;
  }}
  return true;
}}

function updateCounts() {{
  let nInit = 0, nFina = 0, nIsol = 0, nMedi = 0, nRev = 0, nMod = 0;
  for (const row of RAW_DATA) {{
    const cur = state[row.hex];
    if (cur.onlyInit) nInit++;
    if (cur.onlyFina) nFina++;
    if (cur.onlyInit && cur.onlyFina) nIsol++;
    if (!cur.onlyInit && !cur.onlyFina) nMedi++;
    if (row.review) nRev++;
    if (isModified(row)) nMod++;
  }}
  document.getElementById("pillAll").textContent = `${{biText("All", "全部")}} (${{RAW_DATA.length}})`;
  document.getElementById("pillInit").textContent = `${{biText("Only init", "仅 init")}} (${{nInit}})`;
  document.getElementById("pillFina").textContent = `${{biText("Only final", "仅 final")}} (${{nFina}})`;
  document.getElementById("pillIsol").textContent = `${{biText("Isol", "独立 isol")}} (${{nIsol}})`;
  document.getElementById("pillMedi").textContent = `${{biText("Shared/medi", "未勾选")}} (${{nMedi}})`;
  document.getElementById("pillReview").textContent = `${{biText("Key notes", "重点说明")}} (${{nRev}})`;
  document.getElementById("pillMod").textContent = `${{biText("Modified", "已修改")}} (${{nMod}})`;
}}

function renderTable() {{
  const tbody = document.getElementById("tableBody");
  const frag = document.createDocumentFragment();
  let lastGroup = null;

  for (const row of RAW_DATA) {{
    if (!matchesFilter(row)) continue;

    if (row.latin !== lastGroup) {{
      lastGroup = row.latin;
      const gtr = document.createElement("tr");
      gtr.className = "group-header";
      gtr.id = `group-${{row.latin}}`;
      const gtd = document.createElement("td");
      gtd.colSpan = 7;
      gtd.textContent = `${{biText("Group", "字母组")}}: ${{row.latin}}`;
      gtr.appendChild(gtd);
      frag.appendChild(gtr);
    }}

    const cur = state[row.hex];
    const tr = document.createElement("tr");
    tr.className = "data-row" + (isModified(row) ? " modified" : "");
    tr.dataset.hex = row.hex;

    const c = row.counts;
    const statsParts = [];
    if (c.isol) statsParts.push(`isol:<b>${{c.isol}}</b>`);
    if (c.init) statsParts.push(`init:<b>${{c.init}}</b>`);
    if (c.medi) statsParts.push(`medi:<b>${{c.medi}}</b>`);
    if (c.chachlag) statsParts.push(`pre-chachlag/分写前:<b>${{c.chachlag}}</b>`);
    if (c.fina) statsParts.push(`fina:<b>${{c.fina}}</b>`);
    const statsHtml = statsParts.length ? statsParts.join(" · ") : biText("Corpus: 0", "词库未出现 (0)");

    const revBadge = row.review
      ? `<span class="badge badge-rev">${{biText("Key", "重点")}}</span>`
      : "";

    tr.innerHTML = `
      <td class="col-hex">${{row.hex}}</td>
      <td class="col-glyph">
        <div class="glyph-box"><span class="menk-glyph">${{row.char}}</span></div>
      </td>
      <td class="col-shape">
        <div class="shape-code">${{row.shapes.join(" + ")}}</div>
        <div class="utn-sub">[${{row.latin}}] ${{row.utn57}}</div>
      </td>
      <td class="col-chk ${{cur.onlyInit ? 'checked-init' : ''}}" data-role="init">
        <label class="chk-label">
          <input type="checkbox" ${{cur.onlyInit ? "checked" : ""}} data-role="init">
          <span>init</span>
        </label>
      </td>
      <td class="col-chk ${{cur.onlyFina ? 'checked-fina' : ''}}" data-role="fina">
        <label class="chk-label">
          <input type="checkbox" ${{cur.onlyFina ? "checked" : ""}} data-role="fina">
          <span>final</span>
        </label>
      </td>
      <td class="col-status">${{statusBadgeHtml(row)}}</td>
      <td>
        <div class="stats-line">${{statsHtml}}</div>
        <div class="note-text">
          <div class="note-en en-only">${{revBadge}}${{row.noteEn}}</div>
          <div class="note-zh zh-only">${{currentLang === "zh" ? revBadge : ""}}${{row.noteZh}}</div>
        </div>
      </td>
    `;

    tr.querySelectorAll(".col-chk").forEach(td => {{
      td.addEventListener("click", (e) => {{
        const role = td.dataset.role;
        const input = td.querySelector("input");
        if (e.target !== input) {{
          input.checked = !input.checked;
        }}
        if (role === "init") {{
          state[row.hex].onlyInit = input.checked;
        }} else {{
          state[row.hex].onlyFina = input.checked;
        }}
        saveState();
        updateCounts();
        if (currentFilter !== "all") {{
          renderTable();
        }} else {{
          td.classList.toggle("checked-init", role === "init" && input.checked);
          td.classList.toggle("checked-fina", role === "fina" && input.checked);
          tr.classList.toggle("modified", isModified(row));
          tr.querySelector(".col-status").innerHTML = statusBadgeHtml(row);
        }}
      }});
    }});

    frag.appendChild(tr);
  }}

  tbody.innerHTML = "";
  tbody.appendChild(frag);
}}

function buildExportObject() {{
  const only_init = [];
  const only_fina = [];
  for (const row of RAW_DATA) {{
    const cur = state[row.hex];
    if (cur.onlyInit) only_init.push(row.hex);
    if (cur.onlyFina) only_fina.push(row.hex);
  }}
  return {{ only_init, only_fina }};
}}

function showToast(msg) {{
  const t = document.getElementById("toast");
  t.textContent = msg;
  t.classList.add("show");
  clearTimeout(showToast._timer);
  showToast._timer = setTimeout(() => t.classList.remove("show"), 2200);
}}

function renderGroupBar() {{
  const bar = document.getElementById("groupBar");
  for (const g of GROUPS) {{
    const a = document.createElement("a");
    a.href = `#group-${{g}}`;
    a.className = "group-link";
    a.textContent = g;
    a.addEventListener("click", () => {{
      if (currentFilter !== "all" || currentQuery !== "") {{
        currentFilter = "all";
        currentQuery = "";
        document.getElementById("searchInput").value = "";
        document.querySelectorAll(".pill").forEach(p => p.classList.toggle("active", p.dataset.filter === "all"));
        renderTable();
      }}
    }});
    bar.appendChild(a);
  }}
}}

document.getElementById("langSelect").addEventListener("change", (e) => {{
  currentLang = e.target.value;
  document.body.dataset.lang = currentLang;
  localStorage.setItem(LANG_KEY, currentLang);
  updateCounts();
  renderTable();
}});

document.getElementById("filterPills").addEventListener("click", (e) => {{
  const btn = e.target.closest(".pill");
  if (!btn) return;
  currentFilter = btn.dataset.filter;
  document.querySelectorAll(".pill").forEach(p => p.classList.toggle("active", p === btn));
  renderTable();
}});

document.getElementById("searchInput").addEventListener("input", (e) => {{
  currentQuery = e.target.value.trim();
  renderTable();
}});

document.getElementById("btnCopyDiff").addEventListener("click", async () => {{
  const diffs = [];
  for (const row of RAW_DATA) {{
    if (isModified(row)) {{
      const cur = state[row.hex];
      diffs.push(
        `${{row.hex}} (${{row.latin}}, ${{row.shapes.join("+")}}): ` +
        `init ${{row.onlyInit}}->${{cur.onlyInit}}, final ${{row.onlyFina}}->${{cur.onlyFina}}`
      );
    }}
  }}
  const text = diffs.length
    ? `Modified ${{diffs.length}} items / 共修改 ${{diffs.length}} 项：\\n` + diffs.join("\\n")
    : "No changes from default / 与默认一致（0 项修改）";
  await navigator.clipboard.writeText(text);
  showToast(diffs.length ? biText(`Copied ${{diffs.length}} diffs`, `已复制 ${{diffs.length}} 项差异`) : biText("0 diffs copied", "无修改，已复制确认信息"));
}});

document.getElementById("btnCopyJson").addEventListener("click", async () => {{
  const obj = buildExportObject();
  await navigator.clipboard.writeText(JSON.stringify(obj, null, 2));
  showToast(biText("Copied menk_char_positions.json", "已复制 menk_char_positions.json"));
}});

document.getElementById("btnDownloadJson").addEventListener("click", () => {{
  const obj = buildExportObject();
  const blob = new Blob([JSON.stringify(obj, null, 2) + "\\n"], {{ type: "application/json" }});
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "menk_char_positions.json";
  a.click();
  URL.revokeObjectURL(url);
  showToast(biText("Exported menk_char_positions.json", "已导出 menk_char_positions.json"));
}});

document.getElementById("btnImportJson").addEventListener("click", () => {{
  document.getElementById("fileInput").click();
}});

document.getElementById("fileInput").addEventListener("change", (e) => {{
  const file = e.target.files[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = () => {{
    try {{
      const data = JSON.parse(reader.result);
      if (Array.isArray(data.only_init) && Array.isArray(data.only_fina)) {{
        const initSet = new Set(data.only_init);
        const finaSet = new Set(data.only_fina);
        for (const row of RAW_DATA) {{
          state[row.hex] = {{
            onlyInit: initSet.has(row.hex),
            onlyFina: finaSet.has(row.hex),
          }};
        }}
        saveState();
        updateCounts();
        renderTable();
        showToast(biText("Imported JSON", "JSON 导入成功"));
      }} else {{
        alert("JSON must contain 'only_init' and 'only_fina' arrays / JSON 需包含 only_init 和 only_fina 数组");
      }}
    }} catch (err) {{
      alert("Failed to parse JSON / JSON 解析失败: " + err.message);
    }}
    e.target.value = "";
  }};
  reader.readAsText(file);
}});

document.getElementById("btnReset").addEventListener("click", () => {{
  if (!confirm(biText("Reset all checkboxes to default?", "确定重置所有复选框为默认状态吗？"))) return;
  localStorage.removeItem(STORAGE_KEY);
  initDefaultState();
  updateCounts();
  renderTable();
  showToast(biText("Reset to defaults", "已重置为默认状态"));
}});

loadState();
renderGroupBar();
updateCounts();
renderTable();
</script>
</body>
</html>
"""
    OUT_HTML.parent.mkdir(parents=True, exist_ok=True)
    OUT_HTML.write_text(html, encoding="utf-8")
    print(f"Generated {OUT_HTML} ({OUT_HTML.stat().st_size / 1024:.1f} KB)")


if __name__ == "__main__":
    main()
