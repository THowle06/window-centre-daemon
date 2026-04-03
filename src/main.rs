#![windows_subsystem = "windows"]

use std::collections::HashSet;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::{thread, time::Duration};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
    EVENT_SYSTEM_FOREGROUND, EnumWindows, GetCursorPos, GetMessageW, GetSystemMetrics,
    GetWindowRect, HMENU, IDI_APPLICATION, InsertMenuW, IsZoomed, LoadIconW, MENU_ITEM_FLAGS, MSG,
    PostQuitMessage, RegisterClassW, SM_CXSCREEN, SM_CYSCREEN, SWP_NOSIZE, SWP_NOZORDER,
    SetForegroundWindow, SetWindowPos, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TrackPopupMenu,
    TranslateMessage, WINEVENT_OUTOFCONTEXT, WM_COMMAND, WM_DESTROY, WM_RBUTTONUP, WM_USER,
    WNDCLASSW,
};
use windows::core::{PCWSTR, Result};

static SEEN_WINDOWS: OnceLock<Mutex<HashSet<isize>>> = OnceLock::new();

const WM_TRAYICON: u32 = WM_USER + 1;
const ID_TRAY_EXIT: usize = 1;

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
    unsafe {
        if hwnd.0 == 0 {
            return;
        }

        if IsZoomed(hwnd).as_bool() {
            return;
        }

        let seen: &Mutex<HashSet<isize>> = SEEN_WINDOWS.get().unwrap();
        let mut seen: MutexGuard<'_, HashSet<isize>> = seen.lock().unwrap();

        if seen.contains(&hwnd.0) {
            return;
        }

        thread::sleep(Duration::from_millis(50));

        let mut rect: RECT = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return;
        }

        let width: i32 = rect.right - rect.left;
        let height: i32 = rect.bottom - rect.top;

        if width < 200 || height < 200 {
            return;
        }

        let screen_width: i32 = GetSystemMetrics(SM_CXSCREEN);
        let screen_height: i32 = GetSystemMetrics(SM_CYSCREEN);

        let x: i32 = (screen_width - width) / 2;
        let y: i32 = (screen_height - height) / 2;

        let _ = SetWindowPos(hwnd, HWND(0), x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER);

        seen.insert(hwnd.0);
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_TRAYICON => {
                if lparam.0 as u32 == WM_RBUTTONUP {
                    let menu: HMENU = CreatePopupMenu().unwrap();

                    let exit_text: Vec<u16> = "Exit\0".encode_utf16().collect();

                    let _ = InsertMenuW(
                        menu,
                        0,
                        MENU_ITEM_FLAGS(0),
                        ID_TRAY_EXIT,
                        PCWSTR(exit_text.as_ptr()),
                    );

                    let mut point: POINT = Default::default();
                    let _ = GetCursorPos(&mut point);

                    let _ = SetForegroundWindow(hwnd);

                    let _ = TrackPopupMenu(
                        menu,
                        TPM_LEFTALIGN | TPM_BOTTOMALIGN,
                        point.x,
                        point.y,
                        0,
                        hwnd,
                        None,
                    );
                }
                LRESULT(0)
            }

            WM_COMMAND => {
                if wparam.0 == ID_TRAY_EXIT {
                    let _ = DestroyWindow(hwnd);
                }
                LRESULT(0)
            }

            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }

            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn main() -> Result<()> {
    unsafe {
        SEEN_WINDOWS.set(Mutex::new(HashSet::new())).unwrap();

        let seen: &Mutex<HashSet<isize>> = SEEN_WINDOWS.get().unwrap();
        let seen_ptr: *mut Mutex<HashSet<isize>> =
            seen as *const Mutex<HashSet<isize>> as *mut Mutex<HashSet<isize>>;

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
