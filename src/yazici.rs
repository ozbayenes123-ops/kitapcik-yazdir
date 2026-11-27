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
    println!(
        "  Ölçek        : %100 / \"Gerçek boyut\" / \"Sığdır\" KAPALI (küçültme sayfayı kaydırır)"
    );
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
        println!(
            "  Not          : CUPS'a gönderirken kağıt/yön/duplex seçenekleri otomatik verilir"
        );
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

    /// Windows varsayılan yazıcısının adı (yoksa None).
    pub fn varsayilan_yazici() -> Option<String> {
        let out = gizli(std::process::Command::new("powershell"))
            .args([
                "-NoProfile",
                "-Command",
                "(Get-CimInstance Win32_Printer -Filter 'Default=TRUE' | Select-Object -First 1 -ExpandProperty Name)",
            ])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let ad = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if ad.is_empty() {
            None
        } else {
            Some(ad)
        }
    }

    /// Windows'ta hiç yazıcı tanımlı değilse gösterilecek yardım.
    pub fn yazici_yardim_metni() -> String {
        "Windows'ta tanımlı yazıcı bulunamadı.\n  Ayarlar → Bluetooth ve cihazlar → Yazıcılar ve tarayıcılar bölümünden \
         yazıcı ekleyin, sonra Yenile'ye basın."
            .into()
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
                println!(
                    "Adobe bulundu — yazdırılıyor (kopya/duplex'i açılan pencereden doğrulayın)..."
                );
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

    /// Sistem varsayılanı olarak tanımlı yazıcı adı (yoksa None).
    /// `lpstat -d` çıktısı: "system default destination: <ad>".
    pub fn varsayilan_yazici() -> Option<String> {
        let out = std::process::Command::new("lpstat")
            .arg("-d")
            .env("LC_ALL", "C")
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        varsayilan_ayikla(&String::from_utf8_lossy(&out.stdout))
    }

    fn varsayilan_ayikla(metin: &str) -> Option<String> {
        metin.lines().find_map(|satir| {
            let (_, ad) = satir.trim().split_once(':')?;
            let ad = ad.trim();
            let gecersiz = ad.is_empty()
                || ad.contains(char::is_whitespace)
                || ad.to_ascii_lowercase().contains("no system default");
            (!gecersiz).then(|| ad.to_string())
        })
    }

    /// CUPS zamanlayıcısı çalışıyor mu?
    /// `lpstat -r` bağlantı kurulamasa bile 0 döner; bu yüzden çıktı ayrıştırılır.
    fn cups_calisiyor() -> bool {
        std::process::Command::new("lpstat")
            .arg("-r")
            .env("LC_ALL", "C")
            .output()
            .map(|o| {
                o.status.success() && cups_calisiyor_ayikla(&String::from_utf8_lossy(&o.stdout))
            })
            .unwrap_or(false)
    }

    fn cups_calisiyor_ayikla(metin: &str) -> bool {
        let s = metin.to_ascii_lowercase();
        s.contains("running") && !s.contains("not running")
    }

    /// CUPS'a henüz eklenmemiş doğrudan cihazları (usb://, hp:/...) döndürür.
    fn algilanan_cihazlar() -> Vec<String> {
        let Ok(out) = std::process::Command::new("lpinfo").arg("-v").output() else {
            return Vec::new();
        };
        if !out.status.success() {
            return Vec::new();
        }
        cihazlari_ayikla(&String::from_utf8_lossy(&out.stdout))
    }

    fn cihazlari_ayikla(metin: &str) -> Vec<String> {
        metin
            .lines()
            .filter_map(|satir| satir.trim().strip_prefix("direct "))
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    }

    /// "No default destination" / "lp başarısız" durumunda gösterilecek,
    /// adım adım çözüm metni. Yazıcı hiç tanımlı değilse çağrılır.
    pub fn yazici_yardim_metni() -> String {
        if yolda_bul("lpstat").is_none() {
            return "CUPS araçları kurulu değil (lpstat yok).\n  Arch: sudo pacman -S cups cups-filters && sudo systemctl enable --now cups\n  Debian/Ubuntu: sudo apt install cups cups-client".into();
        }
        if !cups_calisiyor() {
            return "CUPS servisi çalışmıyor.\n  Arch: sudo systemctl enable --now cups\n  Debian/Ubuntu: sudo systemctl enable --now cups\n  Ardından --yazici-listele ile kontrol edin.".into();
        }
        let cihazlar = algilanan_cihazlar();
        let mut m = String::from(
            "CUPS'ta tanımlı yazıcı yok (\"No default destination\" hatasının sebebi budur).",
        );
        if cihazlar.is_empty() {
            m.push_str(
                "\nUSB yazıcı algılanmadı: yazıcıyı açıp bağlayın, sonra: sudo systemctl restart cups",
            );
        } else {
            m.push_str("\nAlgılanan doğrudan cihaz(lar):");
            for c in &cihazlar {
                m.push_str(&format!("\n  • {c}"));
            }
            m.push_str(
                "\nYazıcıyı ekleyin: sistem ayarları → Yazıcılar ya da http://localhost:631 \
                 (Administration → Add Printer).",
            );
            if cihazlar.iter().any(|c| c.starts_with("usb://HP/")) {
                m.push_str("\nHP USB yazıcı sürücüsü gerekebilir: sudo pacman -S hplip");
            }
        }
        m
    }

    /// Kullanılacak yazıcıyı çözer: açık seçim → sistem varsayılanı → tek kurulu yazıcı.
    /// Hiçbiri yoksa anlaşılır bir hata üretir; böylece `lp` çıplak
    /// "No default destination" hatasıyla dönmez.
    fn cozulen_yazici(s: &Yazdirma) -> Result<String> {
        if let Some(ad) = s.yazici.map(str::trim).filter(|a| !a.is_empty()) {
            return Ok(ad.to_string());
        }
        if let Some(ad) = varsayilan_yazici() {
            return Ok(ad);
        }
        let adlar = yazici_adlari();
        match adlar.as_slice() {
            [tek] => {
                println!(
                    "Varsayılan yazıcı tanımlı değil; tek kurulu yazıcı '{tek}' kullanılıyor."
                );
                println!("Kalıcı varsayılan için: lpoptions -d {tek}");
                Ok(tek.clone())
            }
            [] => anyhow::bail!("{}", yazici_yardim_metni()),
            _ => anyhow::bail!(
                "Varsayılan yazıcı tanımlı değil. Kurulu yazıcılar: {}.\n\
                 --yazici \"<ad>\" ile birini seçin ya da varsayılan tanımlayın: lpoptions -d <ad>",
                adlar.join(", ")
            ),
        }
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
        if yazici_adlari().is_empty() {
            println!("{}", yazici_yardim_metni());
        } else if let Some(ad) = varsayilan_yazici() {
            println!("Varsayılan: {ad}");
        } else {
            println!(
                "Varsayılan yazıcı tanımlı değil; program tek yazıcı varsa onu otomatik seçer."
            );
        }
        Ok(())
    }

    /// Kuyruğun PPD'sindeki renk seçeneklerinden siyah-beyaz olanı bulur.
    ///
    /// CUPS'ta `ColorModel=Gray` genel bir değer değildir; her sürücü kendi
    /// değerlerini bildirir (HP'de `KGray`, `CMYGray`, `RGB` gibi). Geçersiz
    /// değer sessizce yok sayılır ve iş renkli basılır. Bu yüzden önce
    /// `lpoptions -l` ile sürücünün gerçek değerleri okunur.
    ///
    /// Dönüş: her zaman `print-color-mode=monochrome`; PPD'de bulunursa
    /// ayrıca `ColorModel=KGray` (yalnız siyah mürekkep) veya eşdeğeri.
    fn siyah_beyaz_secenekleri(hedef: &str) -> Vec<String> {
        let Ok(out) = std::process::Command::new("lpoptions")
            .args(["-p", hedef, "-l"])
            .env("LC_ALL", "C")
            .output()
        else {
            return siyah_beyaz_ayikla("");
        };
        if !out.status.success() {
            return siyah_beyaz_ayikla("");
        }
        siyah_beyaz_ayikla(&String::from_utf8_lossy(&out.stdout))
    }

    /// `lpoptions -l` metninden siyah-beyaz job seçeneklerini üretir.
    /// Ayrı fonksiyon: yazıcı olmadan birim testle doğrulanabilir.
    fn siyah_beyaz_ayikla(liste: &str) -> Vec<String> {
        let mut secenekler = vec!["print-color-mode=monochrome".to_string()];
        let mut adaylar: Vec<(String, Vec<String>)> = Vec::new();
        for satir in liste.lines() {
            let Some((anahtar, degerler)) = satir.split_once(':') else {
                continue;
            };
            // "ColorModel/Output Mode" -> "ColorModel"
            let anahtar = anahtar.split('/').next().unwrap_or("").trim();
            if anahtar != "ColorModel" && anahtar != "ColorMode" {
                continue;
            }
            let degerler: Vec<String> = degerler
                .split_whitespace()
                .map(|d| d.trim_start_matches('*').to_string())
                .collect();
            if !degerler.is_empty() {
                adaylar.push((anahtar.to_string(), degerler));
            }
        }
        // Tercih sırası: yalnız siyah mürekkep (KGray), sonra gri tonlar.
        // CMYGray renkli mürekkeplerle gri üretir; metin için KGray daha iyidir.
        for istenen in ["KGray", "Gray", "CMYGray", "Monochrome", "Black"] {
            for (anahtar, degerler) in &adaylar {
                if let Some(d) = degerler
                    .iter()
                    .find(|d| d.eq_ignore_ascii_case(istenen))
                {
                    secenekler.push(format!("{anahtar}={d}"));
                    return secenekler;
                }
            }
        }
        secenekler
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
        let hedef = cozulen_yazici(s)?;
        let pdf_s = pdf
            .to_str()
            .context("çıktı yolu UTF-8 değil, yazdırma atlandı")?;
        let mut cmd = std::process::Command::new(&lp);
        cmd.arg("-n").arg(s.kopya.to_string());
        // `-d` her zaman açıkça verilir: sistem varsayılanı olmasa da çalışır.
        cmd.arg("-d").arg(&hedef);
        cmd.arg("-o").arg("media=A4");
        // Çıktı yüzleri A4 YATAY üretilir; sürücüye bunu açıkça söyle.
        // Söylenmezse bazı sürücüler sayfayı portreye çevirip içeriği yan yatırır.
        cmd.arg("-o").arg("orientation-requested=4");
        cmd.arg("-o").arg("print-scaling=none");
        cmd.arg("-o").arg(match s.duplex {
            "kisa" => "sides=two-sided-short-edge",
            "uzun" => "sides=two-sided-long-edge",
            _ => "sides=one-sided",
        });
        if !s.renkli {
            let renk_sec = siyah_beyaz_secenekleri(&hedef);
            // Yalnız print-color-mode varsa sürücü PPD'sinde renk modeli
            // bildirilmemiş demektir; kullanıcıyı sessizce renkli bırakma.
            if renk_sec.len() == 1 {
                println!(
                    "Uyarı: '{hedef}' sürücüsünde siyah-beyaz renk modeli bildirilmedi; \
                     yalnız print-color-mode=monochrome gönderiliyor. \
                     Renkli çıkarsa yazıcı sürücüsünü kontrol edin."
                );
            } else {
                println!("Renk: siyah-beyaz ({})", renk_sec.join(", "));
            }
            for o in &renk_sec {
                cmd.arg("-o").arg(o);
            }
        }
        cmd.arg(pdf_s);
        println!(
            "CUPS'a gönderiliyor: lp -d {} (kopya {}, duplex {})",
            hedef, s.kopya, s.duplex
        );
        let out = cmd.output().context("lp çalıştırılamadı")?;
        let std_out = String::from_utf8_lossy(&out.stdout);
        let std_err = String::from_utf8_lossy(&out.stderr);
        if !std_out.trim().is_empty() {
            println!("{}", std_out.trim());
        }
        if !out.status.success() {
            let ayrinti = std_err.trim();
            if ayrinti.to_ascii_lowercase().contains("does not exist") {
                anyhow::bail!(
                    "lp başarısız: '{hedef}' adlı yazıcı CUPS'ta yok.\n\
                     Kurulu yazıcıları görmek için: --yazici-listele\n\
                     Ayrıntı: {ayrinti}"
                );
            }
            anyhow::bail!("lp başarısız (yazıcı: {hedef}): {ayrinti}");
        }
        if !std_err.trim().is_empty() {
            // CUPS bilinmeyen seçenekleri uyarı olarak yazar; iş yine de kuyruğa girer.
            println!("Uyarı: {}", std_err.trim());
        }
        println!("Yazdırma kuyruğa verildi (yazıcı: {hedef}).");
        Ok(())
    }

    #[cfg(test)]
    mod testler {
        use super::*;

        #[test]
        fn varsayilan_ayiklanir() {
            assert_eq!(
                varsayilan_ayikla("system default destination: HP_Ink_Tank_310\n"),
                Some("HP_Ink_Tank_310".into())
            );
        }

        #[test]
        fn varsayilan_yoksa_none() {
            assert_eq!(varsayilan_ayikla("no system default destination\n"), None);
            assert_eq!(varsayilan_ayikla(""), None);
        }

        #[test]
        fn zamanlayici_durumu_ayiklanir() {
            assert!(cups_calisiyor_ayikla("scheduler is running\n"));
            assert!(!cups_calisiyor_ayikla("scheduler is not running\n"));
            assert!(!cups_calisiyor_ayikla(""));
        }

        #[test]
        fn usb_cihazlari_ayiklanir() {
            let metin = "network beh\ndirect usb://HP/Ink%20Tank%20310%20series?serial=ABC\n\
                         network ipp\ndirect hp:/usb/HP_Ink_Tank_310_series?serial=ABC\n";
            assert_eq!(
                cihazlari_ayikla(metin),
                vec![
                    "usb://HP/Ink%20Tank%20310%20series?serial=ABC".to_string(),
                    "hp:/usb/HP_Ink_Tank_310_series?serial=ABC".to_string()
                ]
            );
        }

        #[test]
        fn hp_kgray_renk_modeli_secilir() {
            // Gerçek HP PPD satırı: Gray yok, KGray ve CMYGray var.
            let liste = "PageSize/Media Size: *A4 A5\n\
                         ColorModel/Output Mode: CMYGray KGray *RGB\n";
            assert_eq!(
                siyah_beyaz_ayikla(liste),
                vec![
                    "print-color-mode=monochrome".to_string(),
                    "ColorModel=KGray".to_string()
                ]
            );
        }

        #[test]
        fn renk_modeli_yoksa_tek_secenek_kalir() {
            // Sürücü renk modeli bildirmiyorsa uydurma değer gönderme.
            let sec = siyah_beyaz_ayikla("PageSize/Media Size: *A4\n");
            assert_eq!(sec, vec!["print-color-mode=monochrome".to_string()]);
        }
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

    pub fn varsayilan_yazici() -> Option<String> {
        None
    }

    pub fn yazici_yardim_metni() -> String {
        "Bu platformda yazıcı listeleme desteklenmiyor.".into()
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
pub use win::{
    altyapi_kur, arka_uc_bul, varsayilan_yazici, yazdir, yazici_adlari, yazici_yardim_metni,
    yazicilari_listele,
};

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub use unix::{
    altyapi_kur, arka_uc_bul, varsayilan_yazici, yazdir, yazici_adlari, yazici_yardim_metni,
    yazicilari_listele,
};

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub use diger::{
    altyapi_kur, arka_uc_bul, varsayilan_yazici, yazdir, yazici_adlari, yazici_yardim_metni,
    yazicilari_listele,
};

/// PDF'i seçilen yazıcıya gönderir.
pub fn pdf_yazdir_sec(pdf: &Path, s: &Yazdirma) -> Result<()> {
    yazdir(pdf, s)
}
