//! Gửi phím qua SendInput sau khi hook trả về
//!
//! Mặc định dùng Shift+Left để chọn và thay thế ký tự (tránh Chrome autocomplete
//! nuốt VK_BACK). Fallback về VK_BACK cho các app không hỗ trợ Shift+Left (WPS Office, v.v.).
//! Tất cả sự kiện được đánh dấu VNKEY_INJECTED_TAG để hook bỏ qua.

use crate::{PENDING_OUTPUT, VNKEY_INJECTED_TAG};

use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::UI::Input::KeyboardAndMouse::*;

/// Dùng VK_BACK thay vì Shift+Left cho ứng dụng hiện tại.
/// Được cập nhật từ hook khi detect foreground app.
static USE_VK_BACK: AtomicBool = AtomicBool::new(false);

/// Danh sách process cần dùng VK_BACK (Shift+Left không hoạt động)
const VK_BACK_APPS: &[&str] = &[
    "wps.exe", "wpp.exe", "et.exe",     // WPS Office
    "WINWORD.EXE", "EXCEL.EXE", "POWERPNT.EXE", // MS Office
];

/// Kiểm tra và cập nhật phương thức backspace dựa trên ứng dụng hiện tại.
/// Gọi từ hook mỗi lần foreground thay đổi.
pub fn update_backspace_method() {
    let use_vk = crate::blacklist::get_foreground_exe_cached()
        .map(|exe| VK_BACK_APPS.iter().any(|app| app.eq_ignore_ascii_case(&exe)))
        .unwrap_or(false);
    USE_VK_BACK.store(use_vk, Ordering::Relaxed);
}

pub fn send_pending_output() {
    let pending = {
        let Ok(mut guard) = PENDING_OUTPUT.lock() else { return };
        guard.take()
    };

    if let Some(output) = pending {
        if let Some(ref raw_bytes) = output.raw_bytes {
            send_backspaces_and_raw(output.backspaces, raw_bytes);
        } else {
            send_backspaces_and_text(output.backspaces, &output.text);
        }
    }
}

/// Gửi đầu ra trực tiếp (gọi từ hook callback để chuyển ngay).
pub fn send_output(backspaces: usize, text: &str, raw_bytes: Option<&[u8]>) {
    if let Some(raw) = raw_bytes {
        send_backspaces_and_raw(backspaces, raw);
    } else {
        send_backspaces_and_text(backspaces, text);
    }
}

/// Inject lại một ký tự ASCII qua SendInput (KEYEVENTF_UNICODE).
/// Dùng khi engine không xử lý phím nhưng cần mọi ký tự
/// đi qua cùng đường SendInput để backspace đúng trong trình duyệt.
pub fn send_char(ascii: u8) {
    let inputs = [
        make_unicode_input(ascii as u16, KEYBD_EVENT_FLAGS(0), VNKEY_INJECTED_TAG),
        make_unicode_input(ascii as u16, KEYEVENTF_KEYUP, VNKEY_INJECTED_TAG),
    ];
    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

/// Tạo chuỗi INPUT cho backspace: Shift+Left (select) nếu có text thay thế,
/// hoặc VK_BACK thuần nếu không.
fn build_backspace_inputs(inputs: &mut Vec<INPUT>, backspaces: usize, has_replacement: bool) {
    let force_vk_back = USE_VK_BACK.load(Ordering::Relaxed);

    if backspaces > 0 && has_replacement && !force_vk_back {
        // Dùng Shift+Left để chọn ký tự, rồi gõ văn bản thay thế.
        // Tránh VK_BACK vì Chrome Omnibox autocomplete chặn nó
        // (backspace đầu tiên xóa gợi ý thay vì xóa ký tự).
        inputs.push(make_key_input(VK_SHIFT, KEYBD_EVENT_FLAGS(0), VNKEY_INJECTED_TAG));
        for _ in 0..backspaces {
            inputs.push(make_key_input(VK_LEFT, KEYEVENTF_EXTENDEDKEY, VNKEY_INJECTED_TAG));
            inputs.push(make_key_input(VK_LEFT, KEYBD_EVENT_FLAGS(KEYEVENTF_KEYUP.0 | KEYEVENTF_EXTENDEDKEY.0), VNKEY_INJECTED_TAG));
        }
        inputs.push(make_key_input(VK_SHIFT, KEYEVENTF_KEYUP, VNKEY_INJECTED_TAG));
    } else {
        for _ in 0..backspaces {
            inputs.push(make_key_input(VK_BACK, KEYBD_EVENT_FLAGS(0), VNKEY_INJECTED_TAG));
            inputs.push(make_key_input(VK_BACK, KEYEVENTF_KEYUP, VNKEY_INJECTED_TAG));
        }
    }
}

fn send_backspaces_and_text(backspaces: usize, text: &str) {
    let mut inputs: Vec<INPUT> = Vec::new();

    build_backspace_inputs(&mut inputs, backspaces, !text.is_empty());

    // Văn bản dưới dạng ký tự Unicode (KEYEVENTF_UNICODE)
    // Nếu đang chọn văn bản từ Shift+Left, thao tác này thay thế vùng chọn.
    for ch in text.encode_utf16() {
        inputs.push(make_unicode_input(ch, KEYBD_EVENT_FLAGS(0), VNKEY_INJECTED_TAG));
        inputs.push(make_unicode_input(ch, KEYEVENTF_KEYUP, VNKEY_INJECTED_TAG));
    }

    if !inputs.is_empty() {
        unsafe {
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }
}

fn send_backspaces_and_raw(backspaces: usize, raw_bytes: &[u8]) {
    let mut inputs: Vec<INPUT> = Vec::new();

    build_backspace_inputs(&mut inputs, backspaces, !raw_bytes.is_empty());

    for &b in raw_bytes {
        inputs.push(make_unicode_input(b as u16, KEYBD_EVENT_FLAGS(0), VNKEY_INJECTED_TAG));
        inputs.push(make_unicode_input(b as u16, KEYEVENTF_KEYUP, VNKEY_INJECTED_TAG));
    }

    if !inputs.is_empty() {
        unsafe {
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }
}

fn make_key_input(vk: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS, extra: usize) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: extra,
            },
        },
    }
}

fn make_unicode_input(ch: u16, flags: KEYBD_EVENT_FLAGS, extra: usize) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: ch,
                dwFlags: KEYEVENTF_UNICODE | flags,
                time: 0,
                dwExtraInfo: extra,
            },
        },
    }
}
