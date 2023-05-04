/// Most backends happened to have exactly the same fields in their *Display struct
/// Maybe something like this may in some public API some day?
/// (important data from this struct is available through function like Context::screen_size)

#[allow(dead_code)]
pub(crate) struct NativeDisplayData {
    pub screen_width: i32,
    pub screen_height: i32,
    pub dpi_scale: f32,
    pub high_dpi: bool,
    pub quit_requested: bool,
    pub quit_ordered: bool,
}

impl Default for NativeDisplayData {
    fn default() -> NativeDisplayData {
        NativeDisplayData {
            screen_width: 1,
            screen_height: 1,
            dpi_scale: 1.,
            high_dpi: false,
            quit_requested: false,
            quit_ordered: false,
        }
    }
}

pub trait NativeDisplay: std::any::Any {
    fn screen_size(&self) -> (f32, f32);
    fn dpi_scale(&self) -> f32;
    fn high_dpi(&self) -> bool;
    fn order_quit(&mut self);
    fn request_quit(&mut self);
    fn cancel_quit(&mut self);

    fn set_cursor_grab(&mut self, _grab: bool);
    fn show_mouse(&mut self, _shown: bool);
    fn set_mouse_cursor(&mut self, _cursor_icon: crate::CursorIcon);
    fn set_window_size(&mut self, _new_width: u32, _new_height: u32);
    fn set_fullscreen(&mut self, _fullscreen: bool);
    fn clipboard_get(&mut self) -> Option<String>;
    fn clipboard_set(&mut self, _data: &str);
    fn dropped_file_count(&mut self) -> usize {
        0
    }
    fn dropped_file_bytes(&mut self, _index: usize) -> Option<Vec<u8>> {
        None
    }
    fn dropped_file_path(&mut self, _index: usize) -> Option<std::path::PathBuf> {
        None
    }
    fn show_keyboard(&mut self, _show: bool) {}
    #[cfg(target_vendor = "apple")]
    fn apple_gfx_api(&self) -> crate::conf::AppleGfxApi;
    #[cfg(target_vendor = "apple")]
    fn apple_view(&mut self) -> Option<crate::native::apple::frameworks::ObjcId>;

    fn as_any(&mut self) -> &mut dyn std::any::Any;

    fn get_gl_proc_addr(&self, procname: &str) -> Option<unsafe extern "C" fn()>;
}

pub mod module;

#[cfg(target_os = "linux")]
pub mod linux_x11;

/*#[cfg(target_os = "linux")]
pub mod linux_wayland;*/

#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "android")]
pub use android::*;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub mod apple;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "ios")]
pub mod ios;

#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod egl;

pub mod gl;
