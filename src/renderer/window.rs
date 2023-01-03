use crate::resource::Texture;

pub struct WindowAdapter {
    pub(crate) window: winit::window::Window,
    pub(crate) surface: wgpu::Surface,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
    // pub(crate) depth_texture: resource::Texture,
    pub(crate) depth_texture: wgpu::TextureView,
    pub(crate) vsync: bool,
    pub(crate) focused: bool,
}

impl WindowAdapter {
    pub fn resize(&mut self, device: &wgpu::Device, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.config.present_mode = if self.vsync { wgpu::PresentMode::AutoVsync } else { wgpu::PresentMode::AutoNoVsync };
            self.surface.configure(device, &self.config);

            self.depth_texture = Texture::create_depth_texture(device, &self.config, "Depth Texture");
        }
    }
    
    pub fn reconfigure(&mut self, device: &wgpu::Device) {
        self.resize(device, self.size);
    }
}
