//! Cửa sổ chuyển đổi bảng mã và chuyển mã clipboard.

use crate::ui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};

use vnkey_engine::charset::Charset;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::DataExchange::*;
use windows::Win32::System::Memory::*;
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Debug)]
pub struct ConvSettings {
    pub from_charset: usize,
    pub to_charset: usize,
    pub hotkey_vk: u32,
    pub hotkey_modifiers: u32,
}

impl Default for ConvSettings {
    fn default() -> Self {
        Self { from_charset: 0, to_charset: 6, hotkey_vk: 0, hotkey_modifiers: 0 }
    }
}

pub static CONV_SETTINGS: LazyLock<Mutex<ConvSettings>> =
    LazyLock::new(|| Mutex::new(ConvSettings::default()));

static CONV_OPEN: AtomicBool = AtomicBool::new(false);

// ── Bảng mã ───────────────────────────────────────────────────────────

const CS_IDS: [i32; 11] = [0, 1, 2, 3, 5, 10, 20, 21, 22, 40, 43];
const CS_LABELS: [&str; 11] = [
    "Unicode", "UTF-8", "NCR Decimal", "NCR Hex", "CP-1258",
    "VIQR", "TCVN3 (ABC)", "VPS", "VISCII", "VNI Windows", "VNI Mac",
];

fn charset_from_id(id: i32) -> Charset {
    match id {
        0 => Charset::Unicode, 1 => Charset::Utf8,
        2 => Charset::NcrDec, 3 => Charset::NcrHex,
        5 => Charset::WinCP1258, 10 => Charset::Viqr,
        20 => Charset::Tcvn3, 21 => Charset::Vps,
        22 => Charset::Viscii, 40 => Charset::VniWin,
        43 => Charset::VniMac, _ => Charset::Unicode,
    }
}

// ── Mã điều khiển ───────────────────────────────────────────────────────

const ID_CB_FROM: u16     = 200;
const ID_CB_TO: u16       = 201;
const ID_BTN_SWAP: u16    = 202;
const ID_EDIT_TEXT: u16   = 203;
const ID_LBL_STATUS: u16 = 204;
const ID_BTN_CONVERT: u16 = 205;
const ID_BTN_CLIP: u16   = 206;
const ID_BTN_CLOSE: u16  = 207;

// ── Chuyển mã clipboard (gọi từ WM_HOTKEY chính) ───────────────────────────

pub fn convert_clipboard() {
    let (from_idx, to_idx) = {
        let s = match CONV_SETTINGS.lock() { Ok(s) => s, Err(_) => return };
        (s.from_charset, s.to_charset)
    };
    let from_id = CS_IDS.get(from_idx).copied().unwrap_or(0);
    let to_id = CS_IDS.get(to_idx).copied().unwrap_or(0);
    if from_id == to_id { return; }
    let from_cs = charset_from_id(from_id);
    let to_cs = charset_from_id(to_id);
    let from_name = CS_LABELS.get(from_idx).unwrap_or(&"?");
    let to_name = CS_LABELS.get(to_idx).unwrap_or(&"?");
    if let Some(text) = get_clipboard_text() {
        if let Some(converted) = do_convert(&text, from_cs, to_cs) {
            set_clipboard_text(&converted);
            notify_conversion(from_name, to_name, true);
        } else {
            notify_conversion(from_name, to_name, false);
        }
    }
}

fn notify_conversion(from: &str, to: &str, ok: bool) {
    if ok {
        crate::osd::show(&format!("✔ Clipboard: {from} → {to}"));
    } else {
        crate::osd::show(&format!("✘ Lỗi chuyển mã {from} → {to}"));
    }
}

fn do_convert(text: &str, from: Charset, to: Charset) -> Option<String> {
    let (src_bytes, actual_from) = match from {
        Charset::Unicode | Charset::Utf8 => (text.as_bytes().to_vec(), Charset::Utf8),
        Charset::NcrDec | Charset::NcrHex | Charset::Viqr => (text.as_bytes().to_vec(), from),
        _ => (text.chars().map(|c| (c as u32) as u8).collect(), from),
    };
    let result = vnkey_engine::charset::convert(&src_bytes, actual_from, to).ok()?;
    match to {
        Charset::Unicode => {
            let mut u16buf = Vec::with_capacity(result.len() / 2);
            let mut i = 0;
            while i + 1 < result.len() {
                u16buf.push(u16::from_le_bytes([result[i], result[i + 1]]));
                i += 2;
            }
            Some(String::from_utf16_lossy(&u16buf))
        }
        Charset::Utf8 | Charset::NcrDec | Charset::NcrHex | Charset::Viqr => {
            String::from_utf8(result).ok()
        }
        _ => Some(result.iter().map(|&b| b as char).collect()),
    }
}

