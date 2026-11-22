//! Integration tests for exhaustive UTN #57 shape + pure Unicode Mongolian letter (`U+1820..=U+1842`) multi-solution FVS resolution.
//!
//! UTN #57 字形 + 纯蒙古文 Unicode 基础字母串（`U+1820..=U+1842`）全解枚举与最少 FVS 求解的集成测试。

use menk_to_utn57::mongol_norm::{Locale, Shaper};
use menk_to_utn57::{
    get_standard_letters, get_standard_letters_with_k, is_mongolian_letter, menk_char_letter,
    menk_word_to_utn57, utn57_and_letters_to_unicode, utn57_letter_variants, Fvs, KLetter,
    Position, PositionedWrittenUnit, ResolveError, ResolvedLetter, Solution, UnitPosition,
    WrittenUnit,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn expand_test_token_written(raw: &[WrittenUnit], pos: Position) -> Vec<WrittenUnit> {
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

fn unicodes(solutions: &[Solution]) -> Vec<&str> {
    solutions.iter().map(|s| s.unicode.as_str()).collect()
}

#[test]
fn utn57_variant_table_matches_mongol_norm_shaper() {
    // Verify all 35 letters (`U+1820..=U+1842`) and 216 variants match `mongol_norm::Shaper` verbatim.
    // 校验全部 35 个蒙古文 Unicode 字母（`U+1820..=U+1842`）及 216 条变体与 `mongol_norm::Shaper` 完全一致。
    let shaper = Shaper::new(Locale::Mng);
    let mut total_variants = 0usize;

    for cp in 0x1820u32..=0x1842 {
        let ch = char::from_u32(cp).unwrap();
        assert!(is_mongolian_letter(ch));
        for pos in Position::ALL {
            let variants = utn57_letter_variants(ch, pos);
            assert!(
                !variants.is_empty(),
                "missing variants for U+{cp:04X} at {pos:?}"
            );
            assert_eq!(
                variants.iter().filter(|v| v.default).count(),
                1,
                "U+{cp:04X} at {pos:?} must have exactly one default"
            );

            for var in variants {
                total_variants += 1;
                if let Some(fvs) = var.fvs {
                    let fvs_ch = fvs.as_char();
                    let (sample, tok_idx) = match pos {
                        Position::Isol => (format!("{ch}{fvs_ch}"), 0),
                        Position::Init => (format!("{ch}{fvs_ch}\u{200D}"), 0),
                        Position::Medi => (format!("\u{200D}{ch}{fvs_ch}\u{200D}"), 1),
                        Position::Fina => (format!("\u{200D}{ch}{fvs_ch}"), 1),
                    };
                    let details = shaper.shape_detailed(&sample).unwrap();
                    assert_eq!(
                        details[tok_idx].written, var.written,
                        "mismatch for U+{cp:04X} {pos:?} {fvs:?}"
                    );
                } else if var.default {
                    let (sample, tok_idx) = match pos {
                        Position::Isol => (format!("{ch}"), 0),
                        Position::Init => (format!("{ch}\u{200D}"), 0),
                        Position::Medi => (format!("\u{200D}{ch}\u{200D}"), 1),
                        Position::Fina => (format!("\u{200D}{ch}"), 1),
                    };
                    let details = shaper.shape_detailed(&sample).unwrap();
                    assert_eq!(
                        details[tok_idx].written, var.written,
                        "default mismatch for U+{cp:04X} {pos:?}"
                    );
                }
            }
        }
    }

    assert_eq!(total_variants, 216);
}

#[test]
fn standard_letters_helper_covers_all_menk_and_unicode_letters() {
    // Every Menk code point in `U+E264..=U+E34F` maps to a pure Mongolian letter (`U+1820..=U+1842`),
    // and all 35 Unicode Mongolian letters are covered across `KLetter::Ka` (`ᠺ`) + `KLetter::Kha` (`ᠻ`).
    // `U+E264..=U+E34F` 内的全部 236 个孟克字符均返回纯蒙古文基础字母（`U+1820..=U+1842`），且完整覆盖全部 35 个蒙古文字母。
    let mut covered_letters = BTreeSet::new();
    for cp in 0xE264u32..=0xE34F {
        let ch = char::from_u32(cp).unwrap();
        let ka = menk_char_letter(ch, KLetter::Ka).unwrap();
        let kha = menk_char_letter(ch, KLetter::Kha).unwrap();
        assert!(is_mongolian_letter(ka));
        assert!(is_mongolian_letter(kha));
        assert_eq!(
            get_standard_letters(&ch.to_string()).unwrap(),
            ka.to_string()
        );
        assert_eq!(
            get_standard_letters_with_k(&ch.to_string(), KLetter::Kha).unwrap(),
            kha.to_string()
        );
        covered_letters.insert(ka);
        covered_letters.insert(kha);
    }
    let expected: BTreeSet<char> = (0x1820u32..=0x1842)
        .map(|cp| char::from_u32(cp).unwrap())
        .collect();
    assert_eq!(covered_letters, expected);

    assert!(menk_char_letter('a', KLetter::Ka).is_none());
    assert!(get_standard_letters("abc").is_err());
}

#[test]
fn resolves_unique_zero_fvs_and_fvs_words() {
    // 1. Chachlag words `ᠪᠠᠢᠨᠠ` (`ᠪᠠᠢᠨ᠎ᠠ`), `ᠰᠠᠷᠠ` (`ᠰᠠᠷ᠎ᠠ`), and standalone `[Aa:isol]` (`᠎ᠠ`).
    // 1. 分写字尾词 `ᠪᠠᠢᠨᠠ`、`ᠰᠠᠷᠠ` 与独立字尾 `[Aa:isol]`。
    let baina_units = menk_word_to_utn57("\u{E2C1}\u{E26C}\u{E281}\u{E2B6}\u{E26A}").unwrap();
    let baina_sols = utn57_and_letters_to_unicode(&baina_units, "ᠪᠠᠢᠨᠠ").unwrap();
    assert_eq!(
        baina_sols,
        vec![Solution {
            unicode: "ᠪᠠᠢᠨ᠎ᠠ".to_string(),
            fvs_count: 0,
            letters: vec![
                ResolvedLetter {
                    letter: 'ᠪ',
                    position: Position::Init,
                    written: &[WrittenUnit::B],
                    fvs: None,
                },
                ResolvedLetter {
                    letter: 'ᠠ',
                    position: Position::Medi,
                    written: &[WrittenUnit::A],
                    fvs: None,
                },
                ResolvedLetter {
                    letter: 'ᠢ',
                    position: Position::Medi,
                    written: &[WrittenUnit::I, WrittenUnit::I],
                    fvs: None,
                },
                ResolvedLetter {
                    letter: 'ᠨ',
                    position: Position::Fina,
                    written: &[WrittenUnit::N],
                    fvs: None,
                },
                ResolvedLetter {
                    letter: 'ᠠ',
                    position: Position::Isol,
                    written: &[WrittenUnit::Aa],
                    fvs: None,
                },
            ],
            used_greedy_fallback: false,
        }]
    );

    let sara_units = menk_word_to_utn57("\u{E2FD}\u{E26C}\u{E325}\u{E26A}").unwrap();
    assert_eq!(
        unicodes(&utn57_and_letters_to_unicode(&sara_units, "ᠰᠠᠷᠠ").unwrap()),
        vec!["ᠰᠠᠷ᠎ᠠ"]
    );

    let standalone_aa = [PositionedWrittenUnit::new(
        WrittenUnit::Aa,
        UnitPosition::Isol,
    )];
    assert_eq!(
        unicodes(&utn57_and_letters_to_unicode(&standalone_aa, "ᠠ").unwrap()),
        vec!["᠎ᠠ"]
    );

    // 2. Unique-decomposition FVS words (`ᠶᠢᠨ`, `ᠢᠶᠡᠨ`, `ᠢᠶᠡᠷ`, `ᠳᠦ`).
    // 2. 需加 FVS 且字形切分唯一的词（`ᠶᠢᠨ`、`ᠢᠶᠡᠨ`、`ᠢᠶᠡᠷ`、`ᠳᠦ`）。
    let yin_units = menk_word_to_utn57("\u{E281}\u{E2B5}").unwrap();
    assert_eq!(
        unicodes(&utn57_and_letters_to_unicode(&yin_units, "ᠶᠢᠨ").unwrap()),
        vec!["ᠶ᠋ᠢᠨ"]
    );
    let iyen_units = menk_word_to_utn57("\u{E281}\u{E276}\u{E2B5}").unwrap();
    assert_eq!(
        unicodes(&utn57_and_letters_to_unicode(&iyen_units, "ᠢᠶᠡᠨ").unwrap()),
        vec!["ᠢ᠋ᠶ᠋ᠡᠨ"]
    );
    let iyer_units = menk_word_to_utn57("\u{E281}\u{E276}\u{E325}").unwrap();
    assert_eq!(
        unicodes(&utn57_and_letters_to_unicode(&iyer_units, "ᠢᠶᠡᠷ").unwrap()),
        vec!["ᠢ᠋ᠶ᠋ᠡᠷ"]
    );

    assert_eq!(
        unicodes(
            &utn57_and_letters_to_unicode(&menk_word_to_utn57("\u{E310}\u{E268}").unwrap(), "ᠳᠠ")
                .unwrap()
        ),
        vec!["ᠳᠠ"]
    );
    assert_eq!(
        unicodes(
            &utn57_and_letters_to_unicode(&menk_word_to_utn57("\u{E310}\u{E2A5}").unwrap(), "ᠳᠦ")
                .unwrap()
        ),
        vec!["ᠳᠦ"]
    );
    assert_eq!(
        unicodes(
            &utn57_and_letters_to_unicode(&menk_word_to_utn57("\u{E310}\u{E2A3}").unwrap(), "ᠳᠦ")
                .unwrap()
        ),
        vec!["ᠳ᠋ᠦ"]
    );
    assert_eq!(
        unicodes(
            &utn57_and_letters_to_unicode(&menk_word_to_utn57("\u{E310}\u{E2A7}").unwrap(), "ᠳᠦ")
                .unwrap()
        ),
        vec!["ᠳ᠋ᠦ᠌"]
    );
}

#[test]
fn enumerates_all_matching_solutions_without_zero_fvs_pruning() {
    // 1. `[N, A, A, A, L]` with `"ᠨᠠᠠᠯ"` -> returns BOTH minimal-FVS decompositions with full per-letter details!
    // 1. 目标字形 `[N, A, A, A, L]` + `"ᠨᠠᠠᠯ"` -> 完整返回两条切分解法及各自的逐字母子字形与 FVS 明细！
    let naal_units = [
        PositionedWrittenUnit::new(WrittenUnit::N, UnitPosition::Init),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::L, UnitPosition::Fina),
    ];

    let naal_solutions = utn57_and_letters_to_unicode(&naal_units, "ᠨᠠᠠᠯ").unwrap();
    assert_eq!(
        naal_solutions,
        vec![
            Solution {
                unicode: "ᠨᠠᠠ᠋ᠯ".to_string(),
                fvs_count: 1,
                letters: vec![
                    ResolvedLetter {
                        letter: 'ᠨ',
                        position: Position::Init,
                        written: &[WrittenUnit::N],
                        fvs: None,
                    },
                    ResolvedLetter {
                        letter: 'ᠠ',
                        position: Position::Medi,
                        written: &[WrittenUnit::A],
                        fvs: None,
                    },
                    ResolvedLetter {
                        letter: 'ᠠ',
                        position: Position::Medi,
                        written: &[WrittenUnit::A, WrittenUnit::A],
                        fvs: Some(Fvs::Fvs1),
                    },
                    ResolvedLetter {
                        letter: 'ᠯ',
                        position: Position::Fina,
                        written: &[WrittenUnit::L],
                        fvs: None,
                    },
                ],
                used_greedy_fallback: false,
            },
            Solution {
                unicode: "ᠨᠠ᠋ᠠᠯ".to_string(),
                fvs_count: 1,
                letters: vec![
                    ResolvedLetter {
                        letter: 'ᠨ',
                        position: Position::Init,
                        written: &[WrittenUnit::N],
                        fvs: None,
                    },
                    ResolvedLetter {
                        letter: 'ᠠ',
                        position: Position::Medi,
                        written: &[WrittenUnit::A, WrittenUnit::A],
                        fvs: Some(Fvs::Fvs1),
                    },
                    ResolvedLetter {
                        letter: 'ᠠ',
                        position: Position::Medi,
                        written: &[WrittenUnit::A],
                        fvs: None,
                    },
                    ResolvedLetter {
                        letter: 'ᠯ',
                        position: Position::Fina,
                        written: &[WrittenUnit::L],
                        fvs: None,
                    },
                ],
                used_greedy_fallback: false,
            },
        ]
    );

    // 2. Two independent ambiguous spans (`"ᠨᠠᠠᠯᠨᠠᠠᠯ"` on `[N, A, A, A, L, N, A, A, A, L]`) -> 2 * 2 = 4 solutions!
    // 2. 两段独立歧义区间（`"ᠨᠠᠠᠯᠨᠠᠠᠯ"`）返回全部 2 * 2 = 4 种解法！
    let double_units = [
        PositionedWrittenUnit::new(WrittenUnit::N, UnitPosition::Init),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::L, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::N, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::L, UnitPosition::Fina),
    ];
    let double_sols = utn57_and_letters_to_unicode(&double_units, "ᠨᠠᠠᠯᠨᠠᠠᠯ").unwrap();
    assert_eq!(
        unicodes(&double_sols),
        vec![
            "ᠨᠠᠠ\u{180B}ᠯᠨᠠᠠ\u{180B}ᠯ",
            "ᠨᠠᠠ\u{180B}ᠯᠨᠠ\u{180B}ᠠᠯ",
            "ᠨᠠ\u{180B}ᠠᠯᠨᠠᠠ\u{180B}ᠯ",
            "ᠨᠠ\u{180B}ᠠᠯᠨᠠ\u{180B}ᠠᠯ",
        ]
    );

    // 3. No 0-FVS pruning: when both a 0-FVS decomposition and a >0-FVS decomposition exist, BOTH are returned
    //    sorted by `fvs_count` ascending!
    // 3. 不做 0-FVS 剪枝：当 0-FVS 切分与附加 FVS 的切分同时满足目标字形时，按 `fvs_count` 升序全部返回！
    let fruil_units = [
        PositionedWrittenUnit::new(WrittenUnit::F, UnitPosition::Init),
        PositionedWrittenUnit::new(WrittenUnit::R, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::O, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::I, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::I, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::L, UnitPosition::Fina),
    ];
    let fruil_sols = utn57_and_letters_to_unicode(&fruil_units, "ᠹᠷᠦᠢᠯ").unwrap();
    assert_eq!(fruil_sols.len(), 2);
    assert_eq!(fruil_sols[0].unicode, "ᠹᠷᠦᠢᠯ");
    assert_eq!(fruil_sols[0].fvs_count, 0);
    assert_eq!(
        fruil_sols[0].letters[2].written,
        &[WrittenUnit::O, WrittenUnit::I]
    );
    assert_eq!(fruil_sols[0].letters[3].written, &[WrittenUnit::I]);
    assert_eq!(fruil_sols[1].unicode, "ᠹᠷᠦ\u{180D}ᠢᠯ");
    assert_eq!(fruil_sols[1].fvs_count, 1);
    assert_eq!(fruil_sols[1].letters[2].written, &[WrittenUnit::O]);
    assert_eq!(fruil_sols[1].letters[2].fvs, Some(Fvs::Fvs3));
    assert_eq!(
        fruil_sols[1].letters[3].written,
        &[WrittenUnit::I, WrittenUnit::I]
    );
    assert_eq!(fruil_sols[1].letters[3].fvs, None);

    let fruid_units = menk_word_to_utn57("\u{E330}\u{E326}\u{E2AA}\u{E27E}\u{E312}").unwrap();
    let fruid_sols = utn57_and_letters_to_unicode(&fruid_units, "ᠹᠷᠦᠢᠳ").unwrap();
    assert_eq!(
        unicodes(&fruid_sols),
        vec!["ᠹᠷᠦᠢᠳ\u{180B}", "ᠹᠷᠦ\u{180D}ᠢᠳ\u{180B}"]
    );

    // 4. `[S, A, A, I, A]` + `"ᠰᠠᠢᠨ"` -> returns `"ᠰᠠᠢ᠋ᠨ"` (`fvs_count: 1`) and `"ᠰᠠ᠋ᠢ᠍ᠨ"` (`fvs_count: 2`).
    // 4. `[S, A, A, I, A]` + `"ᠰᠠᠢᠨ"` -> 按 FVS 数量升序返回 `"ᠰᠠᠢ᠋ᠨ"`（1 个 FVS）与 `"ᠰᠠ᠋ᠢ᠍ᠨ"`（2 个 FVS）。
    let sain_odd_units = [
        PositionedWrittenUnit::new(WrittenUnit::S, UnitPosition::Init),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::I, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Fina),
    ];
    let sain_sols = utn57_and_letters_to_unicode(&sain_odd_units, "ᠰᠠᠢᠨ").unwrap();
    assert_eq!(
        unicodes(&sain_sols),
        vec!["ᠰᠠᠢ\u{180B}ᠨ", "ᠰᠠ\u{180B}ᠢ\u{180D}ᠨ"]
    );
    assert_eq!(sain_sols[0].fvs_count, 1);
    assert_eq!(sain_sols[1].fvs_count, 2);
}

