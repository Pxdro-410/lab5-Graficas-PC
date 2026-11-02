use crate::color::Color;
use crate::fragment::Fragment;
use crate::shaders::Uniforms;
use crate::noise::*;

/// Shader de los anillos de Saturno
/// Bandas concentricas de color y opacidad segun la distancia al centro
pub fn rings_shader(fragment: &Fragment, _uniforms: &Uniforms) -> Color {
    let pos = fragment.model_pos;

    // Distancia desde el eje central en el plano XZ
    let dist = (pos.x * pos.x + pos.z * pos.z).sqrt();

    // Limites radiales de los anillos interior y exterior
    let inner_edge = 1.38;
    let outer_edge = 2.82;

    // Normalizar distancia dentro del rango del anillo de 0 a 1
    let ring_t = ((dist - inner_edge) / (outer_edge - inner_edge)).clamp(0.0, 1.0);

    // Estructura de bandas concentricas con brillo variable
    let band1 = ((ring_t * 80.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let band2 = ((ring_t * 120.0 + 0.5).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let band3 = ((ring_t * 200.0 + 1.2).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let band4 = ((ring_t * 40.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);

    // Combinar para obtener el patron de densidad
    let band_density = (band1 * 0.35 + band2 * 0.25 + band3 * 0.15 + band4 * 0.25).clamp(0.0, 1.0);

    // Brechas y divisiones de los anillos
    // Division de Cassini brecha prominente cerca de 0.55
    let cassini_center = 0.55;
    let cassini_width = 0.025;
    let cassini_dist = ((ring_t - cassini_center) / cassini_width).abs();
    let cassini_gap = cassini_dist.clamp(0.0, 1.0);

    // Division
    let encke_center = 0.82;
    let encke_width = 0.008;
    let encke_dist = ((ring_t - encke_center) / encke_width).abs();
    let encke_gap = encke_dist.clamp(0.0, 1.0);

    // Division region interior del anillo C
    let colombo_center = 0.15;
    let colombo_width = 0.012;
    let colombo_dist = ((ring_t - colombo_center) / colombo_width).abs();
    let colombo_gap = colombo_dist.clamp(0.0, 1.0);

    let gap_factor = cassini_gap.min(encke_gap).min(colombo_gap);

    // Color del anillo variando de blanco grisaceo y cafe
    // Los anillos interiores son mas brillantes y los exteriores mas ocres
    let inner_color = Color::new(0.85, 0.82, 0.75);   // Blanco grisaceo brillante de hielo
    let mid_color = Color::new(0.75, 0.65, 0.50);      // Ocre calido
    let outer_color = Color::new(0.55, 0.45, 0.35);    // Cafe polvoriento

    let base_ring_color = if ring_t < 0.4 {
        inner_color.lerp(&mid_color, ring_t / 0.4)
    } else {
        mid_color.lerp(&outer_color, (ring_t - 0.4) / 0.6)
    };

    // Variacion de ruido para aspecto natural de particulas
    // Usar angulo azimutal alrededor del anillo
    let angle = pos.z.atan2(pos.x);
    let noise_val = perlin_noise_2d(
        ring_t * 20.0,
        angle * 3.0,
    ) * 0.08;

    let fine_noise = perlin_noise_2d(
        ring_t * 60.0 + 10.0,
        angle * 8.0 + 5.0,
    ) * 0.04;

    let total_noise = noise_val + fine_noise;

    // Perfil de opacidad y brillo a traves de los anillos
    let opacity_profile = if ring_t < 0.20 {
        // Anillo C interior tenue
        0.3 + ring_t * 2.0
    } else if ring_t < 0.53 {
        // Anillo B intermedio mas brillante
        0.85 + ((ring_t - 0.35) * 10.0).sin() * 0.1
    } else if ring_t < 0.58 {
        // Region de la division de Cassini
        0.15
    } else {
        // Anillo A exterior con brillo moderado atenuandose hacia afuera
        0.65 - (ring_t - 0.58) * 0.8
    };

    // Combinar densidad perfil de opacidad brechas y ruido
    let brightness = (band_density * opacity_profile * gap_factor + total_noise).clamp(0.05, 1.0);

    // Atenuacion suave en los bordes interior y exterior del disco
    let edge_fade = if ring_t < 0.03 {
        ring_t / 0.03
    } else if ring_t > 0.97 {
        (1.0 - ring_t) / 0.03
    } else {
        1.0
    };

    // Sombreado sutil para geometria de disco plano
    // Factor de atenuacion suave en lugar de iluminacion difusa completa
    let subtle_shade = 0.75 + fragment.intensity.max(0.0) * 0.25;

    let final_color = Color::new(
        (base_ring_color.r * brightness * edge_fade * subtle_shade).clamp(0.0, 1.0),
        (base_ring_color.g * brightness * edge_fade * subtle_shade).clamp(0.0, 1.0),
        (base_ring_color.b * brightness * edge_fade * subtle_shade).clamp(0.0, 1.0),
    );

    final_color.clamp()
}
