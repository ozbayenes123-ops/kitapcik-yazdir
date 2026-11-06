"""Kitapcik Yazdir tasarim varliklari: logo.svg ile ayni dil.
Cikti: assets/icon.ico (16-256), assets/icon-256.png, assets/ust.bmp (150x57),
assets/sol.bmp (164x314). Yazi yalnizca rakamlardan olusur (font risksiz).
"""
from PIL import Image, ImageDraw, ImageFont
import os

BURADA = os.path.dirname(os.path.abspath(__file__))
TEAL_UST = (14, 116, 144)
TEAL_ALT = (19, 78, 74)
BEYAZ = (255, 255, 255)
SAYFA_ALT = (226, 232, 240)
CIZGI = (148, 163, 184)
KAT = (248, 113, 113)
KESIM = (94, 234, 212)
FOLIO = (15, 118, 110)

try:
    print("Pillow", __import__("PIL").__version__)
    FONT_BUYUK = ImageFont.load_default(size=44)
    FONT_KUCUK = ImageFont.load_default(size=20)
except Exception as e:
    print("buyuk font yok:", e)
    FONT_BUYUK = ImageFont.load_default()
    FONT_KUCUK = ImageFont.load_default()


def dikey_gradyan(w, h):
    img = Image.new("RGB", (w, h))
    px = img.load()
    for y in range(h):
        t = y / max(h - 1, 1)
        px_r = round(TEAL_UST[0] + (TEAL_ALT[0] - TEAL_UST[0]) * t)
        px_g = round(TEAL_UST[1] + (TEAL_ALT[1] - TEAL_UST[1]) * t)
        px_b = round(TEAL_UST[2] + (TEAL_ALT[2] - TEAL_UST[2]) * t)
        for x in range(w):
            px[x, y] = (px_r, px_g, px_b)
    return img


def yuvarlak_maske(w, h, r):
    m = Image.new("L", (w, h), 0)
    ImageDraw.Draw(m).rounded_rectangle([0, 0, w - 1, h - 1], radius=r, fill=255)
    return m


def kesikli_cizgi(dr, x, y0, y1, genis, renk, ciz=10, bos=8):
    y = y0
    while y < y1:
        dr.line([x, y, x, min(y + ciz, y1)], fill=renk, width=genis)
        y += ciz + bos


def kitap_ciz(dr, olcek, ox, oy, folio=True, font=None):
    """256'lik tasarimi olcek+ofset ile ciz. Sayfa: sol(52,72,72,112) sag(132,72,72,112)."""
    def X(v):
        return ox + v * olcek

    def Y(v):
        return oy + v * olcek

    # sayfalar
    for sx in (52, 132):
        dr.rounded_rectangle([X(sx), Y(72), X(sx + 72), Y(184)], radius=max(2, int(8 * olcek)),
                             fill=BEYAZ, outline=SAYFA_ALT, width=max(1, int(2 * olcek)))
    # satirlar
    for sx in (62, 142):
        for i, yy in enumerate((94, 111, 128)):
            gen = 52 if i < 2 else 38
            dr.rounded_rectangle([X(sx), Y(yy), X(sx + gen), Y(yy + 7)],
                                 radius=max(1, int(3.5 * olcek)), fill=CIZGI)
    # folio
    if folio:
        f = font or FONT_BUYUK
        dr.text((X(88), Y(158)), "12", font=f, fill=FOLIO, anchor="ma")
        dr.text((X(168), Y(158)), "1", font=f, fill=FOLIO, anchor="ma")
    # kat cizgisi
    kesikli_cizgi(dr, X(128), Y(62), Y(194), max(2, int(5 * olcek)), KAT)


def logo_kare(boyut):
    img = dikey_gradyan(boyut, boyut)
    img.putalpha(255)
    dr = ImageDraw.Draw(img)
    o = boyut / 256.0
    kitap_ciz(dr, o, 0, 0, folio=boyut >= 64,
              font=FONT_BUYUK if boyut >= 128 else FONT_KUCUK)
    # koselerde kesim isareti
    k = max(2, int(4 * o))
    for x0, x1, y0, y1, dx, dy in ((28, 44, 44, 44, 0, 0),):
        pass
    for cx, cy, sx, sy in ((28, 44, 1, 1), (228, 44, -1, 1), (28, 212, 1, -1), (228, 212, -1, -1)):
        x, y = cx * o, cy * o
        dr.line([x, y, x + sx * 16 * o, y], fill=KESIM, width=k)
        dr.line([x, y, x, y + sy * 16 * o], fill=KESIM, width=k)
    img.putalpha(yuvarlak_maske(boyut, boyut, int(52 * o)))
    return img


def baslik_banneri(w, h):
    img = dikey_gradyan(w, h).convert("RGB")
    dr = ImageDraw.Draw(img)
    o = min(w / 256.0, h / 200.0) * 0.82
    bw, bh = 256 * o, 200 * o
    kitap_ciz(dr, o, (w - bw) / 2, (h - bh) / 2, folio=False)
    return img


if __name__ == "__main__":
    ikon = logo_kare(256)
    ikon.save(os.path.join(BURADA, "icon-256.png"))
    ikon.save(os.path.join(BURADA, "icon.ico"),
              sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    baslik_banneri(150, 57).save(os.path.join(BURADA, "ust.bmp"))
    baslik_banneri(164, 314).save(os.path.join(BURADA, "sol.bmp"))
    print("yazildi:", sorted(os.listdir(BURADA)))