#[test]
fn ka_and_kha_letters_both_align_directly_with_k_glyphs() {
    // Passing `'ᠺ'` (`U+183A`, Inner Mongolia) or `'ᠻ'` (`U+183B`, Mongolia) directly in `letters`
    // works seamlessly without extra solver options!
    // 在 `letters` 中直接传入内蒙古标准 `'ᠺ'`（`U+183A`）或蒙古国标准 `'ᠻ'`（`U+183B`）均可直接求解，无需额外配置项！
    let kino_menk = "\u{E333}\u{E27F}\u{E2B9}\u{E286}";
    let kino_units = menk_word_to_utn57(kino_menk).unwrap();

    assert_eq!(get_standard_letters(kino_menk).unwrap(), "ᠺᠢᠨᠣ");
    assert_eq!(
        get_standard_letters_with_k(kino_menk, KLetter::Kha).unwrap(),
        "ᠻᠢᠨᠣ"
    );

    assert_eq!(
        unicodes(&utn57_and_letters_to_unicode(&kino_units, "ᠺᠢᠨᠣ").unwrap()),
        vec!["ᠺᠢᠨᠣ\u{180B}"]
    );
    assert_eq!(
        unicodes(&utn57_and_letters_to_unicode(&kino_units, "ᠻᠢᠨᠣ").unwrap()),
        vec!["ᠻᠢᠨᠣ\u{180B}"]
    );
}

