import init, {
  infer_standard_latin,
  resolve_menk_to_unicode,
} from "./pkg/menk_to_utn57.js";

const SPECIAL_LATIN_CHARS = [
  "ɵ",
  "ʉ",
  "ë",
  "ŋ",
  "š",
  "č",
  "ž",
  "ħ",
  "ř",
  "ł",
  "ẑ",
  "ĉ",
];

const LATIN_LEGEND = [
  ["a", "ᠠ"],
  ["e", "ᠡ"],
  ["i", "ᠢ"],
  ["o", "ᠣ"],
  ["u", "ᠤ"],
  ["ɵ / ö", "ᠥ"],
  ["ʉ / ü", "ᠦ"],
  ["ë", "ᠧ"],
  ["n", "ᠨ"],
  ["ŋ", "ᠩ"],
  ["b", "ᠪ"],
  ["p", "ᠫ"],
  ["h", "ᠬ"],
  ["g", "ᠭ"],
  ["m", "ᠮ"],
  ["l", "ᠯ"],
  ["s", "ᠰ"],
  ["š", "ᠱ"],
  ["t", "ᠲ"],
  ["d", "ᠳ"],
  ["c", "ᠴ"],
  ["z", "ᠵ"],
  ["y", "ᠶ"],
  ["r", "ᠷ"],
  ["w", "ᠸ"],
  ["f", "ᠹ"],
  ["k", "ᠺ / ᠻ"],
  ["č", "ᠼ"],
  ["ž", "ᠽ"],
  ["ħ", "ᠾ"],
  ["ř", "ᠿ"],
  ["ł", "ᡀ"],
  ["ẑ", "ᡁ"],
  ["ĉ", "ᡂ"],
];

const SAMPLES = [
  {
    id: "zuil",
    menk: "\uE318\uE2A9\uE27E\uE2F9",
    latin: "zʉil",
    label: { zh: "zʉil · 2 解", en: "zʉil · 2 solutions" },
  },
  {
    id: "ulaganudu",
    menk: "\uE28C\uE2FA\uE26C\uE2EA\uE26C\uE2C0\uE290\uE313\uE28D",
    latin: "ulaganudu",
    label: { zh: "ulaganudu · 2 解打平", en: "ulaganudu · 2 tied" },
  },
  {
    id: "baina",
    menk: "\uE2C1\uE26C\uE281\uE2B6\uE26A",
    latin: "baina",
    label: { zh: "baina · 分写 MVS", en: "baina · Chachlag MVS" },
  },
  {
    id: "sain",
    menk: "\uE2FD\uE26C\uE281\uE2B5",
    latin: "sain",
    label: { zh: "sain · 试改 sein", en: "sain · try editing sein" },
  },
  {
    id: "monggol",
    menk: "\uE2F1\uE289\uE2BD\uE2EA\uE289\uE2F9",
    latin: "moŋgol",
    label: { zh: "moŋgol · 试改 muŋgul", en: "moŋgol · try muŋgul" },
  },
  {
    id: "du",
    menk: "\uE310\uE2A7",
    latin: "dʉ",
    label: { zh: "dʉ · 2 FVS", en: "dʉ · 2 FVS" },
  },
  {
    id: "kino",
    menk: "\uE333\uE27F\uE2B9\uE286",
    latin: "kino",
    label: { zh: "kino · ᠺ/ᠻ", en: "kino · ᠺ/ᠻ" },
  },
];

