// ui.rs - UI drawing, tray icon, and window management

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::UI::Shell::*,
    Win32::UI::WindowsAndMessaging::*,
};
use crate::constants::*;
use crate::calendar::get_current_bangla_date;
use crate::{CUSTOM_FONT, MENU_FONT, get_flag_icon};

// Embed the Ekush font directly into the executable
const EKUSH_FONT_DATA: &[u8] = include_bytes!("../fonts/Ekush-Regular.ttf");

/// Install the embedded Ekush font
pub unsafe fn install_embedded_font() {
    let mut num_fonts: u32 = 0;
    let _font_handle = unsafe {
        AddFontMemResourceEx(
            EKUSH_FONT_DATA.as_ptr() as *const std::ffi::c_void,
            EKUSH_FONT_DATA.len() as u32,
            None,
            &mut num_fonts,
        )
    };

    if num_fonts > 0 {
        unsafe {
            CUSTOM_FONT = CreateFontW(
                24,
                0, 0, 0,
                FW_NORMAL.0 as i32,
                0, 0, 0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                CLEARTYPE_QUALITY,
                (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
                w!("Ekush"),
            );
            
            // Create menu font for owner-drawn menus
            MENU_FONT = CreateFontW(
                MENU_FONT_SIZE,
                0, 0, 0,
                FW_NORMAL.0 as i32,
                0, 0, 0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                CLEARTYPE_QUALITY,
                (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
                w!("Ekush"),
            );
        }
    }
}

/// Calculate the optimal widget width based on text content
pub fn calculate_widget_width() -> i32 {
    let bangla_date = get_current_bangla_date();
    let line1 = bangla_date.format_line1();
    let line2 = bangla_date.format_line2();
    let line3 = bangla_date.format_line3();
    
    // Estimate width based on character count (Bangla characters are wider)
    let max_chars = line1.chars().count().max(line2.chars().count()).max(line3.chars().count());
    let text_width = (max_chars as i32 * 11).max(120); // Even tighter width
    
    text_width + (PADDING * 2) // just text + padding on both sides
}

pub unsafe fn create_tray_icon(hwnd: HWND) -> Result<()> {
    let flag_icon = get_flag_icon();
    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: WM_TRAYICON,
        hIcon: unsafe { if !flag_icon.is_invalid() { flag_icon } else { LoadIconW(None, IDI_APPLICATION)? } },
        ..Default::default()
    };

    let tip = "বঙ্গ উইজেট - বাংলা তারিখ";
    let tip_wide: Vec<u16> = tip.encode_utf16().chain(std::iter::once(0)).collect();
    let len = tip_wide.len().min(128);
    nid.szTip[..len].copy_from_slice(&tip_wide[..len]);

    let result = unsafe { Shell_NotifyIconW(NIM_ADD, &nid) };
    if !result.as_bool() {
        return Err(Error::from_win32());
    }
    Ok(())
}

pub unsafe fn remove_tray_icon(hwnd: HWND) -> Result<()> {
    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        ..Default::default()
    };
    let result = unsafe { Shell_NotifyIconW(NIM_DELETE, &nid) };
    if !result.as_bool() {
        return Err(Error::from_win32());
    }
    Ok(())
}

