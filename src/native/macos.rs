//! MacOs implementation is basically a mix between
//! sokol_app's objective C code and Makepad's (https://github.com/makepad/makepad/blob/live/platform/src/platform/apple)
//! platform implementation
//!

use crate::skia::SkiaContext;
use {
    crate::{
        conf::AppleGfxApi,
        event::{EventHandler, MouseButton},
        native::{
            apple::{apple_util::*, frameworks::*},
            gl, NativeDisplay, NativeDisplayData,
        },
        CursorIcon,
    },
    std::{collections::HashMap, os::raw::c_void},
};

pub struct MacosDisplay {
    window: ObjcId,
    view: ObjcId,
    data: NativeDisplayData,
    fullscreen: bool,
    // [NSCursor hide]/unhide calls should be balanced
    // hide/hide/unhide will keep cursor hidden
    // so need to keep internal cursor state to avoid problems from
    // unbalanced show_mouse() calls
    cursor_shown: bool,
    current_cursor: CursorIcon,
    cursors: HashMap<CursorIcon, ObjcId>,
    gfx_api: crate::conf::AppleGfxApi,
}

mod tl_display {
    use super::*;
    use crate::NATIVE_DISPLAY;

    use std::cell::RefCell;

    thread_local! {
        static DISPLAY: RefCell<Option<MacosDisplay>> = RefCell::new(None);
    }

    fn with_native_display(f: &mut dyn FnMut(&mut dyn crate::NativeDisplay)) {
        DISPLAY.with(|d| {
            f(&mut *d.borrow_mut().as_mut().unwrap());
        })
    }

    pub(super) fn with<T>(mut f: impl FnMut(&mut MacosDisplay) -> T) -> T {
        DISPLAY.with(|d| f(&mut *d.borrow_mut().as_mut().unwrap()))
    }

    pub(super) fn set_display(display: MacosDisplay) {
        DISPLAY.with(|d| *d.borrow_mut() = Some(display));
        NATIVE_DISPLAY.with(|d| *d.borrow_mut() = Some(with_native_display));
    }
}

impl NativeDisplay for MacosDisplay {
    fn screen_size(&self) -> (f32, f32) {
        (self.data.screen_width as _, self.data.screen_height as _)
    }
    fn dpi_scale(&self) -> f32 {
        self.data.dpi_scale
    }
    fn high_dpi(&self) -> bool {
        self.data.high_dpi
    }
    fn order_quit(&mut self) {
        self.data.quit_ordered = true;
    }
    fn request_quit(&mut self) {
        self.data.quit_requested = true;
    }
    fn cancel_quit(&mut self) {
        self.data.quit_requested = false;
    }

