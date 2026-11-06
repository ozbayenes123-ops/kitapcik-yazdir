#![windows_subsystem = "windows"]
//! Kitapçık Yazdır — pencereli arayüz (egui).
//! Bölümler: Dosyalar • Sayfalar • Düzen • Yazıcı • Önizleme • Kılavuz • Durum.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use kitapcik_yazdir::{baski, dizilim, kilavuz, onizleme, sayfa, yazici};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mod {
    Kitapcik,
    Ikiup,
    Duz,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cift {
    Kisa,
    Uzun,
    TekYuz,
}

struct Gui {
    girdi: String,
    cikti: String,
    toplam: Option<u32>,
    sayfalar: String,
    mod_: Mod,
    grup: String,
    kenar: f32,
    orta: f32,
    kat_cizgisi: bool,
    // Yazıcı bölümü
    yazicilar: Vec<String>,
    yazici_sec: usize, // 0 = varsayılan
    kopya: String,
    cift: Cift,
    renkli: bool,
    koyu: bool,
    arka_uc: String,
    kurulum: Arc<Mutex<Option<String>>>,
    kuruluyor: bool,
    // Çıktılar
    plan: String,
    log: String,
    secili_sayi: Option<usize>,
}

impl Default for Gui {
    fn default() -> Self {
        let yazicilar = yazici::yazici_adlari();
        Self {
            girdi: String::new(),
            cikti: String::new(),
            toplam: None,
            sayfalar: "hepsi".into(),
            mod_: Mod::Kitapcik,
            grup: "0".into(),
            kenar: 4.0,
            orta: 6.0,
            kat_cizgisi: true,
            yazicilar,
            yazici_sec: 0,
            kopya: "1".into(),
            cift: Cift::Kisa,
            renkli: true,
            koyu: tema_yukle(),
            arka_uc: yazici::arka_uc_adi().into(),
            kurulum: Arc::new(Mutex::new(None)),
            kuruluyor: false,
            plan: String::new(),
            log: "Hoş geldiniz. 1) Girdi PDF'i seçin. Ayrıntılı anlatım için alttaki Kılavuz'u açın.\n".into(),
            secili_sayi: None,
        }
    }
}

impl Gui {
    fn baslik_rengi(&self) -> egui::Color32 {
        if self.koyu {
            egui::Color32::from_rgb(94, 234, 212)
        } else {
            TEAL_KOYU
        }
    }

    fn alt_rengi(&self) -> egui::Color32 {
        if self.koyu {
            egui::Color32::from_rgb(148, 163, 184)
        } else {
            egui::Color32::from_rgb(100, 116, 139)
        }
    }

    fn gunluk(&mut self, s: &str) {
        self.log.push_str(s);
        self.log.push('\n');
    }

