use skia_safe::{
    gpu::gl::FramebufferInfo,
    gpu::{BackendRenderTarget, DirectContext},
    Surface,
};

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
