#![allow(clippy::unusual_byte_groupings)]

use miniquad::skia::SkiaContext;
use miniquad::*;
use skia_safe::canvas::{SaveLayerFlags, SaveLayerRec};
use skia_safe::{image_filters, scalar, Canvas, Color, Paint, RRect, Rect};

#[derive(Default, Clone, Copy)]
struct Pointer {
    color: u32,
    x: f32,
    y: f32,
    on: bool,
}

impl Pointer {
    fn colored(color: u32) -> Self {
        Self {
            color,
            ..Default::default()
        }
    }
}

const N_POINTERS: usize = 11;

struct Stage {
    pointers: [Pointer; N_POINTERS],
}

impl EventHandler for Stage {
    fn update(&mut self, _skia_ctx: &mut SkiaContext) {}

    fn mouse_button_down_event(
        &mut self,
        _skia_ctx: &mut SkiaContext,
        _button: MouseButton,
        x: f32,
        y: f32,
    ) {
        self.pointers[10].on = true;
        self.pointers[10].x = x;
        self.pointers[10].y = y;
    }

    fn mouse_button_up_event(
        &mut self,
        _skia_ctx: &mut SkiaContext,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.pointers[10].on = false;
    }

    fn mouse_motion_event(&mut self, _skia_ctx: &mut SkiaContext, x: f32, y: f32) {
        self.pointers[10].x = x;
        self.pointers[10].y = y;
    }

    fn touch_event(
        &mut self,
        _skia_ctx: &mut SkiaContext,
        phase: TouchPhase,
        id: u64,
        x: f32,
        y: f32,
    ) {
        let id = (id as usize).clamp(0, N_POINTERS - 2);

        match phase {
            TouchPhase::Started => self.pointers[id].on = true,
            TouchPhase::Ended | TouchPhase::Cancelled => self.pointers[id].on = false,
            _ => (),
        }

        self.pointers[id].x = x;
        self.pointers[id].y = y;
    }

    fn resize_event(&mut self, skia_ctx: &mut SkiaContext, width: f32, height: f32) {
        skia_ctx.recreate_surface(width as i32, height as i32);
    }

    fn draw(&mut self, skia_ctx: &mut SkiaContext) {
        let canvas = &mut skia_ctx.surface.canvas();
        canvas.clear(Color::from(0xff_161a1d));

        // simple rectangle
        simple_rectangle(canvas, 0., 0., 200., 200., 0., 10.);

        // blur rectangle
        for pointer in &mut self.pointers {
            if pointer.on {
                blur_rectangle(canvas, pointer);
            }
        }

        skia_ctx.dctx.flush(None);
    }
}

fn blur_rectangle(canvas: &mut Canvas, pointer: &Pointer) {
    let rect = rect_centered(pointer.x, pointer.y, 150., 400.);
    let rrect = RRect::new_rect_xy(rect, 20., 20.);

    canvas.save();
    {
        canvas.clip_rrect(rrect, skia_safe::ClipOp::Intersect, true);
        let image_filter = image_filters::blur((10., 10.), None, None, None);

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_dither(true);
        paint.set_image_filter(image_filter);

        let ext = 20.;
        let layer_rect = Rect::from_xywh(
            rect.x() - ext,
            rect.y() - ext,
            rect.width() + ext * 2.,
            rect.height() + ext * 2.,
        );
        let layer_rec = SaveLayerRec::default()
            .bounds(&layer_rect)
            .paint(&paint)
            .flags(SaveLayerFlags::INIT_WITH_PREVIOUS);

        canvas.save_layer(&layer_rec);
        {
            canvas.draw_color((0x00_ffffff & pointer.color) | 0x80_000000, None);
        }
        canvas.restore();
    }
    canvas.restore();
}

fn rect_centered(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::from_xywh(x - width / 2., y - height / 2., width, height)
}

fn rect_from_center(canvas: &mut Canvas, x: f32, y: f32, width: f32, height: f32) -> Rect {
    let dim = canvas.image_info().dimensions();
    let (cx, cy) = (dim.width as f32 / 2., dim.height as f32 / 2.);
    rect_centered(cx + x, cy + y, width, height)
}

pub fn simple_rectangle(
    canvas: &mut Canvas,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    roundness: scalar,
    stroke_width: scalar,
) -> RRect {
    let rect = rect_from_center(canvas, x, y, width, height);
    let rrect = RRect::new_rect_xy(rect, roundness, roundness);

    let mut paint = Paint::default();
    paint.set_anti_alias(true);

    paint.set_stroke(false);
    paint.set_color(Color::from(0x80_bbddff));
    canvas.draw_rrect(rrect, &paint);

    if stroke_width > 0. {
        paint.set_stroke(true);
        paint.set_stroke_width(stroke_width);
        paint.set_color(Color::from(0xff_bbddff));
        canvas.draw_rrect(rrect, &paint);
    }

    rrect
}

fn main() {
    miniquad::start(
        conf::Conf {
            high_dpi: true,
            ..Default::default()
        },
        || {
            Box::new(Stage {
                pointers: [
                    // pointers for fingers
                    Pointer::colored(0xff3737),
                    Pointer::colored(0xffaf37),
                    Pointer::colored(0xd7ff37),
                    Pointer::colored(0x5fff37),
                    Pointer::colored(0x37ff87),
                    Pointer::colored(0x37ffff),
                    Pointer::colored(0x3787ff),
                    Pointer::colored(0x5f37ff),
                    Pointer::colored(0xd737ff),
                    Pointer::colored(0xff37af),
                    // pointer for mouse
                    Pointer::colored(0xbbddff),
                ],
            })
        },
    );
}