    fn girdi_yukle(&mut self, yol: PathBuf) {
        self.girdi = yol.to_string_lossy().into_owned();
        match lopdf::Document::load(&yol) {
            Ok(doc) => {
                let n = doc.get_pages().len() as u32;
                self.toplam = Some(n);
                self.gunluk(&format!("Girdi: {} ({} sayfa)", yol.display(), n));
                if self.cikti.is_empty() {
                    let mut c = yol.clone();
                    let ad = c
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "kitap".into());
                    c.set_file_name(format!("{ad}-kitapcik.pdf"));
                    self.cikti = c.to_string_lossy().into_owned();
                }
                self.secimi_hesapla();
            }
            Err(e) => {
                self.toplam = None;
                self.secili_sayi = None;
                self.gunluk(&format!("Girdi açılamadı: {e:#}"));
            }
        }
    }

    fn secimi_hesapla(&mut self) {
        match self.toplam {
            Some(t) if t > 0 => match sayfa::secimi_coz(&self.sayfalar, t) {
                Ok(v) => {
                    self.secili_sayi = Some(v.len());
                }
                Err(e) => {
                    self.secili_sayi = None;
                    self.gunluk(&format!("Sayfa seçimi hatalı: {e:#}"));
                }
            },
            _ => {
                self.secili_sayi = None;
            }
        }
    }

    fn secili_sayfalar(&self) -> Option<Vec<u32>> {
        let t = self.toplam?;
        if t == 0 {
            return None;
        }
        sayfa::secimi_coz(&self.sayfalar, t).ok()
    }

    fn grup_sayi(&self) -> usize {
        self.grup.trim().parse().unwrap_or(0)
    }

    fn duplex_adi(&self) -> &'static str {
        match self.cift {
            Cift::Kisa => "kisa",
            Cift::Uzun => "uzun",
            Cift::TekYuz => "yok",
        }
    }

    fn secili_yazici(&self) -> Option<&str> {
        if self.yazici_sec == 0 {
            None
        } else {
            self.yazicilar.get(self.yazici_sec - 1).map(|s| s.as_str())
        }
    }

    // --- 1) Plan + görsel önizleme ---
    fn onizleme_ac(&mut self) {
        let Some(secili) = self.secili_sayfalar() else {
            self.gunluk("Önce geçerli bir girdi PDF ve sayfa seçimi gerekli.");
            return;
        };
        let grup = self.grup_sayi();
        if grup % 4 != 0 && grup != 0 {
            self.gunluk("Fasikül 4'ün katı olmalı (16, 20, 28, 32) ya da 0.");
            return;
        }
        if self.mod_ == Mod::Kitapcik {
            self.plan = dizilim::plan_metni(&secili, grup);
            self.gunluk("Plan hesaplandı (Önizleme bölümünde).");
            let dizin = Path::new("onizleme");
            match onizleme::html_onizleme_yaz(&secili, grup, dizin) {
                Ok(idx) => {
                    self.gunluk(&format!("Görsel önizleme: {}", idx.display()));
                    let _ = yazici::gizli(std::process::Command::new("cmd"))
                        .args(["/C", "start", "", &idx.to_string_lossy()])
                        .spawn();
                    self.gunluk("Önizleme tarayıcıda açıldı: her resim bir A4 yüzdür.");
                }
                Err(e) => self.gunluk(&format!("Önizleme yazılamadı: {e:#}")),
            }
        } else if self.mod_ == Mod::Ikiup {
            self.plan = format!(
                "Ardışık 2-up: {} sayfa yan yana dizilir (kitap sırası yok).\nÇiftler: {} A4 yüz.",
                secili.len(),
                secili.len().div_ceil(2)
            );
            self.gunluk("Plan hesaplandı.");
        } else {
            self.plan = format!("Düz baskı: {} sayfa sırayla.", secili.len());
            self.gunluk("Plan hesaplandı.");
        }
    }

    // --- 2) PDF üret ---
    fn pdf_uret(&mut self) {
        if self.girdi.is_empty() || self.cikti.is_empty() {
            self.gunluk("Girdi ve çıktı yolu gerekli.");
            return;
        }
        let Some(secili) = self.secili_sayfalar() else {
            self.gunluk("Sayfa seçimi geçersiz ya da girdi yok.");
            return;
        };
        let grup = self.grup_sayi();
        let girdi = PathBuf::from(&self.girdi);
        let cikti = PathBuf::from(&self.cikti);
        let ayar = baski::BaskiAyar {
            kenar_boslugu_mm: self.kenar as f64,
            orta_pay_mm: self.orta as f64,
            kat_cizgisi: self.kat_cizgisi,
        };
        let sonuc = match self.mod_ {
            Mod::Kitapcik => baski::kitapcik_pdf_uret(&girdi, &cikti, &secili, grup, &ayar)
                .map(|r| format!("{} fasikül, {} A4 yüz ({} yaprak)", r.fasikul, r.yuz, r.yaprak)),
            Mod::Ikiup => baski::ikiup_pdf_uret(&girdi, &cikti, &secili, &ayar)
                .map(|r| format!("{} A4 yüz", r.yuz)),
            Mod::Duz => duz_kopyala(&girdi, &cikti, &secili).map(|_| "düz sıralı".into()),
        };
        match sonuc {
            Ok(ozet) => {
                self.gunluk(&format!("Tamam: {} yazıldı ({ozet}).", cikti.display()));
                self.gunluk("Sürücüde: A4 • Yatay • Kısa kenar duplex • Ölçek %100.");
            }
            Err(e) => self.gunluk(&format!("Üretim hatası: {e:#}")),
        }
    }

    // --- 3) Yazdır ---
    fn yazdir(&mut self) {
        if self.cikti.is_empty() {
            self.gunluk("Önce çıktı PDF yolunu verin (ya da Üret'e basın).");
            return;
        }
        if !Path::new(&self.cikti).exists() {
            self.gunluk("Çıktı dosyası yok — önce Üret'e basın.");
            return;
        }
        let kopya: u32 = self.kopya.trim().parse().unwrap_or(1);
        let sec = yazici::Yazdirma {
            yazici: self.secili_yazici(),
            kopya,
            duplex: self.duplex_adi(),
            renkli: self.renkli,
        };
        match yazici::pdf_yazdir_sec(Path::new(&self.cikti), &sec) {
            Ok(()) => self.gunluk("Yazdırma kuyruğa verildi."),
            Err(e) => self.gunluk(&format!("Yazdırma hatası: {e:#}")),
        }
    }

    fn ciktiyi_ac(&mut self) {
        if self.cikti.is_empty() {
            return;
        }
        let _ = yazici::gizli(std::process::Command::new("cmd"))
            .args(["/C", "start", "", &self.cikti])
            .spawn();
        self.gunluk("Çıktı varsayılan PDF okuyucuda açıldı.");
    }

    fn yazicilari_yenile(&mut self) {
        self.yazicilar = yazici::yazici_adlari();
        self.yazici_sec = 0;
        self.arka_uc = yazici::arka_uc_adi().to_string();
        if self.yazicilar.is_empty() {
            self.gunluk("Yazıcı bulunamadı (yazıcı kapalı ya da bağlantısız olabilir).");
        } else {
            self.gunluk(&format!("{} yazıcı bulundu.", self.yazicilar.len()));
        }
    }

    fn sumatra_kur_baslat(&mut self) {
        if self.kuruluyor {
            return;
        }
        self.kuruluyor = true;
        self.gunluk("SumatraPDF kuruluyor (1-2 dk sürebilir; pencereyi kapatmayın)...");
        let paylas = Arc::clone(&self.kurulum);
        std::thread::spawn(move || {
            let sonuc = match yazici::sumatra_kur() {
                Ok(()) => "SumatraPDF kuruldu. Şimdi 3) Yazdır'a basın.".to_string(),
                Err(e) => format!(
                    "Kurulum başarısız: {e:#}\nİnterneti kontrol edip tekrar deneyin ya da Adobe Reader kurun."
                ),
            };
            *paylas.lock().unwrap() = Some(sonuc);
        });
    }

    fn kurulum_kontrol(&mut self) {
        if !self.kuruluyor {
            return;
        }
        let msg: Option<String> = match self.kurulum.try_lock() {
            Ok(mut k) => k.take(),
            Err(_) => None,
        };
        if let Some(msg) = msg {
            self.gunluk(&msg);
            self.kuruluyor = false;
            self.arka_uc = yazici::arka_uc_adi().to_string();
        }
    }
}

