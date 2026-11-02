use crate::color::Color;
use crate::fragment::Fragment;
use crate::shaders::Uniforms;
use crate::noise::*;

/// Shader de Marte con 4 capas
/// 1. Color base rojo oxido
/// 2. Variacion de terreno y parches rocosos
/// 3. Casquetes polares blancos
/// 4. Lineas de canones y valles profundos
pub fn mars_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let pos = fragment.model_pos;
    let _t = uniforms.time;
    let intensity = fragment.intensity.clamp(0.18, 1.0);

    // Capa 1: Color base rojo oxido mineral
    let base_color = Color::new(0.75, 0.25, 0.12);

    // Capa 2: Variacion de terreno con parches rocosos
    // Ruido multioctava para terreno natural
    let terrain_noise = fbm(
        pos.x * 5.0,
        pos.y * 5.0,
        pos.z * 5.0,
        5,
        2.2,
        0.45,
    );
    // Detalle fino de superficie
    let detail = perlin_noise_3d(pos.x * 12.0, pos.y * 12.0, pos.z * 12.0);

    let terrain_val = (terrain_noise * 0.7 + detail * 0.3) * 0.5 + 0.5;

    let terrain_dark = Color::new(0.5, 0.15, 0.07);   // Oxido oscuro
    let terrain_light = Color::new(0.88, 0.42, 0.2);   // Rojo arenoso claro
    let terrain_color = terrain_dark.lerp(&terrain_light, terrain_val.clamp(0.0, 1.0));

    let color_after_terrain = base_color.lerp(&terrain_color, 0.65);

    // Capa 3: Casquetes polares blancos
    // Usar la coordenada Y del modelo para que los polos giren con el planeta
    let len = (pos.x * pos.x + pos.y * pos.y + pos.z * pos.z).sqrt().max(0.0001);
    let ny = pos.y / len;
    let abs_lat = ny.abs();

    // Los casquetes de hielo aparecen cerca de los polos
    let ice_threshold = 0.82;
    let ice_blend = if abs_lat > ice_threshold {
        let t = ((abs_lat - ice_threshold) / (1.0 - ice_threshold)).clamp(0.0, 1.0);
        // Agregar ruido al borde del hielo para apariencia natural
        let ice_noise = perlin_noise_3d(pos.x * 8.0, pos.y * 8.0, pos.z * 8.0) * 0.15;
        (t + ice_noise).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let ice_color = Color::new(0.95, 0.97, 1.0); // Blanco con ligero tinte azulado
    let color_after_ice = color_after_terrain.lerp(&ice_color, ice_blend);

    // Capa 4: Lineas de canones y valles estriados oscuros
    // Ruido estirado para crear accidentes geograficos direccionales
    // Estirar a lo largo de X para valles de este a oeste como Valles Marineris
    let canyon_noise = perlin_noise_3d(
        pos.x * 2.0,
        pos.y * 8.0,  // Frecuencia alta en Y para generar estrias horizontales
        pos.z * 3.0,
    );
    let canyon_detail = perlin_noise_3d(
        pos.x * 4.0,
        pos.y * 16.0,
        pos.z * 6.0,
    );
    let canyon_val = canyon_noise * 0.6 + canyon_detail * 0.4;

    // Crear canones solo en una franja estrecha de ruido
    let canyon_factor = if canyon_val < -0.25 && canyon_val > -0.55 {
        let depth = ((-0.25 - canyon_val) / 0.3).clamp(0.0, 1.0);
        depth * 0.5 // Oscurecer hasta un 50 por ciento
    } else {
        0.0
    };

    let canyon_color = Color::new(0.3, 0.1, 0.05); // Cafe rojizo oscuro profundo
    let color_after_canyons = color_after_ice.lerp(&canyon_color, canyon_factor);

    // Aplicar iluminacion calculada segun la intensidad difusa
    Color::new(
        color_after_canyons.r * intensity,
        color_after_canyons.g * intensity,
        color_after_canyons.b * intensity,
    ).clamp()
}
