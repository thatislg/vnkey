#!/bin/bash
# build.sh — Build vnkey-engine + vnkey-fcitx5 and create .deb / .rpm packages
#
# Usage:
#   ./scripts/build.sh build        Build locally (default)
#   ./scripts/build.sh package      Build + create .deb and .rpm
#   ./scripts/build.sh install      Build + install directly (sudo)
#   ./scripts/build.sh docker-deb   Build .deb inside Docker (Debian 11, glibc 2.31)
#   ./scripts/build.sh docker-rpm   Build .rpm inside Docker (Rocky 9, glibc 2.34)
#   ./scripts/build.sh docker-all   Build both .deb and .rpm via Docker
#   ./scripts/build.sh clean        Remove build artifacts
#   ./scripts/build.sh deps         Check build dependencies
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FCITX5_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ROOT_DIR="$(cd "$FCITX5_DIR/.." && pwd)"
ENGINE_DIR="$ROOT_DIR/vnkey-engine"
BUILD_DIR="$FCITX5_DIR/build"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log()   { echo -e "${GREEN}[vnkey]${NC} $*"; }
warn()  { echo -e "${YELLOW}[vnkey]${NC} $*"; }
err()   { echo -e "${RED}[vnkey]${NC} $*" >&2; }

# ── Check build dependencies ──
check_deps() {
    log "Checking build dependencies..."
    local missing=()

    command -v cargo   >/dev/null 2>&1 || missing+=("cargo (rustup)")
    command -v cmake   >/dev/null 2>&1 || missing+=("cmake")
    command -v make    >/dev/null 2>&1 || missing+=("make / build-essential")
    command -v pkg-config >/dev/null 2>&1 || missing+=("pkg-config")

    # Check fcitx5 dev headers (pkg-config name is Fcitx5Core, case-sensitive)
    if ! pkg-config --exists Fcitx5Core 2>/dev/null; then
        missing+=("fcitx5 dev (libfcitx5core-dev / fcitx5-devel)")
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        err "Missing build dependencies:"
        for dep in "${missing[@]}"; do
            err "  - $dep"
        done
        echo ""
        warn "On Debian/Ubuntu:"
        warn "  sudo apt install cargo cmake build-essential pkg-config libfcitx5core-dev fcitx5-modules-dev"
        warn ""
        warn "On Fedora:"
        warn "  sudo dnf install cargo cmake gcc-c++ pkg-config fcitx5-devel"
        warn ""
        warn "On Arch:"
        warn "  sudo pacman -S rust cmake base-devel pkg-config fcitx5"
        exit 1
    fi
    log "All build dependencies found."
}

# ── Build the Rust engine (static library) ──
build_engine() {
    log "Building vnkey-engine (Rust static library)..."
    cd "$ENGINE_DIR"
    cargo build --release
    if [ ! -f target/release/libvnkey_engine.a ]; then
        err "libvnkey_engine.a not found after build!"
        exit 1
    fi
    log "vnkey-engine built: target/release/libvnkey_engine.a"
}

# ── Build the Fcitx5 addon ──
build_fcitx5() {
    log "Building vnkey-fcitx5 (CMake)..."
    mkdir -p "$BUILD_DIR"
    cd "$BUILD_DIR"
    cmake .. \
        -DCMAKE_INSTALL_PREFIX=/usr \
        -DCMAKE_BUILD_TYPE=Release \
        -DVNKEY_ENGINE_LIB_DIR="$ENGINE_DIR/target/release"
    make -j"$(nproc)"
    log "vnkey-fcitx5 built."
}

