# Kitapçık Baskı Kılavuzu (A4'e 2×A5 tel dikiş)

Bu programın iki yüzü vardır: pencereli program **`kitapcik-gui`** (önerilen)
ve komut satırı programı **`kitapcik-yazdir`**. İkisi de aynı işi yapar.

## 1. Mantık (2 dakikada anlayın)

- Her **A4 yaprak YATAY** kullanılır; **ön yüzüne 2, arka yüzüne 2** kaynak sayfa basılır.
- Yaprakları sırayla üst üste koyup **ortadan katlayınca A5 kitap** olur.
- Her yaprak 4 sayfa aldığı için toplam, 4'ün katına tamamlanır
  (eksik kalırsa sona boş sayfa eklenir — normaldir).
- 12 sayfalık örnek: `ÖN [12|1], ARKA [2|11], ÖN [10|3], ARKA [4|9], ÖN [8|5], ARKA [6|7]`

## 2. Pencereli programla baskı (önerilen)

1. Masaüstündeki **Kitapçık Yazdır** kısayoluna çift tıklayın.
2. **1) Dosyalar:** Girdi PDF'i seçin (çıktı adı otomatik dolar).
3. **2) Sayfalar:** `hepsi` bırakın ya da yazın (`1-40,45`, `1-40:tek`).
4. **1) Plan + Önizleme** düğmesine basın:
   - üstteki tabloda hangi sayfanın hangi yaprağın ÖN/ARKA yüzüne düştüğünü görün,
   - açılan tarayıcı sayfasında her A4 yüzün resmini inceleyin.
5. **2) Baskı PDF'i Üret** düğmesine basın.
6. Yazıcı bölümünde yazıcınızı, kopyayı, **Kısa kenar** duplex'i seçip **3) Yazdır**'a basın.
7. Sürücü penceresinde: **A4 • Yatay • %100 / Gerçek boyut** olduğunu doğrulayın.
8. Çıktıları sırayla üst üste koyun → ortadan katlayın → sırttan 2 tel zımba.

## 3. Komut satırıyla baskı

```powershell
cd C:\kitapcik-yazdir
# Plan + görsel önizleme (PDF üretmez):
.\target\release\kitapcik-yazdir.exe --girdi kitap.pdf --sayfalar hepsi --onizleme
# Üretim:
.\target\release\kitapcik-yazdir.exe --girdi kitap.pdf --cikti kitap-kitapcik.pdf --sayfalar 1-40
# Kalın kitap (20'şer sayfalık desteler):
.\target\release\kitapcik-yazdir.exe --girdi kitap.pdf --cikti k.pdf --sayfalar hepsi --grup 20
# Kılavuz metni:
.\target\release\kitapcik-yazdir.exe --kilavuz
```

Tüm seçenekler: `--mod kitapcik|ikiup|duz`, `--grup`, `--kenar-boslugu-mm`,
`--orta-pay-mm`, `--kat-cizgisi`, `--kopya`, `--duplex kisa|uzun|tek-yuz`,
`--siyah-beyaz`, `--yazici "Ad"`, `--yazdir`, `--yazici-listele`.

## 4. Yazıcı ayarları (şart!)

| Ayar | Doğru değer | Yanlışsa ne olur |
|---|---|---|
| Kağıt | A4 | Başka boyutta sayfa taşar |
| Yön | Yatay | Dikeyde yarı sayfalar kesilir |
| Çift taraflı | **KISA kenardan çevir** | Arka yüzler ters gelir |
| Ölçek | **%100 / Gerçek boyut**, Sığdır kapalı | Sayfalar kayar, sıra bozulur |
| Kalite | 600 dpi | Metin için yeterlidir |

## 5. Tek yüzlü yazıcıda manuel duplex

1. Çift taraflıyı **Tek yüz** yapın, üretime aynen devam edin.
2. Önce tüm yaprakların ÖN yüzlerini basın.
3. Desteyi **çevirmeden, aynı yönde** girişe geri koyun, ARKA yüzleri basın.
4. Önce 4 sayfayla prova yapın.

## 6. Sorun giderme

- **Sıra bozuk:** Ölçek %100 mü? Sığdır kapalı mı? Duplex KISA mı?
- **Sayfalar kaymış:** Kenar/orta payını büyütün (önce 4/6 mm).
- **Sonda boş sayfa:** Normaldir (4'ün katına tamamlanır).
- **"İçerik çözülemedi":** PDF'i "Microsoft Print to PDF" ile yeniden kaydedip onu girdi yapın.
- **Yazıcı listesi boş:** Yazıcı kapalı/bağlantısız olabilir; GUI'de Yenile'ye basın.
- **Parolalı PDF:** Desteklenmez; parolasını kaldırıp deneyin.

## 7. Terimler

- **İmpozisyon:** sayfaların katlanınca sıraya girmesi için dizilmesi.
- **Fasikül:** ayrı katlanıp üst üste konan deste (kalın kitaplarda).
- **Duplex:** çift taraflı baskı. **Kısa kenar:** A4 yatayda doğru çevirme yönü.
