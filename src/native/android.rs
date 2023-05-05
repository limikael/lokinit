use crate::{
    event::{EventHandler, KeyCode, TouchPhase},
    native::egl::{self, LibEgl},
    native::NativeDisplay,
};

use std::{cell::RefCell, ffi::CString, sync::mpsc, thread};

pub use crate::gl::{self, *};

mod keycodes;

pub use ndk_sys;

pub mod ndk_utils;

#[no_mangle]
pub unsafe extern "C" fn JNI_OnLoad(
    vm: *mut ndk_sys::JavaVM,
    _: std::ffi::c_void,
) -> ndk_sys::jint {
    VM = vm as *mut _ as _;

    ndk_sys::JNI_VERSION_1_6 as _
}

extern "C" {
    fn loki_main();
}

/// Short recap on how lokinit on Android works
/// There is a MainActivity, a normal Java activity
/// It creates a View and pass a reference to a view to rust.
/// Rust spawn a thread that render things into this view as often as
/// possible.
/// Also MainActivty collects user input events and calls native rust functions.
///
/// This long explanation was to illustrate how we ended up with evets callback
/// and drawing in the different threads.
/// Message enum is used to send data from the callbacks to the drawing thread.
#[derive(Debug)]
enum Message {
    SurfaceChanged {
        window: *mut ndk_sys::ANativeWindow,
        width: i32,
        height: i32,
    },
    SurfaceCreated {
        window: *mut ndk_sys::ANativeWindow,
    },
    SurfaceDestroyed,
    Touch {
        phase: TouchPhase,
        touch_id: u64,
        x: f32,
        y: f32,
    },
    Character {
        character: u32,
    },
    KeyDown {
        keycode: KeyCode,
    },
    KeyUp {
        keycode: KeyCode,
    },
    Pause,
    Resume,
    Destroy,
}
unsafe impl Send for Message {}

thread_local! {
    static MESSAGES_TX: RefCell<Option<mpsc::Sender<Message>>> = RefCell::new(None);
}

fn send_message(message: Message) {
    MESSAGES_TX.with(|tx| {
        let mut tx = tx.borrow_mut();

        let Some(tx) = tx.as_mut() else {
            let msg = CString::new(format!(
                "Tried to send a message ({message:?}), but message channel does not exist"
            ))
            .unwrap();
            unsafe { console_warn(msg.as_ptr()) };
            return;
        };

        tx.send(message).unwrap_or_else(|e| {
            let msg = format!("SendError while trying to send a message: {e}");
            let msg = CString::new(msg).unwrap();
            unsafe { console_error(msg.as_ptr()) };
        });
    })
}

static mut ACTIVITY: ndk_sys::jobject = std::ptr::null_mut();
static mut VM: *mut ndk_sys::JavaVM = std::ptr::null_mut();

struct AndroidDisplay {
    screen_width: f32,
    screen_height: f32,
    fullscreen: bool,
    get_procaddr: Box<dyn Fn(&str) -> Option<unsafe extern "C" fn()>>,
}

mod tl_display {
    use super::*;
    use crate::NATIVE_DISPLAY;

    use std::cell::RefCell;

    thread_local! {
        static DISPLAY: RefCell<Option<AndroidDisplay>> = RefCell::new(None);
    }

    fn with_native_display(f: &mut dyn FnMut(&mut dyn crate::NativeDisplay)) {
        DISPLAY.with(|d| {
            let mut d = d.borrow_mut();
            let d = d.as_mut().unwrap();
            f(&mut *d);
        })
    }

    pub(super) fn with<T>(mut f: impl FnMut(&mut AndroidDisplay) -> T) -> T {
        DISPLAY.with(|d| {
            let mut d = d.borrow_mut();
            let d = d.as_mut().unwrap();
            f(&mut *d)
        })
    }

    pub(super) fn set_display(display: AndroidDisplay) {
        DISPLAY.with(|d| *d.borrow_mut() = Some(display));
        NATIVE_DISPLAY.with(|d| *d.borrow_mut() = Some(with_native_display));
    }
}