fn duz_kopyala(girdi: &Path, cikti: &Path, secili: &[u32]) -> anyhow::Result<()> {
    use anyhow::Context;
    use lopdf::{Document, Object};
    let mut doc = Document::load(girdi).context("girdi açılamadı")?;
    let pages = doc.get_pages();
    let mut num_to_id = std::collections::BTreeMap::new();
    for (n, id) in pages {
        num_to_id.insert(n, id);
    }
    let root = doc.trailer.get(b"Root")?.as_reference()?;
    let pages_id = doc.get_object(root)?.as_dict()?.get(b"Pages")?.as_reference()?;
    let kids: Vec<Object> = secili
        .iter()
        .map(|n| Object::Reference(num_to_id[n]))
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
    doc.save(cikti)?;
    Ok(())
}

// ---------------- Tasarım ----------------
const TEAL: egui::Color32 = egui::Color32::from_rgb(15, 118, 110);
const TEAL_KOYU: egui::Color32 = egui::Color32::from_rgb(19, 78, 74);
const TEAL_ACIK: egui::Color32 = egui::Color32::from_rgb(204, 251, 241);
const KAGIT: egui::Color32 = egui::Color32::from_rgb(248, 250, 252);
const YESIL: egui::Color32 = egui::Color32::from_rgb(21, 128, 61);
const LACI: egui::Color32 = egui::Color32::from_rgb(51, 65, 85);

