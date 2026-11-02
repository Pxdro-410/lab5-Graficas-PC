use crate::color::Color;
use crate::fragment::Fragment;
use crate::shaders::Uniforms;
use crate::noise::*;

/// Shader de la Luna con 3 capas
/// 1. Color base gris claro con variacion sutil
/// 2. Crateres circulares oscuros con bordes claros
/// 3. Mares lunares oscuros mediante ruido de baja frecuencia
pub fn moon_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let pos = fragment.model_pos;
    let _t = uniforms.time;
    let intensity = fragment.intensity.clamp(0.18, 1.0);

    // Capa 1: Color base gris claro
    // Variacion sutil en el gris base para mayor realismo
    let base_noise = perlin_noise_3d(pos.x * 3.0, pos.y * 3.0, pos.z * 3.0);
    let base_val = 0.72 + base_noise * 0.08;
    let base_color = Color::new(base_val, base_val, base_val * 0.98); // Tinte calido casi imperceptible

    // Capa 2: Crateres circulares oscuros
    // Ruido de alta frecuencia donde los crateres aparecen en caidas bruscas
    let crater_noise_1 = perlin_noise_3d(pos.x * 10.0, pos.y * 10.0, pos.z * 10.0);
    let crater_noise_2 = perlin_noise_3d(pos.x * 20.0, pos.y * 20.0, pos.z * 20.0);
    let crater_noise_3 = perlin_noise_3d(pos.x * 40.0, pos.y * 40.0, pos.z * 40.0);

    // Combinar crateres grandes medianos y pequenos
    let crater_combined = crater_noise_1 * 0.5 + crater_noise_2 * 0.3 + crater_noise_3 * 0.2;

    // Los interiores de los crateres son oscuros y los bordes mas brillantes
    let crater_factor = if crater_combined < -0.2 {
        // Interior del crater oscurecido
        let depth = ((-0.2 - crater_combined) / 0.6).clamp(0.0, 1.0);
        -depth * 0.35
    } else if crater_combined > 0.3 && crater_combined < 0.45 {
        // Borde del crater ligeramente mas claro
        let rim = ((crater_combined - 0.3) / 0.15).clamp(0.0, 1.0);
        rim * 0.1
    } else {
        0.0
    };

    let cratered = Color::new(
        (base_color.r + crater_factor).clamp(0.0, 1.0),
        (base_color.g + crater_factor).clamp(0.0, 1.0),
        (base_color.b + crater_factor * 0.95).clamp(0.0, 1.0),
    );

    // Capa 3: Mares lunares o grandes planicies oscuras
    // Ruido de muy baja frecuencia para planicies de escala continental
    let maria_noise = fbm(
        pos.x * 1.2,
        pos.y * 1.2,
        pos.z * 1.2,
        3,
        2.0,
        0.5,
    );
    // Forma secundaria de baja frecuencia
    let maria_shape = perlin_noise_3d(pos.x * 0.8, pos.y * 0.8, pos.z * 0.8);
    let maria_val = maria_noise * 0.6 + maria_shape * 0.4;

    // Los mares lunares corresponden a las planicies basalticas oscuras
    let maria_factor = if maria_val < 0.0 {
        let depth = (-maria_val / 0.8).clamp(0.0, 1.0);
        depth * 0.3 // Oscurecer hasta un 30 por ciento
    } else {
        0.0
    };

    let maria_color = Color::new(0.38, 0.37, 0.35); // Gris oscuro con ligero matiz cafe
    let color_after_maria = cratered.lerp(&maria_color, maria_factor);

    // Aplicar iluminacion calculada segun la intensidad difusa
    Color::new(
        color_after_maria.r * intensity,
        color_after_maria.g * intensity,
        color_after_maria.b * intensity,
    ).clamp()
}
