//! A4'e 2xA5 kitapçık PDF üretimi (lopdf ile).
//!
//! Yaklaşım: kaynak belge aynı `Document` içinde kalır; her kaynak sayfa bir
//! Form XObject'e çevrilir, sonra A4 yatay yüzler (`sol|sağ`) bu formları
//! `Do` ile basar. Böylece font/görüntü kaynakları bozulmaz (her form kendi
//! `/Resources` sözlüğünü taşır, çakışma olmaz).

use anyhow::{bail, Context, Result};
use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream};
use std::collections::BTreeMap;
use std::path::Path;

use crate::dizilim;

// A4 nokta (pt): portrait 595.28 x 841.89
const A4_W: f64 = 595.28;
const A4_H: f64 = 841.89;
const LAND_W: f64 = A4_H; // 841.89 yatay genişlik
const LAND_H: f64 = A4_W; // 595.28 yatay yükseklik

fn mm_to_pt(mm: f64) -> f64 {
    mm * 2.834_646
}

fn obj_f64(o: &Object) -> f64 {
    if let Ok(i) = o.as_i64() {
        return i as f64;
    }
    if let Ok(f) = o.as_float() {
        return f as f64;
    }
    0.0
}

/// Sayfa sözlüğünden (CropBox > MediaBox) genişlik/yükseklik çıkar.
fn sayfa_olcusu(dict: &Dictionary) -> (f64, f64) {
    for key in [b"CropBox".as_slice(), b"MediaBox".as_slice()] {
        if let Ok(obj) = dict.get(key) {
            if let Ok(arr) = obj.as_array() {
                if arr.len() == 4 {
                    let x0 = obj_f64(&arr[0]);
                    let y0 = obj_f64(&arr[1]);
                    let x1 = obj_f64(&arr[2]);
                    let y1 = obj_f64(&arr[3]);
                    let w = (x1 - x0).abs();
                    let h = (y1 - y0).abs();
                    if w > 10.0 && h > 10.0 {
                        return (w, h);
                    }
                }
            }
        }
    }
    (A4_W, A4_H)
}

/// PDF ASCII85Decode filtresi (Adobe varyantı: `z` = 4 sıfır bayt, `~>` = son).
/// lopdf 0.32 bunu çözemez; reportlab gibi üreticiler ASCII85+Flate zinciri yazar.
fn ascii85_coz(girdi: &[u8]) -> Result<Vec<u8>> {
    let mut cikti = Vec::with_capacity(girdi.len());
    let mut grup: Vec<u8> = Vec::with_capacity(5);
    let mut i = 0;
    // 5 karakter -> 4 bayt (büyük uçlu). Kısa son grup 'u' ile doldurulup ilk n-1 bayt alınır.
    let flush = |grup: &[u8], cikti: &mut Vec<u8>| -> Result<()> {
        let n = grup.len();
        if n == 0 {
            return Ok(());
        }
        if n == 1 {
            bail!("ASCII85: tekil karakterlik grup");
        }
        let mut acc: u64 = 0;
        for &c in grup {
            acc = acc * 85 + (c - 33) as u64;
        }
        for _ in n..5 {
            acc = acc * 85 + 84; // 'u' dolgusu
        }
        if n == 5 && acc > u32::MAX as u64 {
            bail!("ASCII85: taşan grup");
        }
        let baytlar = (acc as u32).to_be_bytes();
        cikti.extend_from_slice(&baytlar[..n - 1]);
        Ok(())
    };
    while i < girdi.len() {
        let c = girdi[i];
        i += 1;
        match c {
            b' ' | b'\t' | b'\n' | b'\r' | 0x0c | 0 => continue,
            b'z' => {
                if !grup.is_empty() {
                    bail!("ASCII85: grup içinde 'z'");
                }
                cikti.extend_from_slice(&[0, 0, 0, 0]);
            }
            b'~' => break, // `~>` sonu; `>` varsa tüketilir, yoksa da sorun değil
            _ => {
                if !(33..=117).contains(&c) {
                    bail!("ASCII85: aralık dışı bayt {c}");
                }
                grup.push(c);
                if grup.len() == 5 {
                    flush(&grup, &mut cikti)?;
                    grup.clear();
                }
            }
        }
    }
    flush(&grup, &mut cikti)?;
    Ok(cikti)
}

