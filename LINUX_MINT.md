# VnKey cho Linux Mint

Hướng dẫn cài đặt và sử dụng VnKey Fcitx5 trên Linux Mint.

## Giới thiệu

VnKey là bộ gõ tiếng Việt mã nguồn mở cho Linux, sử dụng framework Fcitx5. Engine được viết bằng Rust, hỗ trợ:

- **Kiểu gõ:** Telex, Simple Telex, VNI, VIQR
- **Bảng mã:** Unicode UTF-8, TCVN3, VNI Windows, VISCII, VPS, VIQR, NCR, CP-1258, và hơn 15 bảng mã khác
- **Tính năng:** Kiểm tra chính tả, bỏ dấu tự do, kiểu mới (oà, uý), chuyển đổi bảng mã clipboard

## Cài đặt từ package (.deb)

### Bước 1: Cài đặt Fcitx5

Nếu chưa cài Fcitx5, mở Terminal và chạy:

```bash
sudo apt update
sudo apt install fcitx5 fcitx5-configtool fcitx5-table-extra
```

### Bước 2: Tải VnKey

Tải file `.deb` mới nhất từ [trang phát hành](https://github.com/marixdev/vnkey/releases):

```bash
wget https://github.com/marixdev/vnkey/releases/latest/download/vnkey-fcitx5.deb -O ~/Downloads/vnkey-fcitx5.deb
```

### Bước 3: Cài đặt

```bash
sudo dpkg -i ~/Downloads/vnkey-fcitx5.deb
sudo apt install -f  # Cài đặt dependencies nếu thiếu
```

### Bước 4: Cấu hình Fcitx5

1. Mở **Fcitx5 Configuration** từ menu ứng dụng
2. Vào tab **Available Input Method**
3. Tìm **VnKey** và nhấn nút `+` để thêm vào **Current Input Method**
4. Dùng nút ↑↓ để di chuyển VnKey lên đầu danh sách (tùy chọn)
5. Đóng cửa sổ cấu hình

### Bước 5: Khởi động lại Fcitx5

```bash
fcitx5 -r
```

Hoặc logout và login lại.

---

## Build từ source

### Yêu cầu

- Rust toolchain
- CMake ≥ 3.16
- Fcitx5 development libraries

### Cách 1: Build tự động (khuyến nghị)

```bash
cd vnkey/vnkey-fcitx5
./scripts/build-linux-mint.sh install
```

Script sẽ tự động:
- Kiểm tra và cài đặt dependencies
- Cài đặt Rust nếu chưa có
- Build engine và Fcitx5 addon
- Cài đặt vào hệ thống

### Cách 2: Build thủ công

#### 1. Cài đặt dependencies

```bash
sudo apt update
sudo apt install -y \
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
```

#### 2. Cài đặt Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### 3. Build engine

```bash
cd vnkey/vnkey-engine
cargo build --release
```

#### 4. Build Fcitx5 addon

```bash
cd vnkey/vnkey-fcitx5
mkdir -p build && cd build
cmake .. \
    -DCMAKE_INSTALL_PREFIX=/usr \
    -DCMAKE_BUILD_TYPE=Release \
    -DVNKEY_ENGINE_LIB_DIR=../../vnkey-engine/target/release
make -j$(nproc)
```

#### 5. Cài đặt

```bash
sudo cmake --install .
sudo ldconfig
sudo bash ../scripts/postinst  # Nếu có file postinst
```

---

## Sử dụng

### Phím tắt mặc định

| Phím | Chức năng |
|------|-----------|
| `Ctrl+Space` | Bật/tắt chế độ tiếng Việt |
| `Shift` | Tạm thời gõ tiếng Anh (khi đang ở chế độ tiếng Việt) |
| `Ctrl+Shift` | Chuyển kiểu gõ (Telex → VNI → VIQR) |

### Mở menu cấu hình

- Click phải vào icon Fcitx5 ở system tray
- Chọn **VnKey** để truy cập các tùy chọn:
  - **Input Method:** Chọn kiểu gõ (Telex, VNI, VIQR, ...)
  - **Charset:** Chọn bảng mã (Unicode, TCVN3, VNI Windows, ...)
  - **Preferences:** Mở cửa sổ cấu hình chi tiết

### Cấu hình chi tiết

1. Mở **Fcitx5 Configuration** từ menu ứng dụng
2. Chọn **VnKey** trong danh sách
3. Nhấn nút **Configure** để mở cấu hình chi tiết

Các tùy chọn có thể cấu hình:
- Kiểu gõ mặc định
- Bảng mã mặc định
- Bật/tắt kiểm tra chính tả
- Bật/tắt kiểu mới (oà, uý)
- Bật/tắt bỏ dấu tự do

---

## Gỡ cài đặt

### Nếu cài từ .deb

```bash
sudo apt remove vnkey-fcitx5
```

### Nếu build từ source

```bash
cd vnkey/vnkey-fcitx5/build
sudo cmake --install --manifest install_manifest.txt --uninstall
sudo ldconfig
```

---

## Troubleshooting

### Fcitx5 không hiển thị icon trong system tray

```bash
sudo apt install fcitx5-frontend-gtk3 fcitx5-frontend-gtk4 fcitx5-frontend-qt5
fcitx5 -r
```

### VnKey không xuất hiện trong danh sách Input Method

1. Kiểm tra file cài đặt:
   ```bash
   ls /usr/share/fcitx5/inputmethod/
   ```
   File `vnkey.conf` phải tồn tại.

2. Chạy lại post-install script:
   ```bash
   sudo bash /usr/share/vnkey/postinst
   ```

3. Restart Fcitx5:
   ```bash
   fcitx5 -r
   ```

### Không gõ được tiếng Việt

1. Kiểm tra VnKey đã được chọn chưa:
   - Click vào icon Fcitx5 ở system tray
   - Đảm bảo **VnKey** được chọn (có dấu tick)

2. Kiểm tra phím tắt:
   - Nhấn `Ctrl+Space` để bật chế độ tiếng Việt
   - Icon Fcitx5 sẽ chuyển từ bàn phím sang chữ V

3. Kiểm tra config:
   ```bash
   cat ~/.config/vnkey/config.json
   ```

### Xung đột với IBus

Nếu bạn đang dùng IBus, có thể xảy ra xung đột với Fcitx5. Gỡ IBus:

```bash
sudo apt remove ibus ibus-gtk ibus-gtk3
```

Hoặc disable IBus trong **System Settings → Language Support**.

---

## Cập nhật

### Từ package

```bash
sudo apt update
sudo apt install --reinstall vnkey-fcitx5
```

### Từ source

```bash
cd vnkey/vnkey-fcitx5
git pull
./scripts/build-linux-mint.sh install
```

---

## Đóng góp

Mọi đóng góp xin gửi về repository: https://github.com/marixdev/vnkey

## Giấy phép

VnKey được phát hành theo giấy phép **GPL-3.0-or-later**.