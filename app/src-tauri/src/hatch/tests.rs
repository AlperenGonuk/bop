use super::*;
use rig::TAG_EYE;

fn errors(issues: &[Issue]) -> Vec<String> {
    issues.iter().filter(|i| i.level == Level::Error).map(validate_line).collect()
}

fn validate_line(i: &Issue) -> String {
    issue_line(i)
}

fn eye_x(f: &Frame) -> f64 {
    let (mut s, mut n) = (0.0, 0.0);
    for y in 0..f.h {
        for x in 0..f.w {
            if f.tag[y * f.w + x] == TAG_EYE {
                s += x as f64;
                n += 1.0;
            }
        }
    }
    s / n
}

#[test]
fn ornekler_hatasiz_cizilir() {
    for name in spec::EXAMPLES {
        let ch = spec::parse(spec::example(name).unwrap()).unwrap_or_else(|e| panic!("{name}: {e}"));
        let h = hatch(&ch);
        assert!(errors(&h.issues).is_empty(), "{name}: {:#?}", errors(&h.issues));
        assert_eq!((h.sheet.w, h.sheet.h), (1536, 2288));
    }
}

#[test]
fn atlas_satir_ve_kare_sayilari() {
    let ch = spec::parse(spec::example("pitir").unwrap()).unwrap();
    let h = hatch(&ch);
    let expected = [6, 8, 8, 4, 5, 8, 6, 6, 6, 8, 8];
    for (r, n) in expected.iter().enumerate() {
        assert_eq!(h.frames[r].len(), *n, "satır {r}");
        for c in 0..8 {
            let used = c < *n || (r, c) == NEUTRAL_CELL;
            let any = (0..atlas::CELL_H).any(|y| (0..atlas::CELL_W).any(|x| h.sheet.get(c * 192 + x, r * 208 + y).3 > 0));
            assert_eq!(any, used, "hücre ({r},{c})");
        }
    }
    // Saydam piksellerde RGB artığı yok.
    assert!(h.sheet.data.chunks_exact(4).all(|p| p[3] != 0 || p[..3] == [0, 0, 0]));
}

#[test]
fn bakis_yonu_koordinattan_olculur() {
    let ch = spec::parse(spec::example("critter").unwrap()).unwrap();
    let h = hatch(&ch);
    let n = eye_x(&h.neutral);
    // 90° (satır 9, kare 4) sağ, 270° (satır 10, kare 4) sol.
    assert!(eye_x(&h.frames[9][4]) > n + 1.0);
    assert!(eye_x(&h.frames[10][4]) < n - 1.0);
    // Koşu: sağa koşan sağa, sola koşan sola bakar.
    assert!(h.frames[1].iter().all(|f| eye_x(f) > n));
    assert!(h.frames[2].iter().all(|f| eye_x(f) < n));
}

#[test]
fn bakista_govde_de_doner() {
    // Pıtır denemesinin zayıflığı: bakışta yalnız yüz kayıyordu. Gövde (ve tepedeki filiz) de
    // hafifçe dönmeli, ama gözlerden az.
    let ch = spec::parse(spec::example("pitir").unwrap()).unwrap();
    let h = hatch(&ch);
    let right = &h.frames[9][4];
    assert_ne!(right.px, h.neutral.px);
    let body_px = |f: &Frame| f.tag.iter().filter(|&&t| t == rig::TAG_PART).count();
    // Filiz/yapraklar yön değiştirince başka piksellere düşer.
    let moved = (0..right.px.len()).filter(|&i| (right.tag[i] == rig::TAG_PART) != (h.neutral.tag[i] == rig::TAG_PART)).count();
    assert!(moved > 0 && body_px(right) > 0);
}

#[test]
fn ayni_spec_ayni_cikti() {
    let ch = spec::parse(spec::example("floaty").unwrap()).unwrap();
    assert_eq!(hatch(&ch).sheet.data, hatch(&ch).sheet.data);
}

#[test]
fn ince_izgara_calisir() {
    let text = spec::example("critter").unwrap().replace("\"archetype\"", "\"grid\": \"96x104\", \"archetype\"");
    let ch = spec::parse(&text).unwrap();
    assert_eq!(ch.grid.w, 96);
    let h = hatch(&ch);
    assert!(errors(&h.issues).is_empty(), "{:#?}", errors(&h.issues));
}

