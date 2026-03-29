//! Hiển thị OSD (toast) — thông báo ngắn giữa màn hình,
//! tự ẩn sau ~1.5 giây.

use std::sync::atomic::{AtomicIsize, Ordering};
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

const OSD_TIMER_ID: usize = 1;
const OSD_DURATION_MS: u32 = 1500;
const OSD_CLASS: PCWSTR = w!("VnKeyOSD");

/// HWND của OSD đang hiển thị (0 = không có)
static CURRENT_OSD: AtomicIsize = AtomicIsize::new(0);

/// Hiển thị toast giữa màn hình. Không chặn — chạy trên thread riêng.
pub fn show(text: &str) {
    // Đóng OSD cũ nếu còn đang hiển thị
    let prev = CURRENT_OSD.swap(0, Ordering::SeqCst);
    if prev != 0 {
        unsafe {
            let _ = PostMessageW(HWND(prev as *mut _), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
    let text = text.to_owned();
    std::thread::spawn(move || run_osd(&text));
}

fn run_osd(text: &str) {
    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap_or_default();

        // Đăng ký window class chỉ 1 lần
        static CLASS_REGISTERED: std::sync::Once = std::sync::Once::new();
        CLASS_REGISTERED.call_once(|| {
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(osd_wnd_proc),
                hInstance: hinstance.into(),
                hbrBackground: HBRUSH::default(),
                lpszClassName: OSD_CLASS,
                ..Default::default()
            };
            RegisterClassExW(&wc);
        });

        // Đo chữ để xác định kích thước cửa sổ
        let hdc = GetDC(HWND::default());
        let font = create_osd_font();
        let old = SelectObject(hdc, font);

        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let mut rc = RECT::default();
        DrawTextW(
            hdc,
            &mut wide[..wide.len() - 1].to_vec(),
            &mut rc,
            DT_CALCRECT | DT_SINGLELINE,
        );
        SelectObject(hdc, old);
        ReleaseDC(HWND::default(), hdc);

        let pad_x = 40;
        let pad_y = 20;
        let w = (rc.right - rc.left) + pad_x * 2;
        let h = (rc.bottom - rc.top) + pad_y * 2;

        // Canh giữa trên màn hình chính
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - w) / 2;
        let y = screen_h / 3 - h / 2; // hơi trên giữa

        // Layered + trong suốt + trên cùng + tool window (không taskbar)
        let ex_style = WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT;

        let hwnd = CreateWindowExW(
            ex_style,
            OSD_CLASS,
            w!(""),
            WS_POPUP,
            x, y, w, h,
            None,
            None,
            hinstance,
            None,
        )
        .unwrap_or_default();

        if hwnd.0.is_null() {
            let _ = DeleteObject(font);
            return;
        }

        // Lưu HWND để có thể đóng từ bên ngoài
        CURRENT_OSD.store(hwnd.0 as isize, Ordering::SeqCst);

        // Lưu font và chữ dưới dạng dữ liệu cửa sổ
        let boxed = Box::new(OsdData {
            text: text.to_owned(),
            font,
        });
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(boxed) as isize);

        // Đặt cửa sổ layered: 85% độ đục
        SetLayeredWindowAttributes(hwnd, COLORREF(0), 216, LWA_ALPHA).ok();

        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        let _ = UpdateWindow(hwnd);

        // Hẹn giờ tự đóng
        let _ = SetTimer(hwnd, OSD_TIMER_ID, OSD_DURATION_MS, None);

        // Vòng lặp tin nhắn
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
            DispatchMessageW(&msg);
        }

        // Dọn dẹp trong WM_DESTROY
    }
}

struct OsdData {
    text: String,
    font: HGDIOBJ,
}

fn create_osd_font() -> HGDIOBJ {
    unsafe {
        let font = CreateFontW(
            32,             // chiều cao
            0,              // chiều rộng (tự động)
            0, 0,           // nghiêng, hướng
            700,            // độ đậm (bold)
            0, 0, 0,        // nghiêng, gạch chân, gạch ngang
            1,              // bảng mã (DEFAULT_CHARSET)
            0, 0, 5,        // độ chính xác, cắt, chất lượng (CLEARTYPE)
            0,              // bước phông
            w!("Segoe UI"),
        );
        HGDIOBJ(font.0)
    }
}

unsafe extern "system" fn osd_wnd_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut rc = RECT::default();
            GetClientRect(hwnd, &mut rc).ok();

            // Nền tối bo góc
            let bg_brush = CreateSolidBrush(COLORREF(0x00403030)); // nâu xám đậm
            FillRect(hdc, &rc, bg_brush);
            let _ = DeleteObject(bg_brush);

            // Vẽ chữ
            let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const OsdData;
            if !data_ptr.is_null() {
                let data = &*data_ptr;
                let old = SelectObject(hdc, data.font);
                SetBkMode(hdc, TRANSPARENT);
                SetTextColor(hdc, COLORREF(0x00FFFFFF)); // trắng

                let mut text_wide: Vec<u16> = data.text.encode_utf16().collect();
                DrawTextW(
                    hdc,
                    &mut text_wide,
                    &mut rc,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );
                SelectObject(hdc, old);
            }

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 == OSD_TIMER_ID {
                KillTimer(hwnd, OSD_TIMER_ID).ok();
                DestroyWindow(hwnd).ok();
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            // Xóa HWND khỏi biến tĩnh
            CURRENT_OSD.store(0, Ordering::SeqCst);
            // Giải phóng OsdData
            let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OsdData;
            if !data_ptr.is_null() {
                let data = Box::from_raw(data_ptr);
                let _ = DeleteObject(data.font);
            }
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
