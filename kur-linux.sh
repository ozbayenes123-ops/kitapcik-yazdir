#!/usr/bin/env bash
# Kitapçık Yazdır — Linux kurulumu (yönetici izni gerekmez).
# Derler, ~/.local/bin'e kurar, uygulama menüsüne kısayol ekler.
set -euo pipefail
cd "$(dirname "$0")"

BIN_DIR="${HOME}/.local/bin"
APP_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor/256x256/apps"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo bulunamadı. Rust kurun: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" >&2
  exit 1
fi

cargo build --release

mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR"
install -m 755 target/release/kitapcik-yazdir "$BIN_DIR/"
install -m 755 target/release/kitapcik-gui "$BIN_DIR/"
install -m 644 assets/icon-256.png "$ICON_DIR/kitapcik-yazdir.png"

cat > "$APP_DIR/kitapcik-yazdir.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Kitapçık Yazdır
GenericName=Kitapçık Baskı Aracı
Comment=PDF'i A4 2×A5 tel dikiş kitapçığı olarak baskıya hazırlar
Exec=${BIN_DIR}/kitapcik-gui
Icon=kitapcik-yazdir
Terminal=false
Categories=Office;
Keywords=pdf;booklet;kitapcik;baski;imposition;
StartupWMClass=kitapcik-gui
EOF
chmod 644 "$APP_DIR/kitapcik-yazdir.desktop"

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$APP_DIR" 2>/dev/null || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true

echo "Kuruldu:"
echo "  GUI : $BIN_DIR/kitapcik-gui  (uygulama menüsünde 'Kitapçık Yazdır')"
echo "  CLI : $BIN_DIR/kitapcik-yazdir"
if ! command -v lp >/dev/null 2>&1; then
  echo
  echo "Not: yazdırma için CUPS istemcisi yok. Kurmak için:"
  echo "  sudo pacman -S cups cups-filters && sudo systemctl enable --now cups"
fi