#[test]
fn hatali_spec_anlasilir_hata_verir() {
    let bad = [
        (r##"{"id":"x","displayName":"X","colour":"#fff"}"##, "unknown field"),
        (r##"{"id":"Bad Id","displayName":"X"}"##, "id 'Bad Id'"),
        (r##"{"id":"x","displayName":"X","archetype":"dragon"}"##, "archetype 'dragon'"),
        (r##"{"id":"x","displayName":"X","palette":{"body":"red"}}"##, "palette.body"),
        (r##"{"id":"x","displayName":"X","options":{"ears":"wings"}}"##, "options.ears"),
        (r##"{"id":"x","displayName":"X","parts":[{"name":"hat","shape":"ellipse","color":"nope","size":[2,2]}]}"##, "unknown palette color 'nope'"),
        (r##"{"id":"x","displayName":"X","parts":[{"name":"hat","shape":"triangle","points":[[0,0]]}]}"##, "3 'points'"),
        (r##"{"id":"x","displayName":"X","remove":["wing"]}"##, "no part named 'wing'"),
        (r##"{"id":"x","displayName":"X","overrides":[{"row":"idle","frame":9,"rows":["a"],"key":{"a":"body"}}]}"##, "frames 0-5"),
        (r##"{"id":"x","displayName":"X","parts":[{"name":"p","shape":"pixels","rows":["ab"],"key":{"a":"body"}}]}"##, "'b' is not in 'key'"),
        (r##"{"id":"x","displayName":"X","archetype":"custom"}"##, "role\": \"body"),
        (r##"{"id":"x","displayName":"X","parts":[{"name":"body","depth":0}]}"##, "'depth' must be a number > 0"),
        (r##"{"id":"x","displayName":"X","parts":[{"name":"t","shape":"capsule","points":[[0,0],[1,1]],"width":-1}]}"##, "'width' must be a number > 0"),
        (r##"{"id":"x","displayName":"X","parts":[{"name":"arm","width":0}]}"##, "'width' must be a number > 0"),
    ];
    for (text, want) in bad {
        let e = spec::parse(text).err().unwrap_or_else(|| panic!("geçmemeliydi: {text}"));
        assert!(e.contains(want), "{text}\n  hata: {e}\n  beklenen: {want}");
    }
}

#[test]
fn parca_ada_gore_degisir_ve_silinir() {
    let ch = spec::parse(
        r#"{"id":"x","displayName":"X","options":{"topper":"sprout"},
            "parts":[{"name":"body","size":[26,24]}],"remove":["leaf2"]}"#,
    )
    .unwrap();
    let body = ch.parts.iter().find(|p| p.name == "body").unwrap();
    assert_eq!(body.size, Some([26.0, 24.0]));
    assert!(!ch.parts.iter().any(|p| p.name == "leaf2"));
    assert!(ch.parts.iter().any(|p| p.name == "foot~mirror"));
}

#[test]
fn override_kareyi_degistirir() {
    let ch = spec::parse(
        r##"{"id":"x","displayName":"X","palette":{"star":"#FFE05A"},
            "overrides":[{"row":"waving","frame":1,"at":[2,2],"rows":["s.s",".s.","s.s"],"key":{"s":"star"}}]}"##,
    )
    .unwrap();
    let h = hatch(&ch);
    let f = &h.frames[3][1];
    assert_eq!(f.tag[2 * f.w + 2], TAG_OVERRIDE);
    assert_eq!(f.tag[2 * f.w + 3], rig::TAG_NONE);
}

#[test]
fn atlas_denetimi_bos_ve_yanlis_boyutu_yakalar() {
    let codes = |img: &Image| validate::check_atlas(img).iter().filter(|i| i.level == Level::Error).map(|i| i.code).collect::<Vec<_>>();
    assert_eq!(codes(&Image::new(1536, 1872)), vec!["size"]);
    let empty = codes(&Image::new(1536, 2288));
    for c in ["frame-empty", "neutral-empty", "look-same-as-neutral"] {
        assert!(empty.contains(&c), "{c} yok: {empty:?}");
    }
    // Kullanılmayan hücrede piksel ve saydam RGB artığı.
    let ch = spec::parse(spec::example("pitir").unwrap()).unwrap();
    let mut h = hatch(&ch);
    h.sheet.put(7 * 192 + 50, 50, color::Rgba(1, 2, 3, 255));
    h.sheet.put(7 * 192 + 60, 60, color::Rgba(9, 9, 9, 0));
    let c = codes(&h.sheet);
    assert!(c.contains(&"unused-not-empty") && c.contains(&"rgb-residue"), "{c:?}");
}

#[test]
fn durgun_satir_yakalanir() {
    let ch = spec::parse(spec::example("pitir").unwrap()).unwrap();
    let mut h = hatch(&ch);
    // waiting satırının bütün karelerini ilk kareyle aynı yap.
    let first: Vec<u8> = (0..208).flat_map(|y| {
        let i = ((6 * 208 + y) * 1536) * 4;
        h.sheet.data[i..i + 192 * 4].to_vec()
    }).collect();
    for c in 1..6 {
        for y in 0..208 {
            let i = ((6 * 208 + y) * 1536 + c * 192) * 4;
            h.sheet.data[i..i + 192 * 4].copy_from_slice(&first[y * 192 * 4..(y + 1) * 192 * 4]);
        }
    }
    assert!(validate::check_atlas(&h.sheet).iter().any(|i| i.code == "static-row" && i.row == Some(6)));
}

#[test]
fn png_gidip_gelir() {
    let ch = spec::parse(spec::example("pitir").unwrap()).unwrap();
    let h = hatch(&ch);
    let p = std::env::temp_dir().join(format!("bop-hatch-{}.png", std::process::id()));
    atlas::write_png(&p, &h.sheet).unwrap();
    let back = atlas::read_png(&p).unwrap();
    let _ = std::fs::remove_file(&p);
    assert_eq!((back.w, back.h), (1536, 2288));
    assert_eq!(back.data, h.sheet.data);
}

#[test]
fn pet_json_codex_uyumlu() {
    let ch = spec::parse(spec::example("pitir").unwrap()).unwrap();
    let info = crate::pet::parse_pet_json(&pet_json(&ch)).unwrap();
    assert_eq!(info.id, "pitir");
    assert_eq!(info.sprite_version_number, Some(2));
    assert_eq!(info.spritesheet_path, "spritesheet.png");
}

#[test]
fn gomulu_pitir_motorun_ciktisi() {
    // default-pet/spritesheet.png motorla üretilir; spec değişip PNG yenilenmezse yakalanır.
    let ch = spec::parse(spec::example("pitir").unwrap()).unwrap();
    let h = hatch(&ch);
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("default-pet").join("spritesheet.png");
    let embedded = atlas::read_png(&p).unwrap();
    assert_eq!((embedded.w, embedded.h), (h.sheet.w, h.sheet.h), "default-pet/spritesheet.png boyutu farklı");
    // Platformlar arası kayan nokta farkı birkaç kenar pikselini oynatabilir: bit-bit değil,
    // farklı piksel oranı ≤ %0.5 aranır. Spec değişip PNG yenilenmezse fark bunun çok üstündedir.
    let differ = embedded.data.chunks_exact(4).zip(h.sheet.data.chunks_exact(4)).filter(|(a, b)| a != b).count();
    let total = h.sheet.w * h.sheet.h;
    assert!(differ * 200 <= total, "default-pet/spritesheet.png eski ({differ}/{total} piksel farklı): bop hatch ile yeniden üretin");
}

#[test]
fn tasan_pet_kuculterek_sigdirilir() {
    let ch = spec::parse(
        r#"{"id":"big","displayName":"Big","archetype":"blob",
            "options":{"body":"wide","size":1.15,"arms":"always","topper":"antenna"}}"#,
    )
    .unwrap();
    assert!(hatch_once(&ch).issues.iter().any(|i| i.code == "clipped"));
    let h = hatch(&ch);
    assert!(errors(&h.issues).is_empty(), "{:#?}", errors(&h.issues));
    assert!(h.issues.iter().any(|i| i.code == "auto-fit"));
}

#[test]
fn kucultme_override_ile_birlikte_tasir() {
    let ch = spec::parse(
        r##"{"id":"x","displayName":"X","overrides":[{"row":"idle","frame":0,"at":[2,2],"rows":["sss","sss","sss"],"key":{"s":"#FFE05A"}}]}"##,
    )
    .unwrap();
    // Merkez (3.5, 3.5), zemin ortası (24, 47.5): yeni merkez (7.6, 12.3), sol üst (6, 11).
    assert_eq!(ch.shrunk(0.8).overrides[0].at, [6, 11]);
    assert_eq!(ch.shrunk(0.8).overrides[0].rows, ch.overrides[0].rows);
}

#[test]
fn yalniz_override_tasarsa_kucultulmez() {
    let ch = spec::parse(
        r##"{"id":"x","displayName":"X","overrides":[{"row":"waving","frame":2,"at":[0,0],"rows":["ss","ss"],"key":{"s":"#FFE05A"}}]}"##,
    )
    .unwrap();
    let h = hatch(&ch);
    assert!(!h.issues.iter().any(|i| i.code == "auto-fit"), "küçültülmemeli");
    let clip = h.issues.iter().find(|i| i.code == "clipped").expect("kırpılma hatası");
    assert_eq!((clip.row, clip.frame), (Some(3), Some(2)));
    assert!(clip.message.contains("override") && clip.message.contains("'at'"), "{}", clip.message);
}

#[test]
fn laptopsuz_calisma_satirinda_gesture_kol_yok() {
    let text = |arms: &str, laptop: bool| {
        format!(r#"{{"id":"x","displayName":"X","options":{{"arms":"{arms}"}},"props":{{"laptop":{laptop}}}}}"#)
    };
    let row = |t: String| hatch(&spec::parse(&t).unwrap()).frames[7].iter().map(|f| f.px.clone()).collect::<Vec<_>>();
    // Dizüstü yok: gesture kollar görünmez (kolsuz petle aynı), always kollar kıpırdar.
    assert_eq!(row(text("gesture", false)), row(text("none", false)));
    assert_ne!(row(text("always", false)), row(text("none", false)));
    // Dizüstü var: kolsuz pete el çizilmez, gesture pete çizilir.
    let hands = |t: String| {
        let ch = spec::parse(&t).unwrap();
        let body = ch.tone("body").unwrap();
        hatch(&ch).frames[7]
            .iter()
            .map(|f| (0..f.px.len()).filter(|&i| f.tag[i] == rig::TAG_PROP && [body.base, body.shade, body.hi].contains(&f.px[i])).count())
            .sum::<usize>()
    };
    assert_eq!(hands(text("none", true)), 0);
    assert!(hands(text("gesture", true)) > 0);
}