const I18N = {
  zh: {
    docTitle: "menk-to-utn57 · 蒙科立转 UTN #57 Unicode 在线演示 (WASM)",
    eyebrow: "PURE RUST · WEBASSEMBLY · 本地浏览器运行",
    heroTitle: "蒙科立（Menk）→ 标准 UTN #57 Unicode 转换演示",
    heroSubtitle:
      "在浏览器内通过 WASM 直接运行 menk_word_and_latin_to_unicode：输入蒙科立词串会自动转写拉丁，可随时手动编辑拉丁字母（例如把 sain 改成 sein），并完整展示全部合法的最小 FVS 解法。",
    wasmLoading: "WASM 加载中...",
    wasmReady: "WASM 已就绪 · 本地运行",
    wasmFailed: "WASM 加载失败",
    inputHeading: "输入 · 蒙科立词串与拉丁转写",
    clear: "清空",
    samplesLabel: "示例词（点击体验多解、分写 MVS、改拉丁）",
    menkLabel: "蒙科立词串（U+E264..=U+E34F）",
    menkPlaceholder: "粘贴或点击上方示例蒙科立词...",
    menkPreviewCaption: "蒙科立",
    utnPreviewCaption: "UTN #57",
    unitsLabel: "UTN #57 字形",
    menkCodesSummary: "查看蒙科立码位",
    latinLabel: "单字符拉丁转写（可编辑）",
    latinPlaceholder: "留空自动转写，或手动编辑拉丁...",
    modeAuto: "自动转写 (get_standard_latin)",
    modeCustom: "已手动编辑",
    resetLatin: "恢复自动转写",
    latinHelp:
      "默认由 get_standard_latin 自动填充；修改同形元音/辅音（如 a↔e、o↔u、ɵ↔ʉ）即可实时重新求解，兼容大小写及 ö/ü。",
    specialLatinLabel: "快捷输入特殊拉丁：",
    kLabel: "字母 k 对应码位",
    kKa: "ᠺ U+183A（内蒙古 Ka）",
    kKha: "ᠻ U+183B（蒙古国 Kha）",
    latinLegendSummary: "展开 34 个单字符拉丁字母对照表",
    outputHeading: "输出 · 全部匹配解法",
    countSolutions: (n) => `共 ${n} 个匹配解法`,
    countError: "无法匹配",
    countEmpty: "等待输入",
    emptyState: "请输入蒙科立词串，或点击左侧任意示例词开始演示。",
    fvsBadge: (n) => `${n} FVS`,
    exactBadge: "精确最少 FVS",
    greedyBadge: "贪心回退 · 建议人工复核",
    copy: "复制",
    copied: "已复制",
    lettersBreakdown: "逐字母笔画切分明细（ResolvedLetter）",
    charCount: (n) => `${n} 字`,
    errInvalidMenkTitle: "包含非蒙科立字符",
    errInvalidMenkDetail: (e) =>
      `第 ${e.index + 1} 个字符 '${e.char}' (${e.codepoint}) 不在蒙科立范围 U+E264..=U+E34F 内。`,
    errInvalidLatinTitle: "包含不支持的拉丁字符",
    errInvalidLatinDetail: (e) =>
      `第 ${e.index + 1} 个字符 '${e.char}' (${e.codepoint}) 不在 34 个单字符拉丁字母表内（不需要输入 MVS / FVS / 连字符）。`,
    errShapeMismatchTitle: "字形与拉丁拼写不匹配",
    errShapeMismatchDetail:
      "当前编辑的拉丁拼写无法生成与左侧蒙科立词串相同的 UTN #57 字形。请检查字母数量或点击下方恢复自动转写。",
    errTargetNormTitle: "目标字形规范化失败",
    errResetBtn: "恢复为自动转写拉丁",
    footerNote: "全程在浏览器本地 WebAssembly 进程内运行，不上传任何文本。",
  },
  en: {
    docTitle: "menk-to-utn57 · Menk to UTN #57 Unicode WASM Demo",
    eyebrow: "PURE RUST · WEBASSEMBLY · RUNS LOCALLY IN BROWSER",
    heroTitle: "Menk PUA → Standard UTN #57 Unicode Converter",
    heroSubtitle:
      "Runs menk_word_and_latin_to_unicode directly in your browser via WASM: auto-transcribes Menk words into editable single-character Latin (try changing sain → sein) and lists every valid minimal-FVS Unicode solution.",
    wasmLoading: "Loading WASM...",
    wasmReady: "WASM Ready · Client-side",
    wasmFailed: "WASM Failed",
    inputHeading: "Input · Menk Word & Editable Latin",
    clear: "Clear",
    samplesLabel: "Sample words (multi-solution, Chachlag MVS, editable Latin)",
    menkLabel: "Menk PUA Word (U+E264..=U+E34F)",
    menkPlaceholder: "Paste a Menk word or pick a sample above...",
    menkPreviewCaption: "Menk",
    utnPreviewCaption: "UTN #57",
    unitsLabel: "UTN #57 Units",
    menkCodesSummary: "Inspect Menk code points",
    latinLabel: "Single-Character Latin (Editable)",
    latinPlaceholder: "Auto-transcribed when empty, or edit freely...",
    modeAuto: "Auto (get_standard_latin)",
    modeCustom: "Custom edited",
    resetLatin: "Reset to auto",
    latinHelp:
      "Auto-filled via get_standard_latin by default. Edit homographic vowels/consonants (e.g. a↔e, o↔u, ɵ↔ʉ) to re-solve live; case-insensitive, ö/ü aliases supported.",
    specialLatinLabel: "Quick insert:",
    kLabel: "Regional letter for 'k'",
    kKa: "ᠺ U+183A (Inner Mongolia Ka)",
    kKha: "ᠻ U+183B (Mongolia Kha)",
    latinLegendSummary: "Show 34-letter single-character Latin reference",
    outputHeading: "Output · All Matching Solutions",
    countSolutions: (n) => (n === 1 ? "1 solution" : `${n} solutions`),
    countError: "Mismatch / Error",
    countEmpty: "Waiting for input",
    emptyState: "Enter a Menk PUA word or pick any sample on the left to inspect all solutions.",
    fvsBadge: (n) => `${n} FVS`,
    exactBadge: "Exact min FVS",
    greedyBadge: "Greedy fallback · Review",
    copy: "Copy",
    copied: "Copied",
    lettersBreakdown: "Per-letter decomposition (ResolvedLetter)",
    charCount: (n) => (n === 1 ? "1 char" : `${n} chars`),
    errInvalidMenkTitle: "Invalid Menk character",
    errInvalidMenkDetail: (e) =>
      `Character #${e.index + 1} '${e.char}' (${e.codepoint}) is outside Menk PUA U+E264..=U+E34F.`,
    errInvalidLatinTitle: "Unsupported Latin character",
    errInvalidLatinDetail: (e) =>
      `Character #${e.index + 1} '${e.char}' (${e.codepoint}) is not in the 34-letter single-character Latin inventory (omit MVS/FVS/hyphens).`,
    errShapeMismatchTitle: "Shape mismatch",
    errShapeMismatchDetail:
      "The edited Latin spelling cannot produce the same UTN #57 glyph shape as this Menk word. Check the letters or reset to auto-transcribed Latin.",
    errTargetNormTitle: "Target shape normalization error",
    errResetBtn: "Reset to auto-transcribed Latin",
    footerNote: "100% client-side WebAssembly — no text ever leaves your browser.",
  },
};

