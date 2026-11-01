use crate::framebuffer::Framebuffer;
use nalgebra_glm::Vec3;

/// Traza una línea entre dos puntos usando el algoritmo de Bresenham.
pub fn draw_line(framebuffer: &mut Framebuffer, p0: (i32, i32), p1: (i32, i32)) {
    let mut x0 = p0.0;
    let mut y0 = p0.1;
    let x1 = p1.0;
    let y1 = p1.1;

    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        framebuffer.point(x0, y0);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// Dibuja un triángulo en modo Wireframe, uniendo sus 3 vértices en coordenadas de pantalla.
pub fn draw_triangle_wireframe(
    framebuffer: &mut Framebuffer,
    v0: &Vec3,
    v1: &Vec3,
    v2: &Vec3,
) {
    let p0 = (v0.x.round() as i32, v0.y.round() as i32);
    let p1 = (v1.x.round() as i32, v1.y.round() as i32);
    let p2 = (v2.x.round() as i32, v2.y.round() as i32);

    draw_line(framebuffer, p0, p1);
    draw_line(framebuffer, p1, p2);
    draw_line(framebuffer, p2, p0);
}

/// Función auxiliar para determinar si un punto (px, py) está en el lado correcto de una arista.
fn edge_function(a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> i32 {
    (c.0 - a.0) * (b.1 - a.1) - (c.1 - a.1) * (b.0 - a.0)
}

/// Dibuja un triángulo sólido
pub fn draw_triangle_filled(
    framebuffer: &mut Framebuffer,
    v0: &Vec3,
    v1: &Vec3,
    v2: &Vec3,
) {
    let p0 = (v0.x.round() as i32, v0.y.round() as i32);
    let p1 = (v1.x.round() as i32, v1.y.round() as i32);
    let p2 = (v2.x.round() as i32, v2.y.round() as i32);

    // Calcular el Bounding Box del triángulo en pantalla
    let min_x = p0.0.min(p1.0).min(p2.0).max(0);
    let max_x = p0.0.max(p1.0).max(p2.0).min(framebuffer.width as i32 - 1);
    let min_y = p0.1.min(p1.1).min(p2.1).max(0);
    let max_y = p0.1.max(p1.1).max(p2.1).min(framebuffer.height as i32 - 1);

    // Área total del triángulo
    let area = edge_function(p0, p1, p2);
    if area == 0 {
        return; // Triángulo degenerado colineal
    }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = (x, y);

            let mut w0 = edge_function(p1, p2, p);
            let mut w1 = edge_function(p2, p0, p);
            let mut w2 = edge_function(p0, p1, p);

            // Corregir orientación si el orden de los vértices es antihorario
            if area < 0 {
                w0 = -w0;
                w1 = -w1;
                w2 = -w2;
            }

            // Si el punto está dentro o en las aristas del triángulo
            if w0 >= 0 && w1 >= 0 && w2 >= 0 {
                framebuffer.point(x, y);
            }
        }
    }
}

/// Función general para pintar triángulos, permitiendo alternar entre wireframe o relleno.
#[allow(dead_code)]
pub fn draw_triangle(
    framebuffer: &mut Framebuffer,
    v0: &Vec3,
    v1: &Vec3,
    v2: &Vec3,
    wireframe: bool,
) {
    if wireframe {
        draw_triangle_wireframe(framebuffer, v0, v1, v2);
    } else {
        draw_triangle_filled(framebuffer, v0, v1, v2);
    }
}
