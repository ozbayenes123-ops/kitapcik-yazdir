//! Kitapçık (saddle-stitch / tel dikiş) sayfa dizilimi.
//!
//! Mantık: A4 yatay tabakanın ön yüzüne 2, arka yüzüne 2 kaynak sayfa basılır,
//! katlanınca A5 kitap elde edilir. Her tabaka 4 mantıksal sayfa alır.
//!
//! Klasik sıra (N = 4'ün katına tamamlanmış sayfa adedi):
//!   tabaka s (0'dan): ön = [N-2s, 2s+1], arka = [2s+2, N-2s-1]
//! Örnek N=8: ön1=[8,1] arka1=[2,7] ön2=[6,3] arka2=[4,5]

/// Bir fasikülün (4'ün katına tamamlanmış) ön/arka yüz planı.
/// Her eleman bir A4 yüzüdür: (sol, sağ). `None` = boş sayfa.
pub type Yuz = (Option<u32>, Option<u32>);

/// Mantıksal sayfa listesini 4'ün katına tamamla (`None` = boş).
pub fn dorte_tamamla(sayfalar: &[u32]) -> Vec<Option<u32>> {
    let mut v: Vec<Option<u32>> = sayfalar.iter().copied().map(Some).collect();
    while v.len() % 4 != 0 {
        v.push(None);
    }
    v
}

/// Tek fasikül için yüz listesini üret. Sıra: ön1, arka1, ön2, arka2...
pub fn kitapcik_yuzleri(tamamlanmis: &[Option<u32>]) -> Vec<Yuz> {
    let n = tamamlanmis.len();
    assert!(n % 4 == 0, "fasikül 4'ün katı olmalı");
    let tabaka = n / 4;
    let mut yuzler = Vec::with_capacity(n / 2);
    for s in 0..tabaka {
        let on_sol = tamamlanmis[n - 2 * s - 1];
        let on_sag = tamamlanmis[2 * s];
        let arka_sol = tamamlanmis[2 * s + 1];
        let arka_sag = tamamlanmis[n - 2 * s - 2];
        yuzler.push((on_sol, on_sag));
        yuzler.push((arka_sol, arka_sag));
    }
    yuzler
}

/// Kalın kitaplar için fasiküllere (grup) böl.
/// `grup` = her fasiküldeki mantıksal sayfa adedi (4'ün katı olmalı, 0 = tek fasikül).
/// Her fasikül ayrı ayrı 4'e tamamlanır.
pub fn gruplara_bol(sayfalar: &[u32], grup: usize) -> Vec<Vec<Option<u32>>> {
    if grup == 0 {
        return vec![dorte_tamamla(sayfalar)];
    }
    assert!(grup % 4 == 0, "grup 4'ün katı olmalı");
    let mut out = Vec::new();
    for chunk in sayfalar.chunks(grup) {
        out.push(dorte_tamamla(chunk));
    }
    out
}

/// İnsan gözüyle kontrol için plan metni.
pub fn plan_metni(sayfalar: &[u32], grup: usize) -> String {
    use std::fmt::Write;
    let mut m = String::new();
    let fasikuller = gruplara_bol(sayfalar, grup);
    let _ = writeln!(
        m,
        "Seçili mantıksal sayfa: {} adet ({} fasikül)",
        sayfalar.len(),
        fasikuller.len()
    );
    for (fi, fas) in fasikuller.iter().enumerate() {
        let bos = fas.iter().filter(|x| x.is_none()).count();
        let _ = writeln!(
            m,
            "Fasikül {}: {} yüz ({} mantıksal + {} boş), {} fiziksel A4 yaprak:",
            fi + 1,
            fas.len() / 2,
            fas.len() - bos,
            bos,
            fas.len() / 4
        );
        for (yi, (sol, sag)) in kitapcik_yuzleri(fas).iter().enumerate() {
            let ad = |o: &Option<u32>| o.map(|n| n.to_string()).unwrap_or("BOŞ".into());
            let yuz_adi = if yi % 2 == 0 {
                format!("Tabaka {} ÖN ", yi / 2 + 1)
            } else {
                format!("Tabaka {} ARKA", yi / 2 + 1)
            };
            let _ = writeln!(m, "  {yuz_adi}: [sol={} | sağ={}]", ad(sol), ad(sag));
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_8_sayfa() {
        let v: Vec<u32> = (1..=8).collect();
        let t = dorte_tamamla(&v);
        let y = kitapcik_yuzleri(&t);
        assert_eq!(y[0], (Some(8), Some(1)));
        assert_eq!(y[1], (Some(2), Some(7)));
        assert_eq!(y[2], (Some(6), Some(3)));
        assert_eq!(y[3], (Some(4), Some(5)));
    }
    #[test]
    fn test_bos_tamamlama() {
        let v: Vec<u32> = (1..=6).collect();
        let t = dorte_tamamla(&v);
        assert_eq!(t.len(), 8);
        assert_eq!(t[6], None);
    }
}
