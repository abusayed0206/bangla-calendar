// punjika.rs - Calendar popup window (পুঞ্জিকা)

use crate::calendar::*;
use crate::constants::*;
use crate::fonts::{
    get_cal_date_font, get_cal_header_font, get_cal_nav_font, get_cal_sub_font,
    get_cal_weekday_font,
};
use crate::get_flag_icon;
use std::sync::atomic::{AtomicI32, AtomicPtr, Ordering};
use windows::{
    Win32::Foundation::*, Win32::Graphics::Dwm::*, Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW, Win32::UI::WindowsAndMessaging::*, core::*,
};

// Calendar state - track which month/year we're viewing
static VIEW_MONTH: AtomicI32 = AtomicI32::new(0); // 0-11
static VIEW_YEAR: AtomicI32 = AtomicI32::new(1431);
static HOVER_DAY: AtomicI32 = AtomicI32::new(-1);
static CALENDAR_HWND_PTR: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());

// Calendar window dimensions
const CAL_WIDTH: i32 = 320;
const CAL_HEIGHT: i32 = 380;
const CAL_HEADER_HEIGHT: i32 = 60;
const CAL_NAV_HEIGHT: i32 = 40;
const CAL_WEEKDAY_HEIGHT: i32 = 30;
const CAL_CELL_SIZE: i32 = 40;
const CAL_PADDING: i32 = 12;

// Colors
const CAL_BG: u32 = 0x00FFFFFF;
const CAL_HEADER_BG: u32 = 0x00006B3C; // Bangladesh green
const CAL_HEADER_TEXT: u32 = 0x00FFFFFF;
const CAL_NAV_BG: u32 = 0x00F5F5F5;
const CAL_NAV_TEXT: u32 = 0x00333333;
const CAL_WEEKDAY_TEXT: u32 = 0x00666666;
const CAL_DATE_TEXT: u32 = 0x00333333;
const CAL_TODAY_BG: u32 = 0x00006B3C;
const CAL_TODAY_TEXT: u32 = 0x00FFFFFF;
const CAL_HOVER_BG: u32 = 0x00E8F5E9;

#[inline]
fn get_calendar_hwnd() -> HWND {
    HWND(CALENDAR_HWND_PTR.load(Ordering::Relaxed))
}

#[inline]
fn set_calendar_hwnd(hwnd: HWND) {
    CALENDAR_HWND_PTR.store(hwnd.0, Ordering::Relaxed);
}

/// Show the calendar popup
pub fn show_calendar(parent: HWND) {
    unsafe {
        let cal_hwnd = get_calendar_hwnd();
        // If already open, just bring to front
        if !cal_hwnd.is_invalid() && IsWindow(Some(cal_hwnd)).as_bool() {
            let _ = SetForegroundWindow(cal_hwnd);
            return;
        }

        // Initialize to current Bangla date
        let current = get_current_bangla_date();
        VIEW_MONTH.store(current.month, Ordering::Relaxed);
        VIEW_YEAR.store(current.year, Ordering::Relaxed);

        let instance = GetModuleHandleW(None).unwrap_or_default();
        let class_name = w!("BongoPunjikaClass");

        // Register window class
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(calendar_wndproc),
            hInstance: instance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: HBRUSH(std::ptr::null_mut()),
            lpszClassName: class_name,
            hIcon: get_flag_icon(),
            hIconSm: get_flag_icon(),
            ..Default::default()
        };
        RegisterClassExW(&wc);

        // Center on screen
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_width - CAL_WIDTH) / 2;
        let y = (screen_height - CAL_HEIGHT) / 2;

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_DLGMODALFRAME,
            class_name,
            w!("পুঞ্জিকা"),
            WS_POPUP | WS_VISIBLE | WS_CAPTION | WS_SYSMENU,
            x,
            y,
            CAL_WIDTH,
            CAL_HEIGHT,
            Some(parent),
            None,
            Some(instance.into()),
            None,
        )
        .unwrap_or_default();

        set_calendar_hwnd(hwnd);

        // Set rounded corners on Windows 11
        let preference = DWM_WINDOW_CORNER_PREFERENCE(2); // DWMWCP_ROUND
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preference as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
        );

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);
    }
}

/// Get the number of days in a Bangla month
fn get_bangla_month_days(month: i32, year: i32) -> i32 {
    // Bangladesh calendar: first 5 months have 31 days, rest have 30
    // Falgun (month 10) has 31 days in leap years
    if month < 5 {
        31
    } else if month == 10 {
        // Falgun - check leap year
        let gregorian_year = year + 594;
        if (gregorian_year % 4 == 0 && gregorian_year % 100 != 0) || (gregorian_year % 400 == 0) {
            31
        } else {
            30
        }
    } else {
        30
    }
}

