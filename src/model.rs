use std::fs::File;
use std::io::{self, BufRead, BufReader};
use nalgebra_glm::Vec3;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Model {
    pub vertices: Vec<Vec3>,
    pub faces: Vec<[usize; 3]>,
    pub min_bound: Vec3,
    pub max_bound: Vec3,
}

#[allow(dead_code)]
impl Model {
    /// Carga y parsea manualmente un archivo OBJ
    /// Recorre líneas de vértices (v) y caras (f)
    pub fn load(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        let mut min_bound = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max_bound = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            // Ignorar comentarios o líneas vacías
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "v" => {
                    // Formato: v x y z
                    if parts.len() >= 4 {
                        let x: f32 = parts[1].parse().unwrap_or(0.0);
                        let y: f32 = parts[2].parse().unwrap_or(0.0);
                        let z: f32 = parts[3].parse().unwrap_or(0.0);
                        let v = Vec3::new(x, y, z);

                        min_bound.x = min_bound.x.min(x);
                        min_bound.y = min_bound.y.min(y);
                        min_bound.z = min_bound.z.min(z);

                        max_bound.x = max_bound.x.max(x);
                        max_bound.y = max_bound.y.max(y);
                        max_bound.z = max_bound.z.max(z);

                        vertices.push(v);
                    }
                }
                "f" => {
                    // Formato en f v1/vt1/vn1 v2/vt2/vn2 v3/vt3/vn3
                    // En OBJ los índices son 1-based, convertimos a 0-based
                    let mut vertex_indices = Vec::new();
                    for token in &parts[1..] {
                        let mut v_part = token.split('/');
                        if let Some(v_str) = v_part.next() {
                            if let Ok(idx) = v_str.parse::<usize>() {
                                if idx > 0 {
                                    vertex_indices.push(idx - 1);
                                }
                            }
                        }
                    }

                    // Si la cara tiene al menos 3 vértices, guardamos como triángulos
                    if vertex_indices.len() >= 3 {
                        for i in 1..vertex_indices.len() - 1 {
                            faces.push([
                                vertex_indices[0],
                                vertex_indices[i],
                                vertex_indices[i + 1],
                            ]);
                        }
                    }
                }
                _ => {}
            }
        }

        if vertices.is_empty() {
            min_bound = Vec3::zeros();
            max_bound = Vec3::zeros();
        }

        Ok(Model {
            vertices,
            faces,
            min_bound,
            max_bound,
        })
    }

    /// Retorna el centro geométrico del Bounding Box del modelo
    pub fn center(&self) -> Vec3 {
        (self.min_bound + self.max_bound) * 0.5
    }

    /// Retorna las dimensiones en X, Y, Z del Bounding Box
    pub fn dimensions(&self) -> Vec3 {
        self.max_bound - self.min_bound
    }

    /// Retorna la dimensión máxima para calcular el escalado óptimo
    pub fn max_extent(&self) -> f32 {
        let dim = self.dimensions();
        dim.x.max(dim.y).max(dim.z)
    }
}
