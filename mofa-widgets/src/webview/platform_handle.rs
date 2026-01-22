//! Platform-specific window handle acquisition
//!
//! This module provides functions to obtain native window handles from the
//! current process without modifying Makepad internals.

use raw_window_handle::RawWindowHandle;

/// Error type for platform handle operations
#[derive(Debug)]
pub enum PlatformHandleError {
    NoWindow,
    NotOnMainThread,
    UnsupportedPlatform,
}

impl std::fmt::Display for PlatformHandleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoWindow => write!(f, "No window available"),
            Self::NotOnMainThread => write!(f, "Must be called from main thread"),
            Self::UnsupportedPlatform => write!(f, "Unsupported platform"),
        }
    }
}

impl std::error::Error for PlatformHandleError {}

/// A wrapper that holds a raw window handle for wry integration
pub struct NativeWindowHandle {
    #[cfg(target_os = "macos")]
    pub ns_view: std::ptr::NonNull<std::ffi::c_void>,
    #[cfg(target_os = "windows")]
    pub hwnd: isize,
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    _phantom: std::marker::PhantomData<()>,
}

unsafe impl Send for NativeWindowHandle {}
unsafe impl Sync for NativeWindowHandle {}

impl NativeWindowHandle {
    /// Get the raw window handle for use with wry
    #[cfg(target_os = "macos")]
    pub fn raw_handle(&self) -> RawWindowHandle {
        use raw_window_handle::AppKitWindowHandle;
        let handle = AppKitWindowHandle::new(self.ns_view);
        RawWindowHandle::AppKit(handle)
    }

    #[cfg(target_os = "windows")]
    pub fn raw_handle(&self) -> RawWindowHandle {
        use raw_window_handle::Win32WindowHandle;
        let handle = Win32WindowHandle::new(
            std::num::NonZeroIsize::new(self.hwnd).unwrap()
        );
        RawWindowHandle::Win32(handle)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    pub fn raw_handle(&self) -> RawWindowHandle {
        unimplemented!("Platform not supported")
    }
}

// ============================================================================
// macOS Implementation
// ============================================================================

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use objc2::rc::Retained;
    use objc2_app_kit::{NSApplication, NSView, NSWindow};
    use objc2_foundation::MainThreadMarker;

    /// Get the NSView from the key window or main window on macOS
    ///
    /// Must be called from the main thread
    pub fn get_native_handle() -> Result<NativeWindowHandle, PlatformHandleError> {
        // Ensure we're on the main thread
        let mtm = MainThreadMarker::new().ok_or(PlatformHandleError::NotOnMainThread)?;

        // Get shared NSApplication
        let app = NSApplication::sharedApplication(mtm);

        // Try key window first, then main window, then first window in list
        let window: Retained<NSWindow> = app
            .keyWindow()
            .or_else(|| app.mainWindow())
            .or_else(|| {
                let windows = app.windows();
                if windows.len() > 0 {
                    Some(windows.objectAtIndex(0))
                } else {
                    None
                }
            })
            .ok_or(PlatformHandleError::NoWindow)?;

        // Get the content view
        let content_view: Option<Retained<NSView>> = window.contentView();
        let view = content_view.ok_or(PlatformHandleError::NoWindow)?;

        // Convert to raw pointer
        let view_ptr = Retained::as_ptr(&view) as *mut std::ffi::c_void;
        let ns_view =
            std::ptr::NonNull::new(view_ptr).ok_or(PlatformHandleError::NoWindow)?;

        Ok(NativeWindowHandle { ns_view })
    }
}

// ============================================================================
// Windows Implementation
// ============================================================================

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    /// Get the HWND from the foreground window on Windows
    pub fn get_native_handle() -> Result<NativeWindowHandle, PlatformHandleError> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0 == std::ptr::null_mut() {
                return Err(PlatformHandleError::NoWindow);
            }
            Ok(NativeWindowHandle { hwnd: hwnd.0 as isize })
        }
    }
}

// ============================================================================
// Unsupported platforms
// ============================================================================

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported {
    use super::*;

    pub fn get_native_handle() -> Result<NativeWindowHandle, PlatformHandleError> {
        Err(PlatformHandleError::UnsupportedPlatform)
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Get the native window handle for the current application
///
/// On macOS, this returns the content view of the key window.
/// On Windows, this returns the foreground window HWND.
///
/// # Errors
/// - `NoWindow`: No window is currently available
/// - `NotOnMainThread`: (macOS) Must be called from the main thread
/// - `UnsupportedPlatform`: Platform is not supported
pub fn get_native_handle() -> Result<NativeWindowHandle, PlatformHandleError> {
    #[cfg(target_os = "macos")]
    {
        macos::get_native_handle()
    }
    #[cfg(target_os = "windows")]
    {
        windows_impl::get_native_handle()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        unsupported::get_native_handle()
    }
}
