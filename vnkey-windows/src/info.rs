//! Cửa sổ giới thiệu (Win32 + Direct2D).

use crate::ui;
use std::sync::atomic::{AtomicBool, Ordering};

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::*;

static INFO_OPEN: AtomicBool = AtomicBool::new(false);

/// Vùng liên kết nhấp được: (top_y, bottom_y, url)
const LINK_EMAIL: (f32, f32, &str) = (178.0, 196.0, "mailto:hi@vnkey.app");
const LINK_WEBSITE: (f32, f32, &str) = (196.0, 214.0, "https://vnkey.app");
const LINK_GITHUB: (f32, f32, &str) = (214.0, 232.0, "https://github.com/marixdev/vnkey");
const LINKS: [(f32, f32, &str); 3] = [LINK_EMAIL, LINK_WEBSITE, LINK_GITHUB];

pub fn open_info_window() {
    if INFO_OPEN.swap(true, Ordering::SeqCst) { return; }
    std::thread::spawn(|| {
        run_info();
        INFO_OPEN.store(false, Ordering::Relaxed);
    });
}

fn run_info() {
    ui::init_common_controls();

    let hwnd = ui::create_dialog_window(
        w!("VnKeyInfo"),
        &format!("Giới thiệu – VnKey {}", crate::gui::VERSION),
        360, 270,
        Some(info_wnd_proc),
    );
    if hwnd.0.is_null() { return; }

    // Lưu render target vào dữ liệu cửa sổ
    let rt = ui::create_render_target(hwnd, 360, 270);
    let rt_boxed = Box::new(rt);
    unsafe {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(rt_boxed) as isize);
    }

    ui::show_and_focus(hwnd);
    ui::run_dialog_loop(hwnd);
}

unsafe extern "system" fn info_wnd_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let rt_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ID2D1HwndRenderTarget;
            if !rt_ptr.is_null() {
                paint_info(&*rt_ptr);
            }
            // Xác nhận vùng vẽ
            let _ = ValidateRect(hwnd, None);
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as f32;
            for &(top, bot, url) in &LINKS {
                if y >= top && y < bot {
                    let url_w: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();
                    ShellExecuteW(hwnd, w!("open"), PCWSTR(url_w.as_ptr()), None, None, SW_SHOW);
                    break;
                }
            }
            LRESULT(0)
        }
        WM_SETCURSOR => {
            // Kiểm tra chuột có nằm trên vùng liên kết không
            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);
            let _ = ScreenToClient(hwnd, &mut pt);
            let y = pt.y as f32;
            let over_link = LINKS.iter().any(|&(top, bot, _)| y >= top && y < bot);
            if over_link {
                SetCursor(LoadCursorW(None, IDC_HAND).unwrap_or_default());
                return LRESULT(1);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_CLOSE => { DestroyWindow(hwnd).ok(); LRESULT(0) }
        WM_DESTROY => {
            // Dọn dẹp render target
            let rt_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ID2D1HwndRenderTarget;
            if !rt_ptr.is_null() {
                let _ = Box::from_raw(rt_ptr);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn paint_info(rt: &ID2D1HwndRenderTarget) {
    let fmt_title = ui::text_format(22.0, DWRITE_FONT_WEIGHT_SEMI_BOLD);
    let fmt_ver = ui::text_format_normal(13.0);
    let fmt_body = ui::text_format_normal(13.0);
    let fmt_small = ui::text_format_normal(12.0);

    // Canh giữa chữ
    unsafe {
        fmt_title.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER).ok();
        fmt_ver.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER).ok();
        fmt_body.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER).ok();
        fmt_small.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER).ok();
    }

    let w = 360.0f32;

    unsafe {
        rt.BeginDraw();
        rt.Clear(Some(&ui::CLR_BG));

        // Tiêu đề
        ui::draw_text(rt, "VnKey", &fmt_title,
            D2D_RECT_F { left: 0.0, top: 24.0, right: w, bottom: 54.0 },
            ui::CLR_ACCENT);

        // Phiên bản
        let ver = format!("Phiên bản {}", crate::gui::VERSION);
        ui::draw_text(rt, &ver, &fmt_ver,
            D2D_RECT_F { left: 0.0, top: 56.0, right: w, bottom: 76.0 },
            ui::CLR_LABEL);

        // Mô tả
        ui::draw_text(rt, "Bộ gõ tiếng Việt đa nền tảng.", &fmt_body,
            D2D_RECT_F { left: 0.0, top: 96.0, right: w, bottom: 116.0 },
            ui::CLR_TEXT);
        ui::draw_text(rt, "Hỗ trợ Telex, VNI, VIQR và nhiều bảng mã.", &fmt_body,
            D2D_RECT_F { left: 0.0, top: 116.0, right: w, bottom: 136.0 },
            ui::CLR_TEXT);

        // Thông tin tác giả
        ui::draw_text(rt, "Tác giả: Vũ Văn Đạt (MarixDev)", &fmt_small,
            D2D_RECT_F { left: 0.0, top: 160.0, right: w, bottom: 178.0 },
            ui::CLR_LABEL);
        ui::draw_text(rt, "Email: hi@vnkey.app", &fmt_small,
            D2D_RECT_F { left: 0.0, top: 178.0, right: w, bottom: 196.0 },
            ui::CLR_ACCENT);
        ui::draw_text(rt, "Website: https://vnkey.app", &fmt_small,
            D2D_RECT_F { left: 0.0, top: 196.0, right: w, bottom: 214.0 },
            ui::CLR_ACCENT);
        ui::draw_text(rt, "GitHub: https://github.com/marixdev/vnkey", &fmt_small,
            D2D_RECT_F { left: 0.0, top: 214.0, right: w, bottom: 232.0 },
            ui::CLR_ACCENT);
        ui::draw_text(rt, "Giấy phép: GPL v3", &fmt_small,
            D2D_RECT_F { left: 0.0, top: 236.0, right: w, bottom: 254.0 },
            ui::CLR_LABEL);

        let _ = rt.EndDraw(None, None);
    }
}