fn zlib_coz(girdi: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;
    let mut d = ZlibDecoder::new(girdi);
    let mut out = Vec::with_capacity(girdi.len() * 2);
    d.read_to_end(&mut out)
        .context("FlateDecode çözülemedi")?;
    Ok(out)
}

fn filtre_adlari(stream: &Stream) -> Vec<String> {
    let mut adlar = Vec::new();
    match stream.dict.get(b"Filter") {
        Ok(Object::Name(n)) => adlar.push(String::from_utf8_lossy(n).into_owned()),
        Ok(Object::Array(arr)) => {
            for o in arr {
                if let Ok(n) = o.as_name() {
                    adlar.push(String::from_utf8_lossy(n).into_owned());
                }
            }
        }
        _ => {}
    }
    adlar
}

/// Bir sayfanın içerik akışlarını filtre zincirini ÇÖZEREK birleştirir.
/// lopdf'nin `get_page_content`ı ASCII85 gibi zincirlerde ham bayt döndürür;
/// bu fonksiyon ASCII85+Flate zincirlerini gerçekten çözer.
fn sayfa_icerik_coz(doc: &Document, sayfa_id: ObjectId) -> Result<Vec<u8>> {
    let mut birlesik = Vec::new();
    for sid in doc.get_page_contents(sayfa_id) {
        let stream = doc
            .get_object(sid)
            .with_context(|| format!("içerik akışı okunamadı: {sid:?}"))?
            .as_stream()
            .context("içerik akış değil")?;
        let mut veri = stream.content.clone();
        for f in filtre_adlari(stream) {
            let ad = f.trim_start_matches('/').to_string();
            veri = match ad.as_str() {
                "ASCII85Decode" | "A85" => ascii85_coz(&veri)?,
                "FlateDecode" | "Fl" => zlib_coz(&veri)?,
                "LZWDecode" | "LZW" => {
                    bail!("LZWDecode desteklenmiyor; PDF'i yeniden kaydedip deneyin")
                }
                "DCTDecode" | "DCT" | "JPXDecode" | "Crypt" => {
                    bail!("görüntü/şifreli filtre içerikte: {ad}")
                }
                "ASCIIHexDecode" | "AHx" => hex_coz(&veri)?,
                "RunLengthDecode" | "RL" => rle_coz(&veri)?,
                diger => bail!("bilinmeyen filtre: {diger}"),
            };
        }
        birlesik.extend_from_slice(&veri);
        birlesik.push(b'\n');
    }
    Ok(birlesik)
}

fn hex_coz(girdi: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut ust: Option<u8> = None;
    for &c in girdi {
        if c == b'>' {
            break;
        }
        let d = match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            b' ' | b'\t' | b'\n' | b'\r' | 0x0c | 0 => continue,
            _ => bail!("ASCIIHex: hatalı karakter"),
        };
        match ust {
            None => ust = Some(d << 4),
            Some(u) => {
                out.push(u | d);
                ust = None;
            }
        }
    }
    if let Some(u) = ust {
        out.push(u); // tek kalan yarım bayt 0 ile dolar
    }
    Ok(out)
}

fn rle_coz(girdi: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < girdi.len() {
        let n = girdi[i] as usize;
        i += 1;
        if n == 128 {
            break;
        } else if n < 128 {
            let adet = n + 1;
            if i + adet > girdi.len() {
                bail!("RunLength: kısa veri");
            }
            out.extend_from_slice(&girdi[i..i + adet]);
            i += adet;
        } else {
            let adet = 257 - n;
            if i >= girdi.len() {
                bail!("RunLength: kısa veri");
            }
            out.extend(std::iter::repeat(girdi[i]).take(adet));
            i += 1;
        }
    }
    Ok(out)
}

fn pages_id_bul(doc: &Document) -> Result<ObjectId> {
    let root_ref = doc
        .trailer
        .get(b"Root")
        .context("trailer Root yok")?
        .as_reference()
        .context("Root referans değil")?;
    let catalog = doc
        .get_object(root_ref)
        .context("katalog okunamadı")?
        .as_dict()
        .context("katalog sözlük değil")?;
    let pages_ref = catalog
        .get(b"Pages")
        .context("katalogda Pages yok")?
        .as_reference()
        .context("Pages referans değil")?;
    Ok(pages_ref)
}

