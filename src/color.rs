use std::ops::{Add, Mul};

/// Representa un color RGB con componentes en punto flotante
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[allow(dead_code)]
impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b }
    }

    pub fn black() -> Self {
        Color { r: 0.0, g: 0.0, b: 0.0 }
    }

    pub fn white() -> Self {
        Color { r: 1.0, g: 1.0, b: 1.0 }
    }

    /// Convierte el color a formato u32 (0xRRGGBB)
    pub fn to_u32(&self) -> u32 {
        let c = self.clamp();
        let r = (c.r * 255.0) as u32;
        let g = (c.g * 255.0) as u32;
        let b = (c.b * 255.0) as u32;
        (r << 16) | (g << 8) | b
    }

    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
        }
    }

    /// Suma aditiva de colores
    pub fn blend_add(&self, other: &Color) -> Color {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }

    /// Clampea los componentes al rango 0.0 a 1.0
    pub fn clamp(&self) -> Color {
        Color {
            r: self.r.clamp(0.0, 1.0),
            g: self.g.clamp(0.0, 1.0),
            b: self.b.clamp(0.0, 1.0),
        }
    }

    /// Multiplica cada componente por un escalar
    pub fn scale(&self, factor: f32) -> Color {
        Color {
            r: self.r * factor,
            g: self.g * factor,
            b: self.b * factor,
        }
    }
}

impl Add for Color {
    type Output = Color;
    fn add(self, other: Color) -> Color {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Color;
    fn mul(self, scalar: f32) -> Color {
        Color {
            r: self.r * scalar,
            g: self.g * scalar,
            b: self.b * scalar,
        }
    }
}

impl Mul<Color> for Color {
    type Output = Color;
    fn mul(self, other: Color) -> Color {
        Color {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b,
        }
    }
}