const CONTROL_LABELS = {
  0x180b: "FVS1",
  0x180c: "FVS2",
  0x180d: "FVS3",
  0x180e: "MVS",
  0x180f: "FVS4",
  0x200c: "ZWNJ",
  0x200d: "ZWJ",
};

const state = {
  lang: navigator.language && navigator.language.toLowerCase().startsWith("en") ? "en" : "zh",
  wasmReady: false,
  menk: SAMPLES[0].menk,
  latinCustom: null, // null => auto (pass None to WASM); string => custom edited
  useKha: false,
};

const els = {
  langZh: document.getElementById("lang-zh"),
  langEn: document.getElementById("lang-en"),
  eyebrowText: document.getElementById("eyebrow-text"),
  heroTitle: document.getElementById("hero-title"),
  heroSubtitle: document.getElementById("hero-subtitle"),
  wasmStatus: document.getElementById("wasm-status-pill"),
  inputHeading: document.getElementById("input-heading"),
  clearBtn: document.getElementById("clear-btn"),
  samplesLabel: document.getElementById("samples-label"),
  samplesList: document.getElementById("samples-list"),
  menkLabel: document.getElementById("menk-label"),
  menkCount: document.getElementById("menk-count"),
  menkInput: document.getElementById("menk-input"),
  menkPreviewCaption: document.getElementById("menk-preview-caption"),
  menkVerticalPreview: document.getElementById("menk-vertical-preview"),
  unitsBar: document.getElementById("units-bar"),
  unitsLabel: document.getElementById("units-label"),
  unitsList: document.getElementById("units-list"),
  menkCodesSummary: document.getElementById("menk-codes-summary"),
  menkCodepoints: document.getElementById("menk-codepoints"),
  latinLabel: document.getElementById("latin-label"),
  latinModeBadge: document.getElementById("latin-mode-badge"),
  resetLatinBtn: document.getElementById("reset-latin-btn"),
  latinInput: document.getElementById("latin-input"),
  latinHelp: document.getElementById("latin-help"),
  specialLatinLabel: document.getElementById("special-latin-label"),
  specialLatinButtons: document.getElementById("special-latin-buttons"),
  kLabel: document.getElementById("k-label"),
  kKa: document.getElementById("k-ka"),
  kKha: document.getElementById("k-kha"),
  latinLegendSummary: document.getElementById("latin-legend-summary"),
  latinLegendGrid: document.getElementById("latin-legend-grid"),
  outputHeading: document.getElementById("output-heading"),
  solutionCountPill: document.getElementById("solution-count-pill"),
  errorBox: document.getElementById("error-box"),
  errorTitle: document.getElementById("error-title"),
  errorDetail: document.getElementById("error-detail"),
  errorResetLatinBtn: document.getElementById("error-reset-latin-btn"),
  emptyState: document.getElementById("empty-state"),
  solutionsList: document.getElementById("solutions-list"),
  footerNote: document.getElementById("footer-note"),
};