#[test]
fn distinct_unicode_letters_and_chachlag_mvs_rules_work() {
    let shaper = Shaper::new(Locale::Mng);

    // 1. `'ᠩ'` (`U+1829`) vs `'ᠨ'` + `'ᠭ'` (`U+1828` + `U+182D`) are distinct Unicode letters directly!
    // 1. `'ᠩ'`（`U+1829`）与 `'ᠨ'` + `'ᠭ'`（`U+1828` + `U+182D`）在 Unicode 字母串中天然无歧义区分！
    let monggol_units =
        menk_word_to_utn57("\u{E2F2}\u{E289}\u{E2BC}\u{E2EA}\u{E289}\u{E2F9}").unwrap();
    assert_eq!(
        unicodes(&utn57_and_letters_to_unicode(&monggol_units, "ᠮᠣᠩᠭᠣᠯ").unwrap()),
        vec!["ᠮᠣᠩᠭᠣᠯ"]
    );

    let zegungar_units =
        menk_word_to_utn57("\u{E319}\u{E276}\u{E2ED}\u{E2AC}\u{E2B8}\u{E2EA}\u{E26C}\u{E325}")
            .unwrap();
    assert_eq!(
        unicodes(&utn57_and_letters_to_unicode(&zegungar_units, "ᠵᠡᠭᠦᠨᠭᠠᠷ").unwrap()),
        vec!["ᠵᠡᠭᠦᠨᠭᠠᠷ"]
    );

    let fem_fina_units = [
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Init),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::G, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Fina),
    ];
    let enge_ang = &utn57_and_letters_to_unicode(&fem_fina_units, "ᠡᠩᠡ").unwrap()[0].unicode;
    let enge_ng = &utn57_and_letters_to_unicode(&fem_fina_units, "ᠡᠨᠭᠡ").unwrap()[0].unicode;
    assert_eq!(enge_ang, "ᠡᠩᠡ");
    assert_eq!(enge_ng, "ᠡᠨᠭᠡ᠌");
    assert!(shaper.same_shape(enge_ang, enge_ng).unwrap());

    // 2. Foreign/Galig Unicode letters: `ᠧ` (`U+1827`), `ᠾ` (`U+183E`), `ᠿ` (`U+183F`), `ᡀ` (`U+1840`), `ᡁ` (`U+1841`), `ᡂ` (`U+1842`)
    // 2. 外来音与嘎礼克字母直接校验：`ᠧ`、`ᠾ`、`ᠿ`、`ᡀ`、`ᡁ`、`ᡂ`
    for (menk_word, expected_letters) in [
        ("\u{E2AD}", "ᠧ"),
        ("\u{E2AE}\u{E325}", "ᠧᠷ"),
        ("\u{E345}\u{E268}", "ᠾᠠ"),
        ("\u{E348}\u{E268}", "ᠿᠠ"),
        ("\u{E34B}\u{E268}", "ᡀᠠ"),
        ("\u{E34E}\u{E27F}\u{E2B5}", "ᡁᠢᠨ"),
        ("\u{E34F}\u{E27F}\u{E2B5}", "ᡂᠢᠨ"),
    ] {
        assert_eq!(get_standard_letters(menk_word).unwrap(), expected_letters);
        let units = menk_word_to_utn57(menk_word).unwrap();
        assert_eq!(
            unicodes(&utn57_and_letters_to_unicode(&units, expected_letters).unwrap()),
            vec![expected_letters]
        );
    }

    // 3. Non-Chachlag suffix MVS (`Mvs:control` + non-`Aa` like `U:isol`) is not a single word and is rejected.
    // 3. 格助词前的 MVS（`Mvs:control` + 非 `Aa` 如 `U:isol`）不属于单词内部 Chachlag，直接返回 `ShapeMismatch`。
    let eguden_mvs_units = [
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Init),
        PositionedWrittenUnit::new(WrittenUnit::G, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::O, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::D, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Medi),
        PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Fina),
        PositionedWrittenUnit::new(WrittenUnit::Mvs, UnitPosition::Control),
        PositionedWrittenUnit::new(WrittenUnit::U, UnitPosition::Isol),
    ];
    assert_eq!(
        utn57_and_letters_to_unicode(&eguden_mvs_units, "ᠡᠭᠦᠳᠡᠨᠦ"),
        Err(ResolveError::ShapeMismatch)
    );
}

