use kitapcik_yazdir::{baski, dizilim, kilavuz, onizleme, sayfa, yazici};

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Mod {
    /// Tel dikiş kitapçık: otomatik imposition + A4 yataya 2-up (önerilen)
    Kitapcik,
    /// Ardışık 2-up (sıra yok, sadece yan yana)
    Ikiup,
    /// Düz: seçili sayfalar olduğu gibi (sıralı baskı / cilt için)
    Duz,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Duplex {
    /// Kısa kenardan çevir (A4 yatay kitapçık için DOĞRUSU budur)
    Kisa,
    /// Uzun kenardan çevir
    Uzun,
    /// Tek yüz (çift taraflı kapalı; kağıt elle çevrilir)
    TekYuz,
}

/// PDF'i A4'e 2xA5 kitapçık düzeninde baskıya hazırla.
///
/// Örnek:
///   kitapcik-yazdir --girdi kitap.pdf --cikti kitap-kitapcik.pdf --sayfalar 1-40
///   kitapcik-yazdir --girdi k.pdf --cikti k-b.pdf --sayfalar hepsi --grup 20 --onizleme
#[derive(Parser, Debug)]
#[command(name = "kitapcik-yazdir", version, about)]
struct Cli {
    /// Girdi PDF
    #[arg(short, long)]
    girdi: Option<PathBuf>,

    /// Çıktı PDF (baskıya hazır)
    #[arg(short, long)]
    cikti: Option<PathBuf>,

    /// Sayfa seçimi: "1-40,45,50-" veya "hepsi". ":tek"/":cift" soneki olur.
    #[arg(short, long, default_value = "hepsi")]
    sayfalar: String,

    /// Baskı modu
    #[arg(short, long, value_enum, default_value_t = Mod::Kitapcik)]
    mod_: Mod,

    /// Fasikül büyüklüğü (mantıksal sayfa). 0 = tek fasikül. 4'ün katı olmalı (örn 16,20,28,32).
    #[arg(short = 'G', long, default_value_t = 0)]
    grup: usize,

    /// Dış kenar boşluğu (mm)
    #[arg(long, default_value_t = 4.0)]
    kenar_boslugu_mm: f64,

    /// Orta cilt payı (mm) — iki A5 arasına bırakılan pay
    #[arg(long, default_value_t = 6.0)]
    orta_pay_mm: f64,

    /// Ortadaki kat çizgisini çizme (varsayılan: çiz)
    #[arg(long, default_value_t = true)]
    kat_cizgisi: bool,

    /// Sadece planı göster, PDF üretme
    #[arg(long, default_value_t = false)]
    onizleme: bool,

    /// Görsel önizleme dizini: her A4 yüz için SVG + index.html yazar.
    /// `--onizleme` ile birlikte verilirse varsayılan "onizleme" kullanılır.
    #[arg(long)]
    onizleme_dizin: Option<PathBuf>,

    /// Yazıcıları listele (Windows Get-Printer)
    #[arg(long, default_value_t = false)]
    yazici_listele: bool,

    /// Çıktıdan sonra yazıcıya göndermeyi dene
    #[arg(long, default_value_t = false)]
    yazdir: bool,

    /// Hedef yazıcı adı (boş = varsayılan)
    #[arg(long)]
    yazici: Option<String>,

    /// Kopya sayısı (bilgi + --yazdir ile Sumatra'ya aktarılır)
    #[arg(long, default_value_t = 1)]
    kopya: u32,

    /// Duplex çevirme yönü (kitapçık için KISA olmalı)
    #[arg(long, value_enum, default_value_t = Duplex::Kisa)]
    duplex: Duplex,

    /// Siyah-beyaz yazdır (varsayılan: renkli)
    #[arg(long, default_value_t = false)]
    siyah_beyaz: bool,

    /// Adım adım kılavuzu yazdır ve çık
    #[arg(long, default_value_t = false)]
    kilavuz: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.kilavuz {
        println!("{kilavuz}", kilavuz = kilavuz::METIN);
        return Ok(());
    }

    if cli.yazici_listele {
        yazici::yazicilari_listele()?;
        return Ok(());
    }

    let girdi = cli.girdi.clone().context(
        "--girdi gerekli (örn --girdi kitap.pdf). Yazıcıları görmek için --yazici-listele kullanın.",
    )?;

    if !girdi.exists() {
        bail!("girdi bulunamadı: {}", girdi.display());
    }
    if cli.grup % 4 != 0 && cli.grup != 0 {
        bail!("--grup 4'ün katı olmalı (örn 16, 20, 28, 32) ya da 0");
    }

    // Toplam sayfa sayısını öğren
    let doc = lopdf::Document::load(&girdi).context("girdi PDF açılamadı")?;
    let toplam = doc.get_pages().len() as u32;
    if toplam == 0 {
        bail!("girdi PDF'de sayfa yok");
    }
    println!("Girdi: {} ({} sayfa)", girdi.display(), toplam);

    let secili = sayfa::secimi_coz(&cli.sayfalar, toplam)?;
    println!(
        "Seçim '{}': {} sayfa ({}..{} arası ilk/son: {} / {})",
        cli.sayfalar,
        secili.len(),
        secili.iter().min().unwrap_or(&0),
        secili.iter().max().unwrap_or(&0),
        secili.first().unwrap_or(&0),
        secili.last().unwrap_or(&0)
    );

    match cli.mod_ {
        Mod::Kitapcik | Mod::Ikiup => {
            println!("{}", dizilim::plan_metni(&secili, cli.grup));
        }
        Mod::Duz => {
            println!("Düz mod: {} sayfa sırayla basılacak.", secili.len());
        }
    }

    if cli.onizleme {
        let dizin = cli
            .onizleme_dizin
            .clone()
            .unwrap_or_else(|| PathBuf::from("onizleme"));
        match cli.mod_ {
            Mod::Kitapcik => {
                let idx = onizleme::html_onizleme_yaz(&secili, cli.grup, &dizin)?;
                println!("Görsel önizleme: {}", idx.display());
                println!("Tarayıcıda açın; her SVG bir A4 yüzdür (ÖN/ARKA aynı yaprağın iki yüzüdür).");
            }
            _ => println!("(Görsel önizleme şimdilik sadece --mod kitapcik içindir.)"),
        }
        println!("--onizleme: PDF üretilmedi.");
        yazici::surucu_hatirlatmasi_full(&cli.duplex_name(), cli.kopya, !cli.siyah_beyaz);
        return Ok(());
    }

    let cikti = cli.cikti.clone().context("--cikti gerekli (örn --cikti kitap-kitapcik.pdf)")?;
    let ayar = baski::BaskiAyar {
        kenar_boslugu_mm: cli.kenar_boslugu_mm,
        orta_pay_mm: cli.orta_pay_mm,
        kat_cizgisi: cli.kat_cizgisi,
    };

    match cli.mod_ {
        Mod::Kitapcik => {
            let r = baski::kitapcik_pdf_uret(&girdi, &cikti, &secili, cli.grup, &ayar)?;
            println!(
                "Tamam: {} yazıldı — {} fasikül, {} A4 yüz (= {} fiziksel yaprak, ön+arka).",
                cikti.display(),
                r.fasikul,
                r.yuz,
                r.yaprak
            );
        }
        Mod::Ikiup => {
            let r = baski::ikiup_pdf_uret(&girdi, &cikti, &secili, &ayar)?;
            println!("Tamam: {} yazıldı — {} A4 yüz.", cikti.display(), r.yuz);
        }
        Mod::Duz => {
            duz_kopyala(&girdi, &cikti, &secili)?;
            println!("Tamam: {} yazıldı (düz sıralı).", cikti.display());
        }
    }

    if let Some(dizin) = &cli.onizleme_dizin {
        if matches!(cli.mod_, Mod::Kitapcik) {
            let idx = onizleme::html_onizleme_yaz(&secili, cli.grup, dizin)?;
            println!("Görsel önizleme: {}", idx.display());
        }
    }

    yazici::surucu_hatirlatmasi_full(&cli.duplex_name(), cli.kopya, !cli.siyah_beyaz);

    if cli.yazdir {
        let sec = yazici::Yazdirma {
            yazici: cli.yazici.as_deref(),
            kopya: cli.kopya,
            duplex: &cli.duplex_name(),
            renkli: !cli.siyah_beyaz,
        };
        yazici::pdf_yazdir_sec(&cikti, &sec)?;
    } else {
        println!("İpucu: çıktıyı Adobe/Sumatra ile açıp yazdırın; ölçek %100 (\"Gerçek boyut\"), kağıt A4, yön Yatay.");
        println!("      Doğrudan göndermek için aynı komuta --yazdir ekleyin.");
    }
    Ok(())
}

impl Cli {
    fn duplex_name(&self) -> String {
        match self.duplex {
            Duplex::Kisa => "kisa".into(),
            Duplex::Uzun => "uzun".into(),
            Duplex::TekYuz => "yok".into(),
        }
    }
}

/// Düz mod: seçili sayfaları sırayla içeren yeni PDF (aynı belge içinde Kids dizerek).
/// Not: orijinal boyutlar korunur, küçültme yapılmaz.
fn duz_kopyala(girdi: &std::path::Path, cikti: &std::path::Path, secili: &[u32]) -> Result<()> {
    use lopdf::{Document, Object};
    let mut doc = Document::load(girdi).context("girdi açılamadı")?;
    let pages = doc.get_pages();
    let mut num_to_id = std::collections::BTreeMap::new();
    for (n, id) in pages {
        num_to_id.insert(n, id);
    }
    let root = doc
        .trailer
        .get(b"Root")?
        .as_reference()
        .context("Root yok")?;
    let pages_id = doc
        .get_object(root)?
        .as_dict()?
        .get(b"Pages")?
        .as_reference()?;
    let kids: Vec<Object> = secili
        .iter()
        .map(|n| {
            num_to_id
                .get(n)
                .map(|id| Object::Reference(*id))
                .expect("sayfa var denetlendi")
        })
        .collect();
    {
        let o = doc.get_object_mut(pages_id)?;
        let d = o.as_dict_mut()?;
        d.set("Kids", kids);
        d.set("Count", Object::Integer(secili.len() as i64));
    }
    if let Some(p) = cikti.parent() {
        if !p.as_os_str().is_empty() {
            std::fs::create_dir_all(p)?;
        }
    }
    doc.save(cikti).context("çıktı yazılamadı")?;
    Ok(())
}
