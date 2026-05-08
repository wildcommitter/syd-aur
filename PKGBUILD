# Maintainer: Guillermo C. <user@wildcommitter.org>
pkgname=syd-git
_pkgname=syd
pkgver=0.0.0
pkgrel=1
pkgdesc="Rock-solid application kernel for sandboxing applications on Linux"
arch=('x86_64')
url="https://gitlab.exherbo.org/sydbox/sydbox"
license=('GPL-3.0-only')
depends=('libseccomp' 'gcc-libs')
makedepends=('git' 'rust' 'scdoc' 'pkgconf')
provides=('syd' 'sydbox')
conflicts=('syd' 'sydbox')
options=('!lto')
source=(
  "$_pkgname::git+https://gitlab.exherbo.org/sydbox/sydbox.git#branch=next"
)
sha256sums=(
  'SKIP'
)

pkgver() {
  cd "$srcdir/$_pkgname"
  local cargo_ver
  cargo_ver=$(awk -F'"' '/^version[[:space:]]*=/ { print $2; exit }' Cargo.toml)
  printf '%s.r%s.g%s\n' \
    "$cargo_ver" \
    "$(git rev-list --count HEAD)" \
    "$(git rev-parse --short=7 HEAD)"
}

prepare() {
  cd "$srcdir/$_pkgname"

  export CARGO_HOME="$srcdir/cargo-home"
  cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
  cd "$srcdir/$_pkgname"
  export CARGO_HOME="$srcdir/cargo-home"
  export CARGO_TARGET_DIR="$srcdir/$_pkgname/target"

  # Link dynamically against the system libseccomp.
  # Upstream's Makefile forces static linking; we don't want that on Arch.
  unset LIBSECCOMP_LINK_TYPE LIBSECCOMP_LIB_PATH

  # Build with default features (asm, log, sh, systemd, utils) plus
  # 'trusted', which lets options like trace/allow_unsafe_caps and
  # trace/allow_unsafe_ptrace take effect (no-ops without it).
  # The 'oci' feature pulls in heavy OCI runtime deps; opt in by adding
  # it to --features below if you want syd-oci built.
  cargo build --locked --release --features trusted

  # Build man pages from scdoc sources (Makefile pattern rules).
  make man
}

package() {
  cd "$srcdir/$_pkgname"

  install -d "$pkgdir/usr/bin"

  # Programs upstream's Makefile considers part of the syd suite.
  # Only install the ones that actually got built — features like 'oci'
  # are off by default, so syd-oci, syd-poc, etc. may be missing.
  local programs=(
    syd syd-aes syd-asm syd-aux syd-bit syd-cap syd-cat syd-cpu syd-dns
    syd-elf syd-emacs syd-env syd-exec syd-fd syd-fork syd-fs syd-hex
    syd-info syd-key syd-ldd syd-lock syd-ls syd-mdwe syd-net syd-mem
    syd-oci syd-ofd syd-path syd-pause syd-pds syd-poc syd-pty syd-read
    syd-rnd syd-run syd-sec syd-sh syd-size syd-stat syd-sum syd-sys
    syd-test syd-test-do syd-tck syd-tor syd-tsc syd-tty syd-utc syd-uts
    syd-x
  )
  local prog
  for prog in "${programs[@]}"; do
    if [[ -f "target/release/$prog" ]]; then
      install -Dm0755 "target/release/$prog" "$pkgdir/usr/bin/$prog"
    fi
  done

  # Man pages and vim syntax files via the upstream Makefile.
  make DESTDIR="$pkgdir" PREFIX=/usr \
    BINDIR=bin MANDIR=share/man VIMDIR=share/vim/vimfiles \
    install-man install-vim

  install -Dm0644 COPYING "$pkgdir/usr/share/licenses/$pkgname/COPYING"
  install -Dm0644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}
