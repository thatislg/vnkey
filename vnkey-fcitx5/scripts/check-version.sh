#!/bin/bash
# check-version.sh — Kiểm tra version VnKey đang cài đặt
#
# Usage:
#   ./scripts/check-version.sh
#
# Output:
#   - Version info từ VERSION_INFO (nếu có)
#   - Version từ package đã cài
#   - So sánh với version từ GitHub
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FCITX5_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ROOT_DIR="$(cd "$FCITX5_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

log()   { echo -e "${GREEN}✓${NC} $*"; }
info()  { echo -e "${BLUE}ℹ${NC} $*"; }
warn()  { echo -e "${YELLOW}⚠${NC} $*"; }
err()   { echo -e "${RED}✗${NC} $*" >&2; }
header() { echo -e "\n${CYAN}═══ $1 ═══${NC}"; }

# Check VERSION_INFO file (từ build process)
check_version_info() {
    local version_file="$FCITX5_DIR/VERSION_INFO"

    if [ -f "$version_file" ]; then
        header "VERSION INFO (từ build)"
        cat "$version_file"
        echo ""

        # Extract key info
        local build_type=$(grep "^VNKEY_BUILD_TYPE=" "$version_file" | cut -d'=' -f2)
        local git_commit=$(grep "^VNKEY_GIT_COMMIT=" "$version_file" | cut -d'=' -f2)
        local build_date=$(grep "^VNKEY_BUILD_DATE=" "$version_file" | cut -d'=' -f2)

        if [ "$build_type" = "CUSTOM_BUILD" ]; then
            echo -e "${YELLOW}→ Đây là CUSTOM BUILD (tự build từ source)${NC}"
        elif [ "$build_type" = "OFFICIAL_RELEASE" ]; then
            echo -e "${GREEN}→ Đây là OFFICIAL RELEASE (từ GitHub)${NC}"
        fi

        echo ""
        echo -e "${BLUE}Git Commit:${NC} $git_commit"
        echo -e "${BLUE}Build Date:${NC} $build_date"
        return 0
    else
        warn "VERSION_INFO not found. Package có thể được cài từ GitHub releases."
        return 1
    fi
}

# Check installed package
check_installed_package() {
    header "INSTALLED PACKAGE"

    if dpkg -l | grep -q vnkey-fcitx5; then
        local pkg_info=$(dpkg -l | grep vnkey-fcitx5 | awk '{print $2, $3, $4}')
        local pkg_name=$(echo "$pkg_info" | awk '{print $1}')
        local pkg_version=$(echo "$pkg_info" | awk '{print $2}')
        local pkg_arch=$(echo "$pkg_info" | awk '{print $3}')

        log "Package: $pkg_name"
        log "Version: $pkg_version"
        log "Arch:    $pkg_arch"

        # Check installation date
        if [ -f /var/lib/dpkg/info/vnkey-fcitx5.list ]; then
            local install_date=$(stat -c %y /var/lib/dpkg/info/vnkey-fcitx5.list 2>/dev/null | cut -d' ' -f1)
            echo -e "${BLUE}Installed:${NC} $install_date"
        fi
    else
        err "VnKey not installed"
        return 1
    fi
}

# Check active library
check_active_library() {
    header "ACTIVE LIBRARY"

    local lib_paths=(
        "/usr/lib/x86_64-linux-gnu/fcitx5/libvnkey.so"
        "/usr/lib/fcitx5/libvnkey.so"
        "/usr/local/lib/fcitx5/libvnkey.so"
    )

    local found=false
    for path in "${lib_paths[@]}"; do
        if [ -f "$path" ]; then
            log "Library: $path"
            ls -lh "$path" | awk '{print "Size: "$5", Modified: "$6, $7, $8}'
            found=true
            break
        fi
    done

    if [ "$found" = false ]; then
        warn "Library not found. Fcitx5 có thể chưa load VnKey."
    fi
}

# Check Fcitx5 status
check_fcitx5_status() {
    header "FCITX5 STATUS"

    if pgrep -x fcitx5 >/dev/null 2>&1; then
        local pid=$(pgrep -x fcitx5 | head -1)
        log "Fcitx5 đang chạy (PID: $pid)"

        # Check current input method
        local im_status=$(fcitx5-remote 2>/dev/null || echo "unknown")
        case "$im_status" in
            1) echo -e "${BLUE}Current IM:${NC} Keyboard (English)" ;;
            2) echo -e "${GREEN}Current IM:${NC} Vietnamese (VnKey/Bamboo)" ;;
            *) echo -e "${YELLOW}Current IM:${NC} Unknown ($im_status)" ;;
        esac

        # Check available input methods
        # VnKey có thể hiển thị là "vnkey" hoặc "Vietnamese - VnKey"
        local vnkey_available=$(fcitx5-remote -l 2>/dev/null | grep -iE "vnkey|vietnamese" || echo "")
        if [ -n "$vnkey_available" ]; then
            echo -e "${GREEN}VnKey available:${NC} $vnkey_available"
        else
            # Fcitx5-remote -l có thể không hiển thị đúng tên, check bằng fcitx5-remote
            local im_status=$(fcitx5-remote 2>/dev/null)
            if [ "$im_status" = "2" ]; then
                echo -e "${GREEN}VnKey đang active (fcitx5-remote = 2)${NC}"
            else
                warn "VnKey không xuất hiện trong danh sách input methods"
            fi
        fi
    else
        warn "Fcitx5 không chạy"
        info "Start với: fcitx5 -d"
    fi
}