function t() {
  return I18N[state.lang];
}

function formatCodepoint(cp) {
  return "U+" + cp.toString(16).toUpperCase().padStart(4, "0");
}

function renderCodepoints(container, text) {
  container.replaceChildren();
  for (const ch of text) {
    const cp = ch.codePointAt(0);
    const pill = document.createElement("span");
    const ctrlName = CONTROL_LABELS[cp];
    pill.className = ctrlName ? "codepoint control" : "codepoint";
    pill.textContent = ctrlName
      ? `${ctrlName} (${formatCodepoint(cp)})`
      : formatCodepoint(cp);
    container.appendChild(pill);
  }
}

function buildStaticControls() {
  els.specialLatinButtons.replaceChildren();
  for (const ch of SPECIAL_LATIN_CHARS) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "char-btn";
    btn.textContent = ch;
    btn.addEventListener("click", () => insertIntoLatin(ch));
    els.specialLatinButtons.appendChild(btn);
  }

  els.latinLegendGrid.replaceChildren();
  for (const [lat, mgl] of LATIN_LEGEND) {
    const item = document.createElement("div");
    item.className = "legend-item";
    const left = document.createElement("span");
    left.className = "legend-latin";
    left.textContent = lat;
    const right = document.createElement("span");
    right.className = "legend-mgl";
    right.textContent = mgl;
    item.appendChild(left);
    item.appendChild(right);
    els.latinLegendGrid.appendChild(item);
  }
}

function renderSamples() {
  els.samplesList.replaceChildren();
  for (const sample of SAMPLES) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = state.menk === sample.menk ? "sample-btn active" : "sample-btn";
    btn.textContent = sample.label[state.lang];
    btn.addEventListener("click", () => {
      state.menk = sample.menk;
      state.latinCustom = null;
      els.menkInput.value = state.menk;
      syncAndSolve();
    });
    els.samplesList.appendChild(btn);
  }
}

