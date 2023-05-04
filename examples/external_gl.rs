use lokinit::{start, conf::Conf, EventHandler, window};

struct Stage {}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        unsafe {
            gl::ClearColor(0.0, 0.25, 0.5, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}

fn main() {
    start(Conf::default(), || {

        // Initialize functions from the gl crate.
        gl::load_with(
            |procname| match window::get_gl_proc_addr(procname) {
                Some(proc) => proc as *const _,
                None => std::ptr::null(),
            },
        );

        Box::new(Stage {})
    });
}
