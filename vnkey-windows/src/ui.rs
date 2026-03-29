//! Các hàm tiện ích Win32 + Direct2D dùng chung cho các cửa sổ.

use std::sync::OnceLock;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;

// ── Màu sắc ───────────────────────────────────────────────────────────────

pub const BG_COLOR: COLORREF = COLORREF(0x00F0F0F0); // #F0F0F0
pub const BG_BRUSH: fn() -> HBRUSH = || unsafe { CreateSolidBrush(BG_COLOR) };

pub const CLR_BG: D2D1_COLOR_F = D2D1_COLOR_F { r: 0.941, g: 0.941, b: 0.941, a: 1.0 };
pub const CLR_TEXT: D2D1_COLOR_F = D2D1_COLOR_F { r: 0.1, g: 0.1, b: 0.1, a: 1.0 };
pub const CLR_LABEL: D2D1_COLOR_F = D2D1_COLOR_F { r: 0.333, g: 0.333, b: 0.333, a: 1.0 };
pub const CLR_BORDER: D2D1_COLOR_F = D2D1_COLOR_F { r: 0.753, g: 0.753, b: 0.753, a: 1.0 };
pub const CLR_ACCENT: D2D1_COLOR_F = D2D1_COLOR_F { r: 0.0, g: 0.471, b: 0.831, a: 1.0 };
pub const CLR_BTN_BG: D2D1_COLOR_F = D2D1_COLOR_F { r: 0.894, g: 0.894, b: 0.894, a: 1.0 };
pub const CLR_WHITE: D2D1_COLOR_F = D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

// ── Factory D2D/DWrite toàn cục ───────────────────────────────────────────

static D2D_FACTORY: OnceLock<ID2D1Factory> = OnceLock::new();
static DW_FACTORY: OnceLock<IDWriteFactory> = OnceLock::new();

pub fn d2d_factory() -> &'static ID2D1Factory {
    D2D_FACTORY.get_or_init(|| unsafe {
        D2D1CreateFactory::<ID2D1Factory>(D2D1_FACTORY_TYPE_MULTI_THREADED, None).unwrap()
    })
}

pub fn dw_factory() -> &'static IDWriteFactory {
    DW_FACTORY.get_or_init(|| unsafe {
        DWriteCreateFactory::<IDWriteFactory>(DWRITE_FACTORY_TYPE_SHARED).unwrap()
    })
}

// ── Trợ giúp định dạng chữ ─────────────────────────────────────────────────

pub fn text_format(size: f32, weight: DWRITE_FONT_WEIGHT) -> IDWriteTextFormat {
    unsafe {
        dw_factory()
            .CreateTextFormat(
                w!("Segoe UI"),
                None,
                weight,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                size,
                w!(""),
            )
            .unwrap()
    }
}

pub fn text_format_normal(size: f32) -> IDWriteTextFormat {
    text_format(size, DWRITE_FONT_WEIGHT_NORMAL)
}

pub fn text_format_bold(size: f32) -> IDWriteTextFormat {
    text_format(size, DWRITE_FONT_WEIGHT_SEMI_BOLD)
}

// ── Render target ───────────────────────────────────────────────────────

pub fn create_render_target(hwnd: HWND, w: u32, h: u32) -> ID2D1HwndRenderTarget {
    let props = D2D1_RENDER_TARGET_PROPERTIES {
        r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
        pixelFormat: D2D1_PIXEL_FORMAT {
            format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
            alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
        },
        dpiX: 0.0,
        dpiY: 0.0,
        usage: D2D1_RENDER_TARGET_USAGE_NONE,
        minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
    };
    let hwnd_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
        hwnd,
        pixelSize: windows::Win32::Graphics::Direct2D::Common::D2D_SIZE_U { width: w, height: h },
        presentOptions: D2D1_PRESENT_OPTIONS_NONE,
    };
    unsafe { d2d_factory().CreateHwndRenderTarget(&props, &hwnd_props).unwrap() }
}

// ── Trợ giúp vẽ D2D ───────────────────────────────────────────────────────

