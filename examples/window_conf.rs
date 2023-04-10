#![allow(clippy::unusual_byte_groupings)]

use miniquad::skia::SkiaContext;
use miniquad::window::set_fullscreen;
use miniquad::*;

struct Stage {
    fullscreen: bool,
}

impl EventHandler for Stage {
    fn update(&mut self, _: &mut SkiaContext) {}

    fn draw(&mut self, skia_ctx: &mut SkiaContext) {
        skia_ctx.surface.canvas().clear(0xff_ff84c6);
        skia_ctx.dctx.flush(None);
    }

    fn key_up_event(&mut self, _: &mut SkiaContext, keycode: KeyCode, _: KeyMods) {
        if keycode == KeyCode::F {
            self.fullscreen = !self.fullscreen;
            set_fullscreen(self.fullscreen);
            dbg!(self.fullscreen);
        }
    }
}

fn main() {
    miniquad::start(
        conf::Conf {
            window_title: "Lokinit".to_string(),
            window_width: 1024,
            window_height: 768,
            fullscreen: true,
            ..Default::default()
        },
        || Box::new(Stage { fullscreen: false }),
    );
}
