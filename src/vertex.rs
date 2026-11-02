use nalgebra_glm::Vec3;

/// Vértice con posición y normal, usado como entrada al pipeline de renderizado
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Vertex {
    /// Posición del vértice en espacio modelo
    pub position: Vec3,
    /// Normal del vértice para la iluminacion
    pub normal: Vec3,
}

/// Vértice ya transformado al espacio de pantalla
#[derive(Debug, Clone)]
pub struct TransformedVertex {
    /// Posición en espacio de pantalla 
    pub screen_pos: Vec3,
    /// Posición original en espacio modelo
    pub model_pos: Vec3,
    /// Posición en espacio mundo 
    pub world_pos: Vec3,
    /// Normal transformada en espacio mundo
    pub normal: Vec3,
}
