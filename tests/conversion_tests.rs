//! Integration tests for Menk shape extraction, sub-chain splitting, and UTN #57 normalization.
//!
//! 孟克字形提取、子链切分与 UTN #57 归一化的集成测试。

use menk_to_utn57::mongol_norm::{Locale, Shaper};
use menk_to_utn57::{
    get_char_shape, get_shape, is_menk_char_only_fina, is_menk_char_only_init, menk_char_shapes,
    menk_shapes, menk_word_to_utn57, parse_menk_shapes, MenkShape, PositionedWrittenUnit,
    UnitPosition, WrittenUnit,
};
use std::fs;
use std::path::Path;

const fn utn(unit: WrittenUnit, pos: UnitPosition) -> PositionedWrittenUnit {
    PositionedWrittenUnit::new(unit, pos)
}

#[test]
fn all_236_menk_codepoints_and_44_shapes_are_consistent() {
    // Verify all 44 MenkShape variants and all 236 Menk PUA code points (`U+E264..=U+E34F`).
    // 校验全部 44 种基础字形与全部 236 个孟克码位（`U+E264..=U+E34F`）的一致性及可归一化性。
    assert_eq!(MenkShape::ALL.len(), 44);
    for &shape in MenkShape::ALL {
        assert_eq!(MenkShape::from_shape_char(shape.shape_char()), Some(shape));
        assert!(!shape.name().is_empty());
    }

    let shaper = Shaper::new(Locale::Mng);
    let mut init_count = 0usize;
    let mut fina_count = 0usize;

    for cp in 0xE264..=0xE34F {
        let ch = char::from_u32(cp).unwrap();
        let pua_shape = get_char_shape(ch);
        assert_eq!(
            parse_menk_shapes(&pua_shape).unwrap(),
            menk_char_shapes(ch).unwrap()
        );

        let only_init = is_menk_char_only_init(ch);
        let only_fina = is_menk_char_only_fina(ch);
        if only_init {
            init_count += 1;
        }
        if only_fina {
            fina_count += 1;
        }

        // Build a valid test word around `ch` according to its positional constraints
        // and verify `mongol-norm` accepts the resulting positioned units.
        // 按 `ch` 的首尾约束构造合法测试词串，校验转换结果可通过 `mongol-norm` 归一化。
        let sample = match (only_init, only_fina) {
            (true, true) => ch.to_string(),
            (true, false) => format!("{ch}\u{E268}"),
            (false, true) => format!("\u{E267}{ch}"),
            (false, false) => format!("\u{E267}{ch}\u{E268}"),
        };
        let units = menk_word_to_utn57(&sample).unwrap();
        shaper
            .normalize_positioned_written_units(&units)
            .unwrap_or_else(|err| panic!("U+{cp:04X} failed normalization: {err}"));
    }

    assert_eq!(init_count, 39);
    assert_eq!(fina_count, 48);
}

#[test]
fn dynamic_aa_and_subchain_positions_work() {
    // Connected `ba`: [B, A_MEDI, Aa] -> [B:init, A:medi, Aa:fina]
    // 连写 `ba`：[B, A_MEDI, Aa] -> [B:init, A:medi, Aa:fina]
    assert_eq!(
        menk_word_to_utn57("\u{E2C1}\u{E26B}").unwrap(),
        vec![
            utn(WrittenUnit::B, UnitPosition::Init),
            utn(WrittenUnit::A, UnitPosition::Medi),
            utn(WrittenUnit::Aa, UnitPosition::Fina),
        ]
    );

    // Chachlag `beye`: [B, A_MEDI, G, Aa] -> G closes stem at fina (I:fina), Aa is isol (Aa:isol)
    // 分写字尾 `beye`：G 在词尾收为 I:fina，后续 Aa 独立为 Aa:isol
    assert_eq!(
        menk_word_to_utn57("\u{E2C1}\u{E276}\u{E31F}\u{E274}").unwrap(),
        vec![
            utn(WrittenUnit::B, UnitPosition::Init),
            utn(WrittenUnit::A, UnitPosition::Medi),
            utn(WrittenUnit::I, UnitPosition::Fina),
            utn(WrittenUnit::Aa, UnitPosition::Isol),
        ]
    );

    // Chachlag `sara`: [S, A_MEDI, K, Aa] -> K closes stem at fina (R:fina), Aa is isol (Aa:isol)
    // 分写字尾 `sara`：K 在词尾收为 R:fina，后续 Aa 独立为 Aa:isol
    assert_eq!(
        menk_word_to_utn57("\u{E2FD}\u{E26C}\u{E325}\u{E26A}").unwrap(),
        vec![
            utn(WrittenUnit::S, UnitPosition::Init),
            utn(WrittenUnit::A, UnitPosition::Medi),
            utn(WrittenUnit::R, UnitPosition::Fina),
            utn(WrittenUnit::Aa, UnitPosition::Isol),
        ]
    );

    // Connected `ab` (`B2:fina`): U+E28C + U+E269 -> [A_INIT, O, Aa] -> [A:init, O:medi, Aa:fina]
    // 连写 `ab`（对应 `B2:fina`）：U+E28C + U+E269 -> [A:init, O:medi, Aa:fina]
    assert_eq!(
        menk_word_to_utn57("\u{E28C}\u{E269}").unwrap(),
        vec![
            utn(WrittenUnit::A, UnitPosition::Init),
            utn(WrittenUnit::O, UnitPosition::Medi),
            utn(WrittenUnit::Aa, UnitPosition::Fina),
        ]
    );

    // Connected `b` final variant (`B2:fina`): U+E266 + U+E2C4 -> [A_INIT, A_MEDI, O, Aa]
    // 连写词尾异体 `b`（U+E2C4，对应 `B2:fina`）：U+E266 + U+E2C4 -> [A:init, A:medi, O:medi, Aa:fina]
    assert_eq!(
        menk_word_to_utn57("\u{E266}\u{E2C4}").unwrap(),
        vec![
            utn(WrittenUnit::A, UnitPosition::Init),
            utn(WrittenUnit::A, UnitPosition::Medi),
            utn(WrittenUnit::O, UnitPosition::Medi),
            utn(WrittenUnit::Aa, UnitPosition::Fina),
        ]
    );

    // Chachlag after `B` (`u`/`w` fina): U+E294 + U+E26A -> [A_INIT, B, Aa] -> [A:init, U:fina, Aa:isol]
    // 大肚 `u`/`w`（`B`）后接分写字尾：U+E294 + U+E26A -> [A:init, U:fina, Aa:isol]
    assert_eq!(
        menk_word_to_utn57("\u{E294}\u{E26A}").unwrap(),
        vec![
            utn(WrittenUnit::A, UnitPosition::Init),
            utn(WrittenUnit::U, UnitPosition::Fina),
            utn(WrittenUnit::Aa, UnitPosition::Isol),
        ]
    );
}

