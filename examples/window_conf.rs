#![allow(clippy::unusual_byte_groupings)]

use lokinit::skia::SkiaContext;
use lokinit::window::set_fullscreen;
use lokinit::*;
use lokinit::conf::Platform;

struct Stage {
    fullscreen: bool,
    skia_ctx: SkiaContext
}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        self.skia_ctx.surface.canvas().clear(0xff_ff84c6);
        self.skia_ctx.dctx.flush(None);
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
        || Box::new(Stage { 
            fullscreen: false,
            skia_ctx: SkiaContext::from_gl_loader()
        }),
    );
}
