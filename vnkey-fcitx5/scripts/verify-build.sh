#!/bin/bash
# verify-build.sh — Kiểm tra build và cài đặt VnKey Fcitx5
#
# Usage:
#   ./scripts/verify-build.sh              Kiểm tra tất cả
#   ./scripts/verify-build.sh engine       Chỉ kiểm tra engine
#   ./scripts/verify-build.sh addon        Chỉ kiểm tra addon
#   ./scripts/verify-build.sh package      Chỉ kiểm tra package
#   ./scripts/verify-build.sh install      Chỉ kiểm tra installation
#   ./scripts/verify-build.sh fcitx5       Chỉ kiểm tra Fcitx5 integration
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
CYAN='\033[0;36m'
NC='\033[0m'

# Counters
PASS=0
FAIL=0
WARN=0

log()   { echo -e "${GREEN}✓${NC} $*"; }
info()  { echo -e "${BLUE}ℹ${NC} $*"; }
warn()  { echo -e "${YELLOW}⚠${NC} $*"; ((WARN++)) || true; }
err()   { echo -e "${RED}✗${NC} $*" >&2; ((FAIL++)) || true; }
header() { echo -e "\n${CYAN}═══ $1 ═══${NC}"; }

# Check if file exists and is non-empty
check_file() {
    local file="$1"
    local desc="$2"
    if [ -f "$file" ] && [ -s "$file" ]; then
        log "$desc: $file ($(du -h "$file" | cut -f1))"
        ((PASS++)) || true
        return 0
    else
        err "$desc không tồn tại hoặc rỗng: $file"
        return 1
    fi
}

# Check if command exists
check_cmd() {
    local cmd="$1"
    local desc="$2"
    if command -v "$cmd" >/dev/null 2>&1; then
        log "$desc: $(command -v "$cmd")"
        ((PASS++)) || true
        return 0
    else
        err "$desc không tìm thấy: $cmd"
        return 1
    fi
}

# Check if string exists in file
check_content() {
    local file="$1"
    local pattern="$2"
    local desc="$3"
    if grep -q "$pattern" "$file" 2>/dev/null; then
        log "$desc"
        ((PASS++)) || true
        return 0
    else
        err "$desc không tìm thấy trong $file"
        return 1
    fi
}

# Check if library exports expected symbols
check_symbols() {
    local lib="$1"
    local symbol="$2"
    local desc="$3"
    if nm "$lib" 2>/dev/null | grep -q "$symbol"; then
        log "$desc: $symbol"
        ((PASS++)) || true
        return 0
    else
        err "$desc không tìm thấy symbol: $symbol"
        return 1
    fi
}

# ─────────────────────────────────────────────────────────────
# Engine verification
# ─────────────────────────────────────────────────────────────
verify_engine() {
    header "KIỂM TRA VNKEY-ENGINE (Rust)"

    info "Kiểm tra Rust toolchain..."
    check_cmd cargo "Cargo"
    check_cmd rustc "Rustc"

    info "\nKiểm tra engine build..."
    local engine_lib="$ENGINE_DIR/target/release/libvnkey_engine.a"

    if [ -f "$engine_lib" ]; then
        check_file "$engine_lib" "Engine library"
        info "\nKiểm tra exported symbols..."
        check_symbols "$engine_lib" "vnkey_engine_new" "FFI function"
        check_symbols "$engine_lib" "vnkey_engine_process" "FFI function"
        check_symbols "$engine_lib" "vnkey_engine_reset" "FFI function"
        check_symbols "$engine_lib" "vnkey_engine_free" "FFI function"
        check_symbols "$engine_lib" "vnkey_charset_from_utf8" "Charset function"
    else
        err "Engine library không tồn tại: $engine_lib"
        echo ""
        info "BUILD ENGINE TRƯỚC:"
        echo "  cd $ENGINE_DIR"
        echo "  cargo build --release"
        echo ""
        info "HOẶC DÙNG SCRIPT TỰ ĐỘNG:"
        echo "  cd $FCITX5_DIR"
        echo "  ./scripts/build-linux-mint.sh"
    fi

    info "\nKiểm tra Cargo.toml..."
    check_file "$ENGINE_DIR/Cargo.toml" "Engine Cargo.toml"

    if [ -f "$ENGINE_DIR/Cargo.toml" ]; then
        check_content "$ENGINE_DIR/Cargo.toml" "vnkey-engine" "Package name"
    fi
}

