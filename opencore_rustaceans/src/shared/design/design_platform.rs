//! Host-platform corner radii (logical points / CSS px).
//!
//! Values are taken from each platform's design documentation or reference
//! implementations, not guessed.
//!
//! | Platform | Window | Controls | Source |
//! |----------|--------|----------|--------|
//! | Windows 11 | 8px | 4px | [Fluent geometry](https://learn.microsoft.com/en-us/windows/apps/design/style/rounded-corner) (`OverlayCornerRadius` / `ControlCornerRadius`) |
//! | macOS 11–15 | 10pt | 6pt | [Electron `native_window_mac.mm`](https://github.com/electron/electron/blob/master/shell/browser/native_window_mac.mm) uses 9pt; Big Sur community measurements ~9–10pt |
//! | macOS 26+ (Tahoe) | 12pt | 6pt | [WWDC 2025 / Apple HIG](https://lapcatsoftware.com/articles/2026/3/4.html) — titlebar windows; toolbar windows are larger and dynamic |
//! | Linux/GNOME | 15px | 6px | [libadwaita `--window-radius`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/css-variables.html) and button radius |
//! | Linux/KDE Breeze | ~10px | 5px | [Breeze `Frame_FrameRadius = 5`](https://invent.kde.org/plasma/breeze/-/commit/21db1883) scaled by `smallSpacing` — GTK apps default to GNOME values |

/// Native top-level window corner radius.
pub fn window_corner_radius() -> f32 {
    #[cfg(target_os = "macos")]
    {
        match macos_major_version() {
            Some(26..) => 12.0,
            Some(11..) => 10.0,
            _ => 10.0,
        }
    }
    #[cfg(target_os = "windows")]
    {
        8.0
    }
    #[cfg(target_os = "linux")]
    {
        linux_window_corner_radius()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        10.0
    }
}

/// In-page control corner radius (buttons, chips, list backplates).
pub fn control_corner_radius() -> f32 {
    #[cfg(target_os = "windows")]
    {
        4.0
    }
    #[cfg(target_os = "macos")]
    {
        6.0
    }
    #[cfg(target_os = "linux")]
    {
        linux_control_corner_radius()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        6.0
    }
}

#[cfg(target_os = "linux")]
fn linux_window_corner_radius() -> f32 {
    if is_kde_desktop() { 10.0 } else { 15.0 }
}

#[cfg(target_os = "linux")]
fn linux_control_corner_radius() -> f32 {
    if is_kde_desktop() { 5.0 } else { 6.0 }
}

#[cfg(target_os = "linux")]
fn is_kde_desktop() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|value| value.to_ascii_lowercase().contains("kde"))
        .unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn macos_major_version() -> Option<u32> {
    use std::sync::OnceLock;

    static VERSION: OnceLock<Option<u32>> = OnceLock::new();
    *VERSION.get_or_init(read_macos_major_version)
}

#[cfg(target_os = "macos")]
fn read_macos_major_version() -> Option<u32> {
    let version = sysctl_string(b"kern.osproductversion\0")?;
    version.split('.').next()?.parse().ok()
}

#[cfg(target_os = "macos")]
fn sysctl_string(name: &[u8]) -> Option<String> {
    use std::os::raw::{c_char, c_int, c_void};

    unsafe extern "C" {
        fn sysctlbyname(
            name: *const c_char,
            oldp: *mut c_void,
            oldlenp: *mut usize,
            newp: *mut c_void,
            newlen: usize,
        ) -> c_int;
    }

    let mut size = 0usize;
    let status = unsafe {
        sysctlbyname(
            name.as_ptr().cast(),
            std::ptr::null_mut(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if status != 0 || size == 0 {
        return None;
    }

    let mut buf = vec![0u8; size];
    let status = unsafe {
        sysctlbyname(
            name.as_ptr().cast(),
            buf.as_mut_ptr().cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if status != 0 {
        return None;
    }
    if buf.last() == Some(&0) {
        buf.pop();
    }

    String::from_utf8(buf).ok()
}

#[cfg(test)]
mod tests {
    use super::{control_corner_radius, window_corner_radius};

    #[test]
    fn window_radius_is_positive() {
        assert!(window_corner_radius() > 0.0);
    }

    #[test]
    fn control_radius_fits_inside_window_radius() {
        let control = control_corner_radius();
        let window = window_corner_radius();
        assert!(control > 0.0);
        assert!(control <= window);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_product_version_is_readable() {
        assert!(super::macos_major_version().is_some());
    }
}