function applyLanguage() {
  const dict = t();
  document.documentElement.lang = state.lang === "zh" ? "zh-CN" : "en";
  document.title = dict.docTitle;
  els.langZh.classList.toggle("active", state.lang === "zh");
  els.langEn.classList.toggle("active", state.lang === "en");

  els.eyebrowText.textContent = dict.eyebrow;
  els.heroTitle.textContent = dict.heroTitle;
  els.heroSubtitle.textContent = dict.heroSubtitle;
  if (!state.wasmReady) {
    if (!els.wasmStatus.classList.contains("failed")) {
      els.wasmStatus.textContent = dict.wasmLoading;
    } else {
      els.wasmStatus.textContent = dict.wasmFailed;
    }
  } else {
    els.wasmStatus.textContent = dict.wasmReady;
  }

  els.inputHeading.textContent = dict.inputHeading;
  els.clearBtn.textContent = dict.clear;
  els.samplesLabel.textContent = dict.samplesLabel;
  els.menkLabel.textContent = dict.menkLabel;
  els.menkInput.placeholder = dict.menkPlaceholder;
  els.menkPreviewCaption.textContent = dict.menkPreviewCaption;
  els.unitsLabel.textContent = dict.unitsLabel;
  els.menkCodesSummary.textContent = dict.menkCodesSummary;
  els.latinLabel.textContent = dict.latinLabel;
  els.latinInput.placeholder = dict.latinPlaceholder;
  els.resetLatinBtn.textContent = dict.resetLatin;
  els.latinHelp.textContent = dict.latinHelp;
  els.specialLatinLabel.textContent = dict.specialLatinLabel;
  els.kLabel.textContent = dict.kLabel;
  els.kKa.textContent = dict.kKa;
  els.kKha.textContent = dict.kKha;
  els.latinLegendSummary.textContent = dict.latinLegendSummary;
  els.outputHeading.textContent = dict.outputHeading;
  els.emptyState.textContent = dict.emptyState;
  els.errorResetLatinBtn.textContent = dict.errResetBtn;
  els.footerNote.textContent = dict.footerNote;

  renderSamples();
  if (state.wasmReady) {
    syncAndSolve();
  }
}

function insertIntoLatin(ch) {
  const input = els.latinInput;
  const start = input.selectionStart ?? input.value.length;
  const end = input.selectionEnd ?? input.value.length;
  const next = input.value.slice(0, start) + ch + input.value.slice(end);
  input.value = next;
  state.latinCustom = next;
  input.focus();
  input.setSelectionRange(start + ch.length, start + ch.length);
  syncAndSolve();
}

function updateLatinBadge() {
  const dict = t();
  const isCustom = state.latinCustom !== null && state.latinCustom.trim() !== "";
  els.latinModeBadge.textContent = isCustom ? dict.modeCustom : dict.modeAuto;
  els.latinModeBadge.classList.toggle("custom", isCustom);
  els.resetLatinBtn.hidden = state.latinCustom === null;
}

function renderUnits(units) {
  els.unitsList.replaceChildren();
  if (!units || units.length === 0) {
    els.unitsBar.hidden = true;
    return;
  }
  els.unitsBar.hidden = false;
  for (const label of units) {
    const chip = document.createElement("span");
    chip.className = "unit-chip";
    chip.textContent = label;
    els.unitsList.appendChild(chip);
  }
}

function renderError(err) {
  const dict = t();
  els.solutionsList.replaceChildren();
  els.emptyState.hidden = true;
  els.errorBox.hidden = false;
  els.solutionCountPill.textContent = dict.countError;
  els.solutionCountPill.className = "count-pill error";

  if (err.kind === "InvalidMenkChar") {
    els.errorTitle.textContent = dict.errInvalidMenkTitle;
    els.errorDetail.textContent = dict.errInvalidMenkDetail(err);
    els.errorResetLatinBtn.hidden = true;
  } else if (err.kind === "InvalidLatin") {
    els.errorTitle.textContent = dict.errInvalidLatinTitle;
    els.errorDetail.textContent = dict.errInvalidLatinDetail(err);
    els.errorResetLatinBtn.hidden = state.latinCustom === null;
  } else if (err.kind === "ShapeMismatch") {
    els.errorTitle.textContent = dict.errShapeMismatchTitle;
    els.errorDetail.textContent = dict.errShapeMismatchDetail;
    els.errorResetLatinBtn.hidden = state.latinCustom === null;
  } else {
    els.errorTitle.textContent = dict.errTargetNormTitle;
    els.errorDetail.textContent = err.message || "Unknown error";
    els.errorResetLatinBtn.hidden = true;
  }
}

function copyText(text, btn) {
  const dict = t();
  if (!navigator.clipboard || !navigator.clipboard.writeText) {
    return;
  }
  navigator.clipboard.writeText(text).then(() => {
    btn.textContent = dict.copied;
    btn.classList.add("copied");
    window.setTimeout(() => {
      btn.textContent = t().copy;
      btn.classList.remove("copied");
    }, 1200);
  });
}

