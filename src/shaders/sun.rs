use crate::color::Color;
use crate::fragment::Fragment;
use crate::shaders::Uniforms;
use crate::noise::*;
use nalgebra_glm::*;

/// Shader del Sol estrella emisiva con 4 capas
/// 1. Color base amarillo anaranjado brillante
/// 2. Turbulencia de plasma con efecto de fuego fluido
/// 3. Manchas solares oscuras a frecuencia de ruido diferente
/// 4. Resplandor de borde y efecto corona
pub fn sun_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let pos = fragment.model_pos;
    let normal = fragment.normal;
    let t = uniforms.time;

    // Capa 1: Color base amarillo anaranjado brillante
    let base_color = Color::new(1.0, 0.65, 0.1);

    // Capa 2: Turbulencia de plasma simulando fuego fluido
    // Usar ruido 3D con coordenadas desplazadas para plasma fluido
    let turb_scale = 3.5;
    let turb = turbulence(
        pos.x * turb_scale + t * 0.4,
        pos.y * turb_scale + t * 0.25,
        pos.z * turb_scale - t * 0.3,
        5,
    );
    // Distorsion secundaria para un remolino mas organico
    let warp = perlin_noise_3d(
        pos.x * 2.0 + turb * 0.8 + t * 0.15,
        pos.y * 2.0 + turb * 0.6 - t * 0.2,
        pos.z * 2.0 + turb * 0.7,
    );
    
    let plasma_val = (turb * 0.6 + warp * 0.4).clamp(0.0, 1.0);
    let hot_color = Color::new(1.0, 0.95, 0.6);
    let cool_color = Color::new(0.9, 0.3, 0.05);
    let plasma_color = cool_color.lerp(&hot_color, plasma_val);

    // Mezclar el plasma sobre el color base
    let color_after_plasma = base_color.lerp(&plasma_color, 0.7);

    // Capa 3: Manchas solares oscuras
    // Ruido de baja frecuencia para regiones amplias de manchas
    let spot_noise = perlin_noise_3d(
        pos.x * 1.2 + t * 0.05,
        pos.y * 1.2,
        pos.z * 1.2 - t * 0.03,
    );
    // Ruido de detalle para los bordes de las manchas
    let spot_detail = perlin_noise_3d(
        pos.x * 4.0 + t * 0.1,
        pos.y * 4.0,
        pos.z * 4.0,
    );
    let spot_combined = spot_noise * 0.7 + spot_detail * 0.3;
    // Oscurecer solo donde el ruido cae bajo un umbral para crear manchas aisladas
    let spot_factor = if spot_combined < -0.3 {
        let depth = ((-0.3 - spot_combined) / 0.5).clamp(0.0, 1.0);
        1.0 - depth * 0.6  // Oscurecer hasta un 60 por ciento
    } else {
        1.0
    };
    let spot_color = Color::new(0.25, 0.08, 0.02); // Tono rojizo muy oscuro
    let color_after_spots = color_after_plasma.lerp(&spot_color, 1.0 - spot_factor);

    // Capa 4: Resplandor de borde y corona solar
    // Efecto tipo Fresnel donde los bordes brillan con mayor intensidad
    let view_dir = vec3(0.0_f32, 0.0, 1.0);
    let n = vec3(normal.x, normal.y, normal.z).normalize();
    let ndotv = n.dot(&view_dir).abs();
    // Invertir para que los bordes tengan mayor intensidad que el centro
    let edge = 1.0 - ndotv;
    let corona_strength = (edge * edge * edge).clamp(0.0, 1.0); // Caida cubica suave
    let corona_color = Color::new(1.0, 0.85, 0.3); // Resplandor dorado brillante

    // Agregar corona como brillo aditivo
    let final_color = Color::new(
        color_after_spots.r + corona_color.r * corona_strength * 0.8,
        color_after_spots.g + corona_color.g * corona_strength * 0.6,
        color_after_spots.b + corona_color.b * corona_strength * 0.5,
    );

    // El Sol es un cuerpo emisivo por lo que no usa iluminacion difusa externa
    final_color.clamp()
}
