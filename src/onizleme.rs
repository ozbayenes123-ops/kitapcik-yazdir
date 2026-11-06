//! Görsel önizleme: her fiziksel A4 yaprağın ÖN/ARKA yüzü + katlanmış okuma sırası.
//! Çıktı: SVG dosyaları + tek `index.html` (tarayıcıda açılır, yazdırılabilir).
//! Ek bağımlılık yok; SVG metin olarak yazılır.

use anyhow::{Context, Result};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::dizilim::Yuz;

fn etiket(o: &Option<u32>) -> String {
    match o {
        Some(n) => n.to_string(),
        None => "BOŞ".into(),
    }
}

fn yuz_svg(
    tabaka_no: usize,
    fasikul_no: usize,
    on_mu: bool,
    sol: &Option<u32>,
    sag: &Option<u32>,
) -> String {
    // 842x595 A4 yatay; iki A5 kutu + orta kat çizgisi + duplex oku
    let baslik = if on_mu { "ÖN YÜZ" } else { "ARKA YÜZ" };
    let sol_t = etiket(sol);
    let sag_t = etiket(sag);
    let sol_renk = if sol.is_some() { "#1a73e8" } else { "#9aa0a6" };
    let sag_renk = if sag.is_some() { "#1a73e8" } else { "#9aa0a6" };
    // Kısa kenar duplex: arka yüz, ön yüze göre yatayda aynalanmış gibi düşünülür;
    // önizlemede sağ/sol yerleri BASKI dosyasındaki gibidir (çevirince üst üste gelir).
    let not = if on_mu {
        "Yazıcıya önce bu yüz (1. geçiş)"
    } else {
        "Aynı yaprağın arkası — kısa kenardan çevirince hizalanır (2. geçiş)"
    };
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 842 615" font-family="sans-serif">
<rect x="1" y="1" width="840" height="613" fill="white" stroke="#444"/>
<text x="21" y="30" font-size="20" font-weight="bold">Fasikül {fasikul_no} • {tabaka_no}. Tabaka — {baslik}</text>
<text x="21" y="52" font-size="13" fill="#555">{not}</text>
<rect x="20" y="65" width="391" height="510" fill="#f8f9fa" stroke="#333" stroke-width="2"/>
<rect x="431" y="65" width="391" height="510" fill="#f8f9fa" stroke="#333" stroke-width="2"/>
<line x1="421" y1="65" x2="421" y2="575" stroke="#c00" stroke-width="2" stroke-dasharray="8 6"/>
<text x="421" y="592" font-size="12" fill="#c00" text-anchor="middle">KAT ÇİZGİSİ (ortadan katla)</text>
<text x="215" y="120" font-size="15" fill="#555" text-anchor="middle">SOL</text>
<text x="215" y="340" font-size="96" font-weight="bold" fill="{sol_renk}" text-anchor="middle">{sol_t}</text>
<text x="215" y="380" font-size="14" fill="#555" text-anchor="middle">kaynak sayfa</text>
<text x="626" y="120" font-size="15" fill="#555" text-anchor="middle">SAĞ</text>
<text x="626" y="340" font-size="96" font-weight="bold" fill="{sag_renk}" text-anchor="middle">{sag_t}</text>
<text x="626" y="380" font-size="14" fill="#555" text-anchor="middle">kaynak sayfa</text>
<text x="21" y="600" font-size="12" fill="#555">A4 yatay • %100 ölçek • kısa kenar duplex</text>
</svg>"##
    )
}

