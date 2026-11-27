use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;

use engine::WgpuStage;
use crate::gpu_profile::{GpuChoice, ServerGpuChoice};
use crate::surface::{PlatformSurface, WgpuPlatformSurface};

/// Interface for platform window and `wgpu` environment setup.
pub trait HostEnvironment {
    fn attach(&self, event_loop: &ActiveEventLoop) -> Box<dyn PlatformSurface>;
}

/// Encapsulated `winit` window and `wgpu` Vulkan/DX12 display surface setup (`3` private fields).
pub struct WgpuHostEnvironment {
    title: &'static str,
    dimensions: (f64, f64),
    gpu_choice: Box<dyn GpuChoice>,
}

impl WgpuHostEnvironment {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_gpu_choice(
        title: &'static str,
        dimensions: (f64, f64),
        gpu_choice: Box<dyn GpuChoice>,
    ) -> Self {
        Self { title, dimensions, gpu_choice }
    }

    pub fn new(title: &'static str, width: f64, height: f64) -> Self {
        Self::with_gpu_choice(title, (width, height), Box::new(ServerGpuChoice::assembled()))
    }
}

impl HostEnvironment for WgpuHostEnvironment {
    fn attach(&self, event_loop: &ActiveEventLoop) -> Box<dyn PlatformSurface> {
        let profile = self.gpu_choice.select_profile();
        let window_attributes = WindowAttributes::default()
            .with_title(self.title)
            .with_transparent(false)
            .with_inner_size(winit::dpi::LogicalSize::new(self.dimensions.0, self.dimensions.1));

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create winit window"),
        );
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: profile.native_backends(),
            ..Default::default()
        });
        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("Failed to create wgpu Vulkan/DX12 window surface");
        let (stage, config) = WgpuStage::initialize_for_surface(
            &instance,
            &surface,
            profile,
            (size.width.max(1), size.height.max(1)),
        );
        Box::new(WgpuPlatformSurface::new(window, surface, stage, config))
    }
}