// ── Trợ giúp clipboard ───────────────────────────────────────────────────

const CF_UNICODETEXT: u32 = 13;

fn get_clipboard_text() -> Option<String> {
    unsafe {
        OpenClipboard(HWND::default()).ok()?;
        let result = (|| {
            let handle = GetClipboardData(CF_UNICODETEXT).ok()?;
            let hmem: HGLOBAL = HGLOBAL(handle.0 as _);
            let ptr = GlobalLock(hmem) as *const u16;
            if ptr.is_null() { return None; }
            let mut len = 0usize;
            while *ptr.add(len) != 0 { len += 1; }
            let slice = std::slice::from_raw_parts(ptr, len);
            let s = String::from_utf16_lossy(slice);
            let _ = GlobalUnlock(hmem);
            Some(s)
        })();
        let _ = CloseClipboard();
        result
    }
}

fn set_clipboard_text(text: &str) {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let byte_len = wide.len() * 2;
    unsafe {
        if OpenClipboard(HWND::default()).is_err() { return; }
        let _ = EmptyClipboard();
        if let Ok(hmem) = GlobalAlloc(GMEM_MOVEABLE, byte_len) {
            let ptr = GlobalLock(hmem) as *mut u16;
            if !ptr.is_null() {
                std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
                let _ = GlobalUnlock(hmem);
                let _ = SetClipboardData(CF_UNICODETEXT, HANDLE(hmem.0 as _));
            }
        }
        let _ = CloseClipboard();
    }
}

// ── Cửa sổ chuyển mã ──────────────────────────────────────────────────────

pub fn open_converter_window() {
    if CONV_OPEN.swap(true, Ordering::SeqCst) { return; }
    std::thread::spawn(|| {
        run_converter();
        CONV_OPEN.store(false, Ordering::Relaxed);
    });
}

fn run_converter() {
    ui::init_common_controls();

    let hwnd = ui::create_dialog_window(
        w!("VnKeyConverter"),
        &format!("Chuyển mã – VnKey {}", crate::gui::VERSION),
        440, 352,
        Some(conv_wnd_proc),
    );
    if hwnd.0.is_null() { return; }
    create_controls(hwnd);
    ui::show_and_focus(hwnd);
    ui::run_dialog_loop(hwnd);
}

fn create_controls(hwnd: HWND) {
    let font = ui::create_ui_font();
    let (from_idx, to_idx) = {
        let s = CONV_SETTINGS.lock().unwrap_or_else(|e| e.into_inner());
        (s.from_charset, s.to_charset)
    };

    // ── "Bảng mã" groupbox ──
    ui::create_groupbox("Bảng mã", 10, 5, 420, 62, hwnd, 250, font);

    ui::create_label("Nguồn", 22, 22, 40, 16, hwnd, 251, font);
    ui::create_combobox(22, 38, 160, 300, hwnd, ID_CB_FROM, font, &CS_LABELS, from_idx);

    ui::create_button("⇄", 192, 36, 36, 24, hwnd, ID_BTN_SWAP, font);

    ui::create_label("Đích", 240, 22, 40, 16, hwnd, 252, font);
    ui::create_combobox(240, 38, 178, 300, hwnd, ID_CB_TO, font, &CS_LABELS, to_idx);

    // ── "Nội dung" groupbox ──
    ui::create_groupbox("Nội dung", 10, 72, 420, 230, hwnd, 253, font);
    ui::create_textarea(22, 90, 396, 170, hwnd, ID_EDIT_TEXT, font);
    ui::create_label("", 22, 265, 396, 18, hwnd, ID_LBL_STATUS, font);

    // ── Các nút ──
    ui::create_button("Chuyển mã", 10, 310, 140, 32, hwnd, ID_BTN_CONVERT, font);
    ui::create_button("Chuyển clipboard", 155, 310, 140, 32, hwnd, ID_BTN_CLIP, font);
    ui::create_button("Đóng", 300, 310, 130, 32, hwnd, ID_BTN_CLOSE, font);
}

