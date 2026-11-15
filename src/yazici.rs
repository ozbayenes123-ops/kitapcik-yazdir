//! Yazıcı yardımcıları: yazıcı listeleme + PDF'i kuyruğa gönderme.
//!
//! - **Windows:** PowerShell `Get-Printer` + SumatraPDF / Adobe / Foxit.
//! - **Linux:** CUPS (`lpstat` ile listeleme, `lp` ile gönderme).
//!
//! Rust, yazıcı sürücüsünün tüm ince ayarlarını tek başına yönetemez;
//! bu modül (1) sürücüde seçmeniz gerekenleri ekrana yazar,
//! (2) isterseniz çıktıyı yazıcıya göndermeyi dener.

use anyhow::{Context, Result};
use std::path::Path;

/// Windows'ta arka plan işlemlerinin siyah konsol penceresi açmasını engeller.
/// Linux'ta etkisizdir; API her iki platformda da aynıdır.
#[allow(unused_mut)]
pub fn gizli(mut cmd: std::process::Command) -> std::process::Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd
}

/// PATH içinde çalıştırılabilir arar (ayırıcı platforma göre doğru işlenir).
#[cfg(not(windows))]
fn yolda_bul(ad: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dizin in std::env::split_paths(&path) {
        let aday = dizin.join(ad);
        if aday.is_file() {
            return Some(aday.to_string_lossy().into_owned());
        }
    }
    None
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
    if cfg!(windows) {
        println!("  Not          : SumatraPDF'e gönderirken ölçek, duplex ve kopya aktarılır");
    } else {
        println!("  Not          : CUPS'a gönderirken kağıt/yön/duplex seçenekleri otomatik verilir");
    }
    println!("==========================================================");
    println!();
}

/// Kullanılabilir yazdırma altyapısı.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArkaUc {
    #[cfg(windows)]
    Sumatra(String),
    #[cfg(windows)]
    Adobe(String),
    #[cfg(windows)]
    Foxit(String),
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    Lp(String),
    Yok,
}

/// Yazdırma altyapısının adı (arayüzde ve günlükte gösterilir).
pub fn arka_uc_adi() -> &'static str {
    match arka_uc_bul() {
        #[cfg(windows)]
        ArkaUc::Sumatra(_) => "SumatraPDF (tam otomatik)",
        #[cfg(windows)]
        ArkaUc::Adobe(_) => "Adobe (yazıcı+kopya aktarılır)",
        #[cfg(windows)]
        ArkaUc::Foxit(_) => "Foxit (yazıcı aktarılır)",
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        ArkaUc::Lp(_) => "CUPS (lp) — doğrudan kuyruğa gönderir",
        ArkaUc::Yok => {
            if cfg!(windows) {
                "YOK — önce SumatraPDF kurun"
            } else {
                "YOK — CUPS araçları eksik (cups, cups-client)"
            }
        }
    }
}

/// PDF'i yazıcıya göndermeyi dener.
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

/// Kurulum düğmesinin etiketi.
pub fn altyapi_kur_etiketi() -> &'static str {
    if cfg!(windows) {
        "SumatraPDF Kur (önerilen, ücretsiz)"
    } else {
        "CUPS kurulum komutunu göster"
    }
}

/// Platforma uygun yazdırma ipucu.
pub fn yazdirma_ipucu() -> &'static str {
    if cfg!(windows) {
        "İpucu: çıktıyı Adobe/Sumatra ile açıp yazdırın; ölçek %100 (\"Gerçek boyut\"), kağıt A4, yön Yatay."
    } else {
        "İpucu: çıktıyı belge görüntüleyiciyle açıp yazdırın; ölçek %100 (\"Gerçek boyut\"), kağıt A4, yön Yatay."
    }
}

/// Altyapı yokken arayüzde gösterilen sebep.
pub fn altyapi_sebep() -> &'static str {
    if cfg!(windows) {
        "Sebep: varsayılan PDF okuyucunuz (Edge) doğrudan yazdırmaz."
    } else {
        "Sebep: CUPS araçları kurulu değil (lp / lpstat)."
    }
}

/// Kurulum sürerken gösterilen satır.
pub fn altyapi_kurulum_mesaji() -> &'static str {
    if cfg!(windows) {
        "SumatraPDF kuruluyor… pencereyi kapatmayın."
    } else {
        "CUPS kurulum komutu hazırlanıyor…"
    }
}

// ---------------------------------------------------------------------------
// Windows
// ---------------------------------------------------------------------------