impl NativeDisplay for AndroidDisplay {
    fn screen_size(&self) -> (f32, f32) {
        (self.screen_width as _, self.screen_height as _)
    }
    fn dpi_scale(&self) -> f32 {
        1.
    }
    fn high_dpi(&self) -> bool {
        true
    }
    fn order_quit(&mut self) {}
    fn request_quit(&mut self) {}
    fn cancel_quit(&mut self) {}
    fn set_cursor_grab(&mut self, _grab: bool) {}
    fn show_mouse(&mut self, _shown: bool) {}
    fn set_mouse_cursor(&mut self, _cursor: crate::CursorIcon) {}
    fn set_window_size(&mut self, _new_width: u32, _new_height: u32) {}
    fn set_fullscreen(&mut self, fullscreen: bool) {
        unsafe {
            let env = attach_jni_env();
            set_full_screen(env, fullscreen);
        }
        self.fullscreen = fullscreen;
    }
    fn clipboard_get(&mut self) -> Option<String> {
        None
    }
    fn clipboard_set(&mut self, _data: &str) {}
    fn show_keyboard(&mut self, show: bool) {
        unsafe {
            let env = attach_jni_env();
            ndk_utils::call_void_method!(env, ACTIVITY, "showKeyboard", "(Z)V", show as i32);
        }
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn get_gl_proc_addr(&self, procname: &str) -> Option<unsafe extern "C" fn()> {
        (self.get_procaddr)(procname)
    }
}

pub unsafe fn console_debug(msg: *const ::std::os::raw::c_char) {
    ndk_sys::__android_log_write(
        ndk_sys::android_LogPriority_ANDROID_LOG_DEBUG as _,
        b"SAPP\0".as_ptr() as _,
        msg,
    );
}

pub unsafe fn console_info(msg: *const ::std::os::raw::c_char) {
    ndk_sys::__android_log_write(
        ndk_sys::android_LogPriority_ANDROID_LOG_INFO as _,
        b"SAPP\0".as_ptr() as _,
        msg,
    );
}

pub unsafe fn console_warn(msg: *const ::std::os::raw::c_char) {
    ndk_sys::__android_log_write(
        ndk_sys::android_LogPriority_ANDROID_LOG_WARN as _,
        b"SAPP\0".as_ptr() as _,
        msg,
    );
}

pub unsafe fn console_error(msg: *const ::std::os::raw::c_char) {
    ndk_sys::__android_log_write(
        ndk_sys::android_LogPriority_ANDROID_LOG_ERROR as _,
        b"SAPP\0".as_ptr() as _,
        msg,
    );
}

// fn log_info(message: &str) {
//     use std::ffi::CString;

//     let msg = CString::new(message).unwrap_or_else(|_| panic!());

//     unsafe { console_info(msg.as_ptr()) };
// }

struct MainThreadState {
    libegl: LibEgl,
    egl_display: egl::EGLDisplay,
    egl_config: egl::EGLConfig,
    egl_context: egl::EGLContext,
    surface: egl::EGLSurface,
    window: *mut ndk_sys::ANativeWindow,
    event_handler: Box<dyn EventHandler>,
    quit: bool,
}

impl MainThreadState {
    unsafe fn destroy_surface(&mut self) {
        (self.libegl.eglMakeCurrent.unwrap())(
            self.egl_display,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        (self.libegl.eglDestroySurface.unwrap())(self.egl_display, self.surface);
        self.surface = std::ptr::null_mut();
    }

    unsafe fn update_surface(&mut self, window: *mut ndk_sys::ANativeWindow) {
        if !self.window.is_null() {
            ndk_sys::ANativeWindow_release(self.window);
        }
        self.window = window;
        if !self.surface.is_null() {
            self.destroy_surface();
        }

        self.surface = (self.libegl.eglCreateWindowSurface.unwrap())(
            self.egl_display,
            self.egl_config,
            window as _,
            std::ptr::null_mut(),
        );

        assert!(!self.surface.is_null());

        let res = (self.libegl.eglMakeCurrent.unwrap())(
            self.egl_display,
            self.surface,
            self.surface,
            self.egl_context,
        );

        assert!(res != 0);
    }

    fn process_message(&mut self, msg: Message) {
        match msg {
            Message::SurfaceCreated { window } => unsafe {
                self.update_surface(window);
            },
            Message::SurfaceDestroyed => unsafe {
                self.destroy_surface();
            },
            Message::SurfaceChanged {
                window,
                width,
                height,
            } => {
                unsafe {
                    self.update_surface(window);
                }

                tl_display::with(|d| {
                    d.screen_width = width as _;
                    d.screen_height = height as _;
                });
                self.event_handler.resize_event(width as _, height as _);
            }
            Message::Touch {
                phase,
                touch_id,
                x,
                y,
            } => {
                self.event_handler.touch_event(phase, touch_id, x, y);
            }
            Message::Character { character } => {
                if let Some(character) = char::from_u32(character) {
                    self.event_handler
                        .char_event(character, Default::default(), false);
                }
            }
            Message::KeyDown { keycode } => {
                self.event_handler
                    .key_down_event(keycode, Default::default(), false);
            }
            Message::KeyUp { keycode } => {
                self.event_handler.key_up_event(keycode, Default::default());
            }
            Message::Pause => self.event_handler.window_minimized_event(),
            Message::Resume => {
                if tl_display::with(|d| d.fullscreen) {
                    unsafe {
                        let env = attach_jni_env();
                        set_full_screen(env, true);
                    }
                }

                self.event_handler.window_restored_event()
            }
            Message::Destroy => {
                self.quit = true;
            }
        }
    }

    fn frame(&mut self) {
        self.event_handler.update();

        if !self.surface.is_null() {
            self.event_handler.draw();

            unsafe {
                (self.libegl.eglSwapBuffers.unwrap())(self.egl_display, self.surface);
            }
        }
    }
}

/// Get the JNI Env by calling ndk's AttachCurrentThread
///
/// Safety note: This function is not exactly correct now, it should be fixed!
///
/// AttachCurrentThread should be called at least once for any given thread that
/// wants to use the JNI and DetachCurrentThread should be called only once, when
/// the thread stack is empty and the thread is about to stop
///
/// calling AttachCurrentThread from the same thread multiple time is very cheap
///
/// BUT! there is no DetachCurrentThread call right now, this code:
/// `thread::spawn(|| attach_jni_env());` will lead to internal jni crash :/
/// thread::spawn(|| { attach_jni_env(); loop {} }); is basically what lokinit
/// is doing. this is not correct, but works
/// TODO: the problem here -
/// TODO:   thread::spawn(|| { Attach(); .. Detach() }); will not work as well.
/// TODO: JNI will check that thread's stack is still alive and will crash.
///
/// TODO: Figure how to get into the thread destructor to correctly call Detach
/// TODO: (this should be a GH issue)
/// TODO: for reference - grep for "pthread_setspecific" in SDL2 sources, SDL fixed it!
pub unsafe fn attach_jni_env() -> *mut ndk_sys::JNIEnv {
    let mut env: *mut ndk_sys::JNIEnv = std::ptr::null_mut();
    let attach_current_thread = (**VM).AttachCurrentThread.unwrap();

    let res = attach_current_thread(VM, &mut env, std::ptr::null_mut());
    assert!(res == 0);

    env
}

pub unsafe fn run<F>(conf: crate::conf::Conf, f: F)
where
    F: 'static + FnOnce() -> Box<dyn EventHandler>,
{
    {
        use std::panic;

        panic::set_hook(Box::new(|info| {
            let msg = CString::new(format!("{:?}", info)).unwrap_or_else(|_| {
                CString::new(format!("MALFORMED ERROR MESSAGE {:?}", info.location())).unwrap()
            });
            console_error(msg.as_ptr());
        }));
    }

    if conf.fullscreen {
        let env = attach_jni_env();
        set_full_screen(env, true);
    }

    // yeah, just adding Send to outer F will do it, but it will brake the API
    // in other backends
    struct SendHack<F>(F);
    unsafe impl<F> Send for SendHack<F> {}

    let f = SendHack(f);

    let (tx, rx) = mpsc::channel();

    MESSAGES_TX.with(move |messages_tx| *messages_tx.borrow_mut() = Some(tx));

    thread::spawn(move || {
        let mut libegl = LibEgl::try_load().expect("Cant load LibEGL");

        // skip all the messages until android will be able to actually open a window
        //
        // sometimes before launching an app android will show a permission dialog
        // it is important to create GL context only after a first SurfaceChanged
        let (window, screen_width, screen_height) = 'a: loop {
            if let Ok(Message::SurfaceChanged {
                window,
                width,
                height,
            }) = rx.try_recv()
            {
                break 'a (window, width as f32, height as f32);
            }
        };

        let (egl_context, egl_config, egl_display) = crate::native::egl::create_egl_context(
            &mut libegl,
            std::ptr::null_mut(), /* EGL_DEFAULT_DISPLAY */
            conf.platform.framebuffer_alpha,
        )
        .expect("Cant create EGL context");

        assert!(!egl_display.is_null());
        assert!(!egl_config.is_null());

        crate::native::gl::load_gl_funcs(|proc| {
            let name = std::ffi::CString::new(proc).unwrap();
            libegl.eglGetProcAddress.expect("non-null function pointer")(name.as_ptr() as _)
        });

        let surface = (libegl.eglCreateWindowSurface.unwrap())(
            egl_display,
            egl_config,
            window as _,
            std::ptr::null_mut(),
        );

        if (libegl.eglMakeCurrent.unwrap())(egl_display, surface, surface, egl_context) == 0 {
            panic!();
        }

        let egl_get_procaddr = (libegl.eglGetProcAddress).expect("non-null function pointer");
        let get_procaddr = Box::new(move |procname: &str| {
            let name = std::ffi::CString::new(procname).unwrap();
            egl_get_procaddr(name.as_ptr())
        });

        let display = AndroidDisplay {
            screen_width,
            screen_height,
            fullscreen: conf.fullscreen,
            get_procaddr,
        };

        tl_display::set_display(display);
        let event_handler = f.0();

        let mut s = MainThreadState {
            libegl,
            egl_display,
            egl_config,
            egl_context,
            surface,
            window,
            event_handler,
            quit: false,
        };

        while !s.quit {
            // process all the messages from the main thread
            while let Ok(msg) = rx.try_recv() {
                s.process_message(msg);
            }

            s.frame();

            thread::yield_now();
        }

        (s.libegl.eglMakeCurrent.unwrap())(
            s.egl_display,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        (s.libegl.eglDestroySurface.unwrap())(s.egl_display, s.surface);
        (s.libegl.eglDestroyContext.unwrap())(s.egl_display, s.egl_context);
        (s.libegl.eglTerminate.unwrap())(s.egl_display);
    });
}

#[no_mangle]
extern "C" fn jni_on_load(vm: *mut std::ffi::c_void) {
    unsafe {
        VM = vm as _;
    }
}

unsafe fn create_native_window(surface: ndk_sys::jobject) -> *mut ndk_sys::ANativeWindow {
    let env = attach_jni_env();

    ndk_sys::ANativeWindow_fromSurface(env, surface)
}

#[no_mangle]
pub unsafe extern "C" fn Java_loki_1native_LokiNative_activityOnCreate(
    _: *mut ndk_sys::JNIEnv,
    _: ndk_sys::jobject,
    activity: ndk_sys::jobject,
) {
    let env = attach_jni_env();
    ACTIVITY = (**env).NewGlobalRef.unwrap()(env, activity);
    loki_main();
}

#[no_mangle]
unsafe extern "C" fn Java_loki_1native_LokiNative_activityOnResume(
    _: *mut ndk_sys::JNIEnv,
    _: ndk_sys::jobject,
) {
    send_message(Message::Resume);
}

#[no_mangle]
unsafe extern "C" fn Java_loki_1native_LokiNative_activityOnPause(
    _: *mut ndk_sys::JNIEnv,
    _: ndk_sys::jobject,
) {
    send_message(Message::Pause);
}

#[no_mangle]
unsafe extern "C" fn Java_loki_1native_LokiNative_activityOnDestroy(
    _: *mut ndk_sys::JNIEnv,
    _: ndk_sys::jobject,
) {
    send_message(Message::Destroy);
}

#[no_mangle]
extern "C" fn Java_loki_1native_LokiNative_surfaceOnSurfaceCreated(
    _: *mut ndk_sys::JNIEnv,
    _: ndk_sys::jobject,
    surface: ndk_sys::jobject,
) {
    let window = unsafe { create_native_window(surface) };
    send_message(Message::SurfaceCreated { window });
}

#[no_mangle]
extern "C" fn Java_loki_1native_LokiNative_surfaceOnSurfaceDestroyed(
    _: *mut ndk_sys::JNIEnv,
    _: ndk_sys::jobject,
) {
    send_message(Message::SurfaceDestroyed);
}

#[no_mangle]
extern "C" fn Java_loki_1native_LokiNative_surfaceOnSurfaceChanged(
    _: *mut ndk_sys::JNIEnv,
    _: ndk_sys::jobject,
    surface: ndk_sys::jobject,
    width: ndk_sys::jint,
    height: ndk_sys::jint,
) {
    let window = unsafe { create_native_window(surface) };

    send_message(Message::SurfaceChanged {
        window,
        width: width as _,
        height: height as _,
    });
}

#[no_mangle]
extern "C" fn Java_loki_1native_LokiNative_surfaceOnTouch(
    _: *mut ndk_sys::JNIEnv,
    _: ndk_sys::jobject,
    touch_id: ndk_sys::jint,
    action: ndk_sys::jint,
    x: ndk_sys::jfloat,
    y: ndk_sys::jfloat,
) {
    let phase = match action {
        0 => TouchPhase::Moved,
        1 => TouchPhase::Ended,
        2 => TouchPhase::Started,
        3 => TouchPhase::Cancelled,
        x => panic!("Unsupported touch phase: {}", x),
    };

    send_message(Message::Touch {
        phase,
        touch_id: touch_id as _,
        x,
        y,
    });
}

#[no_mangle]
extern "C" fn Java_loki_1native_LokiNative_surfaceOnKeyDown(
    _: *mut ndk_sys::JNIEnv,
    _: ndk_sys::jobject,
    keycode: ndk_sys::jint,
) {
    let keycode = keycodes::translate_keycode(keycode as _);

    send_message(Message::KeyDown { keycode });
}

#[no_mangle]
extern "C" fn Java_loki_1native_LokiNative_surfaceOnKeyUp(
    _: *mut ndk_sys::JNIEnv,
    _: ndk_sys::jobject,
    keycode: ndk_sys::jint,
) {
    let keycode = keycodes::translate_keycode(keycode as _);

    send_message(Message::KeyUp { keycode });
}

#[no_mangle]
extern "C" fn Java_loki_1native_LokiNative_surfaceOnCharacter(
    _: *mut ndk_sys::JNIEnv,
    _: ndk_sys::jobject,
    character: ndk_sys::jint,
) {
    send_message(Message::Character {
        character: character as u32,
    });
}

unsafe fn set_full_screen(env: *mut ndk_sys::JNIEnv, fullscreen: bool) {
    ndk_utils::call_void_method!(env, ACTIVITY, "setFullScreen", "(Z)V", fullscreen as i32);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct android_asset {
    pub content: *mut ::std::os::raw::c_char,
    pub content_length: ::std::os::raw::c_int,
}

// According to documentation, AAssetManager_fromJava is as available as an
// AAssetManager_open, which was used before
// For some reason it is missing fron ndk_sys binding
extern "C" {
    pub fn AAssetManager_fromJava(
        env: *mut ndk_sys::JNIEnv,
        assetManager: ndk_sys::jobject,
    ) -> *mut ndk_sys::AAssetManager;
}

pub(crate) unsafe fn load_asset(filepath: *const ::std::os::raw::c_char, out: *mut android_asset) {
    let env = attach_jni_env();

    let get_method_id = (**env).GetMethodID.unwrap();
    let get_object_class = (**env).GetObjectClass.unwrap();
    let call_object_method = (**env).CallObjectMethod.unwrap();

    let mid = (get_method_id)(
        env,
        get_object_class(env, ACTIVITY),
        b"getAssets\0".as_ptr() as _,
        b"()Landroid/content/res/AssetManager;\0".as_ptr() as _,
    );
    let asset_manager = (call_object_method)(env, ACTIVITY, mid);
    let mgr = AAssetManager_fromJava(env, asset_manager);
    let asset = ndk_sys::AAssetManager_open(mgr, filepath, ndk_sys::AASSET_MODE_BUFFER as _);
    if asset.is_null() {
        return;
    }
    let length = ndk_sys::AAsset_getLength64(asset);
    // TODO: memory leak right here! this buffer would never freed
    let buffer = libc::malloc(length as _);
    if ndk_sys::AAsset_read(asset, buffer, length as _) > 0 {
        ndk_sys::AAsset_close(asset);

        (*out).content_length = length as _;
        (*out).content = buffer as _;
    }
}