fn koshe(r: u8) -> egui::CornerRadius {
    egui::CornerRadius {
        nw: r,
        ne: r,
        sw: r,
        se: r,
    }
}

/// Seçili temanın görsel stilleri.
fn gorsel(koyu: bool) -> egui::Visuals {
    let mut g = if koyu {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    if koyu {
        g.window_fill = egui::Color32::from_rgb(15, 23, 42);
        g.panel_fill = egui::Color32::from_rgb(15, 23, 42);
        g.faint_bg_color = egui::Color32::from_rgb(30, 41, 59);
        g.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 41, 59);
        g.widgets.inactive.bg_fill = egui::Color32::from_rgb(30, 41, 59);
        g.widgets.hovered.bg_fill = egui::Color32::from_rgb(19, 78, 74);
        g.widgets.active.bg_fill = egui::Color32::from_rgb(19, 78, 74);
        g.selection.bg_fill = egui::Color32::from_rgb(13, 148, 136);
        g.hyperlink_color = egui::Color32::from_rgb(45, 212, 191);
    } else {
        g.window_fill = KAGIT;
        g.panel_fill = KAGIT;
        g.widgets.noninteractive.bg_fill = egui::Color32::WHITE;
        g.widgets.inactive.bg_fill = egui::Color32::WHITE;
        g.widgets.hovered.bg_fill = TEAL_ACIK;
        g.widgets.active.bg_fill = TEAL_ACIK;
        g.selection.bg_fill = TEAL;
        g.hyperlink_color = TEAL;
    }
    g
}

/// Temayı uygular (yazı boyutları dahil).
fn tema_uygula(ctx: &egui::Context, koyu: bool) {
    ctx.set_visuals(gorsel(koyu));
    ctx.style_mut(|s| {
        s.text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(24.0));
        s.text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
        s.spacing.button_padding = egui::vec2(12.0, 7.0);
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
    });
}

/// Logodaki kitap motifini vektörle çizer (assets/logo.svg ile aynı dil).
fn logo_ciz(ui: &mut egui::Ui, boyut: f32) {
    let (etk, ressam) = ui.allocate_painter(egui::vec2(boyut, boyut), egui::Sense::hover());
    let alan = etk.rect;
    let o = boyut / 256.0;
    let p = |x: f32, y: f32| alan.min + egui::vec2(x * o, y * o);
    ressam.rect_filled(alan, koshe((52.0 * o) as u8), TEAL_KOYU);
    for sx in [52.0, 132.0] {
        ressam.rect_filled(
            egui::Rect::from_min_max(p(sx, 72.0), p(sx + 72.0, 184.0)),
            koshe((8.0 * o).max(1.0) as u8),
            egui::Color32::WHITE,
        );
    }
    for sx in [62.0, 142.0] {
        for (i, yy) in [94.0, 111.0, 128.0].iter().enumerate() {
            let gen = if i < 2 { 52.0 } else { 38.0 };
            ressam.rect_filled(
                egui::Rect::from_min_max(p(sx, *yy), p(sx + gen, yy + 7.0)),
                koshe((3.5 * o).max(1.0) as u8),
                egui::Color32::from_rgb(148, 163, 184),
            );
        }
    }
    let mut y: f32 = 62.0;
    while y < 194.0 {
        ressam.line_segment(
            [p(128.0, y), p(128.0, (y + 10.0).min(194.0))],
            egui::Stroke::new(5.0 * o, egui::Color32::from_rgb(248, 113, 113)),
        );
        y += 18.0;
    }
}