#[test]
fn error_variants_report_invalid_inputs_cleanly() {
    let sa_units = menk_word_to_utn57("\u{E2FD}\u{E268}").unwrap();
    assert_eq!(
        utn57_and_letters_to_unicode(&[], "").unwrap(),
        vec![Solution {
            unicode: String::new(),
            fvs_count: 0,
            letters: Vec::new(),
            used_greedy_fallback: false,
        }]
    );
    // Latin, punctuation, MVS (`U+180E`), and FVS (`U+180B`) are rejected in `letters` (`U+1820..=U+1842` only).
    // 拉丁字符、标点、MVS（`U+180E`）与 FVS（`U+180B`）在纯字母输入中均返回 `InvalidLetter`。
    assert_eq!(
        utn57_and_letters_to_unicode(&sa_units, "sa"),
        Err(ResolveError::InvalidLetter {
            index: 0,
            character: 's'
        })
    );
    assert_eq!(
        utn57_and_letters_to_unicode(&sa_units, "ᠰ\u{180E}ᠠ"),
        Err(ResolveError::InvalidLetter {
            index: 1,
            character: '\u{180E}'
        })
    );
    assert_eq!(
        utn57_and_letters_to_unicode(&sa_units, "ᠰᠠ\u{180B}"),
        Err(ResolveError::InvalidLetter {
            index: 2,
            character: '\u{180B}'
        })
    );
    assert_eq!(
        utn57_and_letters_to_unicode(&sa_units, "ᠪᠠ"),
        Err(ResolveError::ShapeMismatch)
    );

    // Chachlag (`MVS + Aa`) only permits `'ᠠ'` (`U+1820`) or `'ᠡ'` (`U+1821`).
    // 词尾分写 `Chachlag`（`MVS + Aa`）仅允许 `'ᠠ'` 或 `'ᠡ'`。
    let baina_units = menk_word_to_utn57("\u{E2C1}\u{E26C}\u{E281}\u{E2B6}\u{E26A}").unwrap();
    assert_eq!(
        utn57_and_letters_to_unicode(&baina_units, "ᠪᠠᠢᠨᠣ"),
        Err(ResolveError::ShapeMismatch)
    );

    // Incomplete glyph fragments requiring ZWJ or non-word breaks requiring ZWNJ are rejected.
    // 含 ZWJ 的残缺片段或含 ZWNJ 的非单词断口直接返回 `ShapeMismatch`。
    assert_eq!(
        utn57_and_letters_to_unicode(
            &[PositionedWrittenUnit::new(
                WrittenUnit::A,
                UnitPosition::Init
            )],
            "ᠠ",
        ),
        Err(ResolveError::ShapeMismatch)
    );
    assert_eq!(
        utn57_and_letters_to_unicode(
            &[
                PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Isol),
                PositionedWrittenUnit::new(WrittenUnit::A, UnitPosition::Isol),
            ],
            "ᠠᠠ",
        ),
        Err(ResolveError::ShapeMismatch)
    );
    assert!(matches!(
        utn57_and_letters_to_unicode(
            &[PositionedWrittenUnit::new(
                WrittenUnit::Aa,
                UnitPosition::Init
            )],
            "ᠠ",
        ),
        Err(ResolveError::TargetNormalization(_))
    ));
}

