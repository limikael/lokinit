use lokinit::*;

struct Stage {}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        unsafe {
            ::gl::ClearColor(0.0, 0.25, 0.5, 0.0);
            ::gl::Clear(::gl::COLOR_BUFFER_BIT);
        }
    }

    fn char_event(&mut self, character: char, _: KeyMods, _: bool) {
        match character {
            'z' => window::show_mouse(false),
            'x' => window::show_mouse(true),
            _ => (),
        }

        let icon = match character {
            '1' => CursorIcon::Default,
            '2' => CursorIcon::Help,
            '3' => CursorIcon::Pointer,
            '4' => CursorIcon::Wait,
            '5' => CursorIcon::Crosshair,
            '6' => CursorIcon::Text,
            '7' => CursorIcon::Move,
            '8' => CursorIcon::NotAllowed,
            '9' => CursorIcon::EWResize,
            '0' => CursorIcon::NSResize,
            'q' => CursorIcon::NESWResize,
            'w' => CursorIcon::NWSEResize,
            _ => return,
        };
        window::set_mouse_cursor(icon);
    }
}

fn main() {
    lokinit::start(conf::Conf::default(), || {
        ::gl::load_with(window::get_gl_proc_addr);
        Box::new(Stage {})
    });
}
