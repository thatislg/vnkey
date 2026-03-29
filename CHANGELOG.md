# Changelog

## 1.0.0 — 2026-03-29

Phiên bản đầu tiên phát hành công khai.

### vnkey-engine (Rust core)
- 4 kiểu gõ: Telex, Simple Telex, VNI, VIQR
- 15 bảng mã: Unicode UTF-8, TCVN3, VNI Windows, VISCII, VPS, VIQR, NCR, CP-1258, …
- Kiểm tra chính tả tự động
- Bỏ dấu tự do / theo quy tắc
- Kiểu mới (oà, uý)
- Chuyển đổi bảng mã (charset_from_utf8 / charset_to_utf8)
- C FFI (staticlib) cho tích hợp đa ngôn ngữ

### vnkey-windows
- Giao diện Win32 native + Direct2D
- System tray icon với menu đầy đủ
- OSD toast khi chuyển Việt/Anh
- Cửa sổ cấu hình (kiểu gõ, bảng mã, tuỳ chọn)
- Công cụ chuyển đổi bảng mã clipboard
- Loại trừ ứng dụng (blacklist)
- Gán phím tắt tuỳ chỉnh
- Khởi động cùng Windows
- Cửa sổ giới thiệu với link clickable

### vnkey-fcitx5
- Fcitx5 input method addon cho Linux
- Preedit với underline
- Menu chuột phải: kiểu gõ, bảng mã, tuỳ chọn
- Chuyển đổi bảng mã clipboard (wl-paste/xclip/xsel)
- Gói .deb (Debian 12+, Ubuntu 22.04+), .rpm (Fedora 41+), .pkg.tar.zst (Arch Linux), .tar.gz (NixOS)
- GLIBC compat shims cho tương thích rộng (glibc 2.34+)

### vnkey-ibus
- IBus input method engine cho Linux
- Preedit với underline
- Property menu: kiểu gõ, bảng mã, tuỳ chọn
- Chuyển đổi bảng mã clipboard
- Gói .deb (Debian 12+, Ubuntu 22.04+), .rpm (Fedora 41+), .pkg.tar.zst (Arch Linux), .tar.gz (NixOS)
- GLIBC compat shims cho tương thích rộng (glibc 2.34+)
