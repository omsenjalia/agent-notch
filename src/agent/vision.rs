//! Screen awareness: capture screenshots of the desktop and read the
//! foreground window.

use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Gdi as G;
use windows::Win32::UI::WindowsAndMessaging as WM;

use crate::agent::desktop;

/// Screen coordinates of one monitor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn width(self) -> i32 {
        self.right - self.left
    }

    pub fn height(self) -> i32 {
        self.bottom - self.top
    }
}

/// Every monitor, in `EnumDisplayMonitors` order.
pub fn all_screens() -> Vec<Rect> {
    struct Find {
        out: Vec<Rect>,
    }
    unsafe extern "system" fn cb(
        hmonitor: G::HMONITOR,
        _hdc: G::HDC,
        _lprc: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let find: &mut Find = &mut *(lparam.0 as *mut Find);
        let mut mi = G::MONITORINFO {
            cbSize: std::mem::size_of::<G::MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = G::GetMonitorInfoW(hmonitor, &mut mi);
        find.out.push(Rect {
            left: mi.rcMonitor.left,
            top: mi.rcMonitor.top,
            right: mi.rcMonitor.right,
            bottom: mi.rcMonitor.bottom,
        });
        BOOL(1)
    }
    let mut find = Find { out: Vec::new() };
    let lparam = LPARAM((&mut find as *mut Find) as isize);
    let hdc = unsafe { G::GetDC(HWND::default()) };
    let _ = unsafe { G::EnumDisplayMonitors(hdc, None, Some(cb), lparam) };
    let _ = unsafe { G::ReleaseDC(HWND::default(), hdc) };
    find.out
}

/// Capture a screenshot of a monitor as an RGB image.
pub fn capture_screenshot(monitor_index: usize) -> Option<image::RgbImage> {
    let rect = all_screens().get(monitor_index).copied()?;
    let w = rect.width();
    let h = rect.height();
    if w <= 0 || h <= 0 {
        return None;
    }
    unsafe {
        let hdc = G::GetDC(HWND::default());
        let hdc_mem = G::CreateCompatibleDC(hdc);
        let hbitmap = G::CreateCompatibleBitmap(hdc, w, h);
        let _ = G::SelectObject(hdc_mem, hbitmap);
        let _ = G::BitBlt(hdc_mem, 0, 0, w, h, hdc, rect.left, rect.top, G::SRCCOPY);
        let mut bmi = G::BITMAPINFO {
            bmiHeader: G::BITMAPINFOHEADER {
                biSize: std::mem::size_of::<G::BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 24,
                biCompression: 0,
                biSizeImage: (w * h * 3) as u32,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [G::RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }; 1],
        };
        let mut buf = vec![0u8; (w * h * 3) as usize];
        let _ = G::GetDIBits(
            hdc_mem,
            hbitmap,
            0,
            h as u32,
            Some(buf.as_mut_ptr() as *mut std::ffi::c_void),
            &mut bmi,
            G::DIB_RGB_COLORS,
        );
        let _ = G::ReleaseDC(HWND::default(), hdc);
        let _ = G::DeleteObject(hbitmap);
        let _ = G::DeleteDC(hdc_mem);
        // Negative biHeight: DIB rows come back top-down, so `buf` rows map
        // straight to image rows.
        let mut image = image::RgbImage::new(w as u32, h as u32);
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 3) as usize;
                image.put_pixel(
                    x as u32,
                    y as u32,
                    image::Rgb([buf[i], buf[i + 1], buf[i + 2]]),
                );
            }
        }
        Some(image)
    }
}

/// The foreground window's title and class name.
pub fn foreground_info() -> (String, String) {
    unsafe {
        let hwnd = WM::GetForegroundWindow();
        let mut class = [0u16; 260];
        let len = WM::GetClassNameW(hwnd, &mut class);
        (
            desktop::window_title(hwnd),
            String::from_utf16_lossy(&class[..len.max(0) as usize]),
        )
    }
}

/// Encode a screenshot as a PNG data URI, for vision models.
pub fn encode_png_data_uri(image: &image::RgbImage) -> Option<String> {
    let mut buf = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .ok()?;
    Some(format!("data:image/png;base64,{}", base64::encode(&buf)))
}

/// Everything the brain's "what is on screen right now" context is made of.
#[derive(Clone, Debug, Default)]
pub struct ScreenContext {
    pub foreground_title: String,
    pub foreground_app: String,
    pub clipboard: String,
    /// Screenshot data URIs (empty unless the provider takes vision input).
    pub screens: Vec<String>,
}

impl ScreenContext {
    /// Gather the current context. Screenshots are captured only when some
    /// provider in the chain accepts vision input.
    pub fn gather(chain: &crate::agent::ProviderChain) -> Self {
        let (title, app) = foreground_info();
        let clipboard = desktop::clipboard_read();
        let mut context = Self {
            foreground_title: title,
            foreground_app: app,
            clipboard,
            screens: Vec::new(),
        };
        if chain.any_vision() {
            for (index, _rect) in all_screens().iter().enumerate() {
                if let Some(image) = capture_screenshot(index) {
                    if let Some(uri) = encode_png_data_uri(&image) {
                        context.screens.push(uri);
                    }
                }
            }
        }
        context
    }

    /// Text summary used in the system prompt (no screenshot data).
    pub fn summary(&self) -> String {
        let window = if self.foreground_title.is_empty() && self.foreground_app.is_empty() {
            "No foreground window visible".to_owned()
        } else {
            format!(
                "Foreground window: \"{}\" ({})",
                self.foreground_title, self.foreground_app
            )
        };
        let clipboard = if self.clipboard.is_empty() {
            "clipboard: empty".to_owned()
        } else {
            // Don't dump the whole clipboard; a couple of lines is enough.
            let first: String = self
                .clipboard
                .lines()
                .next()
                .unwrap_or_default()
                .chars()
                .take(120)
                .collect();
            let total = self.clipboard.chars().count();
            format!("clipboard: \"{first}…\" ({total} chars)")
        };
        format!("{window}. {clipboard}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn primary_screen_captures() {
        if let Some(img) = capture_screenshot(0) {
            assert!(img.width() > 0 && img.height() > 0);
        }
    }

    #[test]
    fn rect_dimensions() {
        assert_eq!(
            Rect {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080
            }
            .width(),
            1920
        );
        assert_eq!(
            Rect {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080
            }
            .height(),
            1080
        );
    }

    #[test]
    fn data_uri_encoding() {
        let img = image::RgbImage::from_raw(2, 2, vec![0u8; 2 * 2 * 3]).unwrap();
        let uri = encode_png_data_uri(&img).unwrap();
        assert!(uri.starts_with("data:image/png;base64,"));
    }
}
