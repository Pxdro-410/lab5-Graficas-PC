pub mod sun;
pub mod venus;
pub mod earth;
pub mod mars;
pub mod jupiter;
pub mod saturn;
pub mod rings;
pub mod moon;

use crate::color::Color;
use crate::fragment::Fragment;

/// Datos uniformes pasados a todos los shaders constantes por cada cuadro
pub struct Uniforms {
    /// Tiempo de animación en segundos
    pub time: f32,
}

/// Tipos de shader disponibles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShaderType {
    Sun,
    Venus,
    Earth,
    Mars,
    Jupiter,
    Saturn,
    Rings,
    Moon,
}

/// Despacho que ejecuta el fragment shader correspondiente segun el tipo
pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, shader_type: ShaderType) -> Color {
    match shader_type {
        ShaderType::Sun => sun::sun_shader(fragment, uniforms),
        ShaderType::Venus => venus::venus_shader(fragment, uniforms),
        ShaderType::Earth => earth::earth_shader(fragment, uniforms),
        ShaderType::Mars => mars::mars_shader(fragment, uniforms),
        ShaderType::Jupiter => jupiter::jupiter_shader(fragment, uniforms),
        ShaderType::Saturn => saturn::saturn_shader(fragment, uniforms),
        ShaderType::Rings => rings::rings_shader(fragment, uniforms),
        ShaderType::Moon => moon::moon_shader(fragment, uniforms),
    }
}

/// Devuelve el nombre legible del shader
#[allow(dead_code)]
pub fn shader_name(shader: ShaderType) -> &'static str {
    match shader {
        ShaderType::Sun => "Sol",
        ShaderType::Venus => "Venus",
        ShaderType::Earth => "Tierra",
        ShaderType::Mars => "Marte",
        ShaderType::Jupiter => "Jupiter",
        ShaderType::Saturn => "Saturno",
        ShaderType::Rings => "Anillos",
        ShaderType::Moon => "Luna",
    }
}