# ─────────────────────────────────────────────────────────────
# Addon verification
# ─────────────────────────────────────────────────────────────
verify_addon() {
    header "KIỂM TRA FCITX5 ADDON (C++)"

    info "Kiểm tra CMake build..."
    check_file "$FCITX5_DIR/CMakeLists.txt" "CMakeLists.txt"

    info "\nKiểm tra engine library (yêu cầu trước khi build addon)..."
    local engine_lib="$ENGINE_DIR/target/release/libvnkey_engine.a"
    if [ -f "$engine_lib" ]; then
        log "Engine library: $engine_lib ($(du -h "$engine_lib" | cut -f1))"
        ((PASS++)) || true
    else
        err "Engine library chưa build. Build engine trước khi build addon!"
        info "  cd $ENGINE_DIR && cargo build --release"
        return 1
    fi

    info "\nKiểm tra build output..."
    if [ -d "$BUILD_DIR" ]; then
        check_file "$BUILD_DIR/libvnkey-fcitx5.so" "Fcitx5 addon library"
        check_file "$BUILD_DIR/vnkey.conf" "Input method descriptor"

        if [ -f "$BUILD_DIR/vnkey.conf" ]; then
            info "\nKiểm tra vnkey.conf content..."
            check_content "$BUILD_DIR/vnkey.conf" "VnKey" "Addon name"
            check_content "$BUILD_DIR/vnkey.conf" "fcitx5" "Fcitx5 reference"
        fi
    else
        warn "Build directory chưa tồn tại: $BUILD_DIR"
        info "Chạy build script: ./scripts/build-linux-mint.sh"
    fi

    info "\nKiểm tra source files..."
    check_file "$FCITX5_DIR/src/vnkey-fcitx5.cpp" "Main source file"
    check_file "$FCITX5_DIR/src/vnkey-engine.h" "FFI header"
}

# ─────────────────────────────────────────────────────────────
# Package verification
# ─────────────────────────────────────────────────────────────
verify_package() {
    header "KIỂM TRA PACKAGE (.deb)"

    info "Kiểm tra dpkg..."
    check_cmd dpkg "Dpkg"

    info "\nTìm file .deb..."
    local deb_file
    deb_file=$(find "$BUILD_DIR" -name "*.deb" 2>/dev/null | head -1)

    if [ -n "$deb_file" ] && [ -f "$deb_file" ]; then
        log "Tìm thấy package: $deb_file ($(du -h "$deb_file" | cut -f1))"
        ((PASS++)) || true

        info "\nKiểm tra package contents..."
        dpkg -c "$deb_file" | head -20

        info "\nKiểm tra package metadata..."
        dpkg -I "$deb_file" | grep -E "Package:|Version:|Architecture:|Depends:|Installed-Size:"

        info "\nKiểm tra dependencies..."
        if dpkg -I "$deb_file" | grep -q "fcitx5"; then
            log "Package có dependency fcitx5"
            ((PASS++)) || true
        else
            warn "Package có thể thiếu dependency fcitx5"
        fi
    else
        err "Không tìm thấy file .deb trong $BUILD_DIR"
        info "Chạy './scripts/build-linux-mint.sh' để tạo package"
    fi
}

