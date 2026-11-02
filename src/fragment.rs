use nalgebra_glm::Vec3;

/// Fragmento generado por la rasterización de triángulos.
/// Contiene los datos interpolados que el fragment shader usa para calcular el color final.
#[allow(dead_code)]
pub struct Fragment {
    /// Posición en espacio modelo
    pub model_pos: Vec3,
    /// Posición en espacio mundo del fragmento 
    pub position: Vec3,
    /// Normal interpolada en espacio mundo
    pub normal: Vec3,
    /// Profundidad del fragmento
    pub depth: f32,
    /// Intensidad de iluminación calculada
    pub intensity: f32,
    /// Posición en pantalla
    pub screen_x: usize,
    pub screen_y: usize,
}
