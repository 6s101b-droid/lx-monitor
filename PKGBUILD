pkgname=lx-monitor
pkgver=0.1.0
pkgrel=1
pkgdesc="Linux Monitor - Monitor CPU, GPU temperatures and system resources"
arch=('x86_64')
url="https://github.com/yourusername/hwmonitor"
license=('MIT' 'Apache-2.0')
depends=('glibc' 'gcc-libs' 'libgl' 'libx11' 'libxi' 'libxcursor' 'libxrandr' 'libxinerama' 'libxkbcommon' 'pciutils' 'vulkan-icd-loader')
makedepends=('cargo')
source=("Cargo.toml" "Cargo.lock" "lx-monitor.desktop" "lx-monitor.png" "src")
sha256sums=('SKIP' 'SKIP' 'SKIP' 'SKIP' 'SKIP')

build() {
  export CARGO_HOME="$srcdir/cargo"
  cargo build --release --locked
}

package() {
  # Install binary
  install -Dm755 "target/release/lx-monitor" "$pkgdir/usr/bin/lx-monitor"
  
  # Install desktop file
  install -Dm644 "lx-monitor.desktop" "$pkgdir/usr/share/applications/lx-monitor.desktop"
  
  # Install icon
  install -Dm644 "lx-monitor.png" "$pkgdir/usr/share/icons/hicolor/256x256/apps/lx-monitor.png"
}