#[test]
fn optional_local_word_corpus_enumerates_and_verifies_all_solutions() {
    // When `word.txt` (gitignored, 65,148 entries) is present locally, verify that every solvable word
    // returns >= 1 valid `Solution`s sorted by `fvs_count`, and EVERY returned `Solution` matches both
    // `expected_norm` and its per-letter `ResolvedLetter` breakdown!
    // 本地存在 `word.txt`（65,148 条）时，验证所有可匹配词均返回非空且按 `fvs_count` 排序的 `Vec<Solution>`，
    // 且每一条 `Solution` 的整词字形与逐字母 `ResolvedLetter` 明细均与 `Shaper` 完全一致。
    let word_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("word.txt");
    if !word_path.exists() {
        return;
    }

    let word_txt = fs::read_to_string(word_path).expect("read word.txt");
    let shaper = Shaper::new(Locale::Mng);
    let mut solved_words = 0usize;
    let mut multi_solution_words = 0usize;
    let mut mismatch_count = 0usize;

    for (line_idx, line) in word_txt.lines().enumerate() {
        let word = line.trim();
        if word.is_empty() {
            continue;
        }
        let letters = get_standard_letters(word).unwrap();
        let units = menk_word_to_utn57(word).unwrap();
        let expected_units: Vec<PositionedWrittenUnit> = units
            .iter()
            .map(|u| {
                if u.unit == WrittenUnit::K {
                    PositionedWrittenUnit::new(WrittenUnit::K2, u.position)
                } else {
                    *u
                }
            })
            .collect();
        let expected_norm = shaper
            .normalize_positioned_written_units(&expected_units)
            .unwrap();

        match utn57_and_letters_to_unicode(&units, &letters) {
            Ok(solutions) => {
                assert!(!solutions.is_empty());
                solved_words += 1;
                if solutions.len() >= 2 {
                    multi_solution_words += 1;
                }

                let mut prev_fvs = 0usize;
                for sol in &solutions {
                    assert!(sol.fvs_count >= prev_fvs);
                    prev_fvs = sol.fvs_count;
                    assert!(!sol.used_greedy_fallback);
                    assert!(!sol.unicode.contains('\u{200C}') && !sol.unicode.contains('\u{200D}'));
                    assert!(
                        shaper.same_shape(&sol.unicode, &expected_norm).unwrap(),
                        "line {} shape mismatch: unicode={:?}, expected_norm={expected_norm:?}",
                        line_idx + 1,
                        sol.unicode
                    );

                    // Verify per-letter `ResolvedLetter` details match `Shaper::shape_detailed`.
                    let details = shaper.shape_detailed(&sol.unicode).unwrap();
                    let actual_letters: Vec<(Position, Vec<WrittenUnit>)> = details
                        .into_iter()
                        .filter(|tok| tok.alias.is_some())
                        .map(|tok| {
                            (
                                tok.position,
                                expand_test_token_written(&tok.written, tok.position),
                            )
                        })
                        .collect();
                    assert_eq!(actual_letters.len(), sol.letters.len());
                    for (rl, (pos, exp_w)) in sol.letters.iter().zip(actual_letters.iter()) {
                        assert_eq!(rl.position, *pos);
                        assert_eq!(rl.written, exp_w.as_slice());
                    }
                }
            }
            Err(_) => {
                mismatch_count += 1;
            }
        }
    }

    assert!(
        solved_words >= 65_000 && multi_solution_words >= 16 && mismatch_count <= 135,
        "unexpected corpus stats: solved_words={solved_words}, multi_solution_words={multi_solution_words}, mismatch={mismatch_count}"
    );
}

