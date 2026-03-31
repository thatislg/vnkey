# Cấu Trúc Dự Án VnKey

**Phiên bản:** 1.0.1  
**Cập nhật:** 2026-04-01  
**Nền tảng mục tiêu:** Linux (Fcitx5) — Tối ưu cho Linux Mint

---

## 📋 Mục Lục

1. [Tổng Quan](#tổng-quan)
2. [Cấu Trúc Thư Mục](#cấu-trúc-thư-mục)
3. [Mô Tả Các Thành Phần](#mô-tả-các-thành-phần)
4. [Luồng Build](#luồng-build)
5. [Lịch Sử Phát Triển](#lịch-sử-phát-triển)

---

## 📌 Tổng Quan

VnKey là bộ gõ tiếng Việt mã nguồn mở cho Linux, sử dụng framework Fcitx5. Dự án được thiết kế theo kiến trúc module hóa với 2 thành phần chính:

```
┌─────────────────────────────────────────────────────────┐
│                    VnKey Project                         │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────┐    ┌─────────────────────────────┐ │
│  │  vnkey-engine   │    │      vnkey-fcitx5           │ │
│  │     (Rust)      │───▶│         (C++)               │ │
│  │  Core Engine    │FFI │    Fcitx5 Addon             │ │
│  └─────────────────┘    └─────────────────────────────┘ │
│                                                          │
│  • Xử lý tiếng Việt      • Tích hợp Fcitx5             │
│  • 15+ bảng mã           • Menu & UI                   │
│  • 4+ kiểu gõ            • Package (.deb/.rpm)         │
└─────────────────────────────────────────────────────────┘
```

---

## 📁 Cấu Trúc Thư Mục

```
vnkey/
├── docs/                           # Tài liệu dự án
│   ├── CAU_TRUC_DU_AN.md           # File này — Cấu trúc tổng thể
│   ├── LICHSU_PHAT_TRIEN.md        # Lịch sử các phiên bản
│   ├── HƯỚNG_DẪN_BUILD.md           # Hướng dẫn build chi tiết
│   └── TODO.md                     # Kế hoạch phát triển
│
├── vnkey-engine/                   # Core Engine (Rust)
│   ├── src/
│   │   ├── lib.rs                  # Export public API
│   │   ├── engine.rs               # Xử lý logic gõ chính
│   │   ├── input.rs                # Định nghĩa kiểu gõ (Telex, VNI...)
│   │   ├── vnlexi.rs               # Xử lý ngữ pháp tiếng Việt
│   │   ├── ffi.rs                  # C FFI layer — export hàm cho C/C++
│   │   ├── charset/                # Chuyển đổi bảng mã
│   │   │   ├── mod.rs
│   │   │   ├── unicode.rs
│   │   │   ├── tcvn3.rs
│   │   │   └── ...                 # 15+ bảng mã
│   │   └── options.rs              # Cấu hình engine
│   ├── tests/
│   │   └── integration.rs          # Test cases
│   ├── examples/
│   │   └── basic.rs                # Ví dụ sử dụng engine
│   ├── Cargo.toml                  # Rust package manifest
│   ├── Cargo.lock                  # Dependency versions
│   └── .gitignore
│
├── vnkey-fcitx5/                   # Fcitx5 Addon (C++)
│   ├── src/
│   │   ├── vnkey-fcitx5.cpp        # Main addon implementation
│   │   ├── vnkey-fcitx5.h          # Header file
│   │   ├── vnkey-engine.h          # FFI header (từ engine)
│   │   └── glibc-compat.c          # glibc compatibility shim
│   ├── data/
│   │   ├── vnkey-addon.conf        # Fcitx5 addon descriptor
│   │   ├── vnkey.conf              # Input method descriptor
│   │   └── fcitx-vnkey.svg         # Icon
│   ├── scripts/
│   │   ├── build.sh                # Build script chung
│   │   ├── build-linux-mint.sh     # Build script cho Linux Mint
│   │   ├── verify-build.sh         # Script kiểm tra build
│   │   ├── postinst                # Post-install script
│   │   └── prerm                   # Pre-remove script
│   ├── CMakeLists.txt              # CMake build configuration
│   ├── LICENSE
│   └── build/                      # Build output (generated)
│       ├── libvnkey.so             # Fcitx5 addon module
│       └── vnkey-fcitx5_*.deb      # Debian package
│
├── .github/
│   └── workflows/
│       └── build.yml               # CI/CD — Auto build & release
│
├── .gitignore                      # Git ignore rules
├── README.md                       # Giới thiệu & hướng dẫn nhanh
├── LINUX_MINT.md                   # Hướng dẫn chi tiết cho Linux Mint
├── CHANGELOG.md                    # Thay đổi theo phiên bản
├── LICENSE                         # GPL-3.0-or-later
└── analysis_vnkey_architecture.md  # Phân tích kiến trúc
```

---

## 🔧 Mô Tả Các Thành Phần

### 1. `vnkey-engine/` — Core Engine (Rust)

**Chức năng:** Xử lý logic gõ tiếng Việt, chuyển đổi bảng mã, kiểm tra chính tả.

**Đầu ra:** `target/release/libvnkey_engine.a` (static library)

**Các module chính:**

| File | Chức năng |
|------|-----------|
| `engine.rs` | Xử lý phím gõ, backspace, reset trạng thái |
| `input.rs` | Định nghĩa rules cho Telex, VNI, VIQR, Simple Telex |
| `vnlexi.rs` | Phân tích âm, vần, thanh trong tiếng Việt |
| `ffi.rs` | Export C functions cho C/C++ code gọi |
| `charset/` | Bảng chuyển đổi UTF-8 ↔ TCVN3, VNI, VISCII, etc. |
| `options.rs` | Cấu hình: kiểu gõ, bảng mã, chế độ |

**Build:**
```bash
cd vnkey-engine
cargo build --release
# Output: target/release/libvnkey_engine.a
```

---

### 2. `vnkey-fcitx5/` — Fcitx5 Addon (C++)

**Chức năng:** Tích hợp engine vào Fcitx5 framework trên Linux.

**Đầu ra:** 
- `build/libvnkey.so` — Fcitx5 addon module
- `build/vnkey-fcitx5_*.deb` — Debian package

**Các file chính:**

| File | Chức năng |
|------|-----------|
| `vnkey-fcitx5.cpp` | Implement Fcitx5 addon interface |
| `vnkey-engine.h` | C FFI header — khai báo functions từ engine |
| `glibc-compat.c` | Compatibility layer cho glibc versions |
| `data/vnkey.conf` | Input method descriptor cho Fcitx5 |
| `CMakeLists.txt` | Build configuration |

**Build:**
```bash
cd vnkey-fcitx5
./scripts/build-linux-mint.sh
# Output: build/libvnkey.so, build/*.deb
```

---

### 3. `docs/` — Tài Liệu

**Chức năng:** Lưu trữ tài liệu dự án.

| File | Nội dung |
|------|----------|
| `CAU_TRUC_DU_AN.md` | Cấu trúc tổng thể (file này) |
| `LICH_SU_PHAT_TRIEN.md` | Lịch sử phiên bản, milestones |
| `HUONG_DAN_BUILD.md` | Hướng dẫn build chi tiết |
| `TODO.md` | Kế hoạch phát triển tương lai |

---

## ⚙️ Luồng Build

```
┌──────────────────────────────────────────────────────────────────┐
│                        BUILD FLOW                                 │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  1. BUILD ENGINE (Rust)                                          │
│     ┌─────────────────────────────────────────────────────────┐  │
│     │ cd vnkey-engine                                         │  │
│     │ cargo build --release                                   │  │
│     │                                                         │  │
│     │ Output: target/release/libvnkey_engine.a                │  │
│     └─────────────────────────────────────────────────────────┘  │
│                          │                                        │
│                          ▼                                        │
│  2. BUILD ADDON (C++)                                            │
│     ┌─────────────────────────────────────────────────────────┐  │
│     │ cd vnkey-fcitx5                                         │  │
│     │ mkdir -p build && cd build                              │  │
│     │ cmake .. -DVNKEY_ENGINE_LIB_DIR=../../vnkey-engine/...  │  │
│     │ make -j$(nproc)                                         │  │
│     │                                                         │  │
│     │ Output: build/libvnkey.so                               │  │
│     └─────────────────────────────────────────────────────────┘  │
│                          │                                        │
│                          ▼                                        │
│  3. CREATE PACKAGE                                               │
│     ┌─────────────────────────────────────────────────────────┐  │
│     │ cpack -G DEB                                            │  │
│     │                                                         │  │
│     │ Output: build/vnkey-fcitx5_1.0.1_amd64.deb              │  │
│     └─────────────────────────────────────────────────────────┘  │
│                          │                                        │
│                          ▼                                        │
│  4. INSTALL                                                      │
│     ┌─────────────────────────────────────────────────────────┐  │
│     │ sudo dpkg -i build/*.deb                                │  │
│     │ fcitx5 -r                                               │  │
│     └─────────────────────────────────────────────────────────┘  │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

**Build tự động (khuyến nghị):**
```bash
cd vnkey-fcitx5
./scripts/build-linux-mint.sh
```

---

## 📜 Lịch Sử Phát Triển

### Giai Đoạn 1: Tối Ưu Hóa Cho Linux Mint (Hiện Tại)

**Thời gian:** 2026-04-01

**Mục tiêu:** Tập trung phát triển cho Linux Mint, loại bỏ các nền tảng không cần thiết.

**Đã hoàn thành:**
- [x] Xóa `vnkey-windows/`, `vnkey-macos/`, `vnkey-ibus/`
- [x] Xóa `flake.nix` (NixOS config)
- [x] Cập nhật README.md cho Linux Mint
- [x] Tạo script `build-linux-mint.sh`
- [x] Tạo script `verify-build.sh`
- [x] Cập nhật GitHub Actions workflow
- [x] Build và cài đặt thành công trên Linux Mint

**Kết quả:**
- Package .deb: `vnkey-fcitx5_1.0.1_amd64.deb`
- Fcitx5 addon: `libvnkey.so`
- Engine: `libvnkey_engine.a`

---

### Giai Đoạn 2: Cải Thiện Tính Năng (Tương Lai)

**Thời gian:** TBD

**Kế hoạch:**
- [ ] Thêm kiểu gõ Simple Telex cải tiến
- [ ] Hỗ trợ macro/text expansion
- [ ] Cải thiện kiểm tra chính tả
- [ ] Thêm bảng mã mới
- [ ] OSD notification khi chuyển chế độ

---

### Giai Đoạn 3: Mở Rộng Hỗ Trợ (Tương Lai)

**Thời gian:** TBD

**Kế hoạch:**
- [ ] Hỗ trợ Wayland tốt hơn
- [ ] Cấu hình GUI riêng cho Fcitx5
- [ ] Plugin system cho extensions
- [ ] Cloud sync config

---

## 📞 Liên Hệ

- **Website:** https://vnkey.app
- **Repository:** https://github.com/marixdev/vnkey
- **Email:** hi@vnkey.app

---

## 📄 Giấy Phép

Toàn bộ dự án được phát hành theo giấy phép **GPL-3.0-or-later**.