    fn set_cursor_grab(&mut self, _grab: bool) {}
    fn show_mouse(&mut self, show: bool) {
        if show && !self.cursor_shown {
            unsafe {
                let () = msg_send![class!(NSCursor), unhide];
            }
        }
        if !show && self.cursor_shown {
            unsafe {
                let () = msg_send![class!(NSCursor), hide];
            }
        }
        self.cursor_shown = show;
    }
    fn set_mouse_cursor(&mut self, cursor: crate::CursorIcon) {
        if self.current_cursor != cursor {
            self.current_cursor = cursor;
            unsafe {
                let _: () = msg_send![
                    self.window,
                    invalidateCursorRectsForView: self.view
                ];
            }
        }
    }
    fn set_window_size(&mut self, new_width: u32, new_height: u32) {
        let mut frame: NSRect = unsafe { msg_send![self.window, frame] };
        frame.origin.y += frame.size.height;
        frame.origin.y -= new_height as f64;
        frame.size = NSSize {
            width: new_width as f64,
            height: new_height as f64,
        };
        let () = unsafe { msg_send![self.window, setFrame:frame display:true animate:true] };
    }
    fn set_fullscreen(&mut self, fullscreen: bool) {
        if self.fullscreen != fullscreen {
            self.fullscreen = fullscreen;
            unsafe {
                let () = msg_send![self.window, toggleFullScreen: nil];
            }
        }
    }
    fn clipboard_get(&mut self) -> Option<String> {
        unsafe {
            let pasteboard: ObjcId = msg_send![class!(NSPasteboard), generalPasteboard];
            let content: ObjcId = msg_send![pasteboard, stringForType: NSStringPboardType];
            let string = nsstring_to_string(content);
            if string.is_empty() {
                return None;
            }
            Some(string)
        }
    }
    fn clipboard_set(&mut self, data: &str) {
        let str: ObjcId = str_to_nsstring(data);
        unsafe {
            let pasteboard: ObjcId = msg_send![class!(NSPasteboard), generalPasteboard];
            let () = msg_send![pasteboard, clearContents];
            let arr: ObjcId = msg_send![class!(NSArray), arrayWithObject: str];
            let () = msg_send![pasteboard, writeObjects: arr];
        }
    }
    #[cfg(target_vendor = "apple")]
    fn apple_gfx_api(&self) -> crate::conf::AppleGfxApi {
        self.gfx_api
    }
    #[cfg(target_vendor = "apple")]
    fn apple_view(&mut self) -> Option<crate::native::apple::frameworks::ObjcId> {
        Some(self.view)
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl MacosDisplay {
    fn transform_mouse_point(&self, point: &NSPoint) -> (f32, f32) {
        let new_x = point.x as f32 * self.data.dpi_scale;
        let new_y = self.data.screen_height as f32 - (point.y as f32 * self.data.dpi_scale) - 1.;

        (new_x, new_y)
    }

    unsafe fn update_dimensions(&mut self) -> Option<(i32, i32)> {
        if self.data.high_dpi {
            let screen: ObjcId = msg_send![self.window, screen];
            let dpi_scale: f64 = msg_send![screen, backingScaleFactor];
            self.data.dpi_scale = dpi_scale as f32;
        } else {
            self.data.dpi_scale = 1.0;
        }

        let bounds: NSRect = msg_send![self.view, bounds];
        let screen_width = (bounds.size.width as f32 * self.data.dpi_scale) as i32;
        let screen_height = (bounds.size.height as f32 * self.data.dpi_scale) as i32;

        let dim_changed =
            screen_width != self.data.screen_width || screen_height != self.data.screen_height;

        self.data.screen_width = screen_width;
        self.data.screen_height = screen_height;

        if dim_changed {
            Some((screen_width, screen_height))
        } else {
            None
        }
    }
}
struct WindowPayload {
    ctx: Option<(Box<dyn EventHandler>, SkiaContext)>,
    f: Option<Box<dyn 'static + FnOnce() -> Box<dyn EventHandler>>>,
}
impl WindowPayload {
    pub fn context(&mut self) -> Option<&mut (Box<dyn EventHandler>, SkiaContext)> {
        let ctx = self.ctx.as_mut()?;

        Some(ctx)
    }
}
pub fn define_app_delegate() -> *const Class {
    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("NSAppDelegate", superclass).unwrap();
    unsafe {
        decl.add_method(
            sel!(applicationShouldTerminateAfterLastWindowClosed:),
            yes1 as extern "C" fn(&Object, Sel, ObjcId) -> BOOL,
        );
    }

    return decl.register();
}

pub fn define_cocoa_window_delegate() -> *const Class {
    extern "C" fn window_should_close(this: &mut Object, _: Sel, _: ObjcId) -> BOOL {
        let payload = get_window_payload(this);

        unsafe {
            let capture_manager = msg_send_![class![MTLCaptureManager], sharedCaptureManager];
            msg_send_![capture_manager, stopCapture];
        }

        // only give user-code a chance to intervene when sapp_quit() wasn't already called
        if !tl_display::with(|d| d.data.quit_ordered) {
            // if window should be closed and event handling is enabled, give user code
            // a chance to intervene via sapp_cancel_quit()
            tl_display::with(|d| d.data.quit_requested = true);
            if let Some((event_handler, skia_ctx)) = payload.context() {
                event_handler.quit_requested_event(skia_ctx);
            }

            // user code hasn't intervened, quit the app
            if tl_display::with(|d| d.data.quit_requested) {
                tl_display::with(|d| d.data.quit_ordered = true);
            }
        }
        if tl_display::with(|d| d.data.quit_ordered) {
            YES
        } else {
            NO
        }
    }

    extern "C" fn window_did_resize(this: &mut Object, _: Sel, _: ObjcId) {
        let payload = get_window_payload(this);
        if let Some((w, h)) = unsafe { tl_display::with(|d| d.update_dimensions()) } {
            if let Some((event_handler, skia_ctx)) = payload.context() {
                event_handler.resize_event(skia_ctx, w as _, h as _);
            }
        }
    }

    extern "C" fn window_did_change_screen(this: &mut Object, _: Sel, _: ObjcId) {
        let payload = get_window_payload(this);
        if let Some((w, h)) = unsafe { tl_display::with(|d| d.update_dimensions()) } {
            if let Some((event_handler, skia_ctx)) = payload.context() {
                event_handler.resize_event(skia_ctx, w as _, h as _);
            }
        }
    }
    extern "C" fn window_did_enter_fullscreen(_: &Object, _: Sel, _: ObjcId) {
        tl_display::with(|d| d.fullscreen = true);
    }
    extern "C" fn window_did_exit_fullscreen(_: &Object, _: Sel, _: ObjcId) {
        tl_display::with(|d| d.fullscreen = false);
    }
    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("RenderWindowDelegate", superclass).unwrap();

    // Add callback methods
    unsafe {
        decl.add_method(
            sel!(windowShouldClose:),
            window_should_close as extern "C" fn(&mut Object, Sel, ObjcId) -> BOOL,
        );

        decl.add_method(
            sel!(windowDidResize:),
            window_did_resize as extern "C" fn(&mut Object, Sel, ObjcId),
        );
        decl.add_method(
            sel!(windowDidChangeScreen:),
            window_did_change_screen as extern "C" fn(&mut Object, Sel, ObjcId),
        );
        decl.add_method(
            sel!(windowDidEnterFullScreen:),
            window_did_enter_fullscreen as extern "C" fn(&Object, Sel, ObjcId),
        );
        decl.add_method(
            sel!(windowDidExitFullScreen:),
            window_did_exit_fullscreen as extern "C" fn(&Object, Sel, ObjcId),
        );
    }
    // Store internal state as user data
    decl.add_ivar::<*mut c_void>("display_ptr");

    return decl.register();
}

unsafe fn get_proc_address(name: *const u8) -> Option<unsafe extern "C" fn()> {
    mod libc {
        use std::ffi::{c_char, c_int, c_void};

        pub const RTLD_LAZY: c_int = 1;
        extern "C" {
            pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
            pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        }
    }
    static mut OPENGL: *mut std::ffi::c_void = std::ptr::null_mut();

    if OPENGL.is_null() {
        OPENGL = libc::dlopen(
            b"/System/Library/Frameworks/OpenGL.framework/Versions/Current/OpenGL\0".as_ptr() as _,
            libc::RTLD_LAZY,
        );
    }

    assert!(!OPENGL.is_null());

    let symbol = libc::dlsym(OPENGL, name as _);
    if symbol.is_null() {
        return None;
    }
    Some(unsafe { std::mem::transmute_copy(&symbol) })
}

// methods for both metal or opengl view
unsafe fn view_base_decl(decl: &mut ClassDecl) {
    extern "C" fn mouse_moved(this: &mut Object, _sel: Sel, event: ObjcId) {
        let payload = get_window_payload(this);

        unsafe {
            let point: NSPoint = msg_send!(event, locationInWindow);
            let point = tl_display::with(|d| d.transform_mouse_point(&point));
            if let Some((event_handler, skia_ctx)) = payload.context() {
                event_handler.mouse_motion_event(skia_ctx, point.0, point.1);
            }
        }
    }

    fn fire_mouse_event(this: &mut Object, event: ObjcId, down: bool, btn: MouseButton) {
        let payload = get_window_payload(this);

        unsafe {
            let point: NSPoint = msg_send!(event, locationInWindow);
            let point = tl_display::with(|d| d.transform_mouse_point(&point));
            if let Some((event_handler, skia_ctx)) = payload.context() {
                if down {
                    event_handler.mouse_button_down_event(skia_ctx, btn, point.0, point.1);
                } else {
                    event_handler.mouse_button_up_event(skia_ctx, btn, point.0, point.1);
                }
            }
        }
    }
    extern "C" fn mouse_down(this: &mut Object, _sel: Sel, event: ObjcId) {
        fire_mouse_event(this, event, true, MouseButton::Left);
    }
    extern "C" fn mouse_up(this: &mut Object, _sel: Sel, event: ObjcId) {
        fire_mouse_event(this, event, false, MouseButton::Left);
    }
    extern "C" fn right_mouse_down(this: &mut Object, _sel: Sel, event: ObjcId) {
        fire_mouse_event(this, event, true, MouseButton::Right);
    }
    extern "C" fn right_mouse_up(this: &mut Object, _sel: Sel, event: ObjcId) {
        fire_mouse_event(this, event, false, MouseButton::Right);
    }
    extern "C" fn other_mouse_down(this: &mut Object, _sel: Sel, event: ObjcId) {
        fire_mouse_event(this, event, true, MouseButton::Middle);
    }
    extern "C" fn other_mouse_up(this: &mut Object, _sel: Sel, event: ObjcId) {
        fire_mouse_event(this, event, false, MouseButton::Middle);
    }
    extern "C" fn scroll_wheel(this: &mut Object, _sel: Sel, event: ObjcId) {
        let payload = get_window_payload(this);
        unsafe {
            let mut dx: f64 = msg_send![event, scrollingDeltaX];
            let mut dy: f64 = msg_send![event, scrollingDeltaY];

            if !msg_send![event, hasPreciseScrollingDeltas] {
                dx *= 10.0;
                dy *= 10.0;
            }
            if let Some((event_handler, skia_ctx)) = payload.context() {
                event_handler.mouse_wheel_event(skia_ctx, dx as f32, dy as f32);
            }
        }
    }
    extern "C" fn reset_cursor_rects(this: &Object, _sel: Sel) {
        unsafe {
            let cursor_id = tl_display::with(|d| {
                let current_cursor = d.current_cursor;
                let cursor_id = *d
                    .cursors
                    .entry(current_cursor)
                    .or_insert_with(|| load_mouse_cursor(current_cursor));
                assert!(!cursor_id.is_null());
                cursor_id
            });

            let bounds: NSRect = msg_send![this, bounds];
            let _: () = msg_send![
                this,
                addCursorRect: bounds
                cursor: cursor_id
            ];
        }
    }

    extern "C" fn key_down(this: &mut Object, _sel: Sel, event: ObjcId) {
        let payload = get_window_payload(this);
        let mods = unsafe { get_event_key_modifier(event) };
        let repeat: bool = unsafe { msg_send!(event, isARepeat) };
        if let Some(key) = unsafe { get_event_keycode(event) } {
            if let Some((event_handler, skia_ctx)) = payload.context() {
                event_handler.key_down_event(skia_ctx, key, mods, repeat);
            }
        }

        if let Some(character) = unsafe { get_event_char(event) } {
            if let Some((event_handler, skia_ctx)) = payload.context() {
                event_handler.char_event(skia_ctx, character, mods, repeat);
            }
        }
    }
    extern "C" fn key_up(this: &mut Object, _sel: Sel, event: ObjcId) {
        let payload = get_window_payload(this);
        let mods = unsafe { get_event_key_modifier(event) };
        if let Some(key) = unsafe { get_event_keycode(event) } {
            if let Some((event_handler, skia_ctx)) = payload.context() {
                event_handler.key_up_event(skia_ctx, key, mods);
            }
        }
    }

    decl.add_method(
        sel!(canBecomeKey),
        yes as extern "C" fn(&Object, Sel) -> BOOL,
    );
    decl.add_method(
        sel!(acceptsFirstResponder),
        yes as extern "C" fn(&Object, Sel) -> BOOL,
    );
    decl.add_method(sel!(isOpaque), yes as extern "C" fn(&Object, Sel) -> BOOL);
    decl.add_method(
        sel!(resetCursorRects),
        reset_cursor_rects as extern "C" fn(&Object, Sel),
    );
    decl.add_method(
        sel!(mouseMoved:),
        mouse_moved as extern "C" fn(&mut Object, Sel, ObjcId),
    );
    decl.add_method(
        sel!(mouseDragged:),
        mouse_moved as extern "C" fn(&mut Object, Sel, ObjcId),
    );
    decl.add_method(
        sel!(rightMouseDragged:),
        mouse_moved as extern "C" fn(&mut Object, Sel, ObjcId),
    );
    decl.add_method(
        sel!(otherMouseDragged:),
        mouse_moved as extern "C" fn(&mut Object, Sel, ObjcId),
    );
    decl.add_method(
        sel!(mouseDown:),
        mouse_down as extern "C" fn(&mut Object, Sel, ObjcId),
    );
    decl.add_method(
        sel!(mouseUp:),
        mouse_up as extern "C" fn(&mut Object, Sel, ObjcId),
    );
    decl.add_method(
        sel!(rightMouseDown:),
        right_mouse_down as extern "C" fn(&mut Object, Sel, ObjcId),
    );
    decl.add_method(
        sel!(rightMouseUp:),
        right_mouse_up as extern "C" fn(&mut Object, Sel, ObjcId),
    );
    decl.add_method(
        sel!(otherMouseDown:),
        other_mouse_down as extern "C" fn(&mut Object, Sel, ObjcId),
    );
    decl.add_method(
        sel!(otherMouseUp:),
        other_mouse_up as extern "C" fn(&mut Object, Sel, ObjcId),
    );
    decl.add_method(
        sel!(scrollWheel:),
        scroll_wheel as extern "C" fn(&mut Object, Sel, ObjcId),
    );
    decl.add_method(
        sel!(keyDown:),
        key_down as extern "C" fn(&mut Object, Sel, ObjcId),
    );
    decl.add_method(
        sel!(keyUp:),
        key_up as extern "C" fn(&mut Object, Sel, ObjcId),
    );
}

pub fn define_opengl_view_class() -> *const Class {
    //extern "C" fn dealloc(this: &Object, _sel: Sel) {}

    extern "C" fn reshape(this: &mut Object, _sel: Sel) {
        unsafe {
            let superclass = superclass(this);
            let () = msg_send![super(this, superclass), reshape];

            if let Some((w, h)) = tl_display::with(|d| d.update_dimensions()) {
                let payload = get_window_payload(this);
                if let Some((event_handler, skia_ctx)) = payload.context() {
                    event_handler.resize_event(skia_ctx, w as _, h as _);
                }
            }
        }
    }

    extern "C" fn draw_rect(this: &mut Object, _sel: Sel, _rect: NSRect) {
        let payload = get_window_payload(this);
        if let Some((event_handler, skia_ctx)) = payload.context() {
            event_handler.update(skia_ctx);
            event_handler.draw(skia_ctx);
        }

        unsafe {
            let ctx: ObjcId = msg_send![this, openGLContext];
            assert!(!ctx.is_null());
            let () = msg_send![ctx, flushBuffer];

            if tl_display::with(|d| d.data.quit_requested || d.data.quit_ordered) {
                let window = tl_display::with(|d| d.window);
                let () = msg_send![window, performClose: nil];
            }
        }
    }

    extern "C" fn prepare_open_gl(this: &mut Object, _sel: Sel) {
        unsafe {
            let superclass = superclass(this);
            let () = msg_send![super(this, superclass), prepareOpenGL];
            let mut swap_interval = 1;
            let ctx: ObjcId = msg_send![this, openGLContext];
            let () = msg_send![ctx,
                               setValues:&mut swap_interval
                               forParameter:NSOpenGLContextParameterSwapInterval];
            let () = msg_send![ctx, makeCurrentContext];
        }

        gl::load_gl_funcs(|proc| {
            let name = std::ffi::CString::new(proc).unwrap();

            unsafe { get_proc_address(name.as_ptr() as _) }
        });

        let skia_ctx = {
            // Skia initialization on OpenGL ES
            use skia_safe::gpu::{gl::FramebufferInfo, DirectContext};
            use std::convert::TryInto;

            let interface = skia_safe::gpu::gl::Interface::new_load_with(|proc| {
                if proc == "eglGetCurrentDisplay" {
                    return std::ptr::null();
                }
                let name = std::ffi::CString::new(proc).unwrap();
                unsafe {
                    match get_proc_address(name.as_ptr() as _) {
                        Some(procaddr) => procaddr as *const std::ffi::c_void,
                        None => std::ptr::null(),
                    }
                }
            })
            .expect("Failed to create Skia <-> OpenGL interface");

            let dctx = DirectContext::new_gl(Some(interface), None)
                .expect("Failed to create Skia's direct context");

            let fb_info = {
                let mut fboid: gl::GLint = 0;
                unsafe {
                    gl::glGetIntegerv(gl::GL_FRAMEBUFFER_BINDING, &mut fboid);
                }

                FramebufferInfo {
                    fboid: fboid.try_into().unwrap(),
                    format: gl::GL_RGBA8,
                }
            };

            let bounds: NSRect = unsafe { msg_send![this, bounds] };

            SkiaContext::new(
                dctx,
                fb_info,
                bounds.size.width as i32,
                bounds.size.height as i32,
            )
        };

        let payload = get_window_payload(this);
        let f = payload.f.take().unwrap();
        payload.ctx = Some((f(), skia_ctx));
    }

    extern "C" fn timer_fired(this: &Object, _sel: Sel, _: ObjcId) {
        unsafe {
            let () = msg_send!(this, setNeedsDisplay: YES);
        }
    }
    let superclass = class!(NSOpenGLView);
    let mut decl: ClassDecl = ClassDecl::new("RenderViewClass", superclass).unwrap();
    unsafe {
        //decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&Object, Sel));
        decl.add_method(
            sel!(timerFired:),
            timer_fired as extern "C" fn(&Object, Sel, ObjcId),
        );

        decl.add_method(
            sel!(prepareOpenGL),
            prepare_open_gl as extern "C" fn(&mut Object, Sel),
        );
        decl.add_method(sel!(reshape), reshape as extern "C" fn(&mut Object, Sel));
        decl.add_method(
            sel!(drawRect:),
            draw_rect as extern "C" fn(&mut Object, Sel, NSRect),
        );

        view_base_decl(&mut decl);
    }

    decl.add_ivar::<*mut c_void>("display_ptr");

    return decl.register();
}

pub fn define_metal_view_class() -> *const Class {
    let superclass = class!(MTKView);
    let mut decl = ClassDecl::new("RenderViewClass", superclass).unwrap();
    decl.add_ivar::<*mut c_void>("display_ptr");

    extern "C" fn timer_fired(this: &Object, _sel: Sel, _: ObjcId) {
        unsafe {
            let () = msg_send!(this, setNeedsDisplay: YES);
        }
    }

    // TODO: REMOVE THIS LATER
    #[allow(unreachable_code)]
    #[allow(unused_variables)]
    extern "C" fn draw_rect(this: &mut Object, _sel: Sel, _rect: NSRect) {
        let payload = get_window_payload(this);

        if payload.ctx.is_none() {
            let skia_ctx = {
                // Skia initialization on OpenGL ES
                use skia_safe::gpu::{gl::FramebufferInfo, DirectContext};
                use std::convert::TryInto;

                // TODO: Skia <-> Metal interface
                #[allow(clippy::diverging_sub_expression)]
                let interface = unimplemented!();

                // let interface = skia_safe::gpu::gl::Interface::new_load_with(|proc| {
                //     if proc == "eglGetCurrentDisplay" {
                //         return std::ptr::null();
                //     }
                //     let name = std::ffi::CString::new(proc).unwrap();
                //     unsafe {
                //         match get_proc_address(name.as_ptr() as _) {
                //             Some(procaddr) => procaddr as *const std::ffi::c_void,
                //             None => std::ptr::null(),
                //         }
                //     }
                // })
                // .expect("Failed to create Skia <-> Metal interface");

                let dctx = DirectContext::new_gl(Some(interface), None)
                    .expect("Failed to create Skia's direct context");

                let fb_info = {
                    let mut fboid: gl::GLint = 0;
                    unsafe {
                        gl::glGetIntegerv(gl::GL_FRAMEBUFFER_BINDING, &mut fboid);
                    };

                    FramebufferInfo {
                        fboid: fboid.try_into().unwrap(),
                        format: gl::GL_RGBA8,
                    }
                };

                todo!()
                //SkiaContext::new(dctx, fb_info, conf.window_width, conf.window_height)
            };

            let f = payload.f.take().unwrap();
            payload.ctx = Some((f(), skia_ctx));
        }

        if let Some((event_handler, skia_ctx)) = payload.context() {
            event_handler.update(skia_ctx);
            event_handler.draw(skia_ctx);
        }

        unsafe {
            if tl_display::with(|d| d.data.quit_requested || d.data.quit_ordered) {
                let window = tl_display::with(|d| d.window);
                let () = msg_send![window, performClose: nil];
            }
        }
    }

    unsafe {
        //decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&Object, Sel));
        decl.add_method(
            sel!(timerFired:),
            timer_fired as extern "C" fn(&Object, Sel, ObjcId),
        );
        decl.add_method(
            sel!(drawRect:),
            draw_rect as extern "C" fn(&mut Object, Sel, NSRect),
        );

        view_base_decl(&mut decl);
    }

    return decl.register();
}

fn get_window_payload(this: &mut Object) -> &mut WindowPayload {
    unsafe {
        let ptr: *mut c_void = *this.get_ivar("display_ptr");
        &mut *(ptr as *mut WindowPayload)
    }
}

unsafe fn create_metal_view(_window_frame: NSRect, sample_count: i32, _high_dpi: bool) -> ObjcId {
    let mtl_device_obj = MTLCreateSystemDefaultDevice();
    let view_class = define_metal_view_class();
    let view: ObjcId = msg_send![view_class, alloc];
    let view: ObjcId = msg_send![view, init];

    let () = msg_send![view, setDevice: mtl_device_obj];
    let () = msg_send![view, setColorPixelFormat: MTLPixelFormat::BGRA8Unorm];
    let () = msg_send![
        view,
        setDepthStencilPixelFormat: MTLPixelFormat::Depth32Float_Stencil8
    ];
    let () = msg_send![view, setSampleCount: sample_count];

    view
}

unsafe fn create_opengl_view(window_frame: NSRect, sample_count: i32, high_dpi: bool) -> ObjcId {
    use NSOpenGLPixelFormatAttribute::*;

    let mut attrs: Vec<u32> = vec![
        NSOpenGLPFAAccelerated as _,
        NSOpenGLPFADoubleBuffer as _,
        NSOpenGLPFAOpenGLProfile as _,
        NSOpenGLPFAOpenGLProfiles::NSOpenGLProfileVersion3_2Core as _,
        NSOpenGLPFAColorSize as _,
        24,
        NSOpenGLPFAAlphaSize as _,
        8,
        NSOpenGLPFADepthSize as _,
        24,
        NSOpenGLPFAStencilSize as _,
        8,
    ];

    if sample_count > 1 {
        attrs.push(NSOpenGLPFAMultisample as _);
        attrs.push(NSOpenGLPFASampleBuffers as _);
        attrs.push(1 as _);
        attrs.push(NSOpenGLPFASamples as _);
        attrs.push(sample_count as _);
    } else {
        attrs.push(NSOpenGLPFASampleBuffers as _);
        attrs.push(0);
    }
    attrs.push(0);

    let glpixelformat_obj: ObjcId = msg_send![class!(NSOpenGLPixelFormat), alloc];
    let glpixelformat_obj: ObjcId =
        msg_send![glpixelformat_obj, initWithAttributes: attrs.as_ptr()];
    assert!(!glpixelformat_obj.is_null());

    let view_class = define_opengl_view_class();
    let view: ObjcId = msg_send![view_class, alloc];
    let view: ObjcId = msg_send![
        view,
        initWithFrame: window_frame
        pixelFormat: glpixelformat_obj
    ];

    if high_dpi {
        let () = msg_send![view, setWantsBestResolutionOpenGLSurface: YES];
    } else {
        let () = msg_send![view, setWantsBestResolutionOpenGLSurface: NO];
    }

    view
}

pub unsafe fn run<F>(conf: crate::conf::Conf, f: F)
where
    F: 'static + FnOnce() -> Box<dyn EventHandler>,
{
    let mut payload = WindowPayload {
        f: Some(Box::new(f)),
        ctx: None,
    };

    let mut display = MacosDisplay {
        view: std::ptr::null_mut(),
        window: std::ptr::null_mut(),
        data: NativeDisplayData {
            high_dpi: conf.high_dpi,
            ..Default::default()
        },
        fullscreen: false,
        cursor_shown: true,
        current_cursor: CursorIcon::Default,
        cursors: HashMap::new(),
        gfx_api: conf.platform.apple_gfx_api,
    };

    let app_delegate_class = define_app_delegate();
    let app_delegate_instance: ObjcId = msg_send![app_delegate_class, new];

    let ns_app: ObjcId = msg_send![class!(NSApplication), sharedApplication];
    let () = msg_send![ns_app, setDelegate: app_delegate_instance];
    let () = msg_send![
        ns_app,
        setActivationPolicy: NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular
            as i64
    ];
    let () = msg_send![ns_app, activateIgnoringOtherApps: YES];

    let window_masks = NSWindowStyleMask::NSTitledWindowMask as u64
        | NSWindowStyleMask::NSClosableWindowMask as u64
        | NSWindowStyleMask::NSMiniaturizableWindowMask as u64
        | NSWindowStyleMask::NSResizableWindowMask as u64;
    //| NSWindowStyleMask::NSFullSizeContentViewWindowMask as u64;

    let window_frame = NSRect {
        origin: NSPoint { x: 0., y: 0. },
        size: NSSize {
            width: conf.window_width as f64,
            height: conf.window_height as f64,
        },
    };

    let window: ObjcId = msg_send![class!(NSWindow), alloc];
    let window: ObjcId = msg_send![
        window,
        initWithContentRect: window_frame
        styleMask: window_masks
        backing: NSBackingStoreType::NSBackingStoreBuffered as u64
        defer: NO
    ];
    assert!(!window.is_null());

    let window_delegate_class = define_cocoa_window_delegate();
    let window_delegate: ObjcId = msg_send![window_delegate_class, new];
    let () = msg_send![window, setDelegate: window_delegate];

    (*window_delegate).set_ivar("display_ptr", &mut payload as *mut _ as *mut c_void);

    let title = str_to_nsstring(&conf.window_title);
    //let () = msg_send![window, setReleasedWhenClosed: NO];
    let () = msg_send![window, setTitle: title];
    let () = msg_send![window, center];
    let () = msg_send![window, setAcceptsMouseMovedEvents: YES];

    let view = match conf.platform.apple_gfx_api {
        AppleGfxApi::OpenGl => create_opengl_view(window_frame, conf.sample_count, conf.high_dpi),
        AppleGfxApi::Metal => create_metal_view(window_frame, conf.sample_count, conf.high_dpi),
    };
    (*view).set_ivar("display_ptr", &mut payload as *mut _ as *mut c_void);

    display.window = window;
    display.view = view;
    let _ = display.update_dimensions();
    tl_display::set_display(display);

    let nstimer: ObjcId = msg_send![
        class!(NSTimer),
        timerWithTimeInterval: 0.001
        target: view
        selector: sel!(timerFired:)
        userInfo: nil
        repeats: true
    ];
    let nsrunloop: ObjcId = msg_send![class!(NSRunLoop), currentRunLoop];
    let () = msg_send![nsrunloop, addTimer: nstimer forMode: NSDefaultRunLoopMode];
    assert!(!view.is_null());

    let () = msg_send![window, setContentView: view];
    let () = msg_send![window, makeFirstResponder: view];

    if conf.fullscreen {
        let () = msg_send![window, toggleFullScreen: nil];
    }

    let () = msg_send![window, makeKeyAndOrderFront: nil];

    let ns_app: ObjcId = msg_send![class!(NSApplication), sharedApplication];

    let () = msg_send![ns_app, run];

    // run should never return
    // but just in case
    unreachable!();
}
