use crate::color::Color;
use crate::fragment::Fragment;
use crate::shaders::Uniforms;
use crate::noise::*;
use nalgebra_glm::*;

/// Shader de Venus con 3 capas
/// 1. Color base amarillo ambar palido
/// 2. Patrones de nubes densas arremolinadas con distorsion de dominio
/// 3. Gradiente atmosferico entre el ecuador y los polos
pub fn venus_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let pos = fragment.model_pos;
    let normal = fragment.normal;
    let t = uniforms.time;
    let intensity = fragment.intensity.clamp(0.18, 1.0); // Ambiente suave para iluminar la silueta completa

    // Capa 1: Color base amarillo ambar palido
    let base_color = Color::new(0.92, 0.78, 0.45);

    // Capa 2: Patrones densos de nubes arremolinadas
    // Primer paso: distorsionar las coordenadas espaciales con ruido
    let warp_scale = 2.0_f32;
    let warp_x = perlin_noise_3d(
        pos.x * warp_scale + t * 0.08,
        pos.y * warp_scale,
        pos.z * warp_scale,
    );
    let warp_y = perlin_noise_3d(
        pos.x * warp_scale + 5.2,
        pos.y * warp_scale + t * 0.06,
        pos.z * warp_scale + 1.3,
    );
    let warp_z = perlin_noise_3d(
        pos.x * warp_scale + 9.7,
        pos.y * warp_scale + 3.1,
        pos.z * warp_scale + t * 0.07,
    );

    // Segundo paso: muestrear ruido en coordenadas distorsionadas
    let cloud_scale = 3.0_f32;
    let cloud_noise = fbm(
        pos.x * cloud_scale + warp_x * 1.5,
        pos.y * cloud_scale + warp_y * 1.5,
        pos.z * cloud_scale + warp_z * 1.5,
        4,
        2.0,
        0.5,
    );

    // Tercer paso: capa adicional de distorsion para mayor complejidad
    let detail_noise = perlin_noise_3d(
        pos.x * 6.0 + warp_x * 0.8 + t * 0.12,
        pos.y * 6.0 + warp_y * 0.8,
        pos.z * 6.0 + warp_z * 0.8,
    );

    let cloud_val = (cloud_noise * 0.7 + detail_noise * 0.3) * 0.5 + 0.5;

    // Colores de las bandas nubosas
    let cloud_light = Color::new(0.98, 0.92, 0.7);  // Amarillo palido brillante
    let cloud_dark = Color::new(0.75, 0.55, 0.25);   // Ambar mas oscuro
    let cloud_color = cloud_dark.lerp(&cloud_light, cloud_val.clamp(0.0, 1.0));

    let color_after_clouds = base_color.lerp(&cloud_color, 0.75);

    // Capa 3: Gradiente atmosferico entre ecuador y polos
    // Usar la componente Y de la normal para determinar la latitud
    let n = vec3(normal.x, normal.y, normal.z).normalize();
    let latitude = n.y.abs(); // 0 en el ecuador y 1 en los polos

    let equator_tint = Color::new(0.95, 0.65, 0.3);  // Tono mas anaranjado en el ecuador
    let pole_tint = Color::new(0.95, 0.88, 0.55);     // Tono mas amarillo en los polos

    let atmo_color = equator_tint.lerp(&pole_tint, latitude);
    let final_base = color_after_clouds.lerp(&atmo_color, 0.3);

    // Aplicar iluminacion calculada segun la intensidad difusa
    Color::new(
        final_base.r * intensity,
        final_base.g * intensity,
        final_base.b * intensity,
    ).clamp()
}
