pkgname=lx-monitor
pkgver=0.1.1
pkgrel=1
pkgdesc="Linux Monitor - Monitor CPU, GPU temperatures and system resources"
arch=('x86_64')
url="https://github.com/6s101b-droid/lx-monitor"
license=('MIT')
depends=('glibc' 'gcc-libs' 'libgl' 'libx11' 'libxi' 'libxcursor' 'libxrandr' 'libxinerama' 'libxkbcommon' 'pciutils' 'vulkan-icd-loader')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$pkgname-$pkgver"
  export CARGO_HOME="$srcdir/cargo"
  cargo build --release
}

package() {
  cd "$pkgname-$pkgver"
  # Install binary
  install -Dm755 "target/release/lx-monitor" "$pkgdir/usr/bin/lx-monitor"
  
  # Install desktop file
  install -Dm644 "lx-monitor.desktop" "$pkgdir/usr/share/applications/lx-monitor.desktop"
  
  # Install icon
  install -Dm644 "lx-monitor.svg" "$pkgdir/usr/share/icons/hicolor/256x256/apps/lx-monitor.svg"
}