/// Seçili her kaynak sayfayı Form XObject'e çevir.
/// Döner: sayfa_no -> (form_id, w, h)
fn formlari_kur(
    doc: &mut Document,
    num_to_id: &BTreeMap<u32, ObjectId>,
    kullanilan: &[u32],
) -> Result<BTreeMap<u32, (ObjectId, f64, f64)>> {
    let mut map = BTreeMap::new();
    for &n in kullanilan {
        if map.contains_key(&n) {
            continue;
        }
        let pid = *num_to_id
            .get(&n)
            .with_context(|| format!("sayfa {n} belgede yok"))?;
        let page_dict = doc
            .get_object(pid)
            .with_context(|| format!("sayfa {n} nesnesi okunamadı"))?
            .as_dict()
            .context("sayfa sözlük değil")?
            .clone();
        let resources = page_dict
            .get(b"Resources")
            .cloned()
            .unwrap_or_else(|_| Object::Dictionary(Dictionary::new()));
        let (w, h) = sayfa_olcusu(&page_dict);
        let icerik = sayfa_icerik_coz(doc, pid)
            .with_context(|| format!("sayfa {n} içeriği çözülemedi"))?;

        let mut form_dict = Dictionary::new();
        form_dict.set("Type", "XObject");
        form_dict.set("Subtype", "Form");
        form_dict.set(
            "BBox",
            vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Real(w as f32),
                Object::Real(h as f32),
            ],
        );
        form_dict.set("Resources", resources);

        let form_id = doc.new_object_id();
        doc.objects
            .insert(form_id, Object::Stream(Stream::new(form_dict, icerik)));
        map.insert(n, (form_id, w, h));
    }
    Ok(map)
}

/// Bir formu hücreye sığdıran `cm` + `Do` parçasını üret.
/// cell: (x0, y0, cw, ch). inset: her yandan bırakılan pay (pt).
fn do_parcasi(
    buf: &mut Vec<u8>,
    form_adi: &str,
    src_w: f64,
    src_h: f64,
    x0: f64,
    y0: f64,
    cw: f64,
    ch: f64,
    inset: f64,
) {
    let aw = (cw - 2.0 * inset).max(10.0);
    let ah = (ch - 2.0 * inset).max(10.0);
    let s = (aw / src_w).min(ah / src_h);
    let tx = x0 + inset + (aw - src_w * s) / 2.0;
    let ty = y0 + inset + (ah - src_h * s) / 2.0;
    let parca = format!("q\n{s:.4} 0 0 {s:.4} {tx:.2} {ty:.2} cm\n/{form_adi} Do\nQ\n");
    buf.extend_from_slice(parca.as_bytes());
}

pub struct BaskiAyar {
    pub kenar_boslugu_mm: f64,
    pub orta_pay_mm: f64,
    pub kat_cizgisi: bool,
}

