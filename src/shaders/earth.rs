use crate::color::Color;
use crate::fragment::Fragment;
use crate::shaders::Uniforms;
use crate::noise::*;

/// Shader de la Tierra con 4 capas
/// 1. Oceano con variacion de azul mediante ruido
/// 2. Continentes verdes y marrones mediante umbral de ruido
/// 3. Casquetes polares blancos con transicion suave
/// 4. Nubes y atmosfera en capa semitransparente
pub fn earth_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let pos = fragment.model_pos;

    // Normalizar posicion para obtener direccion en esfera unitaria
    let len = (pos.x * pos.x + pos.y * pos.y + pos.z * pos.z).sqrt().max(0.0001);
    let nx = pos.x / len;
    let ny = pos.y / len;
    let nz = pos.z / len;

    // Capa 1: Oceano base azul profundo con sutil variacion de ruido
    let ocean_noise = perlin_noise_3d(nx * 3.0, ny * 3.0, nz * 3.0) * 0.08;
    let ocean_color = Color::new(
        0.04 + ocean_noise.max(0.0),
        0.10 + ocean_noise * 0.5,
        0.45 + ocean_noise,
    );

    // Oceano ligeramente mas claro para aguas poco profundas
    let shallow_ocean = Color::new(0.06, 0.18, 0.55);
    let ocean_blend = (perlin_noise_3d(nx * 5.0, ny * 5.0, nz * 5.0) * 0.5 + 0.5).clamp(0.0, 1.0);
    let ocean_final = ocean_color.lerp(&shallow_ocean, ocean_blend * 0.3);

    // Capa 2: Continentes y masas terrestres
    // Ruido continental primario a frecuencia media
    let continent_scale = 2.5;
    let continent_noise = fbm(
        nx * continent_scale,
        ny * continent_scale,
        nz * continent_scale,
        6,
        2.0,
        0.5,
    );

    // Umbral para determinar tierra frente a oceano
    let land_threshold = 0.10;
    let is_land = continent_noise > land_threshold;

    // Ruido de biomas para variacion de color en la tierra
    let biome_noise = fbm(
        nx * 5.0 + 100.0,
        ny * 5.0 + 100.0,
        nz * 5.0 + 100.0,
        4,
        2.0,
        0.5,
    );
    let biome_val = (biome_noise * 0.5 + 0.5).clamp(0.0, 1.0);

    // Definir colores de los biomas
    let forest_green = Color::new(0.13, 0.42, 0.10);
    let dark_jungle = Color::new(0.05, 0.28, 0.05);
    let desert_brown = Color::new(0.60, 0.45, 0.25);
    let light_green = Color::new(0.25, 0.50, 0.15);

    // Mezclar entre biomas segun el valor de ruido
    let land_color = if biome_val < 0.3 {
        // Desierto
        desert_brown.lerp(&forest_green, biome_val / 0.3)
    } else if biome_val < 0.6 {
        // Bosque
        let t = (biome_val - 0.3) / 0.3;
        forest_green.lerp(&light_green, t)
    } else {
        // Selva
        let t = (biome_val - 0.6) / 0.4;
        light_green.lerp(&dark_jungle, t)
    };

    // Transicion suave en la linea costera
    let coast_width = 0.06;
    let land_factor = if is_land {
        ((continent_noise - land_threshold) / coast_width).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // Mezclar oceano y tierra
    let surface_color = ocean_final.lerp(&land_color, land_factor);

    // Capa 3: Casquetes polares blancos en los polos
    let abs_y = ny.abs();

    // El hielo comienza a mezclarse cerca de 0.70 y blanco total en 0.92
    let ice_start = 0.70;
    let ice_full = 0.92;

    // Agregar ruido para que los bordes del hielo sean irregulares
    let ice_noise = perlin_noise_3d(nx * 8.0, ny * 2.0, nz * 8.0) * 0.08;
    let ice_threshold = abs_y + ice_noise;

    let ice_factor = if ice_threshold > ice_start {
        ((ice_threshold - ice_start) / (ice_full - ice_start)).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let ice_color = Color::new(0.92, 0.95, 0.98);
    let with_ice = surface_color.lerp(&ice_color, ice_factor);

    // Capa 4: Nubes y atmosfera animadas
    // Usar frecuencia de ruido diferente y animar con el tiempo
    let time = uniforms.time;
    let cloud_speed = 0.008;

    // El ruido de nubes usa distorsion de dominio para formas naturales
    let cloud_warp_x = perlin_noise_3d(
        nx * 2.0 + time * cloud_speed,
        ny * 2.0,
        nz * 2.0,
    ) * 0.3;
    let cloud_warp_z = perlin_noise_3d(
        nx * 2.0 + 50.0,
        ny * 2.0,
        nz * 2.0 + time * cloud_speed * 0.7,
    ) * 0.3;

    let cloud_noise = fbm(
        nx * 3.5 + cloud_warp_x + time * cloud_speed * 1.5,
        ny * 4.0,
        nz * 3.5 + cloud_warp_z + time * cloud_speed,
        5,
        2.2,
        0.45,
    );

    // Mostrar nubes solo sobre el umbral haciendolas semitransparentes
    let cloud_threshold = 0.15;
    let cloud_density = if cloud_noise > cloud_threshold {
        ((cloud_noise - cloud_threshold) / 0.45).clamp(0.0, 0.70)
    } else {
        0.0
    };

    let cloud_color = Color::new(0.95, 0.95, 0.97);
    let with_clouds = with_ice.lerp(&cloud_color, cloud_density);

    // Iluminacion difusa con ambiente suave para ver la noche
    let ambient = 0.18;
    let intensity = fragment.intensity.max(0.0);
    let lit = ambient + (1.0 - ambient) * intensity;

    // Resplandor atmosferico en el borde de la silueta
    let view_dot = fragment.normal.z.max(0.0);
    let atmosphere_glow = (1.0 - view_dot).max(0.0).powf(3.0) * 0.25;
    let atmo_tint = Color::new(0.25, 0.60, 1.0);

    let final_color = Color::new(
        with_clouds.r * lit + atmo_tint.r * atmosphere_glow,
        with_clouds.g * lit + atmo_tint.g * atmosphere_glow,
        with_clouds.b * lit + atmo_tint.b * atmosphere_glow,
    );

    final_color.clamp()
}
