use rtxel_gpu::Ctx;
use wgpu::{CommandEncoder, CurrentSurfaceTexture, SurfaceTexture, wgt::CommandEncoderDescriptor};
use winit::window::Window;

/// An error returned by [`Frame::begin`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailedFrame {
    /// Critical error when trying to begin frame
    Critical,
    /// Surface is outdated and must be reconfigured
    Outdated,
    /// Frame must be skipped
    Skip,
}

/// Frame currently being rendered
#[derive(Debug)]
pub struct Frame {
    pub surface: SurfaceTexture,
    pub encoder: CommandEncoder,
}

impl Frame {
    /// Begin a frame
    pub fn begin(ctx: &Ctx) -> Result<Self, FailedFrame> {
        let encoder = ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Main Command Encoder"),
            });

        let surface = match ctx.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(surface) => surface,
            CurrentSurfaceTexture::Suboptimal(surface) => surface,
            CurrentSurfaceTexture::Timeout => return Err(FailedFrame::Skip),
            CurrentSurfaceTexture::Occluded => return Err(FailedFrame::Skip),
            CurrentSurfaceTexture::Outdated => return Err(FailedFrame::Outdated),
            CurrentSurfaceTexture::Lost => return Err(FailedFrame::Critical),
            CurrentSurfaceTexture::Validation => return Err(FailedFrame::Critical),
        };

        Ok(Self { surface, encoder })
    }

    /// Present a frame
    pub fn present(self, ctx: &Ctx, window: &Window) {
        ctx.queue.submit(Some(self.encoder.finish()));
        window.pre_present_notify();
        self.surface.present();
        window.request_redraw();
    }
}
