// menu.rs - Context menu and owner-drawn menu handling

use crate::constants::*;
use crate::fonts::get_menu_font;
use crate::{AUTOSTART_ENABLED, COUNTRY_SELECTION, MENU_STRINGS};
use std::sync::atomic::Ordering;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::UI::Shell::*,
    Win32::UI::WindowsAndMessaging::*, core::*,
};

/// Helper to add an owner-drawn menu item
fn add_owner_drawn_item(
    menu: HMENU,
    position: u32,
    id: u32,
    text: &str,
    is_checked: bool,
    is_disabled: bool,
    is_separator: bool,
) {
    let index = {
        let mut strings = MENU_STRINGS.lock().unwrap();
        let idx = strings.len();
        strings.push(text.to_string());
        idx
    };

    unsafe {
        if is_separator {
            let mii = MENUITEMINFOW {
                cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32,
                fMask: MIIM_FTYPE,
                fType: MFT_SEPARATOR,
                ..Default::default()
            };
            let _ = InsertMenuItemW(menu, position, true, &mii);
        } else {
            let mut state = MENU_ITEM_STATE(0);
            if is_checked {
                state |= MFS_CHECKED;
            }
            if is_disabled {
                state |= MFS_DISABLED;
            }

            let mii = MENUITEMINFOW {
                cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32,
                fMask: MIIM_FTYPE | MIIM_ID | MIIM_STATE | MIIM_DATA,
                fType: MFT_OWNERDRAW,
                fState: state,
                wID: id,
                dwItemData: index,
                ..Default::default()
            };
            let _ = InsertMenuItemW(menu, position, true, &mii);
        }
    }
}

/// Helper to add an owner-drawn submenu
fn add_owner_drawn_submenu(menu: HMENU, position: u32, submenu: HMENU, text: &str) {
    let index = {
        let mut strings = MENU_STRINGS.lock().unwrap();
        let idx = strings.len();
        strings.push(text.to_string());
        idx
    };

    unsafe {
        let mii = MENUITEMINFOW {
            cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32,
            fMask: MIIM_FTYPE | MIIM_SUBMENU | MIIM_DATA,
            fType: MFT_OWNERDRAW,
            hSubMenu: submenu,
            dwItemData: index,
            ..Default::default()
        };
        let _ = InsertMenuItemW(menu, position, true, &mii);
    }
}

pub fn show_context_menu(hwnd: HWND) {
    // Clear previous menu strings
    {
        let mut strings = MENU_STRINGS.lock().unwrap();
        strings.clear();
        strings.reserve(12); // Pre-allocate for expected menu items
    }

    unsafe {
        let menu = CreatePopupMenu().unwrap();
        let autostart = AUTOSTART_ENABLED.load(Ordering::Relaxed);
        let country = COUNTRY_SELECTION.load(Ordering::Relaxed);

        // পুঞ্জিকা (Calendar)
        add_owner_drawn_item(menu, 0, IDM_PUNJIKA, "পুঞ্জিকা", false, false, false);

        // Separator
        add_owner_drawn_item(menu, 1, 0, "", false, false, true);

        // বুট হওয়ার সময়ে খোলো - Submenu
        let autostart_submenu = CreatePopupMenu().unwrap();
        add_owner_drawn_item(
            autostart_submenu,
            0,
            IDM_AUTOSTART_YES,
            "হ্যাঁ",
            autostart,
            false,
            false,
        );
        add_owner_drawn_item(
            autostart_submenu,
            1,
            IDM_AUTOSTART_NO,
            "না",
            !autostart,
            false,
            false,
        );
        add_owner_drawn_submenu(menu, 2, autostart_submenu, "বুট হওয়ার সময়ে খোলো");

        // দেশ - Submenu
        let country_submenu = CreatePopupMenu().unwrap();
        add_owner_drawn_item(
            country_submenu,
            0,
            IDM_COUNTRY_BD,
            "বাংলাদেশ",
            country == 0,
            false,
            false,
        );
        add_owner_drawn_item(
            country_submenu,
            1,
            IDM_COUNTRY_IN,
            "ভারত (শীঘ্রই আসছে)",
            false,
            true,
            false,
        );
        add_owner_drawn_submenu(menu, 3, country_submenu, "দেশ");

        // Separator
        add_owner_drawn_item(menu, 4, 0, "", false, false, true);

        // ফন্ট লাইসেন্স
        add_owner_drawn_item(menu, 5, IDM_FONT_LICENSE, "ফন্ট লাইসেন্স", false, false, false);

        // ওয়েবসাইট
        add_owner_drawn_item(menu, 6, IDM_WEBSITE, "ওয়েবসাইট", false, false, false);

        // Separator
        add_owner_drawn_item(menu, 7, 0, "", false, false, true);

        // বন্ধ করুন
        add_owner_drawn_item(menu, 8, IDM_EXIT, "বন্ধ করুন", false, false, false);

        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        let _ = SetForegroundWindow(hwnd);
        let _ = TrackPopupMenu(menu, TPM_RIGHTBUTTON, pt.x, pt.y, Some(0), hwnd, None);
        let _ = DestroyMenu(menu);
    }
}

