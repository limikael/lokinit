#![allow(clippy::unusual_byte_groupings)]

use lokinit::conf::Platform;
use lokinit::window::set_fullscreen;
use lokinit::*;

struct Stage {
    fullscreen: bool,
}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        unsafe {
            ::gl::ClearColor(0.0, 0.25, 0.5, 0.0);
            ::gl::Clear(::gl::COLOR_BUFFER_BIT);
        }
    }

    fn key_up_event(&mut self, keycode: KeyCode, _: KeyMods) {
        if keycode == KeyCode::F {
            self.fullscreen = !self.fullscreen;
            set_fullscreen(self.fullscreen);
            dbg!(self.fullscreen);
        }
    }
}

fn main() {
    lokinit::start(
        conf::Conf {
            window_title: "Lokinit".to_string(),
            window_width: 800,
            window_height: 600,
            //fullscreen: true,
            platform: Platform {
                linux_x11_gl: lokinit::conf::LinuxX11Gl::GLXOnly,
                //linux_x11_gl: lokinit::conf::LinuxX11Gl::EGLOnly,
                ..Default::default()
            },
            ..Default::default()
        },
        || {
            ::gl::load_with(window::get_gl_proc_addr);
            Box::new(Stage { fullscreen: false })
        },
    );
}