/// Katlanmış okuma sırası şeridi: kitap okunurken 1. sayfadan son sayfaya hangi yaprak/yüz?
/// Basit liste: her mantıksal sayfa için (yaprak, yüz, taraf).
pub fn katlama_aciklamasi(
    fasikuller: &[Vec<Option<u32>>],
    yuzler: &[(usize, Vec<Yuz>)],
) -> String {
    let mut m = String::new();
    let _ = writeln!(
        m,
        "<h2>Katlayınca okuma sırası (tel dikiş)</h2><p>Katlanmış A5 kitabı önden arkaya \
        okurken sayfalar şu fiziksel konumlardan gelir. Boş sayfalar beyaz bırakılmıştır.</p>"
    );
    for (fi, fas) in fasikuller.iter().enumerate() {
        let _ = writeln!(m, "<h3>Fasikül {}</h3><ol>", fi + 1);
        // Mantıksal sıra = fas içindeki sıra (None'lar dahil numaralandırılır)
        for (i, o) in fas.iter().enumerate() {
            let (tabaka, yuz, taraf) = yuzler[fi].1.iter().enumerate().find_map(|(yi, (l, r))| {
                if l == o && o.is_some() || r == o && o.is_some() {
                    let t = yi / 2 + 1;
                    let y = if yi % 2 == 0 { "ÖN" } else { "ARKA" };
                    let s = if l == o { "sol" } else { "sağ" };
                    Some((t, y, s))
                } else {
                    None
                }
            }).unwrap_or((0, "—", "—"));
            let ad = match o {
                Some(n) => format!("<b>{n}</b>. sayfa"),
                None => "boş sayfa".into(),
            };
            let _ = writeln!(
                m,
                "<li>{ad}: {tabaka}. tabaka {yuz} yüz {taraf} (kitap içi sıra {})</li>",
                i + 1
            );
        }
        let _ = writeln!(m, "</ol>");
    }
    m
}

/// Önizlemeyi `dizin` altına yazar, `index.html` yolunu döner.
pub fn html_onizleme_yaz(
    sayfalar: &[u32],
    grup: usize,
    dizin: &Path,
) -> Result<PathBuf> {
    let fasikuller = crate::dizilim::gruplara_bol(sayfalar, grup);
    // fasikül -> yüzler
    let mut yuzler: Vec<(usize, Vec<Yuz>)> = Vec::new();
    for (fi, fas) in fasikuller.iter().enumerate() {
        yuzler.push((fi + 1, crate::dizilim::kitapcik_yuzleri(fas)));
    }

    std::fs::create_dir_all(dizin).with_context(|| format!("{} yazılamadı", dizin.display()))?;

    let mut kartlar = String::new();
    for (fi, ys) in &yuzler {
        for (yi, (sol, sag)) in ys.iter().enumerate() {
            let tabaka = yi / 2 + 1;
            let on_mu = yi % 2 == 0;
            let ad = format!("fas{fi}-tabaka{tabaka}-{}.svg", if on_mu { "on" } else { "arka" });
            let svg = yuz_svg(tabaka, *fi, on_mu, sol, sag);
            std::fs::write(dizin.join(&ad), svg)?;
            let baslik = format!(
                "Fasikül {fi} • {tabaka}. tabaka • {}",
                if on_mu { "ÖN [sol | sağ]" } else { "ARKA [sol | sağ]" }
            );
            let _ = writeln!(
                kartlar,
                "<figure><figcaption>{baslik}: <b>{}</b> | <b>{}</b></figcaption>\
                 <img src=\"{ad}\" alt=\"{baslik}\"/></figure>",
                etiket(sol),
                etiket(sag),
            );
        }
    }

    let katlama = katlama_aciklamasi(&fasikuller, &yuzler);
    let html = format!(
        r##"<!doctype html><html lang="tr"><meta charset="utf-8">
<title>Kitapçık önizleme — A4 ön/arka yüzler</title>
<style>
body{{font-family:sans-serif;max-width:1000px;margin:24px auto;padding:0 16px}}
figure{{border:1px solid #ccc;padding:8px;margin:16px 0}}
img{{width:100%;height:auto;background:#eee}}
figcaption{{margin-bottom:8px}}
.kutu{{background:#fef7e0;border:1px solid #eab308;padding:12px}}
code{{background:#f1f3f4;padding:1px 5px}}
</style>
<h1>A4 kitapçık önizleme (çift taraflı)</h1>
<div class="kutu"><b>Nasıl okunur:</b> Her <b>tabaka</b> 1 fiziksel A4 yapraktır.
<b>ÖN</b> ve <b>ARKA</b> aynı yaprağın iki yüzüdür (otomatik duplex: <code>kısa kenar</code>).
Yüzdeki <b>SOL/SAĞ</b> kutular A4'ün yarısındaki A5 sayfalardır; ortadaki kırmızı çizgiden katlanır.
Kağıtları üst üste koyup ortadan katlayınca alttaki <i>okuma sırası</i> listesi elde edilir.</div>
{kartlar}
{katlama}
<p><b>Baskı:</b> A4 • Yatay • %100 ölçek (Sığdır kapalı) • Kısa kenar duplex • 600 dpi yeterlidir.</p>
"##
    );
    let idx = dizin.join("index.html");
    std::fs::write(&idx, html)?;
    Ok(idx)
}