unsafe extern "system" fn conv_wnd_proc(
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
            let hdc = HDC(wparam.0 as _);
            SetBkColor(hdc, ui::BG_COLOR);
            SetTextColor(hdc, COLORREF(0x001A1A1A));
            LRESULT(ui::BG_BRUSH().0 as _)
        }
        WM_CLOSE => { DestroyWindow(hwnd).ok(); LRESULT(0) }
        WM_DESTROY => { PostQuitMessage(0); LRESULT(0) }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn handle_command(hwnd: HWND, id: u16, notify: u16) {
    match id {
        ID_CB_FROM if notify == CBN_SELCHANGE as u16 => {
            let cb = unsafe { GetDlgItem(hwnd, ID_CB_FROM as i32) }.unwrap_or_default();
            let idx = unsafe { SendMessageW(cb, CB_GETCURSEL, WPARAM(0), LPARAM(0)) }.0 as usize;
            if let Ok(mut s) = CONV_SETTINGS.lock() { s.from_charset = idx; }
            crate::config::save();
        }
        ID_CB_TO if notify == CBN_SELCHANGE as u16 => {
            let cb = unsafe { GetDlgItem(hwnd, ID_CB_TO as i32) }.unwrap_or_default();
            let idx = unsafe { SendMessageW(cb, CB_GETCURSEL, WPARAM(0), LPARAM(0)) }.0 as usize;
            if let Ok(mut s) = CONV_SETTINGS.lock() { s.to_charset = idx; }
            crate::config::save();
        }
        ID_BTN_SWAP => {
            {
                if let Ok(mut s) = CONV_SETTINGS.lock() {
                    let tmp = s.from_charset;
                    s.from_charset = s.to_charset;
                    s.to_charset = tmp;
                }
            }
            // Cập nhật combobox
            let (f, t) = {
                let s = CONV_SETTINGS.lock().unwrap_or_else(|e| e.into_inner());
                (s.from_charset, s.to_charset)
            };
            unsafe {
                let cb_from = GetDlgItem(hwnd, ID_CB_FROM as i32).unwrap_or_default();
                let cb_to = GetDlgItem(hwnd, ID_CB_TO as i32).unwrap_or_default();
                SendMessageW(cb_from, CB_SETCURSEL, WPARAM(f), LPARAM(0));
                SendMessageW(cb_to, CB_SETCURSEL, WPARAM(t), LPARAM(0));
            }
            crate::config::save();
        }
        ID_BTN_CONVERT => {
            let edit = unsafe { GetDlgItem(hwnd, ID_EDIT_TEXT as i32) }.unwrap_or_default();
            let text = ui::get_edit_text(edit);
            if text.is_empty() {
                set_status(hwnd, "Chưa có nội dung");
                return;
            }
            let (from_idx, to_idx) = {
                let cb_from = unsafe { GetDlgItem(hwnd, ID_CB_FROM as i32) }.unwrap_or_default();
                let cb_to = unsafe { GetDlgItem(hwnd, ID_CB_TO as i32) }.unwrap_or_default();
                (
                    unsafe { SendMessageW(cb_from, CB_GETCURSEL, WPARAM(0), LPARAM(0)) }.0 as usize,
                    unsafe { SendMessageW(cb_to, CB_GETCURSEL, WPARAM(0), LPARAM(0)) }.0 as usize,
                )
            };
            if from_idx == to_idx {
                set_status(hwnd, "Nguồn và đích giống nhau");
                return;
            }
            let from_id = CS_IDS.get(from_idx).copied().unwrap_or(0);
            let to_id = CS_IDS.get(to_idx).copied().unwrap_or(0);
            let from_cs = charset_from_id(from_id);
            let to_cs = charset_from_id(to_id);
            match do_convert(&text, from_cs, to_cs) {
                Some(result) => {
                    ui::set_window_text(edit, &result);
                    set_status(hwnd, "Chuyển thành công!");
                }
                None => {
                    set_status(hwnd, "Lỗi chuyển mã. Vui lòng kiểm tra lại bảng mã nguồn.");
                }
            }
        }
        ID_BTN_CLIP => {
            convert_clipboard();
            set_status(hwnd, "Đã chuyển mã clipboard.");
        }
        ID_BTN_CLOSE => { unsafe { DestroyWindow(hwnd).ok(); } }
        _ => {}
    }
}

fn set_status(hwnd: HWND, text: &str) {
    let lbl = unsafe { GetDlgItem(hwnd, ID_LBL_STATUS as i32) }.unwrap_or_default();
    ui::set_window_text(lbl, text);
}
