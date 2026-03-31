#!/bin/bash
# build-linux-mint.sh — Build và cài đặt VnKey Fcitx5 trên Linux Mint
#
# Usage:
#   ./scripts/build-linux-mint.sh          Build và tạo .deb
#   ./scripts/build-linux-mint.sh install  Build và cài đặt ngay
#   ./scripts/build-linux-mint.sh clean    Xóa build artifacts
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FCITX5_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ROOT_DIR="$(cd "$FCITX5_DIR/.." && pwd)"
ENGINE_DIR="$ROOT_DIR/vnkey-engine"
BUILD_DIR="$FCITX5_DIR/build"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log()   { echo -e "${GREEN}[VnKey]${NC} $*"; }
info()  { echo -e "${BLUE}[VnKey]${NC} $*"; }
warn()  { echo -e "${YELLOW}[VnKey]${NC} $*"; }
err()   { echo -e "${RED}[VnKey]${NC} $*" >&2; }

# Check if running on Linux Mint / Ubuntu / Debian
check_os() {
    if [ -f /etc/linuxmint-release ]; then
        log "Phát hiện Linux Mint: $(cat /etc/linuxmint-release)"
    elif [ -f /etc/debian_version ]; then
        log "Phát hiện Debian/Ubuntu"
    elif [ -f /etc/os-release ]; then
        source /etc/os-release
        log "Phát hiện: $PRETTY_NAME"
    else
        warn "Không thể xác định hệ điều hành. Tiếp tục build..."
    fi
}

# Install dependencies
install_deps() {
    log "Đang cài đặt dependencies..."

    if command -v apt-get >/dev/null 2>&1; then
        sudo apt-get update
        sudo apt-get install -y \
            curl \
            build-essential \
            cmake \
            pkg-config \
            fcitx5 \
            fcitx5-configtool \
            libfcitx5core-dev \
            libfcitx5config-dev \
            libfcitx5utils-dev \
            libglib2.0-dev \
            ca-certificates

        log "Dependencies đã được cài đặt."
    else
        err "Không tìm thấy apt-get. Script chỉ hỗ trợ Debian/Ubuntu/Linux Mint."
        exit 1
    fi
}

# Check if Rust is installed
check_rust() {
    if ! command -v cargo >/dev/null 2>&1; then
        log "Rust chưa được cài đặt. Đang cài đặt..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        source "$HOME/.cargo/env"
        log "Rust đã được cài đặt."
    else
        log "Rust đã có: $(rustc --version)"
    fi
}

# Check build dependencies
check_deps() {
    info "Kiểm tra dependencies..."
    local missing=()

    command -v cargo   >/dev/null 2>&1 || missing+=("cargo (rustup)")
    command -v cmake   >/dev/null 2>&1 || missing+=("cmake")
    command -v make    >/dev/null 2>&1 || missing+=("make / build-essential")
    command -v pkg-config >/dev/null 2>&1 || missing+=("pkg-config")

    if ! pkg-config --exists Fcitx5Core 2>/dev/null; then
        missing+=("fcitx5 dev (libfcitx5core-dev)")
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        err "Missing build dependencies:"
        for dep in "${missing[@]}"; do
            err "  - $dep"
        done
        echo ""
        warn "Chạy script với 'install-deps' để cài đặt tự động:"
        warn "  $0 install-deps"
        exit 1
    fi
    log "Tất cả dependencies đã sẵn sàng."
}

# Build the Rust engine
build_engine() {
    log "Đang build vnkey-engine..."
    cd "$ENGINE_DIR"
    cargo build --release

    if [ ! -f target/release/libvnkey_engine.a ]; then
        err "libvnkey_engine.a không được tạo!"
        exit 1
    fi
    log "✓ vnkey-engine built: target/release/libvnkey_engine.a"
}

# Build the Fcitx5 addon
build_fcitx5() {
    log "Đang build vnkey-fcitx5..."
    mkdir -p "$BUILD_DIR"
    cd "$BUILD_DIR"

    cmake .. \
        -DCMAKE_INSTALL_PREFIX=/usr \
        -DCMAKE_BUILD_TYPE=Release \
        -DVNKEY_ENGINE_LIB_DIR="$ENGINE_DIR/target/release"

    make -j"$(nproc)"

    log "✓ vnkey-fcitx5 built."
}

# Create .deb package
build_package() {
    cd "$BUILD_DIR"

    if command -v dpkg >/dev/null 2>&1; then
        log "Đang tạo .deb package..."
        cpack -G DEB

        if [ -f "$BUILD_DIR"/*.deb ]; then
            log "✓ .deb package được tạo:"
            ls -lh "$BUILD_DIR"/*.deb
        else
            err "Không tạo được .deb package"
            exit 1
        fi
    else
        warn "dpkg không có — bỏ qua tạo .deb"
    fi
}

# Install directly
direct_install() {
    cd "$BUILD_DIR"
    log "Đang cài đặt VnKey..."

    sudo cmake --install .
    sudo ldconfig

    # Run post-install if exists
    if [ -f "$FCITX5_DIR/scripts/postinst" ]; then
        sudo bash "$FCITX5_DIR/scripts/postinst"
    fi

    log "✓ Cài đặt hoàn tất!"
    echo ""
    info "HƯỚNG DẪN SỬ DỤNG:"
    echo "  1. Mở Fcitx5 Configuration từ menu ứng dụng"
    echo "  2. Vào tab 'Available Input Method'"
    echo "  3. Tìm và thêm 'VnKey' vào 'Current Input Method'"
    echo "  4. Khởi động lại Fcitx5: fcitx5 -r"
    echo ""
    echo "  Phím tắt:"
    echo "    Ctrl+Space  : Bật/tắt tiếng Việt"
    echo "    Shift       : Tạm thời gõ tiếng Anh"
}

# Clean build artifacts
clean() {
    log "Đang xóa build artifacts..."
    rm -rf "$BUILD_DIR"
    cd "$ENGINE_DIR" && cargo clean
    log "✓ Dọn dẹp hoàn tất."
}

# Show usage
usage() {
    echo "VnKey Fcitx5 — Build Script cho Linux Mint"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  (default)   Build và tạo .deb package"
    echo "  install     Build và cài đặt ngay"
    echo "  install-deps Cài đặt dependencies"
    echo "  clean       Xóa build artifacts"
    echo ""
    echo "Ví dụ:"
    echo "  $0              # Build và tạo .deb"
    echo "  $0 install      # Build và cài đặt"
    echo "  $0 install-deps # Chỉ cài đặt dependencies"
    echo "  $0 clean        # Dọn dẹp"
}

# Main
CMD="${1:-}"

case "$CMD" in
    install-deps)
        check_os
        install_deps
        check_rust
        ;;
    install)
        check_os
        install_deps
        check_rust
        source "$HOME/.cargo/env" 2>/dev/null || true
        check_deps
        build_engine
        build_fcitx5
        direct_install
        ;;
    clean)
        clean
        ;;
    "")
        check_os
        check_rust
        source "$HOME/.cargo/env" 2>/dev/null || true
        check_deps
        build_engine
        build_fcitx5
        build_package
        log "Build hoàn tất! File .deb nằm trong: $BUILD_DIR"
        echo ""
        info "Để cài đặt, chạy:"
        echo "  sudo dpkg -i $BUILD_DIR/vnkey-fcitx5_*.deb"
        echo "  sudo apt install -f  # Nếu thiếu dependencies"
        ;;
    *)
        usage
        exit 1
        ;;
esac
