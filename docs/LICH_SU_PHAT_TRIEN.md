# Lịch Sử Phát Triển VnKey

**Dự án:** VnKey Fcitx5  
**Repository:** https://github.com/marixdev/vnkey  
**Giấy phép:** GPL-3.0-or-later

---

## 📋 Mục Lục

1. [Tổng Quan Các Phiên Bản](#tổng-quan-các-phiên-bản)
2. [Chi Tiết Phiên Bản](#chi-tiết-phiên-bản)
3. [Các Cột Mốc Quan Trọng](#các-cột-mốc-quan-trọng)
4. [Đóng Góp](#đóng-góp)

---

## 📊 Tổng Quan Các Phiên Bản

| Phiên bản | Ngày | Nền tảng | Ghi chú |
|-----------|------|----------|---------|
| 1.0.1 | 2026-04-01 | Linux (Fcitx5) | Tối ưu hóa cho Linux Mint |
| 1.0.0 | TBD | Đa nền tảng | Phiên bản đầu tiên (Windows, macOS, Linux) |

---

## 📝 Chi Tiết Phiên Bản

### v1.0.1 — 2026-04-01

**Chủ đề:** Tối ưu hóa cho Linux Mint

**Thay đổi kiến trúc:**
- Loại bỏ các module không cần thiết: Windows, macOS, IBus
- Tập trung phát triển cho Fcitx5 trên Linux
- Xóa NixOS support (flake.nix)

**Tính năng mới:**
- Script build tự động cho Linux Mint (`build-linux-mint.sh`)
- Script verify build (`verify-build.sh`)
- Tài liệu hóa chi tiết cho Linux Mint

**Cải thiện:**
- Tối ưu CMakeLists.txt cho Fcitx5 5.1.x
- Cải thiện glibc compatibility shim
- Tự động hóa CI/CD chỉ build Fcitx5 packages

**Sửa lỗi:**
- Fix Fcitx5Module detection trong CMake
- Cải thiện post-install script
- Fix icon cache refresh

**Kỹ thuật:**
- Engine: Rust 1.94.1
- Addon: GCC 13.3.0, CMake 3.16+
- Fcitx5: 5.1.7
- glibc: 2.39+ (Linux Mint 22)

**Package:**
- DEB: `vnkey-fcitx5_1.0.1_amd64.deb` (1.1M)
- RPM: `vnkey-fcitx5-1.0.1-1.x86_64.rpm`
- Arch: `vnkey-fcitx5-1.0.1-1-x86_64.pkg.tar.zst`

---

### v1.0.0 — TBD (Phiên bản đầu tiên)

**Chủ đề:** Phát hành đa nền tảng

**Tính năng:**
- Hỗ trợ 4 nền tảng: Windows, macOS, Linux (Fcitx5), Linux (IBus)
- 4 kiểu gõ: Telex, Simple Telex, VNI, VIQR
- 15+ bảng mã: Unicode, TCVN3, VNI Windows, VISCII, VPS, VIQR, NCR, CP-1258...
- Kiểm tra chính tả cơ bản
- Bỏ dấu tự do
- Chuyển đổi bảng mã clipboard

**Kiến trúc:**
- Core engine viết bằng Rust
- Windows: Rust + Win32 API + Direct2D
- macOS: Objective-C + Input Method Kit
- Linux Fcitx5: C++ + Fcitx5 Framework
- Linux IBus: C + IBus Framework

**Package:**
- Windows: `vnkey.exe` (portable)
- macOS: `VnKey.dmg` (universal binary)
- Linux Fcitx5: `.deb`, `.rpm`, `.pkg.tar.zst`
- Linux IBus: `.deb`, `.rpm`, `.pkg.tar.zst`

---

## 🎯 Các Cột Mốc Quan Trọng

### Tháng 4/2026 — Tái cấu trúc cho Linux Mint

- Quyết định tập trung vào Linux Mint/Fcitx5
- Loại bỏ 75% codebase (Windows, macOS, IBus)
- Build thành công trên Linux Mint 22
- Fcitx5 addon hoạt động ổn định

### Tháng ?/202? — Khởi đầu dự án

- Ý tưởng tạo bộ gõ tiếng Việt mã nguồn mở
- Lấy cảm hứng từ Unikey và các bộ gõ hiện có
- Bắt đầu phát triển engine Rust

---

## 👥 Đóng Góp

### Nhóm phát triển chính

| Vai trò | Thành viên |
|---------|------------|
| Founder & Lead Developer | marixdev |
| Rust Engine Developer | (đang tuyển) |
| Fcitx5 Developer | (đang tuyển) |
| QA & Testing | (đang tuyển) |

### Đóng góp từ cộng đồng

Mọi đóng góp về code, tài liệu, báo cáo lỗi đều được chào đón tại:
- **GitHub:** https://github.com/marixdev/vnkey
- **Email:** hi@vnkey.app

---

## 📅 Lộ Trình Phát Triển

### Ngắn hạn (1-3 tháng)

- [ ] Hoàn thiện tài liệu tiếng Việt
- [ ] Thêm test cases cho engine
- [ ] Cải thiện chính tả tiếng Việt
- [ ] OSD notification khi chuyển chế độ

### Trung hạn (3-6 tháng)

- [ ] GUI configuration tool riêng
- [ ] Hỗ trợ macro/text expansion
- [ ] Thêm kiểu gõ mới (nếu cần)
- [ ] Cải thiện hiệu năng

### Dài hạn (6-12 tháng)

- [ ] Hỗ trợ Wayland native
- [ ] Cloud sync cấu hình
- [ ] Plugin system
- [ ] Mobile version (nếu có nhu cầu)

---

## 📞 Liên Hệ

- **Website:** https://vnkey.app
- **Email:** hi@vnkey.app
- **GitHub:** https://github.com/marixdev/vnkey

---

*Document last updated: 2026-04-01*