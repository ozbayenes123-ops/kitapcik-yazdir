//! Uygulama içi adım adım kılavuz (GUI "Kılavuz" bölümü + CLI `--kilavuz`).

pub const METIN: &str = r#"KİTAPÇIK BASKI KILAVUZU (A4'e 2×A5 tel dikiş)
================================================

MANTIK
------
Her A4 yaprak YATAY kullanılır; ön yüzüne 2, arka yüzüne 2 kaynak sayfa basılır.
Yaprakları üst üste koyup ortadan katlayınca A5 kitap olur. Her yaprak 4 sayfa
aldığı için toplam sayfa 4'ün katına tamamlanır (gerekirse sona boş sayfa eklenir).

ADIM ADIM
---------
1) Girdi PDF'i seçin. (Taranmış kitaplar da olur; parolalı PDF olmaz.)
2) Sayfaları yazın: "hepsi" ya da "1-40,45,50-" gibi. Sonekle "1-40:tek".
3) "Plan + Önizleme"ye basın: hangi sayfanın hangi yaprağın ÖN/ARKA yüzüne,
   soluna/sağına düştüğünü tablo olarak görün.
4) Açılan görsel önizlemeye (index.html) bakın: her resim bir A4 yüzdür.
   ÖN ve ARKA aynı yaprağın iki yüzüdür.
5) "Baskı PDF'i Üret"e basın. Çıktı A4 yataydır, dokunmayın.
6) Yazdırırken sürücüde şunları seçin (şart!):
     Kağıt A4 • Yön Yatay • Çift taraflı AÇIK "KISA kenardan çevir"
     Ölçek %100 / "Gerçek boyut" ("Sığdır" KAPALI) • 600 dpi yeterlidir.
7) Çıktıları sırayla üst üste koyun, ortadan katlayın, sırttan 2 tel zımba.

SAYFA SEÇİMİ
------------
  hepsi        tüm kitap            1-40       1'den 40'a
  1-40,45      aralık + tek sayfa   50-        50'den sona kadar
  -10          baştan 10'a kadar    1-40:tek   sadece tek sayfalar
  1-40:cift    sadece çift sayfalar

FASİKÜL (KALIN KİTAP)
---------------------
20 sayfadan kalın işlerde zımba tutmaz; Fasikül'e 16/20/28/32 yazın.
Her deste ayrı basılıp katlanır, desteler üst üste konur.

MANUEL ÇİFT TARAFLI (otomatik duplex yoksa)
-------------------------------------------
HP Ink Tank 310 gibi otomatik duplex ünitesi olmayan yazıcılarda:
1) Yazıcı olarak "Manual_Duplex"i seçin, çift taraflıyı "Uzun kenar" yapın.
2) "3) Yazdır"a basın: ön yüzler basılır, arka yüzler bekletilir.
3) Önler bitince desteyi UZUN kenardan (kitap sayfası gibi) çevirip
   boş yüzü yazıcıya gelecek şekilde tepsiye koyun.
4) Bildirimdeki "Print backs"e veya terminalde `manual-duplex release`
   komutuna basın; arka yüzler basılır.
İlk kullanımda 4 sayfalık test: `manual-duplex test`.
Arkalar ters/karışık gelirse: `manual-duplex config ROTATE_BACKS toggle`
veya `manual-duplex config REVERSE_BACKS toggle`.

SORUN GİDERME
-------------
• Sıra bozuk görünüyor → Ölçek %100 mü? "Sığdır" kapalı mı? Duplex KISA mı?
• Sayfalar kaymış → Kenar/orta payı büyütün (önce 4/6 mm deneyin).
• Sonda boş sayfa var → Normaldir: 4'ün katına tamamlanır.
• "İçerik çözülemedi" → PDF'i bir kez "Yazdır > PDF olarak kaydet" ile yeniden
  kaydedip onu girdi yapın.
• Yazıcı listesi boş → Yazıcınız kapalı, USB'de değil ya da CUPS'a hiç
  eklenmemiş olabilir; "Yenile"ye basın. Program hiç yazıcı bulamazsa
  "Nasıl eklerim?" düğmesiyle adım adım yol gösterir.
• "No default destination" / "lp başarısız" → CUPS'ta tanımlı yazıcı yok ya da
  varsayılan seçili değil. Tek yazıcı varsa program onu kendisi seçer.
  Kontrol: lpstat -p -d   •  Varsayılan tanımla: lpoptions -d <ad>
  Hiç yazıcı yoksa: sistem ayarları → Yazıcılar ya da http://localhost:631

LINUX (CUPS)
------------
• Yazdırma altyapısı CUPS'tır; program `lp` ile kuyruğa gönderir.
• Kurulum: sudo pacman -S cups cups-filters && sudo systemctl enable --now cups
  (Debian/Ubuntu: sudo apt install cups cups-client)
• Yazıcı ekleme: tarayıcıdan http://localhost:631 adresine girin.
• Yazıcı listesi: lpstat -p -d   •  Kuyruk: lpstat -o  •  İptal: cancel -a
• Program kağıt (A4), ölçek (%100) ve çift taraflı seçeneklerini kendisi verir.

TERİMLER
--------
• İmpozisyon: sayfaların katlanınca sıraya girecek şekilde dizilmesi.
• Fasikül: ayrı katlanıp üst üste konan deste. • Duplex: çift taraflı baskı.
• Kısa kenar: A4 yatayda kitabın doğru çevrilme yönü.
"#;
