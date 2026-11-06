//! Windows yazıcı yardımcıları: listeleme + baskıya gönderme.
//! Rust, yazıcı sürücüsünün tüm ince ayarlarını tek başına yönetemez;
//! bu modül (1) sürücüde seçmeniz gerekenleri ekrana yazar,
//! (2) isterseniz çıktıyı yazıcıya göndermeyi dener (SumatraPDF > fallback).

use anyhow::{Context, Result};
use std::path::Path;

/// Windows'ta arka plan işlemlerinin siyah konsol penceresi açmasını engeller.
/// GUI kipinde (konsolsuz) çalışırken parlamaları önler; konsol kipinde etkisizdir.
pub fn gizli(mut cmd: std::process::Command) -> std::process::Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd
}

/// Kurulu yazıcı adlarını döndürür (açılır liste için).
/// Boş liste = yazıcı bulunamadı ya da sorgu çalışmadı.
pub fn yazici_adlari() -> Vec<String> {
    let out = gizli(std::process::Command::new("powershell"))
        .args([
            "-NoProfile",
            "-Command",
            "Get-Printer | Select-Object -ExpandProperty Name",
        ])
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

/// Kurulu yazıcıları tablo olarak listeler.
pub fn yazicilari_listele() -> Result<()> {
    println!("Kurulu yazıcılar (PowerShell Get-Printer):");
    let out = gizli(std::process::Command::new("powershell"))
        .args([
            "-NoProfile",
            "-Command",
            "Get-Printer | Select-Object Name,DriverName,PortName | Format-Table -AutoSize | Out-String -Width 200",
        ])
        .output()
        .context("Get-Printer çalıştırılamadı")?;
    println!("{}", String::from_utf8_lossy(&out.stdout));
    if !out.stderr.is_empty() {
        eprintln!("{}", String::from_utf8_lossy(&out.stderr));
    }
    Ok(())
}

/// Yazdırma seçenekleri (CLI + GUI ortak).
pub struct Yazdirma<'a> {
    /// Hedef yazıcı adı; None = varsayılan yazıcı.
    pub yazici: Option<&'a str>,
    pub kopya: u32,
    /// "kisa" | "uzun" | "yok" (tek yüz)
    pub duplex: &'a str,
    /// true = renkli, false = siyah-beyaz
    pub renkli: bool,
}

/// Sürücüde elle seçmeniz gereken ayarları yazdırır.
/// A4'e 2xA5 tel dikiş kitapçık için kritik olan: YATAY + KISA KENAR duplex + %100 ölçek.
pub fn surucu_hatirlatmasi(duplex: &str, kopya: u32) {
    surucu_hatirlatmasi_full(duplex, kopya, true);
}

pub fn surucu_hatirlatmasi_full(duplex: &str, kopya: u32, renkli: bool) {
    println!();
    println!("=== YAZICI SÜRÜCÜ AYARLARI (kitap görünümü için şart) ===");
    println!("  Kağıt boyutu : A4");
    println!("  Yön          : Yatay (Landscape) — çıktı zaten A4 yatay üretildi");
    match duplex {
        "yok" => println!("  Çift taraflı : KAPALI (tek yüz) — kağıtları elle çevirip 2. yüzleri basın"),
        "kisa" => println!("  Çift taraflı : AÇIK, 'KISA kenardan çevir' (kitapçık için DOĞRUSU budur)"),
        _ => println!("  Çift taraflı : AÇIK, 'UZUN kenardan çevir' (DİKKAT: yatay kitapçıkta genelde KISA gerekir)"),
    }
    println!("  Ölçek        : %100 / \"Gerçek boyut\" / \"Sığdır\" KAPALI (küçültme sayfayı kaydırır)");
    println!(
        "  Renk         : {}",
        if renkli { "Renkli" } else { "Siyah-beyaz" }
    );
    println!("  Kopya        : {kopya}");
    println!("  Kalite       : 600 dpi metin için yeterlidir");
    println!("  Ciltleme     : çıktıdan sonra ortadan katlayıp tel zımba (2 tel) + kenar düzeltme");
    println!("  Kalın kitap  : --grup 20/28/32 ile fasikül fasikül basıp üst üste koyun");
    println!("==========================================================");
    println!();
}