#[test]
fn only_init_and_only_fina_subchain_splitting_works() {
    // 1. Tail-final `only_fina` (U+E2FF `S:fina`) followed without space by `B` (U+E2C1 + U+E26C + U+E2B5)
    // 1. 仅用于词尾的 `U+E2FF`（`S:fina`）无空格紧接 `B` 时，左链在 `S:fina` 闭合、右链从 `B:init` 起笔
    assert_eq!(
        menk_word_to_utn57("\u{E28C}\u{E2FA}\u{E291}\u{E2FF}\u{E2C1}\u{E26C}\u{E2B5}").unwrap(),
        vec![
            utn(WrittenUnit::A, UnitPosition::Init),
            utn(WrittenUnit::O, UnitPosition::Medi),
            utn(WrittenUnit::L, UnitPosition::Medi),
            utn(WrittenUnit::O, UnitPosition::Medi),
            utn(WrittenUnit::S, UnitPosition::Fina),
            utn(WrittenUnit::B, UnitPosition::Init),
            utn(WrittenUnit::A, UnitPosition::Medi),
            utn(WrittenUnit::A, UnitPosition::Fina),
        ]
    );

    // 2. Shared bow (U+E27B, not `only_fina`) followed without space by `only_init` (U+E271 `A_INIT`)
    // 2. 通用弓形 `U+E27B` 后无空格紧接仅用于词首的 `U+E271`（`A_INIT`），在 `U+E271` 前切分使 `U+E27B` 落为 `I:fina`
    assert_eq!(
        menk_word_to_utn57("\u{E271}\u{E27B}\u{E271}\u{E2B5}").unwrap(),
        vec![
            utn(WrittenUnit::A, UnitPosition::Init),
            utn(WrittenUnit::I, UnitPosition::Fina),
            utn(WrittenUnit::A, UnitPosition::Init),
            utn(WrittenUnit::A, UnitPosition::Fina),
        ]
    );

    // 3. Unspaced interchangeable bow/belly pairs join identically as `medi`
    // 3. 无空格连写时，混用的词尾/词中同形弓腹编码等价映射为相同的 `medi` 单元序列
    assert_eq!(
        menk_word_to_utn57("\u{E271}\u{E2ED}\u{E2AC}\u{E2B5}").unwrap(),
        menk_word_to_utn57("\u{E271}\u{E27B}\u{E291}\u{E2B5}").unwrap(),
    );
    assert_eq!(
        menk_word_to_utn57("\u{E271}\u{E325}\u{E289}\u{E2B5}").unwrap(),
        menk_word_to_utn57("\u{E271}\u{E337}\u{E2AC}\u{E2B5}").unwrap(),
    );
}

#[test]
fn optional_local_word_corpus_normalizes_cleanly() {
    // `word.txt` is gitignored; run full 65,148-word verification automatically when present locally.
    // `word.txt` 已加入 `.gitignore`；本地存在该词表时自动对全部词条执行完整转换与归一化验证。
    let word_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("word.txt");
    if !word_path.exists() {
        return;
    }

    let word_txt = fs::read_to_string(word_path).expect("read word.txt");
    let shaper = Shaper::new(Locale::Mng);

    let mut count = 0usize;
    for (idx, line) in word_txt.lines().enumerate() {
        let word = line.trim();
        if word.is_empty() {
            continue;
        }
        count += 1;

        let shape_str = get_shape(word);
        let parsed = parse_menk_shapes(&shape_str).unwrap();
        assert_eq!(parsed, menk_shapes(word).unwrap());

        let units = menk_word_to_utn57(word).unwrap();
        shaper
            .normalize_positioned_written_units(&units)
            .unwrap_or_else(|err| panic!("line {}: {err}", idx + 1));
    }

    assert!(count > 0);
}
