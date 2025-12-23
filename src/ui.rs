// ui.rs - UI drawing, tray icon, and window management

use crate::calendar::get_current_bangla_date;
use crate::constants::*;
use crate::fonts::{get_font_line1, get_font_line2, get_font_line3};
use crate::get_flag_icon;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::UI::Shell::*,
    Win32::UI::WindowsAndMessaging::*, core::*,
};

/// Calculate the optimal widget width based on text content
pub fn calculate_widget_width() -> i32 {
    let bangla_date = get_current_bangla_date();
    let line1 = bangla_date.format_line1();
    let line2 = bangla_date.format_line2();
    let line3 = bangla_date.format_line3();

    // Estimate width based on character count (Bangla characters are wider)
    let max_chars = line1
        .chars()
        .count()
        .max(line2.chars().count())
        .max(line3.chars().count());
    let text_width = (max_chars as i32 * 11).max(120); // Even tighter width

    text_width + (PADDING * 2) // just text + padding on both sides
}

pub fn create_tray_icon(hwnd: HWND) -> Result<()> {
    unsafe {
        let flag_icon = get_flag_icon();
        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_TRAYICON,
            hIcon: if !flag_icon.is_invalid() {
                flag_icon
            } else {
                LoadIconW(None, IDI_APPLICATION)?
            },
            ..Default::default()
        };

        let tip = "বাংলা ক্যালেন্ডার";
        let tip_wide: Vec<u16> = tip.encode_utf16().chain(std::iter::once(0)).collect();
        let len = tip_wide.len().min(128);
        nid.szTip[..len].copy_from_slice(&tip_wide[..len]);

        if !Shell_NotifyIconW(NIM_ADD, &nid).as_bool() {
            return Err(Error::from_win32());
        }
        Ok(())
    }
}

pub fn remove_tray_icon(hwnd: HWND) -> Result<()> {
    unsafe {
        let nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            ..Default::default()
        };
        if !Shell_NotifyIconW(NIM_DELETE, &nid).as_bool() {
            return Err(Error::from_win32());
        }
        Ok(())
    }
}

pub fn set_desktop_level(hwnd: HWND) {
    unsafe {
        let Ok(progman) = FindWindowW(w!("Progman"), None) else {
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_BOTTOM),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
            return;
        };

        if progman.is_invalid() {
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_BOTTOM),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
            return;
        }

        let _ = SendMessageTimeoutW(
            progman,
            0x052C,
            WPARAM(0),
            LPARAM(0),
            SMTO_NORMAL,
            1000,
            None,
        );

        let mut worker_w = HWND::default();
        let mut found_worker = HWND::default();

        loop {
            worker_w = FindWindowExW(None, Some(worker_w), w!("WorkerW"), None).unwrap_or_default();
            if worker_w.is_invalid() {
                break;
            }

            let shell_view = FindWindowExW(Some(worker_w), None, w!("SHELLDLL_DefView"), None)
                .unwrap_or_default();
            if !shell_view.is_invalid() {
                found_worker =
                    FindWindowExW(None, Some(worker_w), w!("WorkerW"), None).unwrap_or_default();
                break;
            }
        }

        if !found_worker.is_invalid() {
            let _ = SetParent(hwnd, Some(found_worker));
        } else {
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_BOTTOM),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }
}

/// Draw rounded rectangle with GDI
#[inline]
fn draw_rounded_rect(hdc: HDC, rect: &RECT, radius: i32, brush: HBRUSH) {
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

/// Handle WM_PAINT - draw the widget using cached fonts
pub fn handle_paint(hwnd: HWND) -> LRESULT {
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
        let _ = RoundRect(
            mem_dc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            CORNER_RADIUS * 2,
            CORNER_RADIUS * 2,
        );
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

        // Use cached fonts (created once at startup - no memory allocation per paint)
        let font_line1 = get_font_line1();
        let font_line2 = get_font_line2();
        let font_line3 = get_font_line3();

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
        DrawTextW(
            mem_dc,
            &mut line1_vec,
            &mut line1_rect,
            DT_CENTER | DT_SINGLELINE,
        );

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
        DrawTextW(
            mem_dc,
            &mut line2_vec,
            &mut line2_rect,
            DT_CENTER | DT_SINGLELINE,
        );

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
        DrawTextW(
            mem_dc,
            &mut line3_vec,
            &mut line3_rect,
            DT_CENTER | DT_SINGLELINE,
        );

        SelectObject(mem_dc, old_font);
        // Note: DO NOT delete cached fonts - they're reused

        // Copy to screen
        let _ = BitBlt(
            hdc,
            0,
            0,
            rect.right,
            rect.bottom,
            Some(mem_dc),
            0,
            0,
            SRCCOPY,
        );

        // Cleanup GDI resources
        SelectObject(mem_dc, old_bitmap);
        let _ = DeleteObject(mem_bitmap.into());
        let _ = DeleteDC(mem_dc);

        let _ = EndPaint(hwnd, &ps);
        LRESULT(0)
    }
}