/// Get the weekday of the first day of a Bangla month
fn get_first_day_weekday(month: i32, year: i32) -> i32 {
    // Convert Bangla date to Gregorian and get weekday
    // 1st Boishakh 1432 was April 14, 2025 (Monday = 1)

    let ref_year = 1432;
    let ref_month = 0; // Boishakh
    let ref_weekday = 1; // Monday

    // Calculate total days from reference
    let mut total_days = 0;

    if year > ref_year || (year == ref_year && month > ref_month) {
        // Forward from reference
        let mut y = ref_year;
        let mut m = ref_month;

        while y < year || (y == year && m < month) {
            total_days += get_bangla_month_days(m, y);
            m += 1;
            if m > 11 {
                m = 0;
                y += 1;
            }
        }
    } else if year < ref_year || (year == ref_year && month < ref_month) {
        // Backward from reference
        let mut y = ref_year;
        let mut m = ref_month;

        while y > year || (y == year && m > month) {
            m -= 1;
            if m < 0 {
                m = 11;
                y -= 1;
            }
            total_days -= get_bangla_month_days(m, y);
        }
    }

    let weekday = (ref_weekday + total_days % 7 + 7) % 7;
    weekday
}

/// Draw the calendar using cached fonts
fn draw_calendar(hdc: HDC, rect: &RECT) {
    unsafe {
        let month = VIEW_MONTH.load(Ordering::Relaxed);
        let year = VIEW_YEAR.load(Ordering::Relaxed);
        let hover_day = HOVER_DAY.load(Ordering::Relaxed);
        let current = get_current_bangla_date();
        let is_current_month = month == current.month && year == current.year;

        // Background
        let bg_brush = CreateSolidBrush(COLORREF(CAL_BG));
        FillRect(hdc, rect, bg_brush);
        let _ = DeleteObject(bg_brush.into());

        // Header background (green)
        let header_rect = RECT {
            left: 0,
            top: 0,
            right: rect.right,
            bottom: CAL_HEADER_HEIGHT,
        };
        let header_brush = CreateSolidBrush(COLORREF(CAL_HEADER_BG));
        FillRect(hdc, &header_rect, header_brush);
        let _ = DeleteObject(header_brush.into());

        // Use cached fonts (no allocation per paint!)
        let header_font = get_cal_header_font();
        let nav_font = get_cal_nav_font();
        let weekday_font = get_cal_weekday_font();
        let date_font = get_cal_date_font();
        let sub_font = get_cal_sub_font();

        SetBkMode(hdc, TRANSPARENT);

        // Draw header: Year, Season, Month
        let old_font = SelectObject(hdc, header_font.into());
        SetTextColor(hdc, COLORREF(CAL_HEADER_TEXT));

        // Month name
        let month_name = if month >= 0 && month < 12 {
            BANGLA_MONTHS[month as usize]
        } else {
            "?"
        };
        let mut month_text: Vec<u16> = month_name.encode_utf16().collect();
        let mut month_rect = RECT {
            left: CAL_PADDING,
            top: 8,
            right: rect.right - CAL_PADDING,
            bottom: 32,
        };
        DrawTextW(
            hdc,
            &mut month_text,
            &mut month_rect,
            DT_CENTER | DT_SINGLELINE,
        );

        // Year and Season
        let year_bangla = to_bangla_number(year);
        let season = if month >= 0 && month < 12 {
            BANGLA_SEASONS[month as usize]
        } else {
            "?"
        };
        let sub_text = format!("{} বঙ্গাব্দ • {}কাল", year_bangla, season);
        let mut sub_vec: Vec<u16> = sub_text.encode_utf16().collect();
        SelectObject(hdc, sub_font.into());
        let mut sub_rect = RECT {
            left: CAL_PADDING,
            top: 34,
            right: rect.right - CAL_PADDING,
            bottom: CAL_HEADER_HEIGHT - 4,
        };
        DrawTextW(hdc, &mut sub_vec, &mut sub_rect, DT_CENTER | DT_SINGLELINE);

        // Navigation bar background
        let nav_rect = RECT {
            left: 0,
            top: CAL_HEADER_HEIGHT,
            right: rect.right,
            bottom: CAL_HEADER_HEIGHT + CAL_NAV_HEIGHT,
        };
        let nav_brush = CreateSolidBrush(COLORREF(CAL_NAV_BG));
        FillRect(hdc, &nav_rect, nav_brush);
        let _ = DeleteObject(nav_brush.into());

        // Navigation arrows
        SelectObject(hdc, nav_font.into());
        SetTextColor(hdc, COLORREF(CAL_NAV_TEXT));

        // Previous button (◀ পূর্ববর্তী)
        let mut prev_text: Vec<u16> = "◀ পূর্ববর্তী".encode_utf16().collect();
        let mut prev_rect = RECT {
            left: CAL_PADDING,
            top: CAL_HEADER_HEIGHT + 8,
            right: CAL_WIDTH / 2 - 10,
            bottom: CAL_HEADER_HEIGHT + CAL_NAV_HEIGHT - 8,
        };
        DrawTextW(
            hdc,
            &mut prev_text,
            &mut prev_rect,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );

        // Next button (পরবর্তী ▶)
        let mut next_text: Vec<u16> = "পরবর্তী ▶".encode_utf16().collect();
        let mut next_rect = RECT {
            left: CAL_WIDTH / 2 + 10,
            top: CAL_HEADER_HEIGHT + 8,
            right: rect.right - CAL_PADDING,
            bottom: CAL_HEADER_HEIGHT + CAL_NAV_HEIGHT - 8,
        };
        DrawTextW(
            hdc,
            &mut next_text,
            &mut next_rect,
            DT_RIGHT | DT_VCENTER | DT_SINGLELINE,
        );

        // Weekday headers
        let weekday_y = CAL_HEADER_HEIGHT + CAL_NAV_HEIGHT + 5;
        SelectObject(hdc, weekday_font.into());
        SetTextColor(hdc, COLORREF(CAL_WEEKDAY_TEXT));

        // Short weekday names
        let weekday_short = ["রবি", "সোম", "মঙ্গল", "বুধ", "বৃহঃ", "শুক্র", "শনি"];
        let cell_width = (rect.right - CAL_PADDING * 2) / 7;

        for (i, day) in weekday_short.iter().enumerate() {
            let mut day_text: Vec<u16> = day.encode_utf16().collect();
            let mut day_rect = RECT {
                left: CAL_PADDING + (i as i32 * cell_width),
                top: weekday_y,
                right: CAL_PADDING + ((i + 1) as i32 * cell_width),
                bottom: weekday_y + CAL_WEEKDAY_HEIGHT,
            };
            DrawTextW(
                hdc,
                &mut day_text,
                &mut day_rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
        }

        // Draw separator line
        let sep_y = weekday_y + CAL_WEEKDAY_HEIGHT;
        let sep_pen = CreatePen(PS_SOLID, 1, COLORREF(0x00E0E0E0));
        let old_pen = SelectObject(hdc, sep_pen.into());
        let _ = MoveToEx(hdc, CAL_PADDING, sep_y, None);
        let _ = LineTo(hdc, rect.right - CAL_PADDING, sep_y);
        SelectObject(hdc, old_pen);
        let _ = DeleteObject(sep_pen.into());

        // Date grid
        let grid_y = sep_y + 5;
        let days_in_month = get_bangla_month_days(month, year);
        let first_weekday = get_first_day_weekday(month, year);

        SelectObject(hdc, date_font.into());

        let mut day = 1;
        let mut row = 0;

        while day <= days_in_month {
            for col in 0..7 {
                if row == 0 && col < first_weekday {
                    continue;
                }
                if day > days_in_month {
                    break;
                }

                let cell_x = CAL_PADDING + col * cell_width;
                let cell_y = grid_y + row * CAL_CELL_SIZE;

                let cell_rect = RECT {
                    left: cell_x + 2,
                    top: cell_y + 2,
                    right: cell_x + cell_width - 2,
                    bottom: cell_y + CAL_CELL_SIZE - 2,
                };

                let is_today = is_current_month && day == current.day;
                let is_hover = day == hover_day;

                // Draw cell background
                if is_today {
                    let today_brush = CreateSolidBrush(COLORREF(CAL_TODAY_BG));
                    let rgn = CreateRoundRectRgn(
                        cell_rect.left,
                        cell_rect.top,
                        cell_rect.right,
                        cell_rect.bottom,
                        8,
                        8,
                    );
                    let _ = FillRgn(hdc, rgn, today_brush);
                    let _ = DeleteObject(rgn.into());
                    let _ = DeleteObject(today_brush.into());
                    SetTextColor(hdc, COLORREF(CAL_TODAY_TEXT));
                } else if is_hover {
                    let hover_brush = CreateSolidBrush(COLORREF(CAL_HOVER_BG));
                    let rgn = CreateRoundRectRgn(
                        cell_rect.left,
                        cell_rect.top,
                        cell_rect.right,
                        cell_rect.bottom,
                        8,
                        8,
                    );
                    let _ = FillRgn(hdc, rgn, hover_brush);
                    let _ = DeleteObject(rgn.into());
                    let _ = DeleteObject(hover_brush.into());
                    SetTextColor(hdc, COLORREF(CAL_DATE_TEXT));
                } else {
                    SetTextColor(hdc, COLORREF(CAL_DATE_TEXT));
                }

                // Draw day number
                let day_str = to_bangla_number(day);
                let mut day_vec: Vec<u16> = day_str.encode_utf16().collect();
                let mut text_rect = cell_rect;
                DrawTextW(
                    hdc,
                    &mut day_vec,
                    &mut text_rect,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );

                day += 1;
            }
            row += 1;
        }

        // Restore original font - do NOT delete cached fonts
        SelectObject(hdc, old_font);
    }
}

/// Get the day number at a mouse position
fn get_day_at_point(x: i32, y: i32, rect: &RECT) -> i32 {
    let month = VIEW_MONTH.load(Ordering::Relaxed);
    let year = VIEW_YEAR.load(Ordering::Relaxed);

    let weekday_y = CAL_HEADER_HEIGHT + CAL_NAV_HEIGHT + 5;
    let sep_y = weekday_y + CAL_WEEKDAY_HEIGHT;
    let grid_y = sep_y + 5;

    if y < grid_y {
        return -1;
    }

    let cell_width = (rect.right - CAL_PADDING * 2) / 7;
    let col = (x - CAL_PADDING) / cell_width;
    let row = (y - grid_y) / CAL_CELL_SIZE;

    if col < 0 || col >= 7 || row < 0 {
        return -1;
    }

    let first_weekday = get_first_day_weekday(month, year);
    let cell_index = row * 7 + col;
    let day = cell_index - first_weekday + 1;

    let days_in_month = get_bangla_month_days(month, year);
    if day >= 1 && day <= days_in_month {
        day
    } else {
        -1
    }
}

/// Navigate to previous month
fn go_prev_month() {
    let mut month = VIEW_MONTH.load(Ordering::Relaxed);
    let mut year = VIEW_YEAR.load(Ordering::Relaxed);

    month -= 1;
    if month < 0 {
        month = 11;
        year -= 1;
    }

    VIEW_MONTH.store(month, Ordering::Relaxed);
    VIEW_YEAR.store(year, Ordering::Relaxed);
}

/// Navigate to next month
fn go_next_month() {
    let mut month = VIEW_MONTH.load(Ordering::Relaxed);
    let mut year = VIEW_YEAR.load(Ordering::Relaxed);

    month += 1;
    if month > 11 {
        month = 0;
        year += 1;
    }

    VIEW_MONTH.store(month, Ordering::Relaxed);
    VIEW_YEAR.store(year, Ordering::Relaxed);
}

/// Calendar window procedure
extern "system" fn calendar_wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match message {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);

                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);

                // Double buffer
                let mem_dc = CreateCompatibleDC(Some(hdc));
                let mem_bitmap = CreateCompatibleBitmap(hdc, rect.right, rect.bottom);
                let old_bitmap = SelectObject(mem_dc, mem_bitmap.into());

                draw_calendar(mem_dc, &rect);

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

                SelectObject(mem_dc, old_bitmap);
                let _ = DeleteObject(mem_bitmap.into());
                let _ = DeleteDC(mem_dc);

                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }

            WM_LBUTTONDOWN => {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

                // Check navigation clicks
                let nav_top = CAL_HEADER_HEIGHT;
                let nav_bottom = CAL_HEADER_HEIGHT + CAL_NAV_HEIGHT;

                if y >= nav_top && y < nav_bottom {
                    if x < CAL_WIDTH / 2 {
                        // Previous
                        go_prev_month();
                        let _ = InvalidateRect(Some(hwnd), None, true);
                    } else {
                        // Next
                        go_next_month();
                        let _ = InvalidateRect(Some(hwnd), None, true);
                    }
                }

                LRESULT(0)
            }

            WM_MOUSEMOVE => {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);

                let new_hover = get_day_at_point(x, y, &rect);
                if new_hover != HOVER_DAY.load(Ordering::Relaxed) {
                    HOVER_DAY.store(new_hover, Ordering::Relaxed);
                    let _ = InvalidateRect(Some(hwnd), None, true);
                }

                LRESULT(0)
            }

            WM_KEYDOWN => {
                match wparam.0 as i32 {
                    0x1B => {
                        // ESC
                        let _ = DestroyWindow(hwnd);
                    }
                    0x25 => {
                        // Left arrow
                        go_prev_month();
                        let _ = InvalidateRect(Some(hwnd), None, true);
                    }
                    0x27 => {
                        // Right arrow
                        go_next_month();
                        let _ = InvalidateRect(Some(hwnd), None, true);
                    }
                    _ => {}
                }
                LRESULT(0)
            }

            WM_DESTROY => {
                set_calendar_hwnd(HWND(std::ptr::null_mut()));
                HOVER_DAY.store(-1, Ordering::Relaxed);
                LRESULT(0)
            }

            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }
}