/// PDF'i yazıcıya göndermeyi dener. Önce SumatraPDF, yoksa varsayılan PDF eylemi.
pub fn pdf_yazdir(pdf: &Path, yazici: Option<&str>, kopya: u32, duplex: &str) -> Result<()> {
    pdf_yazdir_sec(
        pdf,
        &Yazdirma {
            yazici,
            kopya,
            duplex,
            renkli: true,
        },
    )
}

/// Kullanılabilir yazdırma altyapısı.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArkaUc {
    Sumatra(String),
    Adobe(String),
    Foxit(String),
    Yok,
}

/// Kurulmuş PDF yazdırma altyapısını bulur.
pub fn arka_uc_bul() -> ArkaUc {
    if let Some(exe) = sumatra_bul() {
        return ArkaUc::Sumatra(exe);
    }
    for aday in [
        r"C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe",
        r"C:\Program Files (x86)\Adobe\Acrobat Reader DC\Reader\AcroRd32.exe",
        r"C:\Program Files\Adobe\Acrobat Reader DC\Reader\AcroRd32.exe",
    ] {
        if std::path::Path::new(aday).exists() {
            return ArkaUc::Adobe(aday.into());
        }
    }
    for aday in [
        r"C:\Program Files\Foxit Software\Foxit PDF Reader\FoxitReader.exe",
        r"C:\Program Files (x86)\Foxit Software\Foxit PDF Reader\FoxitReader.exe",
    ] {
        if std::path::Path::new(aday).exists() {
            return ArkaUc::Foxit(aday.into());
        }
    }
    ArkaUc::Yok
}

pub fn arka_uc_adi() -> &'static str {
    match arka_uc_bul() {
        ArkaUc::Sumatra(_) => "SumatraPDF (tam otomatik)",
        ArkaUc::Adobe(_) => "Adobe (yazıcı+kopya aktarılır)",
        ArkaUc::Foxit(_) => "Foxit (yazıcı aktarılır)",
        ArkaUc::Yok => "YOK — önce SumatraPDF kurun",
    }
}

/// SumatraPDF'yi winget ile sessizce kurar (ücretsiz, ~10 MB).
/// Birkaç dakika sürebilir; çağıran arayüz donmaması için ayrı iş parçacığında çalıştırmalıdır.
pub fn sumatra_kur() -> Result<()> {
    let st = gizli(std::process::Command::new("winget"))
        .args([
            "install",
            "--id",
            "SumatraPDF.SumatraPDF",
            "--accept-source-agreements",
            "--accept-package-agreements",
            "--silent",
        ])
        .status()
        .context("winget çalıştırılamadı")?;
    if !st.success() {
        anyhow::bail!("winget kurulumu başarısız (kod {st})");
    }
    // PATH'e düşmesi zaman alabilir; dosyayı doğrula
    if matches!(arka_uc_bul(), ArkaUc::Sumatra(_)) {
        Ok(())
    } else {
        anyhow::bail!("kurulum bitti ama SumatraPDF bulunamadı; bilgisayarı yeniden başlatıp tekrar deneyin")
    }
}

