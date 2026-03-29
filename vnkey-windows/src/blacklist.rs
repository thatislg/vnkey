//! Cửa sổ loại trừ ứng dụng — không gõ tiếng Việt trong các app được chỉ định.

use crate::ui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub static BLACKLIST: LazyLock<Mutex<Vec<String>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

static BL_OPEN: AtomicBool = AtomicBool::new(false);

/// Kiểm tra xem ứng dụng hiện tại có nằm trong danh sách loại trừ không.
pub fn is_foreground_blacklisted() -> bool {
    let exe = match get_foreground_exe() {
        Some(e) => e,
        None => return false,
    };
    let list = BLACKLIST.lock().unwrap();
    list.iter().any(|b| b.eq_ignore_ascii_case(&exe))
}

fn get_foreground_exe() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() { return None; }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 { return None; }
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 512];
        let mut len = buf.len() as u32;
        QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len).ok()?;
        let _ = windows::Win32::Foundation::CloseHandle(process);
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        path.rsplit('\\').next().map(|s| s.to_string())
    }
}

/// Trả về exe name của foreground (dùng chung cho blacklist và send.rs)
pub fn get_foreground_exe_cached() -> Option<String> {
    get_foreground_exe()
}

pub fn get_exe_under_cursor() -> Option<String> {
    unsafe {
        let mut pt = POINT::default();
        GetCursorPos(&mut pt).ok()?;
        let hwnd = WindowFromPoint(pt);
        if hwnd.0.is_null() { return None; }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 { return None; }
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 512];
        let mut len = buf.len() as u32;
        QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len).ok()?;
        let _ = windows::Win32::Foundation::CloseHandle(process);
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        path.rsplit('\\').next().map(|s| s.to_string())
    }
}

// ── Mã điều khiển ───────────────────────────────────────────────────────

const ID_EDIT_INPUT: u16  = 400;
const ID_BTN_ADD: u16     = 401;
const ID_BTN_PICK: u16    = 402;
const ID_LISTBOX: u16     = 403;
const ID_BTN_REMOVE: u16  = 404;
const ID_BTN_CLOSE: u16   = 405;

// Tin nhắn tùy chỉnh cho kết quả chọn
const WM_PICK_RESULT: u32 = WM_USER + 100;

// ── Cửa sổ loại trừ ────────────────────────────────────────────────────

pub fn open_blacklist_window() {
    if BL_OPEN.swap(true, Ordering::SeqCst) { return; }
    std::thread::spawn(|| {
        run_blacklist();
        BL_OPEN.store(false, Ordering::Relaxed);
    });
}

fn run_blacklist() {
    ui::init_common_controls();

    let hwnd = ui::create_dialog_window(
        w!("VnKeyBlacklist"),
        &format!("Loại trừ ứng dụng – VnKey {}", crate::gui::VERSION),
        380, 362,
        Some(bl_wnd_proc),
    );
    if hwnd.0.is_null() { return; }
    create_controls(hwnd);
    ui::show_and_focus(hwnd);
    ui::run_dialog_loop(hwnd);
}

fn create_controls(hwnd: HWND) {
    let font = ui::create_ui_font();

    // ── Phần thêm ──
    ui::create_groupbox("Thêm ứng dụng", 10, 5, 360, 62, hwnd, 450, font);
    ui::create_textinput(22, 28, 175, 24, hwnd, ID_EDIT_INPUT, font);
    // Đặt chữ gợi ý (EM_SETCUEBANNER)
    unsafe {
        let placeholder: Vec<u16> = "Tên file .exe (vd: chrome.exe)".encode_utf16().chain(std::iter::once(0)).collect();
        SendMessageW(
            GetDlgItem(hwnd, ID_EDIT_INPUT as i32).unwrap_or_default(),
            0x1501, // EM_SETCUEBANNER
            WPARAM(1),
            LPARAM(placeholder.as_ptr() as _),
        );
    }
    ui::create_button("Thêm", 202, 26, 60, 28, hwnd, ID_BTN_ADD, font);
    ui::create_button("Chọn…", 267, 26, 90, 28, hwnd, ID_BTN_PICK, font);

    // ── Phần danh sách ──
    ui::create_groupbox("Danh sách loại trừ", 10, 72, 360, 240, hwnd, 451, font);
    let lb = ui::create_listbox(22, 90, 336, 200, hwnd, ID_LISTBOX, font);

    // Đổ dữ liệu danh sách
    if let Ok(list) = BLACKLIST.lock() {
        for name in list.iter() {
            unsafe {
                let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
                SendMessageW(lb, LB_ADDSTRING, WPARAM(0), LPARAM(wide.as_ptr() as _));
            }
        }
    }

    // ── Các nút phía dưới ──
    ui::create_button("Xóa", 10, 320, 175, 32, hwnd, ID_BTN_REMOVE, font);
    ui::create_button("Đóng", 195, 320, 175, 32, hwnd, ID_BTN_CLOSE, font);
}

