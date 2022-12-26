
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Rgba {
        Rgba {
            r,
            g,
            b,
            a,
        }
    }
}

impl From<[f32; 4]> for Rgba {
    fn from(colour: [f32; 4]) -> Rgba {
        Rgba::new(colour[0], colour[1], colour[2], colour[3])
    }
}

impl From<[f32; 3]> for Rgba {
    fn from(colour: [f32; 3]) -> Rgba {
        Rgba::new(colour[0], colour[1], colour[2], 1.0)
    }
}

impl From<Rgba> for [f32; 4] {
    fn from(colour: Rgba) -> [f32; 4] {
        [colour.r, colour.g, colour.b, colour.a]
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Rgb {
    pub fn new(r: f32, g: f32, b: f32) -> Rgb {
        Rgb {
            r,
            g,
            b,
        }
    }
}

impl From<[f32; 4]> for Rgb {
    fn from(colour: [f32; 4]) -> Rgb {
        Rgb::new(colour[0], colour[1], colour[2])
    }
}

impl From<[f32; 3]> for Rgb {
    fn from(colour: [f32; 3]) -> Rgb {
        Rgb::new(colour[0], colour[1], colour[2])
    }
}

impl From<Rgb> for [f32; 3] {
    fn from(colour: Rgb) -> [f32; 3] {
        [colour.r, colour.g, colour.b]
    }
}
