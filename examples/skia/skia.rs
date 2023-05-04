use std::convert::TryInto;

use skia_safe::{
    gpu::gl::FramebufferInfo,
    gpu::{BackendRenderTarget, DirectContext},
    Surface,
};

use crate::{gl, window};

pub struct SkiaContext {
    pub(crate) fb_info: FramebufferInfo,
    pub surface: Surface,
    pub dctx: DirectContext,
}

impl SkiaContext {
    pub fn new(mut dctx: DirectContext, fb_info: FramebufferInfo, width: i32, height: i32) -> Self {
        let surface = Self::create_surface(&mut dctx, fb_info, width, height);

        Self {
            fb_info,
            surface,
            dctx,
        }
    }

    pub fn from_gl_loader() -> Self {
        let interface =
            skia_safe::gpu::gl::Interface::new_load_with(
                |procname| match window::get_gl_proc_addr(procname) {
                    Some(proc) => proc as *const _,
                    None => std::ptr::null(),
                },
            )
            .expect("Failed to create Skia <-> OpenGL interface");

        let dctx = DirectContext::new_gl(Some(interface), None)
            .expect("Failed to create Skia's direct context");

        let fb_info = {
            let mut fboid: gl::GLint = 0;
            unsafe { gl::glGetIntegerv(gl::GL_FRAMEBUFFER_BINDING, &mut fboid) };

            FramebufferInfo {
                fboid: fboid.try_into().unwrap(),
                format: gl::GL_RGBA8,
            }
        };

        // TODO! This shouldn't be screen size, it should be window size,
        // but there is currently no way to get it.
        let (w, h) = window::screen_size();

        SkiaContext::new(dctx, fb_info, w as i32, h as i32)
    }

    pub fn recreate_surface(&mut self, width: i32, height: i32) {
        self.surface = Self::create_surface(&mut self.dctx, self.fb_info, width, height);
    }

    fn create_surface(
        dctx: &mut DirectContext,
        fb_info: FramebufferInfo,
        width: i32,
        height: i32,
    ) -> Surface {
        let backend_render_target = BackendRenderTarget::new_gl((width, height), None, 0, fb_info);

        Surface::from_backend_render_target(
            dctx,
            &backend_render_target,
            skia_safe::gpu::SurfaceOrigin::BottomLeft,
            skia_safe::ColorType::RGBA8888,
            None,
            None,
        )
        .expect("Failed to create skia surface")
    }
}
