use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetSystemMetrics, GetWindowRect, SM_CXSCREEN, SM_CYSCREEN, SWP_NOSIZE,
    SWP_NOZORDER, SetWindowPos,
};

fn main() {
    unsafe {
        // Step 1: Get the currently active window
        let hwnd: HWND = GetForegroundWindow();

        if hwnd.0 == 0 {
            println!("No active window found.");
            return;
        }

        // Step 2: Get window dimensions
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            println!("Failed to get window dimensions.");
            return;
        }

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        // Step 3: Get screen dimensions
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);

        // Step 4: Calculate centre position
        let x = (screen_width - width) / 2;
        let y = (screen_height - height) / 2;

        // Step 5: Move the window
        let result = SetWindowPos(hwnd, HWND(0), x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER);

        if result.is_ok() {
            println!("Window centred successfully.");
        } else {
            println!("Failed to move window.");
        }
    }
}