pub fn draw_text(
    rt: &ID2D1HwndRenderTarget,
    text: &str,
    format: &IDWriteTextFormat,
    rect: D2D_RECT_F,
    color: D2D1_COLOR_F,
) {
    unsafe {
        let brush = rt.CreateSolidColorBrush(&color, None).unwrap();
        let wide: Vec<u16> = text.encode_utf16().collect();
        rt.DrawText(&wide, format, &rect, &brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
    }
}

pub fn fill_rect(rt: &ID2D1HwndRenderTarget, rect: D2D_RECT_F, color: D2D1_COLOR_F) {
    unsafe {
        let brush = rt.CreateSolidColorBrush(&color, None).unwrap();
        rt.FillRectangle(&rect, &brush);
    }
}

pub fn draw_rounded_rect(
    rt: &ID2D1HwndRenderTarget,
    rect: D2D_RECT_F,
    radius: f32,
    color: D2D1_COLOR_F,
    width: f32,
) {
    unsafe {
        let brush = rt.CreateSolidColorBrush(&color, None).unwrap();
        let rr = D2D1_ROUNDED_RECT {
            rect,
            radiusX: radius,
            radiusY: radius,
        };
        rt.DrawRoundedRectangle(&rr, &brush, width, None);
    }
}

// ── Trợ giúp cửa sổ Win32 ────────────────────────────────────────────────

/// Bật kiểu hiển thị (Common Controls 6).
pub fn init_common_controls() {
    let icc = INITCOMMONCONTROLSEX {
        dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
        dwICC: ICC_STANDARD_CLASSES,
    };
    unsafe { let _ = InitCommonControlsEx(&icc); }
}

/// Tạo cửa sổ kiểu hộp thoại cấp cao nhất, canh giữa màn hình, ban đầu ẩn.
pub fn create_dialog_window(
    class_name: PCWSTR,
    title: &str,
    width: i32,
    height: i32,
    wndproc: WNDPROC,
) -> HWND {
    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap_or_default();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: wndproc,
            hInstance: hinstance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: CreateSolidBrush(BG_COLOR),
            lpszClassName: class_name,
            hIcon: LoadIconW(hinstance, PCWSTR(101 as _)).unwrap_or_default(),
            hIconSm: LoadIconW(hinstance, PCWSTR(101 as _)).unwrap_or_default(),
            ..Default::default()
        };
        RegisterClassExW(&wc);

        let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

        // Điều chỉnh cho vùng non-client
        let mut rc = RECT { left: 0, top: 0, right: width, bottom: height };
        let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
        let _ = AdjustWindowRectEx(&mut rc, style, false, WINDOW_EX_STYLE::default());
        let adj_w = rc.right - rc.left;
        let adj_h = rc.bottom - rc.top;

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST,
            class_name,
            PCWSTR(title_wide.as_ptr()),
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            adj_w,
            adj_h,
            None,
            None,
            hinstance,
            None,
        )
        .unwrap_or_default();

        // Canh giữa màn hình
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - adj_w) / 2;
        let y = (screen_h - adj_h) / 2;
        SetWindowPos(hwnd, HWND_TOPMOST, x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER).ok();

        hwnd
    }
}

/// Tạo handle font chuẩn cho các điều khiển (Segoe UI 13px).
pub fn create_ui_font() -> HFONT {
    unsafe {
        CreateFontW(
            -13, 0, 0, 0,
            FW_NORMAL.0 as i32,
            0, 0, 0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            CLEARTYPE_QUALITY.0 as u32,
            (FF_SWISS.0 | VARIABLE_PITCH.0) as u32,
            w!("Segoe UI"),
        )
    }
}

/// Tạo handle font đậm.
pub fn create_ui_font_bold() -> HFONT {
    unsafe {
        CreateFontW(
            -13, 0, 0, 0,
            FW_SEMIBOLD.0 as i32,
            0, 0, 0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            CLEARTYPE_QUALITY.0 as u32,
            (FF_SWISS.0 | VARIABLE_PITCH.0) as u32,
            w!("Segoe UI"),
        )
    }
}

/// Tiện ích: đặt font cho điều khiển con.
pub fn set_control_font(hwnd: HWND, font: HFONT) {
    unsafe {
        SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    }
}

/// Tạo cửa sổ con (chung).
pub fn create_child(
    class: PCWSTR,
    text: &str,
    style: WINDOW_STYLE,
    x: i32, y: i32, w: i32, h: i32,
    parent: HWND,
    id: u16,
    font: HFONT,
) -> HWND {
    unsafe {
        let text_wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class,
            PCWSTR(text_wide.as_ptr()),
            WS_CHILD | WS_VISIBLE | style,
            x, y, w, h,
            parent,
            HMENU(id as _),
            GetModuleHandleW(None).unwrap_or_default(),
            None,
        )
        .unwrap_or_default();
        set_control_font(hwnd, font);
        hwnd
    }
}

/// Tạo nhãn STATIC.
pub fn create_label(text: &str, x: i32, y: i32, w: i32, h: i32, parent: HWND, id: u16, font: HFONT) -> HWND {
    create_child(w!("STATIC"), text, WINDOW_STYLE(0), x, y, w, h, parent, id, font)
}

/// Tạo nút BUTTON (nút bấm).
pub fn create_button(text: &str, x: i32, y: i32, w: i32, h: i32, parent: HWND, id: u16, font: HFONT) -> HWND {
    create_child(w!("BUTTON"), text, WINDOW_STYLE(BS_PUSHBUTTON as u32), x, y, w, h, parent, id, font)
}

