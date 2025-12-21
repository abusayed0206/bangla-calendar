#![windows_subsystem = "windows"]

mod constants;
mod calendar;
mod registry;
mod menu;
mod ui;

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicPtr, Ordering};
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::Graphics::Dwm::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::*,
};

use constants::*;
use registry::*;
use menu::*;
use ui::*;

// Embed the ICO file
const FLAG_ICO_DATA: &[u8] = include_bytes!("../assets/Flag_of_Bangladesh.ico");

// Global state
pub static AUTOSTART_ENABLED: AtomicBool = AtomicBool::new(false);
pub static COUNTRY_SELECTION: AtomicU32 = AtomicU32::new(0); // 0 = Bangladesh, 1 = India

// Global handles
pub static mut CUSTOM_FONT: HFONT = HFONT(std::ptr::null_mut());
pub static mut MENU_FONT: HFONT = HFONT(std::ptr::null_mut());
static FLAG_ICON_PTR: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());

// Storage for owner-drawn menu item strings
pub static MENU_STRINGS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

pub fn get_flag_icon() -> HICON {
    HICON(FLAG_ICON_PTR.load(Ordering::SeqCst))
}

fn set_flag_icon(icon: HICON) {
    FLAG_ICON_PTR.store(icon.0, Ordering::SeqCst);
}

/// Load Bangladesh flag icon from embedded ICO file
unsafe fn create_flag_icon() {
    unsafe {
        let icon = CreateIconFromResourceEx(
            FLAG_ICO_DATA,
            true,
            0x00030000, // Icon version
            32,
            32,
            LR_DEFAULTCOLOR,
        );
        
        if let Ok(ico) = icon {
            set_flag_icon(ico);
        }
    }
}

fn main() -> Result<()> {
    unsafe {
        // Enable autostart by default on first run
        let autostart_status = is_autostart_enabled();
        if !autostart_status && !has_run_before() {
            toggle_autostart(true);
            mark_has_run();
        }
        AUTOSTART_ENABLED.store(is_autostart_enabled(), Ordering::SeqCst);
        COUNTRY_SELECTION.store(load_country_selection(), Ordering::SeqCst);

        install_embedded_font();
        create_flag_icon();

        let instance = GetModuleHandleW(None)?;
        let window_class = w!("BongoWidgetClass");

        // Use our custom flag icon for the window class
        let flag_icon = get_flag_icon();
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH(std::ptr::null_mut()),
            lpszClassName: window_class,
            hIcon: flag_icon,
            hIconSm: flag_icon,
            ..Default::default()
        };

        RegisterClassExW(&wc);

        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);

        // Calculate widget width based on text
        let widget_width = calculate_widget_width();

        let (saved_x, saved_y) = load_position();
        let x = if saved_x >= 0 && saved_x < screen_width - 50 {
            saved_x
        } else {
            screen_width - widget_width - 20
        };
        let y = if saved_y >= 0 && saved_y < screen_height - 50 {
            saved_y
        } else {
            screen_height - WIDGET_HEIGHT - 80
        };

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_LAYERED,
            window_class,
            w!("বঙ্গ উইজেট"),
            WS_POPUP | WS_VISIBLE,
            x,
            y,
            widget_width,
            WIDGET_HEIGHT,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        // Explicitly set window icon for Task Manager
        if !flag_icon.is_invalid() {
            SendMessageW(hwnd, WM_SETICON, Some(WPARAM(ICON_BIG as usize)), Some(LPARAM(flag_icon.0 as isize)));
            SendMessageW(hwnd, WM_SETICON, Some(WPARAM(ICON_SMALL as usize)), Some(LPARAM(flag_icon.0 as isize)));
        }

        // Use color key for transparency (black = transparent)
        SetLayeredWindowAttributes(hwnd, COLORREF(0x00000000), 0, LWA_COLORKEY)?;

        set_desktop_level(hwnd);
        create_tray_icon(hwnd)?;
        SetTimer(Some(hwnd), 1, 60000, None);

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        KillTimer(Some(hwnd), 1)?;
        remove_tray_icon(hwnd)?;

        Ok(())
    }
}

extern "system" fn wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => LRESULT(0),
            
            WM_MEASUREITEM => handle_measure_item(hwnd, lparam),
            
            WM_DRAWITEM => handle_draw_item(lparam),
            
            WM_TIMER => {
                let _ = InvalidateRect(Some(hwnd), None, true);
                LRESULT(0)
            }
            
            WM_PAINT => handle_paint(hwnd),
            
            WM_LBUTTONDOWN => {
                let _ = DefWindowProcW(hwnd, WM_SYSCOMMAND, WPARAM(0xF012), LPARAM(0));
                LRESULT(0)
            }
            
            WM_MOVE => {
                let mut rect = RECT::default();
                if GetWindowRect(hwnd, &mut rect).is_ok() {
                    save_position(rect.left, rect.top);
                }
                LRESULT(0)
            }
            
            WM_RBUTTONUP => {
                show_context_menu(hwnd);
                LRESULT(0)
            }
            
            WM_TRAYICON => {
                let event = (lparam.0 & 0xFFFF) as u32;
                if event == WM_RBUTTONUP {
                    show_context_menu(hwnd);
                }
                LRESULT(0)
            }
            
            // Set rounded corners on popup menus (Windows 11)
            WM_ENTERIDLE => {
                if wparam.0 == 2 { // MSGF_MENU
                    let menu_hwnd = HWND(lparam.0 as *mut std::ffi::c_void);
                    if !menu_hwnd.is_invalid() {
                        let preference = DWM_WINDOW_CORNER_PREFERENCE(2); // DWMWCP_ROUND
                        let _ = DwmSetWindowAttribute(
                            menu_hwnd,
                            DWMWA_WINDOW_CORNER_PREFERENCE,
                            &preference as *const _ as *const std::ffi::c_void,
                            std::mem::size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
                        );
                    }
                }
                LRESULT(0)
            }
            
            WM_COMMAND => {
                let cmd = (wparam.0 & 0xFFFF) as u32;
                match cmd {
                    IDM_AUTOSTART_YES => {
                        toggle_autostart(true);
                    }
                    IDM_AUTOSTART_NO => {
                        toggle_autostart(false);
                    }
                    IDM_COUNTRY_BD => {
                        COUNTRY_SELECTION.store(0, Ordering::SeqCst);
                        save_country_selection(0);
                        let _ = InvalidateRect(Some(hwnd), None, true);
                    }
                    IDM_COUNTRY_IN => {
                        // India - disabled for now
                    }
                    IDM_FONT_LICENSE => {
                        open_url("https://codepotro.com/font/ekush/");
                    }
                    IDM_WEBSITE => {
                        open_url("https://sayed.app");
                    }
                    IDM_EXIT => {
                        let _ = DestroyWindow(hwnd);
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }
}
