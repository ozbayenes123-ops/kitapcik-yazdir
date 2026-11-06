//! Sayfa seçim sözdizimi: "1-40,45,50-" gibi.
//! - "hepsi" veya boş = tüm sayfalar
//! - "1-10" aralık, "7" tek sayfa, "50-" = 50'den sona kadar, "-10" = baştan 10'a kadar
//! - ":tek" / ":cift" soneki (örn "1-40:tek") sadece tek/çift sayfaları alır.

use anyhow::{bail, Context, Result};

/// Kullanıcının verdiği seçim dizesini, toplam sayfa sayısına göre 1-bazlı sayfa listesine çevirir.
pub fn secimi_coz(secim: &str, toplam: u32) -> Result<Vec<u32>> {
    let s = secim.trim().to_lowercase();
    if s.is_empty() || s == "hepsi" || s == "all" || s == "*" {
        return Ok((1..=toplam).collect());
    }

    // tek/çift filtresi
    let (govde, filtre) = if let Some(g) = s.strip_suffix(":tek") {
        (g.to_string(), Some(true))
    } else if let Some(g) = s.strip_suffix(":çift") {
        (g.to_string(), Some(false))
    } else if let Some(g) = s.strip_suffix(":cift") {
        (g.to_string(), Some(false))
    } else {
        (s.clone(), None)
    };

    let mut sayfalar: Vec<u32> = Vec::new();
    for parca in govde.split(',') {
        let p = parca.trim();
        if p.is_empty() {
            continue;
        }
        if p.contains('-') {
            let mut it = p.splitn(2, '-');
            let bas_s = it.next().unwrap_or("").trim();
            let bit_s = it.next().unwrap_or("").trim();
            let bas: u32 = if bas_s.is_empty() {
                1
            } else {
                bas_s
                    .parse()
                    .with_context(|| format!("sayfa aralığı başı hatalı: '{p}'"))?
            };
            let bit: u32 = if bit_s.is_empty() {
                toplam
            } else {
                bit_s
                    .parse()
                    .with_context(|| format!("sayfa aralığı sonu hatalı: '{p}'"))?
            };
            if bas < 1 || bit > toplam || bas > bit {
                bail!("aralık kitap dışında: '{p}' (kitap 1-{toplam})");
            }
            sayfalar.extend(bas..=bit);
        } else {
            let n: u32 = p
                .parse()
                .with_context(|| format!("sayfa numarası hatalı: '{p}'"))?;
            if n < 1 || n > toplam {
                bail!("sayfa kitap dışında: {n} (kitap 1-{toplam})");
            }
            sayfalar.push(n);
        }
    }

    if let Some(tek_mi) = filtre {
        sayfalar.retain(|n| (*n % 2 == 1) == tek_mi);
    }

    // Sırayı koru ama tekrarları at (kullanıcı bilerek tekrar isterse --mod duz ile hallederiz;
    // kitapçıkta tekrar genelde istenmez, yine de ilk geçişi koruyoruz)
    let mut goruldu = std::collections::HashSet::new();
    sayfalar.retain(|n| goruldu.insert(*n));

    if sayfalar.is_empty() {
        bail!("seçim boş kaldı (toplam {toplam} sayfa)");
    }
    Ok(sayfalar)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_secim() {
        assert_eq!(secimi_coz("hepsi", 5).unwrap(), vec![1, 2, 3, 4, 5]);
        assert_eq!(secimi_coz("1-3,5", 5).unwrap(), vec![1, 2, 3, 5]);
        assert_eq!(secimi_coz("2-", 4).unwrap(), vec![2, 3, 4]);
        assert_eq!(secimi_coz("1-6:tek", 6).unwrap(), vec![1, 3, 5]);
    }
}
