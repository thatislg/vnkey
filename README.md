# VnKey — Bộ gõ tiếng Việt cho Fcitx5

[![License: GPL v3](https://img.shields.io/badge/license-GPL--3.0-blue)](LICENSE)

VnKey là bộ gõ tiếng Việt mã nguồn mở cho Linux (Fcitx5), engine viết bằng **Rust**. Tối ưu cho **Linux Mint**, Ubuntu, Debian và các bản phân phối dựa trên Debian.

**Website:** [https://vnkey.app](https://vnkey.app)  
**Tài liệu:** [`docs/`](docs/)

## Kiến trúc

```
vnkey-engine/    (Rust)    Core engine + C FFI (staticlib)
vnkey-fcitx5/    (C++)     Fcitx5 addon cho Linux
```

## Tính năng

- **Kiểu gõ:** Telex, Simple Telex, VNI, VIQR
- **Bảng mã:** 15 bảng mã — Unicode UTF-8, TCVN3, VNI Windows, VISCII, VPS, VIQR, NCR, CP-1258, …
- **Kiểm tra chính tả** — tự phục hồi ký tự ASCII nếu từ không hợp lệ
- **Bỏ dấu tự do** — đặt dấu thanh ở vị trí tuỳ ý
- **Kiểu mới** (oà, uý) — theo quy tắc chính tả hiện đại
- **Chuyển đổi bảng mã clipboard** — chuyển text giữa Unicode và các bảng mã legacy
- **Cấu hình chung** `config.json`

## 📚 Tài liệu

Xem thêm tài liệu chi tiết trong thư mục [`docs/`](docs/):

- [`CAU_TRUC_DU_AN.md`](docs/CAU_TRUC_DU_AN.md) — Cấu trúc dự án
- [`LICH_SU_PHAT_TRIEN.md`](docs/LICH_SU_PHAT_TRIEN.md) — Lịch sử phát triển
- [`HUONG_DAN_BUILD.md`](docs/HUONG_DAN_BUILD.md) — Hướng dẫn build
- [`TODO.md`](docs/TODO.md) — Kế hoạch phát triển

---

## Cài đặt

### Linux Mint / Ubuntu / Debian

**Yêu cầu:** Fcitx5 đã được cài đặt. Trên Linux Mint, bạn có thể cài đặt bằng:

```bash
sudo apt update
sudo apt install fcitx5 fcitx5-configtool
```

Sau đó cài đặt VnKey:

```bash
# Tải .deb từ trang phát hành
wget https://github.com/marixdev/vnkey/releases/latest/download/vnkey-fcitx5.deb

# Cài đặt
sudo dpkg -i vnkey-fcitx5.deb
sudo apt install -f  # Cài đặt dependencies nếu thiếu
```

**Cấu hình Fcitx5:**

1. Mở **Fcitx5 Configuration** (từ menu ứng dụng)
2. Vào tab **Available Input Method**
3. Tìm và thêm **VnKey** vào **Current Input Method**
4. Khởi động lại Fcitx5: `fcitx5 -r`

### Linux — Fcitx5 (các bản phân phối khác)

```bash
# Debian/Ubuntu
sudo dpkg -i vnkey-fcitx5_1.0.1_amd64.deb

# Fedora/RHEL/Rocky
sudo dnf install ./vnkey-fcitx5-1.0.1-1.x86_64.rpm

# Arch Linux
sudo pacman -U vnkey-fcitx5-1.0.1-1-x86_64.pkg.tar.zst
```

#### NixOS (Fcitx5)

Thêm vào `flake.nix` của hệ thống:

```nix
{
  inputs.vnkey.url = "github:marixdev/vnkey";

  # Trong nixosConfigurations:
  modules = [
    ({ pkgs, ... }: {
      i18n.inputMethod = {
        enable = true;
        type = "fcitx5";
        fcitx5.addons = [
          vnkey.packages.${pkgs.system}.vnkey-fcitx5
        ];
      };
    })
  ];
}
```

Hoặc dùng `nix profile` (không cần NixOS):
```bash
nix profile install github:marixdev/vnkey#vnkey-fcitx5
```

### Build từ source

#### Trên Linux Mint / Ubuntu

```bash
# Cài đặt dependencies
sudo apt update
sudo apt install -y \
    curl \
    build-essential \
    cmake \
    fcitx5-dev \
    libfcitx5-utils-dev \
    libglib2.0-dev \
    pkg-config

# Cài đặt Rust (nếu chưa có)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Build engine
cd vnkey-engine && cargo build --release && cd ..

# Build Fcitx5 addon
cd vnkey-fcitx5
bash scripts/build.sh package    # → vnkey-fcitx5_*.deb

# Cài đặt bản build
sudo dpkg -i build/*.deb
```

#### Trên các bản phân phối khác

```bash
# Yêu cầu: Rust toolchain, CMake ≥ 3.16, GCC, Fcitx5 dev libraries

# Build engine
cd vnkey-engine && cargo build --release && cd ..

# Build Fcitx5 addon
cd vnkey-fcitx5
bash scripts/build.sh package    # → .deb hoặc .rpm
```

## Sử dụng

### Trên Linux Mint

| Thao tác | Cách thực hiện |
|----------|---------------|
| Chuyển Việt/Anh | `Ctrl+Space` |
| Mở menu cấu hình | Click phải icon tray |
| Chọn kiểu gõ (Telex/VNI) | Click phải icon tray → Input Method |
| Chọn bảng mã | Click phải icon tray → Charset |

### Phím tắt mặc định

| Phím | Chức năng |
|------|-----------|
| `Ctrl+Space` | Bật/tắt tiếng Việt |
| `Ctrl+Shift` | Chuyển kiểu gõ |
| `Shift` | Tạm thời gõ tiếng Anh |

## Cấu trúc dự án

```
vnkey-engine/              Rust engine — core xử lý tiếng Việt
  src/
    engine.rs              Engine chính (process key, backspace, reset)
    input.rs               Định nghĩa kiểu gõ (Telex, VNI, …)
    vnlexi.rs              Xử lý ngữ pháp tiếng Việt (âm, vần, thanh)
    charset/               Chuyển đổi bảng mã
    ffi.rs                 C FFI layer
  tests/integration.rs     Test cases

vnkey-fcitx5/              Fcitx5 addon (Linux)
  src/vnkey-fcitx5.cpp     Engine implementation
```

## Test

```bash
cd vnkey-engine
cargo test
```

## Giấy phép

Toàn bộ dự án được phát hành theo giấy phép **GPL-3.0-or-later**.

## Privacy Policy

This program will not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it.

## Lời cảm ơn

Dự án lấy cảm hứng từ [Unikey](https://www.unikey.org/) của Phạm Kim Long và các bộ gõ tiếng Việt khác.