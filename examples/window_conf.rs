use miniquad::*;
use miniquad::skia::SkiaContext;

struct Stage {
    ctx: GlContext,
}
impl EventHandler for Stage {
    fn update(&mut self, _skia_ctx: &mut SkiaContext) {}

    fn draw(&mut self, _skia_ctx: &mut SkiaContext) {
        self.ctx.clear(Some((0., 1., 0., 1.)), None, None);
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
        || {
            Box::new(Stage {
                ctx: GlContext::new(),
            })
        },
    );
}
