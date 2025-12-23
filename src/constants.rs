// constants.rs - All application constants

use windows::Win32::UI::WindowsAndMessaging::WM_USER;

// Custom message for tray icon
pub const WM_TRAYICON: u32 = WM_USER + 1;

// Menu item IDs
pub const IDM_PUNJIKA: u32 = 1000;
pub const IDM_AUTOSTART_YES: u32 = 1001;
pub const IDM_AUTOSTART_NO: u32 = 1002;
pub const IDM_COUNTRY_BD: u32 = 1003;
pub const IDM_COUNTRY_IN: u32 = 1004;
pub const IDM_FONT_LICENSE: u32 = 1005;
pub const IDM_WEBSITE: u32 = 1006;
pub const IDM_EXIT: u32 = 1007;

// App constants
pub const APP_NAME: &str = "BanglaCalendar";
pub const POS_KEY_X: &str = "PosX";
pub const POS_KEY_Y: &str = "PosY";
pub const COUNTRY_KEY: &str = "Country";

// UI Colors - Modern dark theme
pub const BG_COLOR: u32 = 0x00201A18; // Dark brown-black background
pub const TEXT_PRIMARY: u32 = 0x00FFFFFF; // White text
pub const TEXT_SECONDARY: u32 = 0x00B0B0B0; // Light gray
pub const BORDER_COLOR: u32 = 0x00404040; // Subtle border

// Widget dimensions
pub const WIDGET_HEIGHT: i32 = 90;
pub const CORNER_RADIUS: i32 = 12;
pub const PADDING: i32 = 8;

// Owner-drawn menu constants
pub const MENU_ITEM_HEIGHT: i32 = 28;
pub const MENU_FONT_SIZE: i32 = 18;
pub const MENU_BG_COLOR: u32 = 0x00FFFFFF;
pub const MENU_TEXT_COLOR: u32 = 0x00000000;
pub const MENU_HIGHLIGHT_BG: u32 = 0x00FFE0C0;
pub const MENU_DISABLED_TEXT: u32 = 0x00808080;
pub const MENU_CHECK_COLOR: u32 = 0x00008800;

// Bangla ordinal suffixes (১লা, ২রা, etc.)
pub const BANGLA_ORDINALS: &[&str] = &[
    "",     // 0 (not used)
    "১লা",   // 1
    "২রা",   // 2
    "৩রা",   // 3
    "৪ঠা",   // 4
    "৫ই",   // 5
    "৬ই",   // 6
    "৭ই",   // 7
    "৮ই",   // 8
    "৯ই",   // 9
    "১০ই",  // 10
    "১১ই",  // 11
    "১২ই",  // 12
    "১৩ই",  // 13
    "১৪ই",  // 14
    "১৫ই",  // 15
    "১৬ই",  // 16
    "১৭ই",  // 17
    "১৮ই",  // 18
    "১৯শে", // 19
    "২০শে", // 20
    "২১শে", // 21
    "২২শে", // 22
    "২৩শে", // 23
    "২৪শে", // 24
    "২৫শে", // 25
    "২৬শে", // 26
    "২৭শে", // 27
    "২৮শে", // 28
    "২৯শে", // 29
    "৩০শে", // 30
    "৩১শে", // 31
];

// Bangla month names
pub const BANGLA_MONTHS: &[&str] = &[
    "বৈশাখ",   // 0 - Boishakh (Apr-May)
    "জ্যৈষ্ঠ",  // 1 - Jyoishtho (May-Jun)
    "আষাঢ়",    // 2 - Asharh (Jun-Jul)
    "শ্রাবণ",   // 3 - Shrabon (Jul-Aug)
    "ভাদ্র",    // 4 - Bhadro (Aug-Sep)
    "আশ্বিন",  // 5 - Ashwin (Sep-Oct)
    "কার্তিক",  // 6 - Kartik (Oct-Nov)
    "অগ্রহায়ণ", // 7 - Ogrohayon (Nov-Dec)
    "পৌষ",    // 8 - Poush (Dec-Jan)
    "মাঘ",     // 9 - Magh (Jan-Feb)
    "ফাল্গুন",   // 10 - Falgun (Feb-Mar)
    "চৈত্র",   // 11 - Choitro (Mar-Apr)
];

// Bangla seasons (ঋতু)
pub const BANGLA_SEASONS: &[&str] = &[
    "গ্রীষ্ম", // 0 - Grishmo (Summer) - Boishakh
    "গ্রীষ্ম", // 1 - Jyoishtho
    "বর্ষা",   // 2 - Borsha (Rainy) - Asharh
    "বর্ষা",   // 3 - Shrabon
    "শরৎ",   // 4 - Shorot (Autumn) - Bhadro
    "শরৎ",   // 5 - Ashwin
    "হেমন্ত", // 6 - Hemonto (Late Autumn) - Kartik
    "হেমন্ত", // 7 - Ogrohayon
    "শীত",   // 8 - Sheet (Winter) - Poush
    "শীত",   // 9 - Magh
    "বসন্ত",  // 10 - Boshonto (Spring) - Falgun
    "বসন্ত",  // 11 - Choitro
];

// Bangla weekday names
pub const BANGLA_WEEKDAYS: &[&str] = &[
    "রবিবার",    // 0 - Sunday
    "সোমবার",    // 1 - Monday
    "মঙ্গলবার",   // 2 - Tuesday
    "বুধবার",     // 3 - Wednesday
    "বৃহস্পতিবার", // 4 - Thursday
    "শুক্রবার",    // 5 - Friday
    "শনিবার",    // 6 - Saturday
];

// Bangla numerals
pub const BANGLA_DIGITS: &[char] = &['০', '১', '২', '৩', '৪', '৫', '৬', '৭', '৮', '৯'];