# ─────────────────────────────────────────────────────────────
# Installation verification
# ─────────────────────────────────────────────────────────────
verify_install() {
    header "KIỂM TRA INSTALLATION"

    info "Kiểm tra files đã install..."

    # Check input method descriptor
    local im_dirs=(
        "/usr/share/fcitx5/inputmethod"
        "/usr/local/share/fcitx5/inputmethod"
    )
    local im_found=false
    for dir in "${im_dirs[@]}"; do
        if [ -f "$dir/vnkey.conf" ]; then
            log "Input method descriptor: $dir/vnkey.conf"
            ((PASS++)) || true
            im_found=true
            break
        fi
    done
    if [ "$im_found" = false ]; then
        warn "Không tìm thấy vnkey.conf trong các thư mục inputmethod"
    fi

    # Check addon library
    local addon_dirs=(
        "/usr/lib/x86_64-linux-gnu/fcitx5"
        "/usr/lib/fcitx5"
        "/usr/local/lib/fcitx5"
    )
    local addon_found=false
    for dir in "${addon_dirs[@]}"; do
        if [ -f "$dir/libvnkey-fcitx5.so" ]; then
            log "Addon library: $dir/libvnkey-fcitx5.so"
            ((PASS++)) || true
            addon_found=true
            break
        fi
    done
    if [ "$addon_found" = false ]; then
        warn "Không tìm thấy libvnkey-fcitx5.so trong các thư mục fcitx5"
    fi

    # Check engine library
    local engine_dirs=(
        "/usr/lib"
        "/usr/local/lib"
    )
    local engine_found=false
    for dir in "${engine_dirs[@]}"; do
        if ls "$dir"/libvnkey_engine.a 2>/dev/null || ls "$dir"/libvnkey_engine.so 2>/dev/null; then
            log "Engine library: $dir/libvnkey_engine.*"
            ((PASS++)) || true
            engine_found=true
            break
        fi
    done
    if [ "$engine_found" = false ]; then
        info "Engine library có thể được link tĩnh vào addon"
    fi

    # Check ldconfig
    info "\nKiểm tra library cache..."
    if ldconfig -p 2>/dev/null | grep -q "vnkey"; then
        log "Engine library được register trong ldconfig"
        ((PASS++)) || true
    else
        info "Engine library không trong ldconfig (có thể là static link)"
    fi

    # Check config directory
    if [ -d "/usr/share/vnkey" ]; then
        log "VnKey data directory: /usr/share/vnkey"
        ((PASS++)) || true
    fi
}

# ─────────────────────────────────────────────────────────────
# Fcitx5 integration verification
# ─────────────────────────────────────────────────────────────
verify_fcitx5() {
    header "KIỂM TRA FCITX5 INTEGRATION"

    info "Kiểm tra Fcitx5..."
    check_cmd fcitx5 "Fcitx5"
    check_cmd fcitx5-remote "Fcitx5-remote"

    info "\nKiểm tra Fcitx5 đang chạy..."
    if pgrep -x fcitx5 >/dev/null 2>&1; then
        log "Fcitx5 đang chạy (PID: $(pgrep -x fcitx5 | head -1))"
        ((PASS++)) || true
    else
        warn "Fcitx5 không chạy. Khởi động bằng: fcitx5 -r"
    fi

    info "\nKiểm tra input methods available..."
    if fcitx5-remote -l 2>/dev/null | grep -qi "vnkey"; then
        log "VnKey xuất hiện trong danh sách input methods"
        ((PASS++)) || true
    else
        warn "VnKey không xuất hiện trong danh sách input methods"
        info "Thử restart Fcitx5: fcitx5 -r"
    fi

    info "\nKiểm tra Fcitx5 addons..."
    if fcitx5-remote 2>/dev/null | grep -qi "vnkey"; then
        log "VnKey addon được load"
        ((PASS++)) || true
    else
        info "Không thể kiểm tra addon status qua fcitx5-remote"
    fi

    info "\nKiểm tra Fcitx5 logs..."
    if journalctl -xe --no-pager 2>/dev/null | grep -qi "vnkey" | head -5; then
        log "Tìm thấy log entries về VnKey"
        ((PASS++)) || true
    else
        info "Không tìm thấy log entries về VnKey (có thể là bình thường)"
    fi
}

