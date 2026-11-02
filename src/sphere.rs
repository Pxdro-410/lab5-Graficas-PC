use nalgebra_glm::{Vec3, vec3};

/// Genera los vértices, normales e índices de una esfera UV con el radio y resolución dados.
/// Retorna donde cada triángulo tiene 3 índices.
pub fn create_sphere(radius: f32, sectors: u32, stacks: u32) -> (Vec<Vec3>, Vec<Vec3>, Vec<[usize; 3]>) {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    let pi = std::f32::consts::PI;

    // Generar vértices y normales
    for i in 0..=stacks {
        let stack_angle = pi / 2.0 - (i as f32) * pi / (stacks as f32); 
        let xy = radius * stack_angle.cos();
        let z = radius * stack_angle.sin();

        for j in 0..=sectors {
            let sector_angle = 2.0 * pi * (j as f32) / (sectors as f32); 
            let x = xy * sector_angle.cos();
            let y = xy * sector_angle.sin();

            vertices.push(vec3(x, z, y)); // +y como eje vertical
            
            // La normal de una esfera es simplemente la posición normalizada
            let nx = stack_angle.cos() * sector_angle.cos();
            let ny = stack_angle.sin();
            let nz = stack_angle.cos() * sector_angle.sin();
            normals.push(vec3(nx, ny, nz));
        }
    }

    // Generar índices de triángulos
    for i in 0..stacks {
        for j in 0..sectors {
            let first = (i * (sectors + 1) + j) as usize;
            let second = (first + (sectors + 1) as usize) as usize;

            // Primer triángulo del quad
            if i != 0 {
                indices.push([first, second, first + 1]);
            }
            // Segundo triángulo del quad
            if i != (stacks - 1) {
                indices.push([first + 1, second, second + 1]);
            }
        }
    }

    (vertices, normals, indices)
}

/// Genera un disco plano para los anillos de Saturno.
/// El disco tiene un radio interior y exterior, y se genera en el plano XZ
pub fn create_ring(inner_radius: f32, outer_radius: f32, segments: u32, rings: u32) -> (Vec<Vec3>, Vec<Vec3>, Vec<[usize; 3]>) {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    let pi = std::f32::consts::PI;

    // Generar vértices del anillo
    for i in 0..=rings {
        let t = i as f32 / rings as f32;
        let radius = inner_radius + t * (outer_radius - inner_radius);

        for j in 0..=segments {
            let angle = 2.0 * pi * (j as f32) / (segments as f32);
            let x = radius * angle.cos();
            let z = radius * angle.sin();

            vertices.push(vec3(x, 0.0, z));
            normals.push(vec3(0.0, 1.0, 0.0)); // normal apunta hacia arriba
        }
    }

    // Generar índices
    for i in 0..rings {
        for j in 0..segments {
            let first = (i * (segments + 1) + j) as usize;
            let second = first + (segments + 1) as usize;

            indices.push([first, second, first + 1]);
            indices.push([first + 1, second, second + 1]);
        }
    }

    (vertices, normals, indices)
}