/// Tạo ô chọn.
pub fn create_checkbox(text: &str, x: i32, y: i32, w: i32, h: i32, parent: HWND, id: u16, font: HFONT, checked: bool) -> HWND {
    let hwnd = create_child(w!("BUTTON"), text, WINDOW_STYLE(BS_AUTOCHECKBOX as u32), x, y, w, h, parent, id, font);
    if checked {
        unsafe { SendMessageW(hwnd, BM_SETCHECK, WPARAM(1), LPARAM(0)); }
    }
    hwnd
}

/// Tạo COMBOBOX (danh sách thả xuống).
pub fn create_combobox(x: i32, y: i32, w: i32, h: i32, parent: HWND, id: u16, font: HFONT, items: &[&str], selected: usize) -> HWND {
    let hwnd = create_child(
        w!("COMBOBOX"), "",
        WINDOW_STYLE(CBS_DROPDOWNLIST as u32 | CBS_HASSTRINGS as u32),
        x, y, w, h,
        parent, id, font,
    );
    unsafe {
        for item in items {
            let wide: Vec<u16> = item.encode_utf16().chain(std::iter::once(0)).collect();
            SendMessageW(hwnd, CB_ADDSTRING, WPARAM(0), LPARAM(wide.as_ptr() as _));
        }
        SendMessageW(hwnd, CB_SETCURSEL, WPARAM(selected), LPARAM(0));
    }
    hwnd
}

/// Tạo EDIT nhiều dòng (textarea).
pub fn create_textarea(x: i32, y: i32, w: i32, h: i32, parent: HWND, id: u16, font: HFONT) -> HWND {
    unsafe {
        let hwnd = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            w!("EDIT"),
            PCWSTR::null(),
            WS_CHILD | WS_VISIBLE | WS_VSCROLL
                | WINDOW_STYLE(ES_MULTILINE as u32 | ES_AUTOVSCROLL as u32 | ES_WANTRETURN as u32),
            x, y, w, h,
            parent,
            HMENU(id as _),
            GetModuleHandleW(None).unwrap_or_default(),
            None,
        )
        .unwrap_or_default();
        set_control_font(hwnd, font);
        hwnd
    }
}

/// Tạo EDIT một dòng (ô nhập liệu).
pub fn create_textinput(x: i32, y: i32, w: i32, h: i32, parent: HWND, id: u16, font: HFONT) -> HWND {
    unsafe {
        let hwnd = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            w!("EDIT"),
            PCWSTR::null(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
            x, y, w, h,
            parent,
            HMENU(id as _),
            GetModuleHandleW(None).unwrap_or_default(),
            None,
        )
        .unwrap_or_default();
        set_control_font(hwnd, font);
        hwnd
    }
}

/// Tạo LISTBOX.
pub fn create_listbox(x: i32, y: i32, w: i32, h: i32, parent: HWND, id: u16, font: HFONT) -> HWND {
    unsafe {
        let hwnd = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            w!("LISTBOX"),
            PCWSTR::null(),
            WS_CHILD | WS_VISIBLE | WS_VSCROLL
                | WINDOW_STYLE(LBS_NOTIFY as u32 | LBS_NOINTEGRALHEIGHT as u32),
            x, y, w, h,
            parent,
            HMENU(id as _),
            GetModuleHandleW(None).unwrap_or_default(),
            None,
        )
        .unwrap_or_default();
        set_control_font(hwnd, font);
        hwnd
    }
}

/// Tạo GROUPBOX (nhóm).
pub fn create_groupbox(text: &str, x: i32, y: i32, w: i32, h: i32, parent: HWND, id: u16, font: HFONT) -> HWND {
    create_child(w!("BUTTON"), text, WINDOW_STYLE(BS_GROUPBOX as u32), x, y, w, h, parent, id, font)
}

/// Hiển thị cửa sổ và đưa lên trước.
pub fn show_and_focus(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
    }
}

/// Lấy chữ từ điều khiển EDIT.
pub fn get_edit_text(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd) as usize;
        if len == 0 {
            return String::new();
        }
        let mut buf = vec![0u16; len + 1];
        GetWindowTextW(hwnd, &mut buf);
        String::from_utf16_lossy(&buf[..len])
    }
}

/// Đặt chữ cho điều khiển.
pub fn set_window_text(hwnd: HWND, text: &str) {
    unsafe {
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        SetWindowTextW(hwnd, PCWSTR(wide.as_ptr())).ok();
    }
}

/// Chạy vòng lặp tin nhắn kiểu modal cho cửa sổ hộp thoại trên thread hiện tại.
/// Trả về khi cửa sổ bị hủy.
pub fn run_dialog_loop(hwnd: HWND) {
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
            if !IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