/// Kitapçık modunda A4 yatay yüzler üretir ve `cikti` yoluna yazar.
/// `sayfalar`: seçili mantıksal sayfa numaraları (1-bazlı, orijinal belgeye göre).
/// `grup`: fasikül büyüklüğü (0 = tek fasikül).
pub fn kitapcik_pdf_uret(
    girdi: &Path,
    cikti: &Path,
    sayfalar: &[u32],
    grup: usize,
    ayar: &BaskiAyar,
) -> Result<KitapcikRapor> {
    let mut doc =
        Document::load(girdi).with_context(|| format!("girdi açılamadı: {}", girdi.display()))?;
    let pages = doc.get_pages();
    let toplam = pages.len() as u32;
    if toplam == 0 {
        bail!("girdi PDF'de sayfa yok");
    }
    // 1-bazlı numara -> ObjectId
    let mut num_to_id: BTreeMap<u32, ObjectId> = BTreeMap::new();
    for (n, id) in pages {
        num_to_id.insert(n, id);
    }
    for &n in sayfalar {
        if !num_to_id.contains_key(&n) {
            bail!("seçilen sayfa {n} belgede yok (1-{toplam})");
        }
    }

    let fasikuller = crate::dizilim::gruplara_bol(sayfalar, grup);
    // Tüm yüzleri sırayla topla
    let mut tum_yuzler: Vec<dizilim::Yuz> = Vec::new();
    for fas in &fasikuller {
        tum_yuzler.extend(dizilim::kitapcik_yuzleri(fas));
    }

    // Kullanılan benzersiz sayfalar
    let mut kullanilan: Vec<u32> = tum_yuzler
        .iter()
        .flat_map(|(a, b)| [a, b])
        .flatten()
        .copied()
        .collect();
    kullanilan.sort_unstable();
    kullanilan.dedup();

    let formlar = formlari_kur(&mut doc, &num_to_id, &kullanilan)?;
    let pages_id = pages_id_bul(&doc)?;

    let dis = mm_to_pt(ayar.kenar_boslugu_mm);
    let orta = mm_to_pt(ayar.orta_pay_mm);
    let cell_w = LAND_W / 2.0;
    let inset = dis;

    let mut yeni_sayfa_idler: Vec<ObjectId> = Vec::new();
    for (yi, (sol, sag)) in tum_yuzler.iter().enumerate() {
        let mut icerik: Vec<u8> = Vec::new();
        let mut xobj = Dictionary::new();

        if let Some(n) = sol {
            let (fid, w, h) = formlar[&n];
            xobj.set("FmL", Object::Reference(fid));
            // sol hücre: [0, W/2], orta payın yarısını sağdan düş
            do_parcasi(
                &mut icerik,
                "FmL",
                w,
                h,
                0.0,
                0.0,
                cell_w - orta / 2.0,
                LAND_H,
                inset,
            );
        }
        if let Some(n) = sag {
            let (fid, w, h) = formlar[&n];
            xobj.set("FmR", Object::Reference(fid));
            do_parcasi(
                &mut icerik,
                "FmR",
                w,
                h,
                cell_w + orta / 2.0,
                0.0,
                cell_w - orta / 2.0,
                LAND_H,
                inset,
            );
        }
        if ayar.kat_cizgisi {
            // Ortada ince kesikli kat çizgisi
            let cizgi = format!(
                "q\n0.4 w\n[3 3] 0 d\n0.6 G\n{:.2} 10 m\n{:.2} {:.2} l\nS\nQ\n",
                LAND_W / 2.0,
                LAND_W / 2.0,
                LAND_H - 10.0
            );
            icerik.extend_from_slice(cizgi.as_bytes());
        }

        let icerik_id = doc.new_object_id();
        doc.objects.insert(
            icerik_id,
            Object::Stream(Stream::new(Dictionary::new(), icerik)),
        );

        let mut res = Dictionary::new();
        res.set("XObject", Object::Dictionary(xobj));

        let mut page = Dictionary::new();
        page.set("Type", "Page");
        page.set("Parent", Object::Reference(pages_id));
        page.set(
            "MediaBox",
            vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Real(LAND_W as f32),
                Object::Real(LAND_H as f32),
            ],
        );
        page.set("Contents", Object::Reference(icerik_id));
        page.set("Resources", Object::Dictionary(res));

        let pid = doc.new_object_id();
        doc.objects.insert(pid, Object::Dictionary(page));
        yeni_sayfa_idler.push(pid);

        let _ = yi;
    }

    // Pages Kids'i yeni yüzlerle değiştir
    {
        let pages_obj = doc
            .get_object_mut(pages_id)
            .context("Pages düğümü yazılamadı")?;
        let pages_dict = pages_obj.as_dict_mut().context("Pages sözlük değil")?;
        let kids: Vec<Object> = yeni_sayfa_idler
            .iter()
            .map(|id| Object::Reference(*id))
            .collect();
        pages_dict.set("Kids", kids);
        pages_dict.set("Count", Object::Integer(yeni_sayfa_idler.len() as i64));
    }

    if let Some(parent) = cikti.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    doc.save(cikti)
        .with_context(|| format!("çıktı yazılamadı: {}", cikti.display()))?;

    // Eski sayfalar yetim kalır (dosya biraz şişer); bilinçli tercih:
    // kaynak font/görüntü nesneleri formlar tarafından hâlâ kullanılıyor.
    Ok(KitapcikRapor {
        secili: sayfalar.len(),
        fasikul: fasikuller.len(),
        yuz: tum_yuzler.len(),
        yaprak: tum_yuzler.len().div_ceil(2),
    })
}