# ── Create packages ──
build_packages() {
    cd "$BUILD_DIR"

    # .deb
    if command -v dpkg >/dev/null 2>&1; then
        log "Creating .deb package..."
        cpack -G DEB
        log ".deb package created:"
        ls -1 "$BUILD_DIR"/*.deb 2>/dev/null || warn "No .deb found"
    else
        warn "dpkg not found — skipping .deb"
    fi

    # .rpm
    if command -v rpmbuild >/dev/null 2>&1; then
        log "Creating .rpm package..."
        cpack -G RPM
        log ".rpm package created:"
        ls -1 "$BUILD_DIR"/*.rpm 2>/dev/null || warn "No .rpm found"
    else
        warn "rpmbuild not found — skipping .rpm"
    fi

    echo ""
    log "Packages in $BUILD_DIR:"
    ls -lh "$BUILD_DIR"/*.deb "$BUILD_DIR"/*.rpm 2>/dev/null || true
}

# ── Install directly (no package) ──
direct_install() {
    cd "$BUILD_DIR"
    log "Installing directly via cmake --install ..."
    sudo cmake --install .
    sudo ldconfig
    log "Installed. Running post-install setup..."
    sudo bash "$FCITX5_DIR/scripts/postinst"
}

# ── Docker builds for cross-distro compatibility ──
docker_build_deb() {
    log "Building .deb package inside Docker (Debian 11, glibc 2.31)..."
    cd "$FCITX5_DIR"

    docker build -t vnkey-builder-deb -f docker/Dockerfile.deb "$ROOT_DIR"
    docker run --rm \
        -v "$ROOT_DIR:/src:rw" \
        -w /src/vnkey-fcitx5 \
        vnkey-builder-deb \
        bash -c '
            set -e
            cd /src/vnkey-engine && cargo build --release
            mkdir -p /src/vnkey-fcitx5/build && cd /src/vnkey-fcitx5/build
            cmake .. -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_BUILD_TYPE=Release \
                     -DVNKEY_ENGINE_LIB_DIR=/src/vnkey-engine/target/release
            make -j$(nproc)
            cpack -G DEB
        '

    log "DEB package built:"
    ls -lh "$BUILD_DIR"/*.deb 2>/dev/null || warn "No .deb found"
}

docker_build_rpm() {
    log "Building .rpm package inside Docker (Rocky 9, glibc 2.34)..."
    cd "$FCITX5_DIR"

    docker build -t vnkey-builder-rpm -f docker/Dockerfile.rpm "$ROOT_DIR"
    docker run --rm \
        -v "$ROOT_DIR:/src:rw" \
        -w /src/vnkey-fcitx5 \
        vnkey-builder-rpm \
        bash -c '
            set -e
            cd /src/vnkey-engine && cargo build --release
            mkdir -p /src/vnkey-fcitx5/build && cd /src/vnkey-fcitx5/build
            cmake .. -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_BUILD_TYPE=Release \
                     -DVNKEY_ENGINE_LIB_DIR=/src/vnkey-engine/target/release
            make -j$(nproc)
            cpack -G RPM
        '

    log "RPM package built:"
    ls -lh "$BUILD_DIR"/*.rpm 2>/dev/null || warn "No .rpm found"
}

# ── Usage ──
usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  build       Build vnkey-engine + vnkey-fcitx5 locally (default)"
    echo "  package     Build + create .deb and .rpm packages"
    echo "  install     Build + install directly (requires sudo)"
    echo "  docker-deb  Build .deb inside Docker (Debian 11, glibc 2.31)"
    echo "  docker-rpm  Build .rpm inside Docker (Rocky 9, glibc 2.34)"
    echo "  docker-all  Build both .deb and .rpm via Docker"
    echo "  clean       Remove build artifacts"
    echo "  deps        Check build dependencies"
    echo ""
    echo "Docker builds produce portable packages with minimal glibc requirements."
    echo "Minimum supported: Debian 11+, Ubuntu 22.04+, Fedora 35+, RHEL/Rocky/Alma 9+"
    echo ""
}

# ── Clean ──
clean() {
    log "Cleaning..."
    rm -rf "$BUILD_DIR"
    cd "$ENGINE_DIR" && cargo clean
    log "Clean complete."
}

# ── Main ──
CMD="${1:-build}"

case "$CMD" in
    build)
        check_deps
        build_engine
        build_fcitx5
        log "Build complete. Run '$0 package' to create packages or '$0 install' to install directly."
        ;;
    package)
        check_deps
        build_engine
        build_fcitx5
        build_packages
        ;;
    install)
        check_deps
        build_engine
        build_fcitx5
        direct_install
        ;;
    docker-deb)
        docker_build_deb
        ;;
    docker-rpm)
        docker_build_rpm
        ;;
    docker-all)
        docker_build_deb
        docker_build_rpm
        log "All Docker builds complete."
        ls -lh "$BUILD_DIR"/*.deb "$BUILD_DIR"/*.rpm 2>/dev/null || true
        ;;
    clean)
        clean
        ;;
    deps)
        check_deps
        ;;
    *)
        usage
        exit 1
        ;;
esac
