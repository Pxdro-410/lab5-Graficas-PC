use crate::color::Color;
use crate::fragment::Fragment;
use crate::shaders::Uniforms;
use crate::noise::*;

/// Shader del cuerpo de Saturno con 3 capas
/// 1. Color base dorado y beige mas calido que Jupiter
/// 2. Bandas horizontales sutiles con variacion suave de color
/// 3. Variacion de ruido de baja frecuencia dentro de las bandas
pub fn saturn_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let pos = fragment.model_pos;

    // Normalizar posicion para mapeo esferico
    let len = (pos.x * pos.x + pos.y * pos.y + pos.z * pos.z).sqrt().max(0.0001);
    let nx = pos.x / len;
    let ny = pos.y / len;
    let nz = pos.z / len;

    let time = uniforms.time;

    // Capa 1: Color base dorado y beige calido
    let base_color = Color::new(0.85, 0.72, 0.45);

    // Capa 2: Bandas horizontales sutiles
    // Ondulacion suave en las bandas latitudinales
    let wave_noise = perlin_noise_3d(
        nx * 3.0 + time * 0.001,
        ny * 1.5,
        nz * 3.0,
    ) * 0.04;

    let warped_y = ny + wave_noise;

    // Bandas mas suaves y de menor contraste que en Jupiter
    let band_freq = 8.0;
    let band1 = ((warped_y * band_freq).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let band2 = ((warped_y * band_freq * 1.7 + 2.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);

    let band_val = (band1 * 0.65 + band2 * 0.35).clamp(0.0, 1.0);

    // Paleta de Saturno con dorados calidos y amarillos tenues
    let light_band = Color::new(0.92, 0.82, 0.55);   // Dorado palido
    let dark_band = Color::new(0.72, 0.58, 0.32);     // Ambar profundo
    let warm_band = Color::new(0.82, 0.68, 0.40);     // Tono medio calido

    // Gradiente suave a traves de los colores de las bandas
    let band_color = if band_val < 0.35 {
        dark_band.lerp(&warm_band, band_val / 0.35)
    } else if band_val < 0.65 {
        let t = (band_val - 0.35) / 0.30;
        warm_band.lerp(&base_color, t)
    } else {
        let t = (band_val - 0.65) / 0.35;
        base_color.lerp(&light_band, t)
    };

    // Capa 3: Variacion de ruido de baja frecuencia dentro de las bandas
    let color_var = fbm(
        nx * 4.0 + time * 0.002,
        ny * 2.0,
        nz * 4.0 + time * 0.001,
        3,
        2.0,
        0.5,
    );

    // Variacion sutil para mantener la apariencia tersa de Saturno
    let var_amount = color_var * 0.06;

    let varied_color = Color::new(
        (band_color.r + var_amount).clamp(0.0, 1.0),
        (band_color.g + var_amount * 0.8).clamp(0.0, 1.0),
        (band_color.b + var_amount * 0.4).clamp(0.0, 1.0),
    );

    // Ligero oscurecimiento polar caracteristico de Saturno
    let polar_dark = 1.0 - ny.abs() * 0.15;

    let surface = Color::new(
        varied_color.r * polar_dark,
        varied_color.g * polar_dark,
        varied_color.b * polar_dark,
    );

    // Iluminacion difusa con luz ambiental suave
    let ambient = 0.18;
    let intensity = fragment.intensity.max(0.0);
    let lit = ambient + (1.0 - ambient) * intensity;

    let final_color = Color::new(
        surface.r * lit,
        surface.g * lit,
        surface.b * lit,
    );

    final_color.clamp()
}