/// Ardışık 2-up (kitapçık sırası yok): [1,2],[3,4]... A4 yatay.
pub fn ikiup_pdf_uret(
    girdi: &Path,
    cikti: &Path,
    sayfalar: &[u32],
    ayar: &BaskiAyar,
) -> Result<KitapcikRapor> {
    let mut doc =
        Document::load(girdi).with_context(|| format!("girdi açılamadı: {}", girdi.display()))?;
    let pages = doc.get_pages();
    let mut num_to_id: BTreeMap<u32, ObjectId> = BTreeMap::new();
    for (n, id) in pages {
        num_to_id.insert(n, id);
    }
    let mut kullanilan = sayfalar.to_vec();
    kullanilan.sort_unstable();
    kullanilan.dedup();
    let formlar = formlari_kur(&mut doc, &num_to_id, &kullanilan)?;
    let pages_id = pages_id_bul(&doc)?;

    let dis = mm_to_pt(ayar.kenar_boslugu_mm);
    let orta = mm_to_pt(ayar.orta_pay_mm);
    let cell_w = LAND_W / 2.0;

    let mut ciftler: Vec<(Option<u32>, Option<u32>)> = Vec::new();
    let mut it = sayfalar.iter().copied().map(Some);
    loop {
        let a = it.next().flatten();
        let b = it.next().flatten();
        if a.is_none() && b.is_none() {
            break;
        }
        // `it.next().flatten()` çift Option'ı düzleştirir; tek kalan sayfa (a=Some,b=None) olur.
        ciftler.push((a, b));
        if a.is_none() {
            break;
        }
    }

    let mut yeni: Vec<ObjectId> = Vec::new();
    for (sol, sag) in &ciftler {
        let mut icerik = Vec::new();
        let mut xobj = Dictionary::new();
        if let Some(n) = sol {
            let (fid, w, h) = formlar[&n];
            xobj.set("FmL", Object::Reference(fid));
            do_parcasi(&mut icerik, "FmL", w, h, 0.0, 0.0, cell_w - orta / 2.0, LAND_H, dis);
        }
        if let Some(n) = sag {
            let (fid, w, h) = formlar[&n];
            xobj.set("FmR", Object::Reference(fid));
            do_parcasi(
                &mut icerik,
                "FmR",
                w,
                h,
                cell_w + orta / 2.0,
                0.0,
                cell_w - orta / 2.0,
                LAND_H,
                dis,
            );
        }
        let icerik_id = doc.new_object_id();
        doc.objects.insert(
            icerik_id,
            Object::Stream(Stream::new(Dictionary::new(), icerik)),
        );
        let mut res = Dictionary::new();
        res.set("XObject", Object::Dictionary(xobj));
        let mut page = dictionary! {
            "Type" => "Page",
            "Parent" => Object::Reference(pages_id),
            "MediaBox" => vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Real(LAND_W as f32),
                Object::Real(LAND_H as f32),
            ],
            "Contents" => Object::Reference(icerik_id),
            "Resources" => Object::Dictionary(res)
        };
        // dictionary! makrosu Parent referansını doğru yazar; ek kontrol gerekmez.
        let _ = &mut page;
        let pid = doc.new_object_id();
        doc.objects.insert(pid, Object::Dictionary(page));
        yeni.push(pid);
    }
    {
        let o = doc.get_object_mut(pages_id).context("Pages yazılamadı")?;
        let d = o.as_dict_mut().context("Pages sözlük değil")?;
        d.set(
            "Kids",
            yeni.iter().map(|id| Object::Reference(*id)).collect::<Vec<_>>(),
        );
        d.set("Count", Object::Integer(yeni.len() as i64));
    }
    if let Some(parent) = cikti.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    doc.save(cikti)?;
    Ok(KitapcikRapor {
        secili: sayfalar.len(),
        fasikul: 1,
        yuz: yeni.len(),
        yaprak: yeni.len().div_ceil(2),
    })
}

pub struct KitapcikRapor {
    pub secili: usize,
    pub fasikul: usize,
    pub yuz: usize,
    pub yaprak: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ascii85_temel() {
        // "Man " <-> "9jqo^" (klasik vektör)
        assert_eq!(ascii85_coz(b"9jqo^").unwrap(), b"Man ");
        // 'z' = 4 sıfır bayt
        assert_eq!(ascii85_coz(b"z").unwrap(), vec![0, 0, 0, 0]);
        // Beyaz boşluklar yoksayılır
        assert_eq!(ascii85_coz(b"9jqo^\n 9jqo^").unwrap(), b"Man Man ");
    }
}