pub fn pdf_yazdir_sec(pdf: &Path, s: &Yazdirma) -> Result<()> {
    let yazici = s.yazici;
    let kopya = s.kopya;
    let duplex = s.duplex;
    let pdf_s = pdf
        .to_str()
        .context("çıktı yolu UTF-8 değil, yazdırma atlandı")?;
    match arka_uc_bul() {
        ArkaUc::Sumatra(exe) => {
            println!("SumatraPDF bulundu: {exe} — yazıcıya gönderiliyor...");
            let mut args: Vec<String> = Vec::new();
            match yazici {
                Some(ad) => {
                    args.push("-print-to".into());
                    args.push(ad.into());
                }
                None => args.push("-print-to-default".into()),
            }
            // Sumatra print-settings: duplex + renk + kopya + ölçek yok
            let duplex_opt = match duplex {
                "kisa" => "duplexshort",
                "uzun" => "duplexlong",
                _ => "simplex",
            };
            let renk_opt = if s.renkli { "color" } else { "monochrome" };
            args.push("-print-settings".into());
            args.push(format!("noscale,{duplex_opt},{renk_opt},copies={kopya}"));
            args.push(pdf_s.into());
            println!("  {exe} {}", args.join(" "));
            let st = std::process::Command::new(&exe)
                .args(&args)
                .status()
                .context("SumatraPDF çalıştırılamadı")?;
            if !st.success() {
                anyhow::bail!("SumatraPDF yazdırma başarısız (kod {st})");
            }
            println!("Yazdırma kuyruğa verildi.");
            Ok(())
        }
        ArkaUc::Adobe(exe) => {
            // AcroRd32/Acrobat: /t "pdf" "yazıcı" — kopya/duplex sürücüden alınır
            println!("Adobe bulundu — yazdırılıyor (kopya/duplex'i açılan pencereden doğrulayın)...");
            let mut cmd = std::process::Command::new(&exe);
            cmd.arg("/t").arg(pdf_s);
            if let Some(ad) = yazici {
                cmd.arg(ad);
            }
            let st = cmd.status().context("Adobe çalıştırılamadı")?;
            if !st.success() {
                anyhow::bail!("Adobe yazdırma başarısız (kod {st})");
            }
            if kopya > 1 {
                println!("Not: Adobe'ye kopya sayısı aktarılamaz; kopyayı sürücü penceresinden seçin.");
            }
            Ok(())
        }
        ArkaUc::Foxit(exe) => {
            println!("Foxit bulundu — yazdırılıyor...");
            let mut cmd = std::process::Command::new(&exe);
            cmd.arg("/t").arg(pdf_s);
            if let Some(ad) = yazici {
                cmd.arg(ad);
            }
            let st = cmd.status().context("Foxit çalıştırılamadı")?;
            if !st.success() {
                anyhow::bail!("Foxit yazdırma başarısız (kod {st})");
            }
            Ok(())
        }
        ArkaUc::Yok => {
            // Son çare: Windows Print eylemi (Edge varsayılıyken genelde çalışmaz).
            println!("Uygun yazdırma programı yok; Windows yazdırma eylemi deneniyor...");
            let deneme = std::process::Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!("Start-Process -FilePath '{pdf_s}' -Verb Print"),
                ])
                .status();
            match deneme {
                Ok(st) if st.success() => {
                    println!("Yazdırma penceresi tetiklendi; sürücüden A4 Yatay + Kısa kenar duplex seçin.");
                    Ok(())
                }
                _ => anyhow::bail!(
                    "Yazdırma altyapısı yok: varsayılan PDF okuyucunuz (Edge) yazdırmayı desteklemiyor. \
                    Çözüm: programdaki 'SumatraPDF Kur' düğmesine basın ya da Adobe Reader kurun; \
                    ya da çıktıyı elle açıp Ctrl+P ile yazdırın."
                ),
            }
        }
    }
}

fn sumatra_bul() -> Option<String> {
    // PATH'te mi?
    if std::process::Command::new("SumatraPDF.exe")
        .arg("-h")
        .output()
        .map(|o| o.status.success() || !o.stdout.is_empty() || !o.stderr.is_empty())
        .unwrap_or(false)
    {
        return Some("SumatraPDF.exe".into());
    }
    let adaylar = [
        r"C:\Program Files\SumatraPDF\SumatraPDF.exe",
        r"C:\Program Files (x86)\SumatraPDF\SumatraPDF.exe",
    ];
    for a in adaylar {
        if std::path::Path::new(a).exists() {
            return Some(a.into());
        }
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let p = format!("{local}\\SumatraPDF\\SumatraPDF.exe");
        if std::path::Path::new(&p).exists() {
            return Some(p);
        }
    }
    None
}