pub unsafe fn set_desktop_level(hwnd: HWND) {
    let progman = unsafe { FindWindowW(w!("Progman"), None) };

    if let Ok(progman) = progman {
        if !progman.is_invalid() {
            unsafe {
                let _ = SendMessageTimeoutW(
                    progman,
                    0x052C,
                    WPARAM(0),
                    LPARAM(0),
                    SMTO_NORMAL,
                    1000,
                    None,
                );
            }

            let mut worker_w = HWND::default();
            let mut found_worker = HWND::default();

            loop {
                worker_w = unsafe { FindWindowExW(None, Some(worker_w), w!("WorkerW"), None) }
                    .unwrap_or_default();
                if worker_w.is_invalid() {
                    break;
                }

                let shell_view = unsafe {
                    FindWindowExW(Some(worker_w), None, w!("SHELLDLL_DefView"), None)
                }
                .unwrap_or_default();
                if !shell_view.is_invalid() {
                    found_worker =
                        unsafe { FindWindowExW(None, Some(worker_w), w!("WorkerW"), None) }
                            .unwrap_or_default();
                    break;
                }
            }

            if !found_worker.is_invalid() {
                unsafe { let _ = SetParent(hwnd, Some(found_worker)); }
                return;
            }
        }
    }

    unsafe {
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_BOTTOM),
            0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

/// Draw rounded rectangle with GDI
pub unsafe fn draw_rounded_rect(hdc: HDC, rect: &RECT, radius: i32, brush: HBRUSH) {
    unsafe {
        let rgn = CreateRoundRectRgn(
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            radius * 2,
            radius * 2,
        );
        let _ = FillRgn(hdc, rgn, brush);
        let _ = DeleteObject(rgn.into());
    }
}

/// Handle WM_PAINT - draw the widget
pub unsafe fn handle_paint(hwnd: HWND) -> LRESULT {
    unsafe {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);

        let mut rect = RECT::default();
        let _ = GetClientRect(hwnd, &mut rect);

        // Create memory DC for double buffering
        let mem_dc = CreateCompatibleDC(Some(hdc));
        let mem_bitmap = CreateCompatibleBitmap(hdc, rect.right, rect.bottom);
        let old_bitmap = SelectObject(mem_dc, mem_bitmap.into());

        // Fill with transparent color (black will be transparent)
        let trans_brush = CreateSolidBrush(COLORREF(0x00000000));
        FillRect(mem_dc, &rect, trans_brush);
        let _ = DeleteObject(trans_brush.into());

        // Draw main rounded background (dark)
        let bg_brush = CreateSolidBrush(COLORREF(BG_COLOR));
        draw_rounded_rect(mem_dc, &rect, CORNER_RADIUS, bg_brush);
        let _ = DeleteObject(bg_brush.into());

        // Draw subtle border
        let border_pen = CreatePen(PS_SOLID, 1, COLORREF(BORDER_COLOR));
        let old_pen = SelectObject(mem_dc, border_pen.into());
        let null_brush = GetStockObject(NULL_BRUSH);
        let old_brush = SelectObject(mem_dc, null_brush);
        let _ = RoundRect(mem_dc, rect.left, rect.top, rect.right, rect.bottom, CORNER_RADIUS * 2, CORNER_RADIUS * 2);
        SelectObject(mem_dc, old_pen);
        SelectObject(mem_dc, old_brush);
        let _ = DeleteObject(border_pen.into());

        // Set text properties
        SetBkMode(mem_dc, TRANSPARENT);

        // Get current Bangla date
        let bangla_date = get_current_bangla_date();

        // Text area with padding
        let text_left = rect.left + PADDING;
        let text_right = rect.right - PADDING;

        // Create fonts
        let font_line1 = CreateFontW(
            26, // Slightly bigger for line 1
            0, 0, 0,
            FW_SEMIBOLD.0 as i32,
            0, 0, 0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            w!("Ekush"),
        );

        let font_line2 = CreateFontW(
            22,
            0, 0, 0,
            FW_NORMAL.0 as i32,
            0, 0, 0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            w!("Ekush"),
        );

        let font_line3 = CreateFontW(
            18,
            0, 0, 0,
            FW_NORMAL.0 as i32,
            0, 0, 0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            w!("Ekush"),
        );

        // Line 1: ০৬ই পৌষ, (bigger, white)
        let old_font = SelectObject(mem_dc, font_line1.into());
        SetTextColor(mem_dc, COLORREF(TEXT_PRIMARY));
        let line1_text = bangla_date.format_line1();
        let mut line1_vec: Vec<u16> = line1_text.encode_utf16().collect();
        let mut line1_rect = RECT {
            left: text_left,
            top: 12,
            right: text_right,
            bottom: 38,
        };
        DrawTextW(mem_dc, &mut line1_vec, &mut line1_rect, DT_CENTER | DT_SINGLELINE);

        // Line 2: ১৪৩২ বঙ্গাব্দ (normal, white)
        SelectObject(mem_dc, font_line2.into());
        let line2_text = bangla_date.format_line2();
        let mut line2_vec: Vec<u16> = line2_text.encode_utf16().collect();
        let mut line2_rect = RECT {
            left: text_left,
            top: 38,
            right: text_right,
            bottom: 60,
        };
        DrawTextW(mem_dc, &mut line2_vec, &mut line2_rect, DT_CENTER | DT_SINGLELINE);

        // Line 3: শনিবার, হেমন্তকাল (smaller, gray)
        SelectObject(mem_dc, font_line3.into());
        SetTextColor(mem_dc, COLORREF(TEXT_SECONDARY));
        let line3_text = bangla_date.format_line3();
        let mut line3_vec: Vec<u16> = line3_text.encode_utf16().collect();
        let mut line3_rect = RECT {
            left: text_left,
            top: 62,
            right: text_right,
            bottom: 82,
        };
        DrawTextW(mem_dc, &mut line3_vec, &mut line3_rect, DT_CENTER | DT_SINGLELINE);

        SelectObject(mem_dc, old_font);
        let _ = DeleteObject(font_line1.into());
        let _ = DeleteObject(font_line2.into());
        let _ = DeleteObject(font_line3.into());

        // Copy to screen
        let _ = BitBlt(hdc, 0, 0, rect.right, rect.bottom, Some(mem_dc), 0, 0, SRCCOPY);

        // Cleanup
        SelectObject(mem_dc, old_bitmap);
        let _ = DeleteObject(mem_bitmap.into());
        let _ = DeleteDC(mem_dc);

        let _ = EndPaint(hwnd, &ps);
        LRESULT(0)
    }
}