# Check GitHub latest release
check_github_release() {
    header "GITHUB LATEST RELEASE"

    info "Checking GitHub releases..."

    if command -v curl >/dev/null 2>&1; then
        local latest_version=$(curl -s https://api.github.com/repos/marixdev/vnkey/releases/latest 2>/dev/null | grep '"tag_name"' | cut -d'"' -f4 || echo "unknown")

        if [ "$latest_version" != "unknown" ]; then
            log "Latest Release: $latest_version"

            # Compare with installed version
            if dpkg -l | grep -q vnkey-fcitx5; then
                local installed_version=$(dpkg -l | grep vnkey-fcitx5 | awk '{print $3}')

                # So sánh version (loại bỏ 'v' prefix nếu có)
                local installed_clean=$(echo "$installed_version" | sed 's/^v//')
                local latest_clean=$(echo "$latest_version" | sed 's/^v//')

                if [ "$installed_clean" = "$latest_clean" ]; then
                    echo -e "${GREEN}→ Installed version is UP TO DATE${NC}"
                else
                    echo -e "${YELLOW}→ Installed version ($installed_version) differs from latest ($latest_version)${NC}"
                    echo ""
                    info "Update với:"
                    echo "  wget https://github.com/marixdev/vnkey/releases/latest/download/vnkey-fcitx5.deb"
                    echo "  sudo dpkg -i vnkey-fcitx5.deb"
                fi
            fi
        else
            warn "Cannot fetch GitHub releases"
        fi
    else
        warn "curl not available"
    fi
}

# Check git repository status (nếu build từ source)
check_git_status() {
    header "GIT REPOSITORY STATUS"

    if [ -d "$ROOT_DIR/.git" ]; then
        cd "$ROOT_DIR"

        local current_branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
        local current_commit=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
        local commit_date=$(git show -s --format=%ci "$current_commit" 2>/dev/null | cut -d' ' -f1)

        log "Branch: $current_branch"
        log "Commit: $current_commit ($commit_date)"

        # Check if working tree is clean
        local git_status=$(git status --porcelain 2>/dev/null | wc -l)
        if [ "$git_status" -eq 0 ]; then
            echo -e "${GREEN}→ Working tree is CLEAN${NC}"
        else
            echo -e "${YELLOW}→ Working tree has UNCOMMITTED changes ($git_status files)${NC}"
            echo ""
            info "Uncommitted files:"
            git status --porcelain 2>/dev/null | head -10
        fi

        # Check if branch is behind/ahead of origin
        if git rev-parse --abbrev-ref --symbolic-full-name @{u} >/dev/null 2>&1; then
            local behind=$(git rev-list --count HEAD..@{u} 2>/dev/null || echo "0")
            local ahead=$(git rev-list --count @{u}..HEAD 2>/dev/null || echo "0")

            if [ "$behind" -gt 0 ] || [ "$ahead" -gt 0 ]; then
                echo -e "${YELLOW}→ Branch diverged: $ahead ahead, $behind behind origin${NC}"
            else
                echo -e "${GREEN}→ Branch is UP TO DATE with origin${NC}"
            fi
        fi
    else
        info "Not a git repository"
    fi
}

# Summary
print_summary() {
    header "SUMMARY"

    local is_custom=false
    local is_official=false

    # Check if custom build
    if [ -f "$FCITX5_DIR/VERSION_INFO" ]; then
        local build_type=$(grep "^VNKEY_BUILD_TYPE=" "$FCITX5_DIR/VERSION_INFO" 2>/dev/null | cut -d'=' -f2)
        if [ "$build_type" = "CUSTOM_BUILD" ]; then
            is_custom=true
        elif [ "$build_type" = "OFFICIAL_RELEASE" ]; then
            is_official=true
        fi
    fi

    if [ "$is_custom" = true ]; then
        echo -e "${YELLOW}╔════════════════════════════════════════════╗${NC}"
        echo -e "${YELLOW}║  CUSTOM BUILD (tự build từ source)         ║${NC}"
        echo -e "${YELLOW}╚════════════════════════════════════════════╝${NC}"
        echo ""
        info "Đây là version bạn tự build từ source code."
        echo "Để check version chính thức từ GitHub:"
        echo "  curl -s https://api.github.com/repos/marixdev/vnkey/releases/latest"
    elif [ "$is_official" = true ]; then
        echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
        echo -e "${GREEN}║  OFFICIAL RELEASE (từ GitHub)              ║${NC}"
        echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
        echo ""
        info "Đây là version chính thức từ GitHub releases."
    else
        echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
        echo -e "${BLUE}║  UNKNOWN BUILD                             ║${NC}"
        echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
        echo ""
        warn "Không thể xác định nguồn gốc build."
        info "Có thể package được cài từ GitHub releases mà không có VERSION_INFO."
    fi
}

# Main
main() {
    echo -e "${CYAN}"
    echo "╔═══════════════════════════════════════════╗"
    echo "║     VnKey Version Check Tool              ║"
    echo "╚═══════════════════════════════════════════╝"
    echo -e "${NC}"

    check_version_info || true
    check_installed_package || true
    check_active_library || true
    check_fcitx5_status || true
    check_git_status || true
    check_github_release || true
    print_summary
}

# Run
main "$@"