pub fn open_url(url: &str) {
    let url_wide: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();
    let operation = w!("open");

    unsafe {
        ShellExecuteW(
            None,
            operation,
            PCWSTR(url_wide.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        );
    }
}

/// Handle WM_MEASUREITEM for owner-drawn menus
pub fn handle_measure_item(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    unsafe {
        let mis = lparam.0 as *mut windows::Win32::UI::Controls::MEASUREITEMSTRUCT;
        if !mis.is_null() {
            (*mis).itemHeight = MENU_ITEM_HEIGHT as u32;
            (*mis).itemWidth = 200;

            let index = (*mis).itemData;
            if let Ok(strings) = MENU_STRINGS.lock() {
                if let Some(text) = strings.get(index) {
                    let hdc = GetDC(Some(hwnd));
                    let menu_font = get_menu_font();
                    let old_font = SelectObject(hdc, menu_font.into());
                    let text_wide: Vec<u16> = text.encode_utf16().collect();
                    let mut size = SIZE::default();
                    let _ = GetTextExtentPoint32W(hdc, &text_wide, &mut size);
                    SelectObject(hdc, old_font);
                    let _ = ReleaseDC(Some(hwnd), hdc);
                    (*mis).itemWidth = (size.cx + 40) as u32;
                }
            }
        }
        LRESULT(1)
    }
}

/// Handle WM_DRAWITEM for owner-drawn menus
pub fn handle_draw_item(lparam: LPARAM) -> LRESULT {
    unsafe {
        let dis = lparam.0 as *const windows::Win32::UI::Controls::DRAWITEMSTRUCT;
        if !dis.is_null() && (*dis).CtlType == windows::Win32::UI::Controls::ODT_MENU {
            let hdc = (*dis).hDC;
            let rc = (*dis).rcItem;
            let is_selected =
                ((*dis).itemState.0 & windows::Win32::UI::Controls::ODS_SELECTED.0) != 0;
            let is_disabled =
                ((*dis).itemState.0 & windows::Win32::UI::Controls::ODS_DISABLED.0) != 0;
            let is_checked =
                ((*dis).itemState.0 & windows::Win32::UI::Controls::ODS_CHECKED.0) != 0;

            // Draw background
            let bg_color = if is_selected {
                MENU_HIGHLIGHT_BG
            } else {
                MENU_BG_COLOR
            };
            let bg_brush = CreateSolidBrush(COLORREF(bg_color));
            FillRect(hdc, &rc, bg_brush);
            let _ = DeleteObject(bg_brush.into());

            // Draw text
            let index = (*dis).itemData;
            if let Ok(strings) = MENU_STRINGS.lock() {
                if let Some(text) = strings.get(index) {
                    let menu_font = get_menu_font();
                    let old_font = SelectObject(hdc, menu_font.into());
                    SetBkMode(hdc, TRANSPARENT);

                    let text_color = if is_disabled {
                        MENU_DISABLED_TEXT
                    } else {
                        MENU_TEXT_COLOR
                    };
                    SetTextColor(hdc, COLORREF(text_color));

                    let mut text_rect = rc;
                    text_rect.left += 28;
                    text_rect.right -= 10;

                    let mut text_wide: Vec<u16> = text.encode_utf16().collect();
                    DrawTextW(
                        hdc,
                        &mut text_wide,
                        &mut text_rect,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    );

                    SelectObject(hdc, old_font);

                    // Draw checkmark if checked (using system font for checkmark)
                    if is_checked {
                        let check_font = CreateFontW(
                            16,
                            0,
                            0,
                            0,
                            FW_BOLD.0 as i32,
                            0,
                            0,
                            0,
                            DEFAULT_CHARSET,
                            OUT_DEFAULT_PRECIS,
                            CLIP_DEFAULT_PRECIS,
                            CLEARTYPE_QUALITY,
                            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
                            w!("Segoe UI Symbol"),
                        );
                        let old_check_font = SelectObject(hdc, check_font.into());
                        SetTextColor(hdc, COLORREF(MENU_CHECK_COLOR));
                        let mut check_rect = rc;
                        check_rect.left += 6;
                        check_rect.right = check_rect.left + 20;
                        let mut check_mark: Vec<u16> = "✓".encode_utf16().collect();
                        DrawTextW(
                            hdc,
                            &mut check_mark,
                            &mut check_rect,
                            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                        );
                        SelectObject(hdc, old_check_font);
                        let _ = DeleteObject(check_font.into());
                    }
                }
            }
        }
        LRESULT(1)
    }
}
