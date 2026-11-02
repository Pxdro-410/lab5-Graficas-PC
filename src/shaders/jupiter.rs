use crate::color::Color;
use crate::fragment::Fragment;
use crate::shaders::Uniforms;
use crate::noise::*;

/// Shader de Jupiter con 4 capas
/// 1. Color base beige y crema
/// 2. Bandas horizontales onduladas de gas
/// 3. Gran Mancha Roja
/// 4. Turbulencia y distorsion de dominio para nubes en remolino
pub fn jupiter_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let pos = fragment.model_pos;

    // Normalizar posicion para mapeo esferico
    let len = (pos.x * pos.x + pos.y * pos.y + pos.z * pos.z).sqrt().max(0.0001);
    let nx = pos.x / len;
    let ny = pos.y / len;
    let nz = pos.z / len;

    let time = uniforms.time;

    // Capa 4: Turbulencia y distorsion de dominio aplicada a las bandas
    // Distorsionar la coordenada Y antes de calcular las bandas
    let warp_scale = 4.0;
    let warp_x = perlin_noise_3d(
        nx * warp_scale + time * 0.003,
        ny * warp_scale,
        nz * warp_scale,
    ) * 0.12;
    let warp_z = perlin_noise_3d(
        nx * warp_scale + 30.0,
        ny * warp_scale,
        nz * warp_scale + time * 0.002,
    ) * 0.12;

    // Turbulencia fina adicional
    let fine_turb = turbulence(
        nx * 8.0 + time * 0.005,
        ny * 6.0,
        nz * 8.0,
        4,
    ) * 0.04;

    // Latitud distorsionada para el calculo de bandas
    let warped_y = ny + warp_x + fine_turb;

    // Capa 1: Color base beige y cafe claro
    let _base_color = Color::new(0.78, 0.65, 0.45);

    // Capa 2: Bandas horizontales con franjas alternadas cafes y anaranjadas
    let band_freq = 12.0;

    // Crear patrones de bandas superpuestas para mayor complejidad
    let band1 = ((warped_y * band_freq).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let band2 = ((warped_y * band_freq * 2.3 + 1.5).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let band3 = ((warped_y * band_freq * 0.7 + 0.8).sin() * 0.5 + 0.5).clamp(0.0, 1.0);

    // Combinar bandas
    let band_val = (band1 * 0.5 + band2 * 0.3 + band3 * 0.2).clamp(0.0, 1.0);

    // Definir colores de las bandas
    let light_band = Color::new(0.88, 0.78, 0.58);  // Crema palido
    let dark_band = Color::new(0.62, 0.38, 0.18);    // Cafe anaranjado oscuro
    let mid_band = Color::new(0.75, 0.55, 0.30);     // Cafe anaranjado medio

    // Gradiente de bandas con multiples paradas
    let band_color = if band_val < 0.4 {
        dark_band.lerp(&mid_band, band_val / 0.4)
    } else {
        mid_band.lerp(&light_band, (band_val - 0.4) / 0.6)
    };

    // Agregar variacion de ruido dentro de cada banda
    let intra_noise = fbm(
        nx * 10.0 + warp_z * 3.0 + time * 0.004,
        ny * 2.0,
        nz * 10.0 + time * 0.003,
        3,
        2.0,
        0.5,
    ) * 0.12;

    let banded_color = Color::new(
        (band_color.r + intra_noise).clamp(0.0, 1.0),
        (band_color.g + intra_noise * 0.7).clamp(0.0, 1.0),
        (band_color.b + intra_noise * 0.3).clamp(0.0, 1.0),
    );

    // Capa 3: La Gran Mancha Roja
    // Ubicar la mancha en una latitud y longitud especificas
    let spot_lat: f32 = -0.22;   // Hemisferio sur
    let spot_lon_x: f32 = 0.85;  // Direccion longitudinal componente X
    let spot_lon_z: f32 = 0.45;  // Componente Z para la longitud

    // Calcular distancia angular desde el centro de la mancha
    // Utilizando la posicion en la esfera
    let dy = ny - spot_lat;
    let dx = nx - spot_lon_x;
    let dz = nz - spot_lon_z;

    // Forma eliptica mas ancha en longitud que en latitud
    let spot_dist_sq = dx * dx * 1.0 + dy * dy * 4.5 + dz * dz * 1.0;
    let spot_radius_sq = 0.06;

    // Ruido en remolino dentro de la mancha
    let spot_swirl = fbm(
        nx * 15.0 + time * 0.008,
        ny * 15.0,
        nz * 15.0 + time * 0.006,
        4,
        2.0,
        0.5,
    ) * 0.15;

    let spot_factor = if spot_dist_sq < spot_radius_sq {
        let t = 1.0 - (spot_dist_sq / spot_radius_sq).sqrt();
        // Caida suave con atenuacion cubica
        let smooth_t = t * t * (3.0 - 2.0 * t);
        smooth_t.clamp(0.0, 1.0)
    } else {
        0.0
    };

    // Colores de la mancha roja
    let spot_outer = Color::new(0.75, 0.30, 0.12);
    let spot_inner = Color::new(0.85, 0.20, 0.08);
    let spot_color_blend = (spot_swirl * 3.0 + 0.5).clamp(0.0, 1.0);
    let spot_color = spot_outer.lerp(&spot_inner, spot_color_blend);

    // Mezclar la mancha sobre la superficie con bandas
    let final_surface = banded_color.lerp(&spot_color, spot_factor);

    // Iluminacion difusa con luz ambiental suave
    let ambient = 0.18;
    let intensity = fragment.intensity.max(0.0);
    let lit = ambient + (1.0 - ambient) * intensity;

    let final_color = Color::new(
        final_surface.r * lit,
        final_surface.g * lit,
        final_surface.b * lit,
    );

    final_color.clamp()
}