#[test]
fn long_word_budget_exact_and_greedy_fallback_work() {
    let shaper = Shaper::new(Locale::Mng);

    // 1. Long word with > 16 FVS-capable slots (18 medial `ᠨ` + 1 medial `ᠠ` requiring `FVS1`),
    //    needing only `k = 1` FVS: finishes well within the 65,536-check exact budget (`used_greedy_fallback == false`).
    // 1. 超过 16 个可加 FVS 槽位但实际只需 1 个 FVS 的长词：在 65,536 次精确预算内完成，`used_greedy_fallback == false`。
    let mut exact_units = vec![PositionedWrittenUnit::new(
        WrittenUnit::B,
        UnitPosition::Init,
    )];
    let mut exact_letters = String::from("ᠪ");
    for i in 0..18 {
        exact_units.push(PositionedWrittenUnit::new(
            WrittenUnit::N,
            UnitPosition::Medi,
        ));
        exact_units.push(PositionedWrittenUnit::new(
            WrittenUnit::A,
            UnitPosition::Medi,
        ));
        if i == 17 {
            exact_units.push(PositionedWrittenUnit::new(
                WrittenUnit::A,
                UnitPosition::Medi,
            ));
        }
        exact_letters.push_str("ᠨᠠ");
    }
    exact_units.push(PositionedWrittenUnit::new(
        WrittenUnit::L,
        UnitPosition::Fina,
    ));
    exact_letters.push('ᠯ');

    let exact_sols = utn57_and_letters_to_unicode(&exact_units, &exact_letters).unwrap();
    assert_eq!(exact_sols.len(), 1);
    assert_eq!(exact_sols[0].fvs_count, 1);
    assert!(!exact_sols[0].used_greedy_fallback);
    assert_eq!(
        shaper.shape(&exact_sols[0].unicode).unwrap(),
        shaper.canonical_positioned_units(&exact_units).unwrap()
    );

    // 2. Extreme word with 17 FVS-capable slots (`ᠨ` before consonant `ᠰ`) where ALL 17 FVS are
    //    required: exhausts the 65,536-check exact budget and falls back to greedy stripping
    //    (`used_greedy_fallback == true`), while still returning the exact valid rendering.
    // 2. 17 个槽位全部必选 FVS 的极端长词：耗尽 65,536 次精确搜索预算后回退至贪心剥离，返回合法解并标记 `used_greedy_fallback == true`。
    let mut greedy_units = vec![PositionedWrittenUnit::new(
        WrittenUnit::B,
        UnitPosition::Init,
    )];
    let mut greedy_letters = String::from("ᠪ");
    for _ in 0..17 {
        greedy_units.push(PositionedWrittenUnit::new(
            WrittenUnit::N,
            UnitPosition::Medi,
        ));
        greedy_units.push(PositionedWrittenUnit::new(
            WrittenUnit::S,
            UnitPosition::Medi,
        ));
        greedy_letters.push_str("ᠨᠰ");
    }
    greedy_units.push(PositionedWrittenUnit::new(
        WrittenUnit::L,
        UnitPosition::Fina,
    ));
    greedy_letters.push('ᠯ');

    let greedy_sols = utn57_and_letters_to_unicode(&greedy_units, &greedy_letters).unwrap();
    assert_eq!(greedy_sols.len(), 1);
    assert_eq!(greedy_sols[0].fvs_count, 17);
    assert!(greedy_sols[0].used_greedy_fallback);
    assert_eq!(
        shaper.shape(&greedy_sols[0].unicode).unwrap(),
        shaper.canonical_positioned_units(&greedy_units).unwrap()
    );
}