function createSolutionCard(sol, index, menkText) {
  const dict = t();
  const card = document.createElement("article");
  card.className = "solution-card";

  // Header
  const head = document.createElement("div");
  head.className = "solution-head";

  const badges = document.createElement("div");
  badges.className = "solution-badges";

  const idxBadge = document.createElement("span");
  idxBadge.className = "sol-index";
  idxBadge.textContent = `#${index + 1}`;
  badges.appendChild(idxBadge);

  const fvsBadge = document.createElement("span");
  fvsBadge.className = sol.fvs_count === 0 ? "sol-badge fvs-zero" : "sol-badge fvs-nonzero";
  fvsBadge.textContent = dict.fvsBadge(sol.fvs_count);
  badges.appendChild(fvsBadge);

  const methodBadge = document.createElement("span");
  methodBadge.className = sol.used_greedy_fallback ? "sol-badge greedy" : "sol-badge exact";
  methodBadge.textContent = sol.used_greedy_fallback ? dict.greedyBadge : dict.exactBadge;
  badges.appendChild(methodBadge);

  const copyBtn = document.createElement("button");
  copyBtn.type = "button";
  copyBtn.className = "copy-btn";
  copyBtn.textContent = dict.copy;
  copyBtn.addEventListener("click", () => copyText(sol.unicode, copyBtn));

  head.appendChild(badges);
  head.appendChild(copyBtn);
  card.appendChild(head);

  // Body
  const body = document.createElement("div");
  body.className = "solution-body";

  const visual = document.createElement("div");
  visual.className = "solution-visual";

  // Side-by-side vertical comparison
  const compare = document.createElement("div");
  compare.className = "glyph-compare";

  const menkCol = document.createElement("div");
  menkCol.className = "glyph-col";
  const menkCap = document.createElement("span");
  menkCap.className = "vertical-caption";
  menkCap.textContent = dict.menkPreviewCaption;
  const menkGlyph = document.createElement("span");
  menkGlyph.className = "menk-vertical";
  menkGlyph.textContent = menkText;
  menkCol.appendChild(menkCap);
  menkCol.appendChild(menkGlyph);

  const utnCol = document.createElement("div");
  utnCol.className = "glyph-col utn-col";
  const utnCap = document.createElement("span");
  utnCap.className = "vertical-caption";
  utnCap.textContent = dict.utnPreviewCaption;
  const utnGlyph = document.createElement("span");
  utnGlyph.className = "utn-vertical";
  utnGlyph.textContent = sol.unicode;
  utnCol.appendChild(utnCap);
  utnCol.appendChild(utnGlyph);

  compare.appendChild(menkCol);
  compare.appendChild(utnCol);

  // Horizontal text + code points
  const textCol = document.createElement("div");
  textCol.className = "solution-text-col";

  const unicodeStr = document.createElement("p");
  unicodeStr.className = "unicode-string";
  unicodeStr.textContent = sol.unicode;
  textCol.appendChild(unicodeStr);

  const cps = document.createElement("div");
  cps.className = "codepoints";
  renderCodepoints(cps, sol.unicode);
  textCol.appendChild(cps);

  visual.appendChild(compare);
  visual.appendChild(textCol);
  body.appendChild(visual);

  // Per-letter breakdown
  if (sol.letters && sol.letters.length > 0) {
    const lettersSec = document.createElement("div");
    lettersSec.className = "letters-section";

    const title = document.createElement("p");
    title.className = "letters-title";
    title.textContent = dict.lettersBreakdown;
    lettersSec.appendChild(title);

    const grid = document.createElement("div");
    grid.className = "letters-grid";
    for (const item of sol.letters) {
      const chip = document.createElement("div");
      chip.className = item.fvs ? "letter-chip has-fvs" : "letter-chip";

      const glyph = document.createElement("span");
      glyph.className = "letter-glyph";
      glyph.textContent = item.letter;
      chip.appendChild(glyph);

      const meta = document.createElement("span");
      meta.className = "letter-meta";
      meta.textContent = item.position;
      chip.appendChild(meta);

      const written = document.createElement("span");
      written.className = "letter-written";
      written.textContent = item.written.join("+");
      chip.appendChild(written);

      if (item.fvs) {
        const fvs = document.createElement("span");
        fvs.className = "letter-fvs";
        fvs.textContent = item.fvs;
        chip.appendChild(fvs);
      }

      grid.appendChild(chip);
    }

    lettersSec.appendChild(grid);
    body.appendChild(lettersSec);
  }

  card.appendChild(body);
  return card;
}

