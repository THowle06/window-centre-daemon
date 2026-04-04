#![windows_subsystem = "windows"]

mod app_state;
mod tray;
mod window;

use std::collections::HashSet;
use std::sync::{Mutex, MutexGuard};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, EVENT_SYSTEM_FOREGROUND, EnumWindows,
    GetMessageW, IDI_APPLICATION, LoadIconW, MSG, RegisterClassW, TranslateMessage,
    WINEVENT_OUTOFCONTEXT, WM_COMMAND, WM_DESTROY, WM_USER, WNDCLASSW,
};
use windows::core::{PCWSTR, Result};

const WM_TRAYICON: u32 = WM_USER + 1;

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let seen_ptr: *mut Mutex<HashSet<isize>> = lparam.0 as *mut Mutex<HashSet<isize>>;

    if !seen_ptr.is_null() {
        let seen: &Mutex<HashSet<isize>> = unsafe { &*seen_ptr };
        let mut seen: MutexGuard<'_, HashSet<isize>> = seen.lock().unwrap();
        seen.insert(hwnd.0);
    }

    true.into()
}

unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _thread_id: u32,
    _time: u32,
) {
    window::handle_window(hwnd);
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAYICON => tray::handle_tray_event(hwnd, lparam),
        WM_COMMAND => tray::handle_command(hwnd, wparam),
        WM_DESTROY => tray::handle_destroy(),
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn main() -> Result<()> {
    unsafe {
        app_state::init();

        let state: &app_state::AppState = app_state::get();
        let seen_ptr: *mut Mutex<HashSet<isize>> =
            &state.seen_windows as *const Mutex<HashSet<isize>> as *mut Mutex<HashSet<isize>>;

        let _ = EnumWindows(Some(enum_windows_proc), LPARAM(seen_ptr as isize));

        let class_name: Vec<u16> = "TrayWindow\0".encode_utf16().collect();

        let wc: WNDCLASSW = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        RegisterClassW(&wc);

        let hwnd: HWND = CreateWindowExW(
            Default::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(class_name.as_ptr()),
            Default::default(),
            0,
            0,
            0,
            0,
            HWND(0),
            None,
            None,
            None,
        );

        let mut nid: NOTIFYICONDATAW = NOTIFYICONDATAW::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_MESSAGE | NIF_TIP | NIF_ICON;
        nid.uCallbackMessage = WM_TRAYICON;

        let tip: Vec<u16> = "Window Centre\0".encode_utf16().collect();
        nid.szTip[..tip.len()].copy_from_slice(&tip);

        nid.hIcon = LoadIconW(None, IDI_APPLICATION).unwrap();

        let _ = Shell_NotifyIconW(NIM_ADD, &mut nid);

        let hook: HWINEVENTHOOK = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        if hook.0 == 0 {
            return Ok(());
        }

        let mut msg: MSG = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = UnhookWinEvent(hook);
        let _ = Shell_NotifyIconW(NIM_DELETE, &mut nid);
    }

    Ok(())
}
