use lokinit::*;

struct Stage {}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        unsafe {
            gl::glClearColor(0.0, 0.25, 0.5, 0.0);
            gl::glClear(gl::GL_COLOR_BUFFER_BIT);
        }

        std::thread::sleep(std::time::Duration::from_millis(1000));
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
    let conf=conf::Conf {
        platform: conf::Platform {
            //linux_x11_gl: lokinit::conf::LinuxX11Gl::GLXOnly,
            //linux_x11_gl: lokinit::conf::LinuxX11Gl::EGLOnly,
            linux_backend: lokinit::conf::LinuxBackend::WaylandOnly,
            ..Default::default()
        },
        ..Default::default()
    };

    lokinit::start(conf, || {
        Box::new(Stage {})
    });
}
