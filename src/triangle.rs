use crate::framebuffer::Framebuffer;
use crate::fragment::Fragment;
use crate::vertex::TransformedVertex;
use crate::shaders::{Uniforms, ShaderType, fragment_shader};
use nalgebra_glm::Vec3;

/// Función de arista para determinar si un punto está dentro del triángulo
fn edge_function(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> f32 {
    (c.0 - a.0) * (b.1 - a.1) - (c.1 - a.1) * (b.0 - a.0)
}

/// Para cada píxel dentro del triángulo
/// - Calcula coordenadas baricéntricas
/// - Interpola posición modelo, posición mundo, normal y profundidad
/// - Llama al fragment shader
/// - Escribe el resultado con test de z-buffer
pub fn rasterize_triangle(
    framebuffer: &mut Framebuffer,
    v0: &TransformedVertex,
    v1: &TransformedVertex,
    v2: &TransformedVertex,
    uniforms: &Uniforms,
    shader: ShaderType,
) {
    let p0 = (v0.screen_pos.x, v0.screen_pos.y);
    let p1 = (v1.screen_pos.x, v1.screen_pos.y);
    let p2 = (v2.screen_pos.x, v2.screen_pos.y);

    // Calcular el área del triángulo (2x)
    let area = edge_function(p0, p1, p2);
    if area.abs() < 1e-8 {
        return; // Triángulo degenerado
    }

    // Bounding box del triángulo
    let min_x = p0.0.min(p1.0).min(p2.0).max(0.0) as usize;
    let max_x = p0.0.max(p1.0).max(p2.0).min(framebuffer.width as f32 - 1.0) as usize;
    let min_y = p0.1.min(p1.1).min(p2.1).max(0.0) as usize;
    let max_y = p0.1.max(p1.1).max(p2.1).min(framebuffer.height as f32 - 1.0) as usize;

    // Dirección de luz natural y cinematográfica 
    let light_dir = nalgebra_glm::normalize(&nalgebra_glm::vec3(0.75, 0.35, 0.55));

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let p = (px, py);

            // Coordenadas baricéntricas
            let w0 = edge_function(p1, p2, p);
            let w1 = edge_function(p2, p0, p);
            let w2 = edge_function(p0, p1, p);

            // Verificar si el punto está dentro del triángulo
            let inside = if area > 0.0 {
                w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0
            } else {
                w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0
            };

            if !inside {
                continue;
            }

            // Normalizar coordenadas baricéntricas
            let inv_area = 1.0 / area;
            let lambda0 = w0 * inv_area;
            let lambda1 = w1 * inv_area;
            let lambda2 = w2 * inv_area;

            // Interpolar profundidad
            let depth = lambda0 * v0.screen_pos.z + lambda1 * v1.screen_pos.z + lambda2 * v2.screen_pos.z;

            // Interpolar posición en espacio modelo 
            let model_pos = Vec3::new(
                lambda0 * v0.model_pos.x + lambda1 * v1.model_pos.x + lambda2 * v2.model_pos.x,
                lambda0 * v0.model_pos.y + lambda1 * v1.model_pos.y + lambda2 * v2.model_pos.y,
                lambda0 * v0.model_pos.z + lambda1 * v1.model_pos.z + lambda2 * v2.model_pos.z,
            );

            // Interpolar posición en espacio mundo
            let world_pos = Vec3::new(
                lambda0 * v0.world_pos.x + lambda1 * v1.world_pos.x + lambda2 * v2.world_pos.x,
                lambda0 * v0.world_pos.y + lambda1 * v1.world_pos.y + lambda2 * v2.world_pos.y,
                lambda0 * v0.world_pos.z + lambda1 * v1.world_pos.z + lambda2 * v2.world_pos.z,
            );

            // Interpolar normal en espacio mundo
            let normal = Vec3::new(
                lambda0 * v0.normal.x + lambda1 * v1.normal.x + lambda2 * v2.normal.x,
                lambda0 * v0.normal.y + lambda1 * v1.normal.y + lambda2 * v2.normal.y,
                lambda0 * v0.normal.z + lambda1 * v1.normal.z + lambda2 * v2.normal.z,
            );
            let normal = nalgebra_glm::normalize(&normal);

            // Intensidad de iluminación difusa directa
            let intensity = normal.dot(&light_dir).max(0.0);

            // Crear fragmento
            let fragment = Fragment {
                model_pos,
                position: world_pos,
                normal,
                depth,
                intensity,
                screen_x: x,
                screen_y: y,
            };

            // Ejecutar fragment shader
            let color = fragment_shader(&fragment, uniforms, shader);

            // Escribir al framebuffer con z-buffer test
            framebuffer.point_with_depth(x, y, depth, color.to_u32());
        }
    }
}