unsafe extern "system" fn bl_wnd_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as u16;
            handle_command(hwnd, id);
            LRESULT(0)
        }
        WM_PICK_RESULT => {
            // lparam = con trỏ tới chuỗi wide (đã cấp phát)
            let ptr = lparam.0 as *const u16;
            if !ptr.is_null() {
                let mut len = 0;
                while *ptr.add(len) != 0 { len += 1; }
                let name = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
                // Giải phóng bộ nhớ
                let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u16, len + 1));
                // Đưa vào ô nhập liệu
                let edit = GetDlgItem(hwnd, ID_EDIT_INPUT as i32).unwrap_or_default();
                ui::set_window_text(edit, &name);
            }
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

fn handle_command(hwnd: HWND, id: u16) {
    match id {
        ID_BTN_ADD => {
            let edit = unsafe { GetDlgItem(hwnd, ID_EDIT_INPUT as i32) }.unwrap_or_default();
            let name = ui::get_edit_text(edit).trim().to_string();
            if name.is_empty() { return; }

            // Thêm vào danh sách loại trừ
            {
                let mut list = BLACKLIST.lock().unwrap();
                if !list.iter().any(|b| b.eq_ignore_ascii_case(&name)) {
                    list.push(name.clone());
                }
            }
            crate::config::save();

            // Thêm vào listbox
            let lb = unsafe { GetDlgItem(hwnd, ID_LISTBOX as i32) }.unwrap_or_default();
            unsafe {
                let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
                SendMessageW(lb, LB_ADDSTRING, WPARAM(0), LPARAM(wide.as_ptr() as _));
            }
            // Xóa ô nhập liệu
            ui::set_window_text(edit, "");
        }
        ID_BTN_PICK => {
            let hwnd_val = hwnd.0 as isize;
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(1500));
                if let Some(exe) = get_exe_under_cursor() {
                    // Gửi lại thread UI qua tin nhắn tùy chỉnh
                    let wide: Vec<u16> = exe.encode_utf16().chain(std::iter::once(0)).collect();
                    let boxed = wide.into_boxed_slice();
                    let ptr = Box::into_raw(boxed) as *const u16;
                    unsafe {
                        let _ = PostMessageW(
                            HWND(hwnd_val as _),
                            WM_PICK_RESULT,
                            WPARAM(0),
                            LPARAM(ptr as _),
                        );
                    }
                }
            });
        }
        ID_BTN_REMOVE => {
            let lb = unsafe { GetDlgItem(hwnd, ID_LISTBOX as i32) }.unwrap_or_default();
            let sel = unsafe { SendMessageW(lb, LB_GETCURSEL, WPARAM(0), LPARAM(0)) }.0;
            if sel < 0 { return; }

            // Lấy nội dung chữ
            let len = unsafe { SendMessageW(lb, LB_GETTEXTLEN, WPARAM(sel as _), LPARAM(0)) }.0 as usize;
            let mut buf = vec![0u16; len + 1];
            unsafe { SendMessageW(lb, LB_GETTEXT, WPARAM(sel as _), LPARAM(buf.as_mut_ptr() as _)); }
            let name = String::from_utf16_lossy(&buf[..len]);

            // Xóa khỏi danh sách loại trừ
            {
                let mut list = BLACKLIST.lock().unwrap();
                list.retain(|b| !b.eq_ignore_ascii_case(&name));
            }
            crate::config::save();

            // Xóa khỏi listbox
            unsafe { SendMessageW(lb, LB_DELETESTRING, WPARAM(sel as _), LPARAM(0)); }
        }
        ID_BTN_CLOSE => { unsafe { DestroyWindow(hwnd).ok(); } }
        _ => {}
    }
}