#[cfg(windows)]
mod win {
    use super::*;

    /// Kurulu yazıcı adlarını döndürür (açılır liste için).
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

    pub fn arka_uc_bul() -> ArkaUc {
        if let Some(exe) = sumatra_bul() {
            return ArkaUc::Sumatra(exe);
        }
        for aday in [
            r"C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe",
            r"C:\Program Files (x86)\Adobe\Acrobat Reader DC\Reader\AcroRd32.exe",
            r"C:\Program Files\Adobe\Acrobat Reader DC\Reader\AcroRd32.exe",
        ] {
            if Path::new(aday).exists() {
                return ArkaUc::Adobe(aday.into());
            }
        }
        for aday in [
            r"C:\Program Files\Foxit Software\Foxit PDF Reader\FoxitReader.exe",
            r"C:\Program Files (x86)\Foxit Software\Foxit PDF Reader\FoxitReader.exe",
        ] {
            if Path::new(aday).exists() {
                return ArkaUc::Foxit(aday.into());
            }
        }
        ArkaUc::Yok
    }

    /// SumatraPDF'yi winget ile sessizce kurar (ücretsiz, ~10 MB).
    /// Birkaç dakika sürebilir; çağıran arayüz donmaması için ayrı iş parçacığında çalıştırmalıdır.
    pub fn altyapi_kur() -> Result<()> {
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
            anyhow::bail!(
                "kurulum bitti ama SumatraPDF bulunamadı; bilgisayarı yeniden başlatıp tekrar deneyin"
            )
        }
    }

    pub fn yazdir(pdf: &Path, s: &Yazdirma) -> Result<()> {
        let yazici = s.yazici;
        let kopya = s.kopya;
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
                let duplex_opt = match s.duplex {
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
            if Path::new(a).exists() {
                return Some(a.into());
            }
        }
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let p = format!("{local}\\SumatraPDF\\SumatraPDF.exe");
            if Path::new(&p).exists() {
                return Some(p);
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Linux / macOS (CUPS)
// ---------------------------------------------------------------------------

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod unix {
    use super::*;

    /// Kurulu yazıcı adlarını döndürür (CUPS).
    /// Boş liste = yazıcı yok, CUPS kapalı ya da araçlar kurulu değil.
    pub fn yazici_adlari() -> Vec<String> {
        use std::process::Command;
        let mut adlar: Vec<String> = Vec::new();
        // `lpstat -e` yalnızca adları verir (CUPS 2.x+).
        if let Ok(o) = Command::new("lpstat").arg("-e").output() {
            if o.status.success() {
                for satir in String::from_utf8_lossy(&o.stdout).lines() {
                    let s = satir.trim();
                    if !s.is_empty() && !s.contains(char::is_whitespace) {
                        adlar.push(s.to_string());
                    }
                }
            }
        }
        // Eski sürümler için `lpstat -a`: "<ad> accepting requests since ..."
        if adlar.is_empty() {
            if let Ok(o) = Command::new("lpstat").arg("-a").output() {
                if o.status.success() {
                    for satir in String::from_utf8_lossy(&o.stdout).lines() {
                        if let Some(ad) = satir.split_whitespace().next() {
                            if !ad.is_empty() && !ad.ends_with(':') {
                                adlar.push(ad.to_string());
                            }
                        }
                    }
                }
            }
        }
        adlar.sort();
        adlar.dedup();
        adlar
    }

    /// Kurulu yazıcıları ve varsayılanı listeler.
    pub fn yazicilari_listele() -> Result<()> {
        if yolda_bul("lpstat").is_none() {
            anyhow::bail!(
                "CUPS araçları kurulu değil. Arch: sudo pacman -S cups cups-filters • \
                 Debian/Ubuntu: sudo apt install cups cups-client • \
                 Fedora: sudo dnf install cups"
            );
        }
        println!("Kurulu yazıcılar (CUPS lpstat):");
        let out = std::process::Command::new("lpstat")
            .args(["-p", "-d"])
            .output()
            .context("lpstat çalıştırılamadı")?;
        println!("{}", String::from_utf8_lossy(&out.stdout));
        let hata = String::from_utf8_lossy(&out.stderr);
        if !hata.trim().is_empty() {
            eprintln!("{hata}");
        }
        if !out.status.success() && hata.trim().is_empty() {
            println!("(yazıcı yok — CUPS servisi kapalı olabilir: systemctl status cups)");
        }
        Ok(())
    }

    pub fn arka_uc_bul() -> ArkaUc {
        match yolda_bul("lp") {
            Some(lp) => ArkaUc::Lp(lp),
            None => ArkaUc::Yok,
        }
    }

    /// Linux'ta kurulum yönetici izni ister; komutu gösterir, kendisi kurmaz.
    pub fn altyapi_kur() -> Result<()> {
        anyhow::bail!(
            "CUPS kurulumu yönetici izni ister. Terminalde şunu çalıştırın:\n  \
             Arch/CachyOS : sudo pacman -S cups cups-filters && sudo systemctl enable --now cups\n  \
             Debian/Ubuntu: sudo apt install cups cups-client\n  \
             Fedora       : sudo dnf install cups\n\
             Sonra yazıcıyı ekleyin: http://localhost:631"
        )
    }

    pub fn yazdir(pdf: &Path, s: &Yazdirma) -> Result<()> {
        let lp = match arka_uc_bul() {
            ArkaUc::Lp(lp) => lp,
            ArkaUc::Yok => anyhow::bail!(
                "CUPS komutu 'lp' bulunamadı. Arch: sudo pacman -S cups cups-filters • \
                 Debian/Ubuntu: sudo apt install cups-client"
            ),
        };
        let pdf_s = pdf
            .to_str()
            .context("çıktı yolu UTF-8 değil, yazdırma atlandı")?;
        let mut cmd = std::process::Command::new(&lp);
        cmd.arg("-n").arg(s.kopya.to_string());
        if let Some(ad) = s.yazici {
            cmd.arg("-d").arg(ad);
        }
        cmd.arg("-o").arg("media=A4");
        cmd.arg("-o").arg("print-scaling=none");
        cmd.arg("-o").arg(match s.duplex {
            "kisa" => "sides=two-sided-short-edge",
            "uzun" => "sides=two-sided-long-edge",
            _ => "sides=one-sided",
        });
        if !s.renkli {
            cmd.arg("-o").arg("ColorModel=Gray");
            cmd.arg("-o").arg("ColorMode=Monochrome");
        }
        cmd.arg(pdf_s);
        println!(
            "CUPS'a gönderiliyor: lp {} (kopya {}, duplex {})",
            s.yazici.unwrap_or("(varsayılan)"),
            s.kopya,
            s.duplex
        );
        let out = cmd.output().context("lp çalıştırılamadı")?;
        let std_out = String::from_utf8_lossy(&out.stdout);
        let std_err = String::from_utf8_lossy(&out.stderr);
        if !std_out.trim().is_empty() {
            println!("{}", std_out.trim());
        }
        if !out.status.success() {
            anyhow::bail!("lp başarısız: {}", std_err.trim());
        }
        if !std_err.trim().is_empty() {
            // CUPS bilinmeyen seçenekleri uyarı olarak yazar; iş yine de kuyruğa girer.
            println!("Uyarı: {}", std_err.trim());
        }
        println!("Yazdırma kuyruğa verildi.");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Diğer platformlar
// ---------------------------------------------------------------------------

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
mod diger {
    use super::*;

    pub fn yazici_adlari() -> Vec<String> {
        Vec::new()
    }

    pub fn yazicilari_listele() -> Result<()> {
        anyhow::bail!("Bu platformda yazıcı listeleme desteklenmiyor")
    }

    pub fn arka_uc_bul() -> ArkaUc {
        ArkaUc::Yok
    }

    pub fn altyapi_kur() -> Result<()> {
        anyhow::bail!("Bu platformda otomatik kurulum yok")
    }

    pub fn yazdir(_pdf: &Path, _s: &Yazdirma) -> Result<()> {
        anyhow::bail!("Bu platformda yazdırma desteklenmiyor")
    }
}

// ---------------------------------------------------------------------------
// Ortak arayüz
// ---------------------------------------------------------------------------

#[cfg(windows)]
pub use win::{altyapi_kur, arka_uc_bul, yazdir, yazici_adlari, yazicilari_listele};

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub use unix::{altyapi_kur, arka_uc_bul, yazdir, yazici_adlari, yazicilari_listele};

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub use diger::{altyapi_kur, arka_uc_bul, yazdir, yazici_adlari, yazicilari_listele};

/// PDF'i seçilen yazıcıya gönderir.
pub fn pdf_yazdir_sec(pdf: &Path, s: &Yazdirma) -> Result<()> {
    yazdir(pdf, s)
}