function syncAndSolve() {
  const dict = t();
  const menkText = state.menk.trim();
  const charLen = [...menkText].length;
  els.menkCount.textContent = dict.charCount(charLen);
  els.menkVerticalPreview.textContent = menkText;
  renderCodepoints(els.menkCodepoints, menkText);
  renderSamples();
  updateLatinBadge();

  if (!state.wasmReady) {
    return;
  }

  if (menkText.length === 0) {
    if (state.latinCustom === null) {
      els.latinInput.value = "";
    }
    renderUnits([]);
    els.errorBox.hidden = true;
    els.solutionsList.replaceChildren();
    els.emptyState.hidden = false;
    els.solutionCountPill.textContent = dict.countEmpty;
    els.solutionCountPill.className = "count-pill";
    return;
  }

  if (state.latinCustom === null) {
    try {
      els.latinInput.value = infer_standard_latin(menkText);
    } catch {
      els.latinInput.value = "";
    }
  }

  const trimmedLatin = state.latinCustom === null ? "" : state.latinCustom.trim();
  const latinArg = trimmedLatin === "" ? undefined : trimmedLatin;
  const rawJson = resolve_menk_to_unicode(menkText, latinArg, state.useKha);
  const payload = JSON.parse(rawJson);

  if (!payload.ok) {
    if (payload.error.kind === "InvalidMenkChar") {
      renderUnits([]);
    }
    renderError(payload.error);
    return;
  }

  renderUnits(payload.units);
  els.errorBox.hidden = true;
  els.emptyState.hidden = true;
  els.solutionsList.replaceChildren();

  const solutions = payload.solutions || [];
  els.solutionCountPill.textContent = dict.countSolutions(solutions.length);
  els.solutionCountPill.className =
    solutions.length > 1 ? "count-pill multi" : "count-pill";

  for (let i = 0; i < solutions.length; i += 1) {
    els.solutionsList.appendChild(createSolutionCard(solutions[i], i, menkText));
  }
}

function bindEvents() {
  els.langZh.addEventListener("click", () => {
    state.lang = "zh";
    applyLanguage();
  });
  els.langEn.addEventListener("click", () => {
    state.lang = "en";
    applyLanguage();
  });

  els.menkInput.addEventListener("input", () => {
    state.menk = els.menkInput.value;
    state.latinCustom = null;
    syncAndSolve();
  });

  els.latinInput.addEventListener("input", () => {
    state.latinCustom = els.latinInput.value;
    syncAndSolve();
  });

  const resetLatin = () => {
    state.latinCustom = null;
    syncAndSolve();
  };
  els.resetLatinBtn.addEventListener("click", resetLatin);
  els.errorResetLatinBtn.addEventListener("click", resetLatin);

  els.clearBtn.addEventListener("click", () => {
    state.menk = "";
    state.latinCustom = null;
    els.menkInput.value = "";
    els.latinInput.value = "";
    syncAndSolve();
  });

  els.kKa.addEventListener("click", () => {
    state.useKha = false;
    els.kKa.classList.add("active");
    els.kKha.classList.remove("active");
    syncAndSolve();
  });
  els.kKha.addEventListener("click", () => {
    state.useKha = true;
    els.kKha.classList.add("active");
    els.kKa.classList.remove("active");
    syncAndSolve();
  });
}

async function bootstrap() {
  buildStaticControls();
  bindEvents();
  els.menkInput.value = state.menk;
  els.wasmStatus.classList.add("loading");
  applyLanguage();

  try {
    await init();
    state.wasmReady = true;
    els.wasmStatus.classList.remove("loading");
    els.wasmStatus.textContent = t().wasmReady;
    syncAndSolve();
  } catch {
    els.wasmStatus.classList.remove("loading");
    els.wasmStatus.classList.add("failed");
    els.wasmStatus.textContent = t().wasmFailed;
  }
}

bootstrap();
