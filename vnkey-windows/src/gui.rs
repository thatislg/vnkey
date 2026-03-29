//! Cửa sổ cấu hình — dùng Win32 native controls.\n//! Chạy trên thread nền, có message loop riêng.

use crate::ui;
use crate::ENGINE;
use std::sync::atomic::{AtomicBool, Ordering};

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

static GUI_OPEN: AtomicBool = AtomicBool::new(false);

// ── Mã điều khiển ───────────────────────────────────────────────────────

const ID_CB_CHARSET: u16   = 100;
const ID_CB_INPUT: u16     = 101;
const ID_CHK_SPELL: u16    = 110;
const ID_CHK_FREE: u16     = 111;
const ID_CHK_MODERN: u16   = 112;
const ID_CHK_AUTO: u16     = 113;
const ID_BTN_BLACKLIST: u16 = 120;
const ID_BTN_CONVERTER: u16 = 121;
const ID_BTN_HOTKEY: u16   = 122;
const ID_BTN_CLOSE: u16    = 130;
const ID_BTN_EXIT: u16     = 131;
const ID_LINK_INFO: u16    = 140;
const ID_GROUPBOX1: u16    = 150;
const ID_GROUPBOX2: u16    = 151;
const ID_GROUPBOX3: u16    = 152;

pub fn open_config_window() {
    if GUI_OPEN.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(|| {
        run_config();
        GUI_OPEN.store(false, Ordering::Relaxed);
    });
}

fn run_config() {
    ui::init_common_controls();

    let hwnd = ui::create_dialog_window(
        w!("VnKeyConfig"),
        &format!("VnKey {VERSION}"),
        410, 272,
        Some(config_wnd_proc),
    );

    if hwnd.0.is_null() { return; }

    create_controls(hwnd);

    ui::show_and_focus(hwnd);
    ui::run_dialog_loop(hwnd);
}

fn create_controls(hwnd: HWND) {
    let font = ui::create_ui_font();

    // Đọc trạng thái hiện tại
    let (im, cs, spell, free, modern) = {
        let g = ENGINE.lock().unwrap_or_else(|e| e.into_inner());
        match g.as_ref() {
            Some(s) => (s.input_method, s.output_charset, s.spell_check, s.free_marking, s.modern_style),
            None => (0, 1, true, true, true),
        }
    };
    let auto_start = is_auto_start_enabled();

    // ── Groupbox: Bảng mã ──
    ui::create_groupbox("Bảng mã", 10, 5, 190, 52, hwnd, ID_GROUPBOX1, font);
    let cs_items = &[
        "Unicode", "UTF-8", "NCR Decimal", "NCR Hex", "CP-1258",
        "VIQR", "TCVN3 (ABC)", "VPS", "VISCII", "VNI Windows", "VNI Mac",
    ];
    ui::create_combobox(22, 22, 166, 300, hwnd, ID_CB_CHARSET, font, cs_items, cs_index(cs));

    // ── Groupbox: Kiểu gõ ──
    ui::create_groupbox("Kiểu gõ", 210, 5, 190, 52, hwnd, ID_GROUPBOX2, font);
    let im_items = &["Telex", "Simple Telex", "VNI", "VIQR", "MS Vietnamese"];
    ui::create_combobox(222, 22, 166, 200, hwnd, ID_CB_INPUT, font, im_items, im as usize);

    // ── Groupbox: Tùy chọn ──
    ui::create_groupbox("Tùy chọn", 10, 62, 390, 130, hwnd, ID_GROUPBOX3, font);

    // Ô chọn (2 cột)
    ui::create_checkbox("Kiểm tra chính tả", 22, 80, 180, 20, hwnd, ID_CHK_SPELL, font, spell);
    ui::create_checkbox("Bỏ dấu tự do", 210, 80, 180, 20, hwnd, ID_CHK_FREE, font, free);
    ui::create_checkbox("Kiểu mới (oà, uý)", 22, 102, 180, 20, hwnd, ID_CHK_MODERN, font, modern);
    ui::create_checkbox("Khởi động cùng Windows", 210, 102, 180, 20, hwnd, ID_CHK_AUTO, font, auto_start);

    // Hàng nút chức năng
    ui::create_button("🚫 Loại trừ", 22, 130, 118, 28, hwnd, ID_BTN_BLACKLIST, font);
    ui::create_button("🔄 Chuyển mã", 146, 130, 118, 28, hwnd, ID_BTN_CONVERTER, font);
    ui::create_button("⌨ Gán phím", 270, 130, 118, 28, hwnd, ID_BTN_HOTKEY, font);

    // ── Các nút phía dưới ──
    ui::create_button("Đóng", 10, 200, 190, 32, hwnd, ID_BTN_CLOSE, font);
    ui::create_button("Thoát VnKey", 210, 200, 190, 32, hwnd, ID_BTN_EXIT, font);

    // ── Liên kết giới thiệu (dùng nút phẳng) ──
    ui::create_button("ℹ Giới thiệu", 150, 240, 110, 22, hwnd, ID_LINK_INFO, font);
}

