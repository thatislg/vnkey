# Hướng Dẫn Build VnKey Fcitx5

**Phiên bản:** 1.0.1  
**Cập nhật:** 2026-04-01  
**Nền tảng:** Linux Mint / Ubuntu / Debian

---

## 📋 Mục Lục

1. [Yêu Cầu Hệ Thống](#yêu-cầu-hệ-thống)
2. [Cài Đặt Dependencies](#cài-đặt-dependencies)
3. [Build Engine (Rust)](#build-engine-rust)
4. [Build Addon (C++)](#build-addon-c)
5. [Tạo Package](#tạo-package)
6. [Cài Đặt](#cài-đặt)
7. [Verify Build](#verify-build)
8. [Kiểm Tra Version](#kiểm-tra-version)
9. [Troubleshooting](#troubleshooting)

---

## 🖥️ Yêu Cầu Hệ Thống

### Tối thiểu

- **Hệ điều hành:** Linux Mint 21+, Ubuntu 22.04+, Debian 12+
- **CPU:** x86_64 (AMD64)
- **RAM:** 2GB
- **Dung lượng:** 500MB trống

### Khuyến nghị

- **RAM:** 4GB+
- **CMake:** ≥ 3.16
- **Rust:** Phiên bản mới nhất (rustup)
- **Fcitx5:** ≥ 5.0

---

## 📦 Cài Đặt Dependencies

### Cách 1: Tự động (Khuyến nghị)

Sử dụng script tự động cài đặt tất cả dependencies:

```bash
cd ~/Developer/vnkey/vnkey-fcitx5
./scripts/build-linux-mint.sh install-deps
```

Script sẽ tự động:
- Cài đặt build tools (cmake, make, gcc)
- Cài đặt Fcitx5 development libraries
- Cài đặt Rust toolchain (nếu chưa có)

### Cách 2: Thủ công

#### Bước 1: Cài đặt build tools

```bash
sudo apt update
sudo apt install -y \
    curl \
    build-essential \
    cmake \
    pkg-config \
    ca-certificates
```

#### Bước 2: Cài đặt Fcitx5 development libraries

```bash
sudo apt install -y \
    fcitx5 \
    fcitx5-configtool \
    libfcitx5core-dev \
    libfcitx5config-dev \
    libfcitx5utils-dev \
    fcitx5-modules-dev \
    extra-cmake-modules \
    libglib2.0-dev
```

#### Bước 3: Cài đặt Rust

```bash
# Download và chạy rustup installer
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Load Rust environment vào shell hiện tại
source $HOME/.cargo/env

# Kiểm tra cài đặt
rustc --version
cargo --version
```

---

## ⚙️ Build Engine (Rust)

### Build release (khuyến nghị)

```bash
cd ~/Developer/vnkey/vnkey-engine
cargo build --release
```

**Output:**
```
target/release/libvnkey_engine.a  (~2-5MB)
```

### Build debug (cho development)

```bash
cd ~/Developer/vnkey/vnkey-engine
cargo build
```

**Output:**
```
target/debug/libvnkey_engine.a
```

### Clean build (xóa cache và build lại từ đầu)

```bash
cd ~/Developer/vnkey/vnkey-engine
cargo clean
cargo build --release
```

### Kiểm tra build thành công

```bash
# Kiểm tra file tồn tại
ls -lh target/release/libvnkey_engine.a

# Kiểm tra symbols
nm target/release/libvnkey_engine.a | grep vnkey_engine | head -10
```

**Kết quả mong đợi:**
```
-rw-r--r-- 1 user user 2.5M ... libvnkey_engine.a
0000000000001234 T vnkey_engine_new
0000000000001567 T vnkey_engine_process
0000000000001890 T vnkey_engine_reset
```

---

## 🔨 Build Addon (C++)

### Yêu cầu trước khi build

Engine phải được build trước và file `libvnkey_engine.a` phải tồn tại:

```bash
ls ~/Developer/vnkey/vnkey-engine/target/release/libvnkey_engine.a
```

### Build tự động (Khuyến nghị)

```bash
cd ~/Developer/vnkey/vnkey-fcitx5
./scripts/build-linux-mint.sh
```

Script sẽ tự động:
1. Kiểm tra dependencies
2. Build engine (nếu chưa build)
3. Build Fcitx5 addon
4. Tạo package .deb

### Build thủ công

#### Bước 1: Tạo build directory

```bash
cd ~/Developer/vnkey/vnkey-fcitx5
mkdir -p build
cd build
```

#### Bước 2: Chạy CMake configure

```bash
cmake .. \
    -DCMAKE_INSTALL_PREFIX=/usr \
    -DCMAKE_BUILD_TYPE=Release \
    -DVNKEY_ENGINE_LIB_DIR=../../vnkey-engine/target/release
```

**Giải thích options:**
- `CMAKE_INSTALL_PREFIX`: Đường dẫn cài đặt
- `CMAKE_BUILD_TYPE`: Release (optimized) hoặc Debug
- `VNKEY_ENGINE_LIB_DIR`: Đường dẫn tới engine library

#### Bước 3: Build

```bash
make -j$(nproc)
```

**Output:**
```
[ 33%] Building CXX object CMakeFiles/vnkey.dir/src/vnkey-fcitx5.cpp.o
[ 66%] Building C object CMakeFiles/vnkey.dir/src/glibc-compat.c.o
[100%] Linking CXX shared module libvnkey.so
[100%] Built target vnkey
```

#### Bước 4: Kiểm tra build

```bash
ls -lh libvnkey.so
# Output: -rwxrwxr-x 1 user user 1.5M ... libvnkey.so
```

---

## 📦 Tạo Package

### Tạo .deb package (Debian/Ubuntu/Linux Mint)

```bash
cd ~/Developer/vnkey/vnkey-fcitx5/build
cpack -G DEB
```

**Output:**
```
CPack: Create package using DEB
CPack: Install projects
CPack: - package: /path/to/vnkey-fcitx5_1.0.1_amd64.deb generated.
```

### Tạo .rpm package (Fedora/RHEL)

```bash
cd ~/Developer/vnkey/vnkey-fcitx5/build
cpack -G RPM
```

### Kiểm tra package

```bash
# Xem thông tin package
dpkg -I vnkey-fcitx5_*.deb

# Xem nội dung package
dpkg -c vnkey-fcitx5_*.deb
```

---

## 📥 Cài Đặt

### Cài đặt từ package .deb

```bash
cd ~/Developer/vnkey/vnkey-fcitx5/build
sudo dpkg -i vnkey-fcitx5_*.deb
sudo apt install -f  # Cài dependencies nếu thiếu
```

### Cài đặt trực tiếp (không tạo package)

```bash
cd ~/Developer/vnkey/vnkey-fcitx5/build
sudo cmake --install .
sudo ldconfig
sudo bash ../scripts/postinst
```

### Restart Fcitx5

```bash
fcitx5 -r
```

Hoặc logout và login lại.

---

## ✅ Verify Build

### Sử dụng verify script

```bash
cd ~/Developer/vnkey/vnkey-fcitx5
./scripts/verify-build.sh
```

### Kiểm tra thủ công

```bash
# 1. Kiểm tra engine library
ls -lh vnkey-engine/target/release/libvnkey_engine.a

# 2. Kiểm tra addon library
ls -lh vnkey-fcitx5/build/libvnkey.so

# 3. Kiểm tra package
ls -lh vnkey-fcitx5/build/*.deb

# 4. Kiểm tra installation
ls /usr/share/fcitx5/inputmethod/vnkey.conf
ls /usr/lib/x86_64-linux-gnu/fcitx5/libvnkey.so

# 5. Kiểm tra Fcitx5 nhận diện
fcitx5-remote -l | grep -i vnkey
```

---

## 🔍 Kiểm Tra Version

Để phân biệt **VnKey custom build** (tự build từ source) và **VnKey official** (từ GitHub releases), sử dụng các lệnh sau:

### 1. Kiểm tra nhanh version

```bash
# Xem version package đã cài
dpkg -l | grep vnkey

# Output:
# ii  vnkey-fcitx5  1.0.1  amd64  VnKey Vietnamese Input Method for Fcitx5
```

### 2. Kiểm tra version chi tiết

```bash
cd ~/Developer/vnkey/vnkey-fcitx5
./scripts/check-version.sh
```

**Script sẽ hiển thị:**
- Version info từ `VERSION_INFO` (nếu tự build)
- Version từ package đã cài
- Library đang active
- Fcitx5 status
- Git commit hash (nếu tự build)
- So sánh với GitHub latest release

### 3. Phân biệt Custom vs Official

| Đặc điểm | Custom Build | Official Release |
|----------|--------------|------------------|
| **VERSION_INFO** | ✅ Có file | ❌ Không có |
| **BUILD_TYPE** | `CUSTOM_BUILD` | `OFFICIAL_RELEASE` |
| **Git Commit** | Hiển thị commit hash | N/A |
| **Build Date** | Hiển thị ngày build | N/A |
| **Package Source** | Tự build | GitHub Releases |

### 4. Kiểm tra từ terminal

```bash
# Check version package
dpkg -l | grep vnkey-fcitx5 | awk '{print "Version: "$3}'

# Check installation date
stat -c %y /var/lib/dpkg/info/vnkey-fcitx5.list

# Check git commit (nếu build từ source)
cd ~/Developer/vnkey && git rev-parse --short HEAD

# Check uncommitted changes
cd ~/Developer/vnkey && git status --porcelain
```

### 5. Ví dụ output

**Custom Build:**
```
═══ VERSION INFO (từ build) ═══
VNKEY_VERSION=1.0.1
VNKEY_BUILD_DATE=2026-04-01 01:30:00 UTC
VNKEY_GIT_COMMIT=a1b2c3d
VNKEY_GIT_BRANCH=main
VNKEY_BUILD_TYPE=CUSTOM_BUILD

→ Đây là CUSTOM BUILD (tự build từ source)
```

**Official Release:**
```
═══ INSTALLED PACKAGE ═══
✓ Package: vnkey-fcitx5
✓ Version: 1.0.1
✓ Arch:    amd64

═══ SUMMARY ═══
→ Đây là OFFICIAL RELEASE (từ GitHub)
```

---

## 🐛 Troubleshooting

### Lỗi: Không thể xác định version

**Nguyên nhân:** Package được cài từ GitHub releases (không có VERSION_INFO).

**Giải pháp:**
```bash
# Check version từ package
dpkg -l | grep vnkey-fcitx5

# Check GitHub latest release
curl -s https://api.github.com/repos/marixdev/vnkey/releases/latest | grep '"tag_name"'
```

### Lỗi: "libvnkey_engine.a not found"

**Nguyên nhân:** Engine chưa được build.

**Giải pháp:**
```bash
cd ~/Developer/vnkey/vnkey-engine
cargo build --release
```

### Lỗi: "Fcitx5Core not found"

**Nguyên nhân:** Thiếu Fcitx5 development libraries.

**Giải pháp:**
```bash
sudo apt install libfcitx5core-dev libfcitx5config-dev libfcitx5utils-dev
```

### Lỗi: "cmake not found"

**Nguyên nhân:** Chưa cài CMake.

**Giải pháp:**
```bash
sudo apt install cmake
```

### Lỗi: "cargo not found"

**Nguyên nhân:** Chưa cài Rust.

**Giải pháp:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Lỗi: "package configuration file provided by Fcitx5Module not found"

**Nguyên nhân:** Thiếu `extra-cmake-modules`.

**Giải pháp:**
```bash
sudo apt install extra-cmake-modules
```

### Build thành công nhưng Fcitx5 không nhận diện

**Kiểm tra:**
```bash
# 1. File cài đặt có tồn tại không?
ls /usr/share/fcitx5/inputmethod/vnkey.conf

# 2. Fcitx5 có đang chạy không?
pgrep -x fcitx5

# 3. Restart Fcitx5
fcitx5 -r

# 4. Kiểm tra logs
journalctl -xe | grep -i vnkey
```

### Gõ không được tiếng Việt

**Kiểm tra:**
```bash
# 1. VnKey đã được chọn chưa?
fcitx5-remote  # 1 = keyboard, 2 = vnkey

# 2. Thêm VnKey vào Fcitx5 Configuration
# Mở Fcitx5 Configuration → Available Input Method → Thêm VnKey

# 3. Restart Fcitx5
fcitx5 -r
```

---

## 📊 Build Time Reference

| Thành phần | Lần đầu | Lần sau (incremental) |
|------------|---------|----------------------|
| Engine (Rust) | 2-5 phút | 30-60 giây |
| Addon (C++) | 30-60 giây | 10-20 giây |
| Package (.deb) | 5-10 giây | 5-10 giây |

---

## 📝 Tips

### Build nhanh cho development

```bash
# Build debug (nhanh hơn, có debug symbols)
cd vnkey-engine && cargo build
cd ../vnkey-fcitx5 && ./scripts/build-linux-mint.sh
```

### Rebuild chỉ khi có thay đổi

```bash
# Chỉ build lại addon (engine không đổi)
cd vnkey-fcitx5/build
make -j$(nproc)
```

### Xem log chi tiết khi build

```bash
# Rust build với verbose output
RUST_LOG=debug cargo build --release

# CMake build với verbose output
cmake .. -DCMAKE_VERBOSE_MAKEFILE=ON
```

---

## 📞 Hỗ Trợ

- **Documentation:** `/docs/` folder
- **Issues:** https://github.com/marixdev/vnkey/issues
- **Website:** https://vnkey.app