# Kitapçık Yazdır

PDF kitapları **A4'e 2×A5 tel dikiş kitapçığı** düzeninde baskıya hazırlar.
Rust ile yazıldı. Her A4 yaprağın ön yüzüne 2, arka yüzüne 2 kaynak sayfa yerleştirir;
ortadan katlayıp zımbalayınca A5 kitap elde edilir. Sayfa sırasını (imposition)
otomatik hesaplar.

*A booklet imposer for Windows: turns a PDF book into A4 sheets with two A5 pages per
side, ready to print, fold and staple. Rust, CLI + GUI, MIT.*

**İki program vardır:** pencereli **`kitapcik-gui`** (önerilen — masaüstündeki
*Kitapçık Yazdır* kısayolu bunu açar) ve komut satırı **`kitapcik-yazdir`**.
Ayrıntılı adım adım anlatım: **`KILAVUZ.md`** (ayrıca programların içindeki *Kılavuz* bölümü
ve `kitapcik-yazdir --kilavuz`).

## Ne yapar?

- **Otomatik sayfa dizilimi:** 12 sayfalık kitap → 3 A4 yaprak:
  `ÖN [12|1], ARKA [2|11], ÖN [10|3], ARKA [4|9], ÖN [8|5], ARKA [6|7]`
- **Sayfa seçimi:** `1-40,45,50-`, `:tek` / `:cift` soneki, hızlı Tümü/Tekler/Çiftler düğmeleri
- **Fasikül:** kalın kitaplarda 16/20/28/32'şer sayfalık desteler
- **Görsel önizleme:** her A4 yüzün resmi (SVG) + `index.html`, katlanınca okuma sırası listesi
- **Yazıcı özellikleri:** yazıcı listesi/seçimi, kopya, duplex (kısa/uzun/tek yüz),
  renkli/siyah-beyaz, kenar + orta pay, kat çizgisi, doğrudan yazdırma
- **Gömülü kılavuz:** programın içinde adım adım Kılavuz bölümü

## Pencereli programla baskı (önerilen)

1. Masaüstündeki **Kitapçık Yazdır** kısayoluna çift tıklayın.
2. **1) Dosyalar:** Girdi PDF'i seçin (çıktı adı otomatik dolar).
3. **2) Sayfalar:** `hepsi` bırakın ya da yazın.
4. **1) Plan + Önizleme:** tabloyu inceleyin, açılan tarayıcı önizlemesine bakın.
5. **2) Baskı PDF'i Üret.**
6. **4) Yazıcı** bölümünde yazıcıyı seçip **3) Yazdır**'a basın.
7. Sürücüde **A4 • Yatay • %100** doğrulaması yapın; katlayıp 2 tel zımba.

## Komut satırıyla baskı

```powershell
cd C:\kitapcik-yazdir
.\target\release\kitapcik-yazdir.exe --girdi kitap.pdf --cikti kitap-kitapcik.pdf --sayfalar 1-40
.\target\release\kitapcik-yazdir.exe --girdi kitap.pdf --cikti k.pdf --sayfalar hepsi --grup 20
.\target\release\kitapcik-yazdir.exe --kilavuz
```

Seçenekler: `--mod kitapcik|ikiup|duz`, `--grup`, `--kenar-boslugu-mm`,
`--orta-pay-mm`, `--kopya`, `--duplex kisa|uzun|tek-yuz`, `--siyah-beyaz`,
`--yazici "Ad"`, `--yazdir`, `--yazici-listele`, `--onizleme`, `--kilavuz`.

## Yazıcı sürücü ayarları (şart!)

Kağıt **A4** • Yön **Yatay** • Çift taraflı **KISA kenardan** • Ölçek **%100**
(Sığdır kapalı) • 600 dpi yeterlidir.

## Kurulum

**Hazır kurulum (Windows):** [Releases](../../releases) sayfasındaki
`KitapcikYazdir-0.3-Kurulum.exe`. Kullanıcı düzeyinde kurulur, yönetici izni istemez
(`%LOCALAPPDATA%\Programs\Kitapçık Yazdır`).

**Kaynaktan derleme:**

```bash
git clone https://github.com/ozbayenes123-ops/kitapcik-yazdir.git
cd kitapcik-yazdir
cargo build --release
```

Çıktılar: `target/release/kitapcik-yazdir.exe` (CLI) ve `target/release/kitapcik-gui.exe` (GUI).

- Simge için `windres` (MinGW binutils) gerekir; yoksa derleme simgesiz devam eder.
- Windows notu: MinGW `ld`, kullanıcı yolunda **boşluk veya Türkçe karakter** varsa
  (`C:\Users\Ad Soyad\...`) kütüphane yollarını böler ve derleme patlar. Böyle bir hesapta
  `rust-lld` ile derleyin (`RUSTFLAGS="-C linker=rust-lld ..."`) ya da projeyi
  `C:\kitapcik-yazdir` gibi sade bir dizinde derleyin.
- Kurulum paketi (NSIS) üretmek için `makensis` gerekir; `kurulum.nsi` ve
  `kurulum/install-scoop.ps1` (scoop ile NSIS kurulumu) repoda.

## Sınırlamalar

- Şifreli (parolalı) PDF'ler açılamaz.
- Sayfa `/Rotate` değeri yoksayılır (çoğu kitapta 0'dır).
- Eski sayfa nesneleri dosyada yetim kalır; çıktı biraz şişer, zararsızdır.

## Lisans

MIT — bkz. [LICENSE](LICENSE).