// ── Thủ tục cửa sổ ────────────────────────────────────────────────────────

unsafe extern "system" fn config_wnd_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as u16;
            let notify = ((wparam.0 >> 16) & 0xFFFF) as u16;
            handle_command(hwnd, id, notify);
            LRESULT(0)
        }
        WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => {
            // Đặt nền cho ô chọn / nhãn / nhóm
            let hdc = HDC(wparam.0 as _);
            SetBkColor(hdc, ui::BG_COLOR);
            SetTextColor(hdc, COLORREF(0x001A1A1A));
            LRESULT(ui::BG_BRUSH().0 as _)
        }
        WM_CLOSE => {
            DestroyWindow(hwnd).ok();
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn handle_command(hwnd: HWND, id: u16, notify: u16) {
    match id {
        ID_CB_CHARSET if notify == CBN_SELCHANGE as u16 => {
            let cb = unsafe { GetDlgItem(hwnd, ID_CB_CHARSET as i32) }.unwrap_or_default();
            let idx = unsafe { SendMessageW(cb, CB_GETCURSEL, WPARAM(0), LPARAM(0)) }.0 as usize;
            let cs_val = cs_value(idx);
            if let Ok(mut g) = ENGINE.lock() {
                if let Some(s) = g.as_mut() {
                    s.output_charset = cs_val;
                }
            }
            crate::config::save();
        }
        ID_CB_INPUT if notify == CBN_SELCHANGE as u16 => {
            let cb = unsafe { GetDlgItem(hwnd, ID_CB_INPUT as i32) }.unwrap_or_default();
            let idx = unsafe { SendMessageW(cb, CB_GETCURSEL, WPARAM(0), LPARAM(0)) }.0 as i32;
            if let Ok(mut g) = ENGINE.lock() {
                if let Some(s) = g.as_mut() {
                    s.set_input_method(idx);
                }
            }
            crate::config::save();
        }
        ID_CHK_SPELL if notify == BN_CLICKED as u16 => {
            let chk = unsafe { GetDlgItem(hwnd, ID_CHK_SPELL as i32) }.unwrap_or_default();
            let checked = unsafe { SendMessageW(chk, BM_GETCHECK, WPARAM(0), LPARAM(0)) }.0 == 1;
            if let Ok(mut g) = ENGINE.lock() {
                if let Some(s) = g.as_mut() { s.spell_check = checked; s.sync_options(); }
            }
            crate::config::save();
        }
        ID_CHK_FREE if notify == BN_CLICKED as u16 => {
            let chk = unsafe { GetDlgItem(hwnd, ID_CHK_FREE as i32) }.unwrap_or_default();
            let checked = unsafe { SendMessageW(chk, BM_GETCHECK, WPARAM(0), LPARAM(0)) }.0 == 1;
            if let Ok(mut g) = ENGINE.lock() {
                if let Some(s) = g.as_mut() { s.free_marking = checked; s.sync_options(); }
            }
            crate::config::save();
        }
        ID_CHK_MODERN if notify == BN_CLICKED as u16 => {
            let chk = unsafe { GetDlgItem(hwnd, ID_CHK_MODERN as i32) }.unwrap_or_default();
            let checked = unsafe { SendMessageW(chk, BM_GETCHECK, WPARAM(0), LPARAM(0)) }.0 == 1;
            if let Ok(mut g) = ENGINE.lock() {
                if let Some(s) = g.as_mut() { s.modern_style = checked; s.sync_options(); }
            }
            crate::config::save();
        }
        ID_CHK_AUTO if notify == BN_CLICKED as u16 => {
            let chk = unsafe { GetDlgItem(hwnd, ID_CHK_AUTO as i32) }.unwrap_or_default();
            let checked = unsafe { SendMessageW(chk, BM_GETCHECK, WPARAM(0), LPARAM(0)) }.0 == 1;
            set_auto_start(checked);
        }
        ID_BTN_BLACKLIST => { crate::blacklist::open_blacklist_window(); }
        ID_BTN_CONVERTER => { crate::converter::open_converter_window(); }
        ID_BTN_HOTKEY => { crate::hotkey::open_hotkey_window(); }
        ID_BTN_CLOSE => { unsafe { DestroyWindow(hwnd).ok(); } }
        ID_BTN_EXIT => {
            unsafe {
                DestroyWindow(hwnd).ok();
                if let Ok(mw) = FindWindowW(w!("VnKeyHiddenWindow"), w!("VnKey")) {
                    PostMessageW(mw, WM_CLOSE, WPARAM(0), LPARAM(0)).ok();
                }
            }
        }
        ID_LINK_INFO => { crate::info::open_info_window(); }
        _ => {}
    }
}

// ── Ánh xạ chỉ mục bảng mã ─────────────────────────────────────────────────

const CS_IDS: [i32; 11] = [0, 1, 2, 3, 5, 10, 20, 21, 22, 40, 43];

fn cs_index(cs: i32) -> usize {
    CS_IDS.iter().position(|&v| v == cs).unwrap_or(0)
}

fn cs_value(idx: usize) -> i32 {
    CS_IDS.get(idx).copied().unwrap_or(1)
}

// ── Tự khởi động (registry) ──────────────────────────────────────────────

fn is_auto_start_enabled() -> bool {
    use std::os::windows::process::CommandExt;
    std::process::Command::new("reg")
        .args(["query", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run", "/v", "VnKey"])
        .creation_flags(0x08000000)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn set_auto_start(enable: bool) {
    use std::os::windows::process::CommandExt;
    if enable {
        let exe = std::env::current_exe()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let _ = std::process::Command::new("reg")
            .args(["add", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                   "/v", "VnKey", "/t", "REG_SZ", "/d", &format!("\"{exe}\""), "/f"])
            .creation_flags(0x08000000).output();
    } else {
        let _ = std::process::Command::new("reg")
            .args(["delete", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                   "/v", "VnKey", "/f"])
            .creation_flags(0x08000000).output();
    }
}

/// Đặt icon cửa sổ từ tài nguyên exe.
#[allow(dead_code)]
pub fn set_window_icon(hwnd: HWND) {
    unsafe {
        let hinstance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
            .unwrap_or_default();
        if let Ok(icon) = LoadIconW(hinstance, PCWSTR(101 as *const u16)) {
            SendMessageW(hwnd, WM_SETICON, WPARAM(0), LPARAM(icon.0 as isize));
            SendMessageW(hwnd, WM_SETICON, WPARAM(1), LPARAM(icon.0 as isize));
        }
    }
}