/// Büyük renkli işlem düğmesi.
fn islem_dugmesi(ui: &mut egui::Ui, metin: &str, renk: egui::Color32) -> egui::Response {
    ui.add_sized(
        [208.0, 44.0],
        egui::Button::new(
            egui::RichText::new(metin)
                .size(16.0)
                .strong()
                .color(egui::Color32::WHITE),
        )
        .fill(renk),
    )
}

fn bolum_baslik(metin: &str, acik: bool, renk: egui::Color32) -> egui::CollapsingHeader {
    egui::CollapsingHeader::new(
        egui::RichText::new(metin)
            .size(16.0)
            .strong()
            .color(renk),
    )
    .default_open(acik)
}

/// Tema seçimi %LOCALAPPDATA%\KitapcikYazdir\tema.txt içinde saklanır.
fn ayar_dizini() -> PathBuf {
    let taban = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    taban.join("KitapcikYazdir")
}

fn tema_yukle() -> bool {
    std::fs::read_to_string(ayar_dizini().join("tema.txt"))
        .map(|s| s.trim() == "koyu")
        .unwrap_or(false)
}

fn tema_kaydet(koyu: bool) {
    let dz = ayar_dizini();
    let _ = std::fs::create_dir_all(&dz);
    let _ = std::fs::write(dz.join("tema.txt"), if koyu { "koyu" } else { "acik" });
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.kurulum_kontrol();
        // İşletim sistemi temayı ezerse geri al (yalnızca uyuşmazlıkta çalışır):
        if ctx.style().visuals.dark_mode != self.koyu {
            tema_uygula(ctx, self.koyu);
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
            // Marka başlığı
            egui::Frame::new()
                .fill(TEAL_KOYU)
                .corner_radius(koshe(12))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        logo_ciz(ui, 56.0);
                        let baslik_genislik = (ui.available_width() - 180.0).max(300.0);
                        ui.allocate_ui_with_layout(
                            egui::vec2(baslik_genislik, 56.0),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                ui.label(
                                    egui::RichText::new("Kitapçık Yazdır")
                                        .size(26.0)
                                        .strong()
                                        .color(egui::Color32::WHITE),
                                );
                                ui.label(
                                    egui::RichText::new(
                                        "A4'e 2×A5 tel dikiş  •  Dosya  •  Sayfa  •  Önizle  •  Üret  •  Yazdır",
                                    )
                                    .size(14.0)
                                    .color(TEAL_ACIK),
                                );
                            },
                        );
                    });
                });
            ui.add_space(2.0);
            // Görünüm düğmesi (başlık bandının hemen altında, her zaman görünür)
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Görünüm:")
                        .size(14.0)
                        .color(self.alt_rengi()),
                );
                let once = self.koyu;
                ui.radio_value(&mut self.koyu, false, "Açık");
                ui.radio_value(&mut self.koyu, true, "Koyu");
                if self.koyu != once {
                    tema_kaydet(self.koyu);
                    tema_uygula(ui.ctx(), self.koyu);
                }
            });
            ui.add_space(4.0);

            // DOSYALAR
            bolum_baslik("1)  Dosyalar", true, self.baslik_rengi()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Girdi PDF:");
                    ui.text_edit_singleline(&mut self.girdi);
                    if ui.button("Dosya Seç…").clicked() {
                        if let Some(yol) =
                            rfd::FileDialog::new().add_filter("PDF", &["pdf"]).pick_file()
                        {
                            self.girdi_yukle(yol);
                        }
                    }
                    ui.label(match self.toplam {
                        Some(n) => format!("({n} sayfa)"),
                        None => "(dosya yok)".into(),
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("Çıktı PDF:");
                    ui.text_edit_singleline(&mut self.cikti);
                    if ui.button("Farklı Kaydet…").clicked() {
                        if let Some(yol) = rfd::FileDialog::new()
                            .add_filter("PDF", &["pdf"])
                            .set_file_name("kitap-kitapcik.pdf")
                            .save_file()
                        {
                            self.cikti = yol.to_string_lossy().into_owned();
                        }
                    }
                    if ui.button("Çıktıyı Aç").clicked() {
                        self.ciktiyi_ac();
                    }
                });
            });

            // SAYFALAR
            bolum_baslik("2)  Sayfalar", true, self.baslik_rengi()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Seçim:");
                    if ui.text_edit_singleline(&mut self.sayfalar).changed() {
                        self.secimi_hesapla();
                    }
                    if ui.button("Tümü").clicked() {
                        self.sayfalar = "hepsi".into();
                        self.secimi_hesapla();
                    }
                    if ui.button("Tekler").clicked() {
                        self.sayfalar = "hepsi:tek".into();
                        self.secimi_hesapla();
                    }
                    if ui.button("Çiftler").clicked() {
                        self.sayfalar = "hepsi:cift".into();
                        self.secimi_hesapla();
                    }
                    ui.label(match self.secili_sayi {
                        Some(n) => format!("→ {n} sayfa seçili"),
                        None => "→ seçim geçersiz".into(),
                    });
                });
                ui.label("Örnek: 1-40,45,50-   •   1-40:tek   •   -10 (baştan 10'a)");
            });

            // DÜZEN
            bolum_baslik("3)  Düzen", false, self.baslik_rengi()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Kip:");
                    ui.radio_value(&mut self.mod_, Mod::Kitapcik, "Kitapçık (önerilen)");
                    ui.radio_value(&mut self.mod_, Mod::Ikiup, "Ardışık 2-up");
                    ui.radio_value(&mut self.mod_, Mod::Duz, "Düz");
                });
                ui.horizontal(|ui| {
                    ui.label("Fasikül (4'ün katı, 0=tek parça):");
                    ui.text_edit_singleline(&mut self.grup);
                    ui.label("Kalın kitapta 16/20/28/32.");
                });
                ui.horizontal(|ui| {
                    ui.label("Kenar payı:");
                    ui.add(egui::Slider::new(&mut self.kenar, 0.0..=15.0).suffix(" mm"));
                    ui.label("Orta pay:");
                    ui.add(egui::Slider::new(&mut self.orta, 0.0..=20.0).suffix(" mm"));
                    ui.checkbox(&mut self.kat_cizgisi, "Kat çizgisi");
                });
            });

            // YAZICI
            bolum_baslik("4)  Yazıcı", true, self.baslik_rengi()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Yazdırma altyapısı:");
                    ui.label(&self.arka_uc);
                });
                if self.arka_uc.starts_with("YOK") && !self.kuruluyor {
                    ui.horizontal(|ui| {
                        ui.label("Sebep: varsayılan PDF okuyucunuz (Edge) doğrudan yazdırmaz.");
                        if ui.button("SumatraPDF Kur (önerilen, ücretsiz)").clicked() {
                            self.sumatra_kur_baslat();
                        }
                    });
                }
                if self.kuruluyor {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("SumatraPDF kuruluyor… pencereyi kapatmayın.");
                    });
                }
                ui.horizontal(|ui| {
                    ui.label("Yazıcı:");
                    let mut sec = self.yazici_sec;
                    egui::ComboBox::from_id_salt("yazici")
                        .selected_text(if sec == 0 {
                            "Varsayılan yazıcı".to_string()
                        } else {
                            self.yazicilar.get(sec - 1).cloned().unwrap_or_default()
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut sec, 0, "Varsayılan yazıcı");
                            for (i, ad) in self.yazicilar.iter().enumerate() {
                                ui.selectable_value(&mut sec, i + 1, ad);
                            }
                        });
                    self.yazici_sec = sec.min(self.yazicilar.len());
                    if ui.button("Yenile").clicked() {
                        self.yazicilari_yenile();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Çift taraflı:");
                    ui.radio_value(&mut self.cift, Cift::Kisa, "Kısa kenar (önerilen)");
                    ui.radio_value(&mut self.cift, Cift::Uzun, "Uzun kenar");
                    ui.radio_value(&mut self.cift, Cift::TekYuz, "Tek yüz (manuel çevirme)");
                });
                ui.horizontal(|ui| {
                    ui.label("Renk:");
                    ui.radio_value(&mut self.renkli, true, "Renkli");
                    ui.radio_value(&mut self.renkli, false, "Siyah-beyaz");
                    ui.label("Kopya:");
                    ui.text_edit_singleline(&mut self.kopya);
                });
                ui.label("Kağıt A4 • Yön Yatay • Ölçek %100 (çıktı hazır gelir; sürücüde Sığdır kapalı olmalı)");
            });

            // İŞLEMLER
            ui.separator();
            ui.horizontal(|ui| {
                if islem_dugmesi(ui, "1) Plan + Önizleme", LACI).clicked() {
                    self.onizleme_ac();
                }
                if islem_dugmesi(ui, "2) Baskı PDF'i Üret", TEAL).clicked() {
                    self.pdf_uret();
                }
                if islem_dugmesi(ui, "3) Yazdır", YESIL).clicked() {
                    self.yazdir();
                }
            });
            ui.separator();

            // ÖNİZLEME (plan tablosu)
            bolum_baslik("Önizleme: hangi sayfa hangi yüzde", false, self.baslik_rengi()).show(ui, |ui| {
                egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                    ui.monospace(&self.plan);
                });
            });

            // KILAVUZ
            bolum_baslik("Kılavuz (adım adım)", false, self.baslik_rengi()).show(ui, |ui| {
                egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                    ui.monospace(kilavuz::METIN);
                });
            });

            // DURUM
            ui.label(
                egui::RichText::new("Durum")
                    .size(16.0)
                    .strong()
                    .color(self.baslik_rengi()),
            );
            egui::ScrollArea::vertical()
                .max_height(120.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.monospace(&self.log);
                });
            ui.separator();
            ui.horizontal(|ui| {
                logo_ciz(ui, 22.0);
                ui.label(
                    egui::RichText::new(
                        "Kitapçık Yazdır v0.3  •  A4 Yatay  •  Kısa kenar duplex  •  Ölçek %100",
                    )
                    .size(13.0)
                    .color(self.alt_rengi()),
                );
            });
            }); // dış kaydırma
        });
    }
}

fn main() -> eframe::Result<()> {
    let sec = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([980.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Kitapçık Yazdır — A4'e 2×A5",
        sec,
        Box::new(|cc| {
            let g = Gui::default();
            // İşletim sistemi tema olaylarında bizim stiller uygulansın:
            cc.egui_ctx.set_visuals_of(egui::Theme::Dark, gorsel(true));
            cc.egui_ctx
                .set_visuals_of(egui::Theme::Light, gorsel(false));
            tema_uygula(&cc.egui_ctx, g.koyu);
            Ok(Box::new(g) as Box<dyn eframe::App>)
        }),
    )
}
