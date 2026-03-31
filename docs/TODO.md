# Kế Hoạch Phát Triển VnKey

**Cập nhật:** 2026-04-01  
**Phiên bản hiện tại:** 1.0.1  
**Nền tảng:** Linux (Fcitx5) — Linux Mint

---

## 📋 Mục Lục

1. [Tổng Quan](#tổng-quan)
2. [Short-term (1-3 tháng)](#short-term-1-3-tháng)
3. [Medium-term (3-6 tháng)](#medium-term-3-6-tháng)
4. [Long-term (6-12 tháng)](#long-term-6-12-tháng)
5. [Backlog](#backlog)
6. [Known Issues](#known-issues)

---

## 🎯 Tổng Quan

VnKey đang trong giai đoạn phát triển tập trung cho Linux Mint với Fcitx5. Tài liệu này ghi lại kế hoạch phát triển theo từng giai đoạn.

### Trạng Thái Hiện Tại

| Hạng mục | Trạng thái |
|----------|------------|
| Core Engine (Rust) | ✅ Ổn định |
| Fcitx5 Addon (C++) | ✅ Ổn định |
| Build System | ✅ Hoàn thiện |
| CI/CD | ✅ Hoạt động |
| Tài liệu | 🔄 Đang cập nhật |

---

## 📅 Short-term (1-3 tháng)

### Sprint 1: Hoàn Thiện Build & Test (Tháng 4/2026)

**Mục tiêu:** Cải thiện quy trình build và testing.

- [ ] **Unit Tests cho Engine**
  - [ ] Test các kiểu gõ (Telex, VNI, VIQR)
  - [ ] Test chuyển đổi bảng mã
  - [ ] Test xử lý backspace
  - [ ] Test surrounding text recovery
  - **Độ phủ:** ≥80%

- [ ] **Integration Tests**
  - [ ] Test end-to-end với Fcitx5
  - [ ] Test trên nhiều distro (Ubuntu, Debian, Fedora)
  - [ ] Automated testing trong CI

- [ ] **Cải Thiện Build Script**
  - [ ] Thêm option `--debug` cho build script
  - [ ] Thêm logging chi tiết hơn
  - [ ] Support build incremental nhanh hơn
  - [ ] Docker build cho reproducibility

- [ ] **Tài Liệu**
  - [ ] Hướng dẫn debug
  - [ ] API documentation cho engine
  - [ ] Contributing guidelines

**Definition of Done:**
- Tất cả tests pass trong CI
- Build time < 2 phút cho incremental build
- Documentation đầy đủ

---

### Sprint 2: Cải Thiện UX (Tháng 5/2026)

**Mục tiêu:** Nâng cao trải nghiệm người dùng.

- [ ] **OSD Notification**
  - [ ] Hiển thị toast khi chuyển chế độ Việt/Anh
  - [ ] Hiển thị kiểu gõ hiện tại
  - [ ] Hiển thị bảng mã hiện tại
  - [ ] Customizable position & style

- [ ] **Cấu Hình GUI**
  - [ ] Cửa sổ cấu hình riêng (Qt/GTK)
  - [ ] Preview kiểu gõ
  - [ ] Import/Export config
  - [ ] Reset về mặc định

- [ ] **Phím Tắt**
  - [ ] Customizable hotkeys
  - [ ] Global hotkeys registration
  - [ ] Conflict detection

- [ ] **Performance**
  - [ ] Giảm latency khi gõ (<10ms)
  - [ ] Optimize memory usage
  - [ ] Profile và fix bottlenecks

**Definition of Done:**
- OSD hoạt động mượt mà
- GUI config hoàn chỉnh
- Latency < 10ms

---

### Sprint 3: Tính Năng Nâng Cao (Tháng 6/2026)

**Mục tiêu:** Thêm các tính năng nâng cao.

- [ ] **Macro / Text Expansion**
  - [ ] Define custom macros
  - [ ] Import/Export macro list
  - [ ] Pre-defined macros (địa chỉ, email, chữ ký)

- [ ] **Smart Spell Check**
  - [ ] Tích hợp từ điển tiếng Việt
  - [ ] Gợi ý từ đúng chính tả
  - [ ] Học từ mới từ user

- [ ] **Clipboard Converter**
  - [ ] Convert text giữa các bảng mã
  - [ ] Hotkey để convert clipboard
  - [ ] Support 15+ bảng mã

- [ ] **Profile System**
  - [ ] Multiple profiles (work, personal, dev)
  - [ ] Auto-switch profile per application
  - [ ] Sync profiles giữa các máy

**Definition of Done:**
- Macro system hoạt động
- Spell check gợi ý chính xác
- Clipboard converter hỗ trợ đủ bảng mã

---

## 📅 Medium-term (3-6 tháng)

### Tính Năng Chính

- [ ] **Wayland Support Hoàn Chỉnh**
  - [ ] Fix các issues với Wayland
  - [ ] Test trên GNOME Wayland, KDE Wayland
  - [ ] Support input method protocol mới

- [ ] **Cloud Sync**
  - [ ] Sync config qua cloud (optional)
  - [ ] Backup/Restore tự động
  - [ ] End-to-end encryption

- [ ] **Plugin System**
  - [ ] API cho plugins
  - [ ] Plugin marketplace (community)
  - [ ] Built-in plugins: emoji, math symbols, etc.

- [ ] **Multi-language Support**
  - [ ] UI localization (EN, VI, FR, JA, etc.)
  - [ ] Support gõ các ngôn ngữ khác (Thai, Khmer)
  - [ ] Language detection tự động

---

## 📅 Long-term (6-12 tháng)

### Tái Cấu Trúc & Mở Rộng

- [ ] **Engine V2**
  - [ ] Refactor engine architecture
  - [ ] Support dynamic loading của rules
  - [ ] Plugin-based input methods

- [ ] **Mobile Support**
  - [ ] Android keyboard (AOSP)
  - [ ] iOS keyboard (nếu có thể)
  - [ ] Cross-platform config sync

- [ ] **AI-Powered Features**
  - [ ] Predictive text với ML
  - [ ] Context-aware input
  - [ ] Auto-correct thông minh

- [ ] **Enterprise Features**
  - [ ] Centralized management (cho công ty)
  - [ ] Custom dictionary deployment
  - [ ] Audit logging

---

## 📦 Backlog

### Ý Tưởng Tính Năng

- [ ] Gõ tắt theo ngữ cảnh
- [ ] Hỗ trợ gõ chữ Hán-Nôm
- [ ] Chuyển đổi chữ thường/hoa thông minh
- [ ] Auto-complete từ
- [ ] Voice input integration
- [ ] Gesture typing (trên mobile)
- [ ] Theme system cho UI
- [ ] Statistics & analytics (gõ bao nhiêu từ/ngày)

### Cải Thiện Kỹ Thuật

- [ ] Migration sang Rust 2024 edition
- [ ] Async processing cho engine
- [ ] Better error handling & reporting
- [ ] Performance profiling dashboard
- [ ] Memory leak detection
- [ ] Fuzz testing cho engine

---

## 🐛 Known Issues

### Issues Hiện Tại

| Issue | Mức độ | Trạng thái | Ghi chú |
|-------|--------|------------|---------|
| Xung đột với IBus | Medium | Open | Cần disable IBus khi dùng Fcitx5 |
| Wayland support hạn chế | Medium | Investigating | Một số apps không nhận input |
| OSD chưa có | Low | Planned | Sẽ thêm trong Sprint 2 |
| GUI config chưa có | Low | Planned | Sẽ thêm trong Sprint 2 |

### Báo Cáo Issue

Người dùng có thể báo issue tại:
- **GitHub Issues:** https://github.com/marixdev/vnkey/issues
- **Email:** support@vnkey.app

---

## 📊 Metrics & Goals

### Mục Tiêu 2026

| Metric | Current | Target |
|--------|---------|--------|
| Users | 100+ | 10,000+ |
| GitHub Stars | 50+ | 500+ |
| Issues Resolved | 20+ | 100+ |
| Release Frequency | 1/quarter | 1/month |
| Test Coverage | 40% | 80%+ |

---

## 🤝 Đóng Góp

Chúng tôi chào đón mọi đóng góp! Xem [`CONTRIBUTING.md`](../CONTRIBUTING.md) để biết cách đóng góp.

### Cần Giúp Đỡ

- [ ] Documentation writers
- [ ] Testers (trên các distro khác nhau)
- [ ] UI/UX designers
- [ ] Rust developers
- [ ] C++ developers

---

## 📞 Liên Hệ

- **Website:** https://vnkey.app
- **GitHub:** https://github.com/marixdev/vnkey
- **Discord:** (coming soon)
- **Email:** hi@vnkey.app

---

**Cập nhật cuối:** 2026-04-01  
**Người cập nhật:** VnKey Development Team