# ─────────────────────────────────────────────────────────────
# Dependencies check
# ─────────────────────────────────────────────────────────────
verify_deps() {
    header "KIỂM TRA DEPENDENCIES"

    info "Build dependencies..."
    check_cmd cargo "Rust/Cargo"
    check_cmd cmake "CMake"
    check_cmd make "Make"
    check_cmd pkg-config "pkg-config"

    info "\nFcitx5 development libraries..."
    if pkg-config --exists Fcitx5Core 2>/dev/null; then
        log "Fcitx5Core: $(pkg-config --modversion Fcitx5Core)"
        ((PASS++)) || true
    else
        warn "Fcitx5Core không tìm thấy qua pkg-config"
    fi

    if pkg-config --exists glib-2.0 2>/dev/null; then
        log "GLib: $(pkg-config --modversion glib-2.0)"
        ((PASS++)) || true
    else
        warn "GLib không tìm thấy qua pkg-config"
    fi

    info "\nRuntime dependencies..."
    check_cmd fcitx5 "Fcitx5"
}

# ─────────────────────────────────────────────────────────────
# Summary
# ─────────────────────────────────────────────────────────────
print_summary() {
    header "TỔNG KẾT"

    local total=$((PASS + FAIL + WARN))
    echo -e "  ${GREEN}✓ Pass:${NC}   $PASS"
    echo -e "  ${RED}✗ Fail:${NC}   $FAIL"
    echo -e "  ${YELLOW}⚠ Warn:${NC}   $WARN"
    echo -e "  ─────────"
    echo -e "  Total:    $total"
    echo ""

    if [ $FAIL -eq 0 ]; then
        echo -e "${GREEN}═══ BUILD OK! TẤT CẢ KIỂM TRA ĐẠT ═══${NC}"
        echo ""
        info "Để cài đặt, chạy:"
        echo "  sudo dpkg -i $BUILD_DIR/*.deb"
        echo "  fcitx5 -r"
        return 0
    else
        echo -e "${RED}═══ BUILD CÓ VẤN ĐỀ ═══${NC}"
        echo ""
        warn "Có $FAIL kiểm tra không đạt. Xem lại các lỗi bên trên."
        return 1
    fi
}

# ─────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────
usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  (default)  Kiểm tra tất cả"
    echo "  engine     Chỉ kiểm tra vnkey-engine"
    echo "  addon      Chỉ kiểm tra fcitx5 addon"
    echo "  package    Chỉ kiểm tra package .deb"
    echo "  install    Chỉ kiểm tra installation"
    echo "  fcitx5     Chỉ kiểm tra Fcitx5 integration"
    echo "  deps       Chỉ kiểm tra dependencies"
    echo ""
}

CMD="${1:-all}"

case "$CMD" in
    engine)
        verify_engine
        print_summary
        ;;
    addon)
        # Check engine first (required for addon)
        info "Kiểm tra engine trước (yêu cầu cho addon)..."
        if [ -f "$ENGINE_DIR/target/release/libvnkey_engine.a" ]; then
            verify_addon
        else
            err "Engine chưa build. Chạy 'verify-build.sh engine' để kiểm tra chi tiết."
        fi
        print_summary
        ;;
    package)
        # Check addon first (required for package)
        if [ -d "$BUILD_DIR" ] && [ -f "$BUILD_DIR/libvnkey-fcitx5.so" ]; then
            verify_package
        else
            err "Addon chưa build. Chạy './scripts/build-linux-mint.sh' trước."
            ((FAIL++)) || true
        fi
        print_summary
        ;;
    install)
        verify_install
        print_summary
        ;;
    fcitx5)
        verify_fcitx5
        print_summary
        ;;
    deps)
        verify_deps
        print_summary
        ;;
    all|"")
        verify_deps
        verify_engine
        # Only continue if engine exists
        if [ -f "$ENGINE_DIR/target/release/libvnkey_engine.a" ]; then
            verify_addon
            verify_package
            verify_install
            verify_fcitx5
        else
            echo ""
            warn "Engine chưa build. Bỏ qua các kiểm tra addon/package/install."
            echo ""
            info "HƯỚNG DẪN BUILD:"
            echo "  1. Build engine: cd vnkey-engine && cargo build --release"
            echo "  2. Build addon:  cd vnkey-fcitx5 && ./scripts/build-linux-mint.sh"
        fi
        print_summary
        ;;
    *)
        usage
        exit 1
        ;;
esac
