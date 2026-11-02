mod framebuffer;
mod color;
mod fragment;
mod vertex;
mod triangle;
mod noise;
mod sphere;
mod shaders;
mod font;

use framebuffer::Framebuffer;
use vertex::TransformedVertex;
use shaders::{Uniforms, ShaderType};
use triangle::rasterize_triangle;
use sphere::{create_sphere, create_ring};
use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{vec3, vec4, Mat4, Vec3};

const WINDOW_WIDTH: usize = 900;
const WINDOW_HEIGHT: usize = 700;

/// Cuerpo celeste con su geometría y propiedades
struct CelestialBody {
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    indices: Vec<[usize; 3]>,
    shader: ShaderType,
    position: Vec3,
    scale: f32,
}

/// Luna orbitando un planeta
struct Moon {
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    indices: Vec<[usize; 3]>,
    shader: ShaderType,
    orbit_radius: f32,
    scale: f32,
}

/// Datos de la escena actual que se renderiza
struct Scene {
    name: &'static str,
    bodies: Vec<CelestialBody>,
    moons: Vec<Moon>,
    has_rings: bool,
    ring_vertices: Vec<Vec3>,
    ring_normals: Vec<Vec3>,
    ring_indices: Vec<[usize; 3]>,
    ring_parent_idx: usize,
}

/// Transforma vértices al espacio de pantalla
fn vertex_shader(
    position: &Vec3,
    normal: &Vec3,
    model_matrix: &Mat4,
    view_scale: f32,
    screen_w: f32,
    screen_h: f32,
) -> TransformedVertex {
    let v4 = vec4(position.x, position.y, position.z, 1.0);
    let transformed = model_matrix * v4;
    let world_pos = vec3(transformed.x, transformed.y, transformed.z);

    // Transformar normal en espacio mundo usando la parte rotacional
    let n4 = vec4(normal.x, normal.y, normal.z, 0.0);
    let transformed_normal = model_matrix * n4;
    let normal_out = nalgebra_glm::normalize(&vec3(
        transformed_normal.x,
        transformed_normal.y,
        transformed_normal.z,
    ));

    // Proyección ortográfica centrada en pantalla
    let screen_x = world_pos.x * view_scale + (screen_w / 2.0);
    let screen_y = -world_pos.y * view_scale + (screen_h / 2.0);
    // Para que los objetos más cercanos a la cámara +Z tengan menor depth en el zbuffer
    let screen_z = -world_pos.z;

    TransformedVertex {
        screen_pos: vec3(screen_x, screen_y, screen_z),
        model_pos: *position,
        world_pos,
        normal: normal_out,
    }
}

/// Renderiza un conjunto de triángulos con el shader dado
fn render_mesh(
    framebuffer: &mut Framebuffer,
    vertices: &[Vec3],
    normals: &[Vec3],
    indices: &[[usize; 3]],
    model_matrix: &Mat4,
    view_scale: f32,
    uniforms: &Uniforms,
    shader: ShaderType,
) {
    let screen_w = framebuffer.width as f32;
    let screen_h = framebuffer.height as f32;

    // Transformar todos los vértices
    let transformed: Vec<TransformedVertex> = vertices
        .iter()
        .zip(normals.iter())
        .map(|(pos, norm)| vertex_shader(pos, norm, model_matrix, view_scale, screen_w, screen_h))
        .collect();

    // Rasterizar cada triángulo
    for face in indices {
        let v0 = &transformed[face[0]];
        let v1 = &transformed[face[1]];
        let v2 = &transformed[face[2]];

        // Backface culling en espacio de pantalla 
        let edge1 = v1.screen_pos - v0.screen_pos;
        let edge2 = v2.screen_pos - v0.screen_pos;
        let cross_z = edge1.x * edge2.y - edge1.y * edge2.x;

        if shader != ShaderType::Rings && cross_z < 0.0 {
            continue;
        }

        rasterize_triangle(framebuffer, v0, v1, v2, uniforms, shader);
    }
}

/// Crea la matriz de modelo a partir de posición, rotación 3D y escala
fn create_model_matrix(position: &Vec3, rotation_x: f32, rotation_y: f32, scale: f32) -> Mat4 {
    let mut mat = Mat4::identity();
    mat = nalgebra_glm::translate(&mat, position);
    mat = nalgebra_glm::rotate_x(&mat, rotation_x);
    mat = nalgebra_glm::rotate_y(&mat, rotation_y);
    mat = nalgebra_glm::scale(&mat, &vec3(scale, scale, scale));
    mat
}

/// Renderiza completamente una escena en el framebuffer sin movimiento automático
fn render_scene(
    framebuffer: &mut Framebuffer,
    scene: &Scene,
    rotation_x: f32,
    rotation_y: f32,
    view_scale: f32,
) {
    // tiempo 0.0 para anular cualquier movimiento automático procedural
    let uniforms = Uniforms { time: 0.0 };
    framebuffer.clear();

    // Renderizar cuerpo principal
    for body in &scene.bodies {
        let model_matrix = create_model_matrix(
            &body.position,
            rotation_x,
            rotation_y,
            body.scale,
        );

        render_mesh(
            framebuffer,
            &body.vertices,
            &body.normals,
            &body.indices,
            &model_matrix,
            view_scale,
            &uniforms,
            body.shader,
        );
    }

    // Renderizar anillos si esta saturno en escena
    if scene.has_rings {
        let parent = &scene.bodies[scene.ring_parent_idx];

        let mut ring_matrix = Mat4::identity();
        ring_matrix = nalgebra_glm::translate(&ring_matrix, &parent.position);
        ring_matrix = nalgebra_glm::rotate_x(&ring_matrix, rotation_x);
        ring_matrix = nalgebra_glm::rotate_y(&ring_matrix, rotation_y);
        // Inclinación característica de los anillos de Saturno 
        ring_matrix = nalgebra_glm::rotate_x(&ring_matrix, 0.47);
        ring_matrix = nalgebra_glm::rotate_z(&ring_matrix, 0.15);
        ring_matrix = nalgebra_glm::scale(&ring_matrix, &vec3(parent.scale, parent.scale, parent.scale));

        render_mesh(
            framebuffer,
            &scene.ring_vertices,
            &scene.ring_normals,
            &scene.ring_indices,
            &ring_matrix,
            view_scale,
            &uniforms,
            ShaderType::Rings,
        );
    }

    // Renderizar satélites naturales 
    for moon_data in &scene.moons {
        let base_moon_offset = vec3(moon_data.orbit_radius, 0.25, 0.2);
        let rot_mat = nalgebra_glm::rotate_y(&nalgebra_glm::rotate_x(&Mat4::identity(), rotation_x), rotation_y);
        let rotated_offset4 = rot_mat * vec4(base_moon_offset.x, base_moon_offset.y, base_moon_offset.z, 0.0);
        let rotated_offset = vec3(rotated_offset4.x, rotated_offset4.y, rotated_offset4.z);

        let parent_pos = &scene.bodies[0].position;
        let moon_pos = parent_pos + rotated_offset;

        let model_matrix = create_model_matrix(&moon_pos, rotation_x, rotation_y, moon_data.scale);

        render_mesh(
            framebuffer,
            &moon_data.vertices,
            &moon_data.normals,
            &moon_data.indices,
            &model_matrix,
            view_scale,
            &uniforms,
            moon_data.shader,
        );
    }
}

/// Dibuja el panel informativo y el menú de controles en pantalla
fn draw_hud(
    framebuffer: &mut Framebuffer,
    scene_name: &str,
    scene_idx: usize,
    rot_x: f32,
    rot_y: f32,
    show_menu: bool,
) {
    // Barra superior de estado
    font::draw_panel(framebuffer, 15, 12, 450, 28, 0x070B1E, 0x244284);
    let title = format!("[{}/6] {}", scene_idx + 1, scene_name.to_uppercase());
    font::draw_text(framebuffer, 25, 21, &title, 0x00FFCC);

    let deg_x = (rot_x.to_degrees() % 360.0) as i32;
    let deg_y = (rot_y.to_degrees() % 360.0) as i32;
    let angles = format!("PITCH(W/S): {:>4}  YAW(A/D): {:>4}", deg_x, deg_y);
    font::draw_text(framebuffer, 230, 21, &angles, 0xCCDDEE);

    // Menú de controles
    if show_menu {
        font::draw_panel(framebuffer, 15, 485, 370, 200, 0x070B1E, 0x2A5298);
        font::draw_text(framebuffer, 25, 498, "        GUIA DE CONTROLES", 0xFFD700);
        font::draw_text(framebuffer, 25, 520, "[1-6]   Cambiar planeta activo", 0xFFFFFF);
        font::draw_text(framebuffer, 25, 542, "[W / S] Rotar verticalmente (Pitch eje X)", 0xFFFFFF);
        font::draw_text(framebuffer, 25, 564, "[A / D] Rotar horizontalmente (Yaw eje Y)", 0xFFFFFF);
        font::draw_text(framebuffer, 25, 586, "[R]     Resetear rotacion al origen", 0xAADDEE);
        font::draw_text(framebuffer, 25, 608, "[P]     Guardar captura actual en PNG", 0xAADDEE);
        font::draw_text(framebuffer, 25, 630, "[H]     Ocultar este menu de ayuda", 0x77FFAA);
        font::draw_text(framebuffer, 25, 652, "[ESC]   Salir de la aplicacion", 0xFFAAAA);
    } else {
        font::draw_panel(framebuffer, 15, 660, 220, 26, 0x070B1E, 0x244284);
        font::draw_text(framebuffer, 25, 669, "Presiona [H] para ver menu", 0x77FFAA);
    }
}

/// Escena del Sol
fn create_sun_scene() -> Scene {
    let (verts, norms, indices) = create_sphere(1.0, 40, 30);
    Scene {
        name: "Sol",
        bodies: vec![CelestialBody {
            vertices: verts,
            normals: norms,
            indices,
            shader: ShaderType::Sun,
            position: vec3(0.0, 0.0, 0.0),
            scale: 1.5,
        }],
        moons: vec![],
        has_rings: false,
        ring_vertices: vec![],
        ring_normals: vec![],
        ring_indices: vec![],
        ring_parent_idx: 0,
    }
}

/// Escena de Venus
fn create_venus_scene() -> Scene {
    let (verts, norms, indices) = create_sphere(1.0, 40, 30);
    Scene {
        name: "Venus",
        bodies: vec![CelestialBody {
            vertices: verts,
            normals: norms,
            indices,
            shader: ShaderType::Venus,
            position: vec3(0.0, 0.0, 0.0),
            scale: 1.2,
        }],
        moons: vec![],
        has_rings: false,
        ring_vertices: vec![],
        ring_normals: vec![],
        ring_indices: vec![],
        ring_parent_idx: 0,
    }
}

/// Escena de la Tierra con su Luna
fn create_earth_scene() -> Scene {
    let (verts, norms, indices) = create_sphere(1.0, 40, 30);
    let (moon_v, moon_n, moon_i) = create_sphere(1.0, 30, 20);
    Scene {
        name: "Tierra + Luna",
        bodies: vec![CelestialBody {
            vertices: verts,
            normals: norms,
            indices,
            shader: ShaderType::Earth,
            position: vec3(0.0, 0.0, 0.0),
            scale: 1.3,
        }],
        moons: vec![Moon {
            vertices: moon_v,
            normals: moon_n,
            indices: moon_i,
            shader: ShaderType::Moon,
            orbit_radius: 2.3,
            scale: 0.35,
        }],
        has_rings: false,
        ring_vertices: vec![],
        ring_normals: vec![],
        ring_indices: vec![],
        ring_parent_idx: 0,
    }
}

/// Escena de Marte
fn create_mars_scene() -> Scene {
    let (verts, norms, indices) = create_sphere(1.0, 40, 30);
    Scene {
        name: "Marte",
        bodies: vec![CelestialBody {
            vertices: verts,
            normals: norms,
            indices,
            shader: ShaderType::Mars,
            position: vec3(0.0, 0.0, 0.0),
            scale: 1.1,
        }],
        moons: vec![],
        has_rings: false,
        ring_vertices: vec![],
        ring_normals: vec![],
        ring_indices: vec![],
        ring_parent_idx: 0,
    }
}

/// Escena de Júpiter
fn create_jupiter_scene() -> Scene {
    let (verts, norms, indices) = create_sphere(1.0, 50, 35);
    Scene {
        name: "Jupiter",
        bodies: vec![CelestialBody {
            vertices: verts,
            normals: norms,
            indices,
            shader: ShaderType::Jupiter,
            position: vec3(0.0, 0.0, 0.0),
            scale: 1.8,
        }],
        moons: vec![],
        has_rings: false,
        ring_vertices: vec![],
        ring_normals: vec![],
        ring_indices: vec![],
        ring_parent_idx: 0,
    }
}

/// Escena de Saturno con anillos
fn create_saturn_scene() -> Scene {
    let (verts, norms, indices) = create_sphere(1.0, 50, 35);
    let (ring_v, ring_n, ring_i) = create_ring(1.4, 2.8, 80, 20);
    Scene {
        name: "Saturno + Anillos",
        bodies: vec![CelestialBody {
            vertices: verts,
            normals: norms,
            indices,
            shader: ShaderType::Saturn,
            position: vec3(0.0, 0.0, 0.0),
            scale: 1.5,
        }],
        moons: vec![],
        has_rings: true,
        ring_vertices: ring_v,
        ring_normals: ring_n,
        ring_indices: ring_i,
        ring_parent_idx: 0,
    }
}

fn main() {
    println!("");
    println!("Lab 5 - Sistema Solar con Shaders Procedurales");
    println!("Pedro Caso - 241286");
    println!("");

    // Crear las 6 escenas
    let scenes = vec![
        create_sun_scene(),
        create_venus_scene(),
        create_earth_scene(),
        create_mars_scene(),
        create_jupiter_scene(),
        create_saturn_scene(),
    ];

    let view_scale = (WINDOW_HEIGHT as f32).min(WINDOW_WIDTH as f32) * 0.22;

    // Modo sin interfaz gráfica para generar capturas
    if std::env::args().any(|arg| arg == "--screenshots") {
        println!("Generando capturas de pantalla para todos los cuerpos celestes...");
        let mut fb = Framebuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT);

        let configs = [
            ("screenshot_Sol.png", 0.0, 0.0),
            ("screenshot_Venus.png", 0.1, 0.5),
            ("screenshot_Tierra_y_Luna.png", 0.15, 0.4),
            ("screenshot_Marte.png", 0.1, 0.5),
            ("screenshot_Jupiter.png", 0.0, 0.6),
            ("screenshot_Saturno_y_Anillos.png", 0.25, 0.4),
        ];

        for (i, (filename, rot_x, rot_y)) in configs.iter().enumerate() {
            println!("Renderizando {}...", scenes[i].name);
            render_scene(&mut fb, &scenes[i], *rot_x, *rot_y, view_scale);
            if let Err(e) = fb.save_to_file(filename) {
                eprintln!("Error al guardar {}: {}", filename, e);
            } else {
                println!(" Guardado: {}", filename);
            }
        }
        println!("\n¡Todas las capturas se generaron exitosamente!");
        return;
    }

    let mut current_scene: usize = 0;
    let mut show_menu: bool = true;

    let mut window = Window::new(
        "Lab 5 - Sistema Solar - Pedro Caso",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("No se pudo crear la ventana: {}", e);
    });

    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT);

    let mut rotation_x: f32 = 0.0;
    let mut rotation_y: f32 = 0.0;

    println!("Controles interactivos:");
    println!("  [1-6]: Seleccionar planeta");
    println!("  [W / S]: Rotar verticalmente");
    println!("  [A / D]: Rotar horizontalmente");
    println!("  [R]: Resetear rotacion");
    println!("  [H]: Ocultar / Mostrar menu en pantalla");
    println!("  [P]: Guardar captura de pantalla PNG");
    println!("  [ESC]: Salir");
    println!();
    println!("Mostrando: {}", scenes[current_scene].name);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Cambiar escena con teclas numéricas 1..6
        let prev_scene = current_scene;
        if window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No) { current_scene = 0; }
        if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) { current_scene = 1; }
        if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) { current_scene = 2; }
        if window.is_key_pressed(Key::Key4, minifb::KeyRepeat::No) { current_scene = 3; }
        if window.is_key_pressed(Key::Key5, minifb::KeyRepeat::No) { current_scene = 4; }
        if window.is_key_pressed(Key::Key6, minifb::KeyRepeat::No) { current_scene = 5; }

        if current_scene != prev_scene {
            println!("Cambiado a: {}", scenes[current_scene].name);
            rotation_x = 0.0;
            rotation_y = 0.0;
        }

        // Rotación 3D completamente manual controlada por el usuario
        let rot_speed = 0.04;
        if window.is_key_down(Key::W) || window.is_key_down(Key::Up) {
            rotation_x -= rot_speed;
        }
        if window.is_key_down(Key::S) || window.is_key_down(Key::Down) {
            rotation_x += rot_speed;
        }
        if window.is_key_down(Key::A) || window.is_key_down(Key::Left) {
            rotation_y -= rot_speed;
        }
        if window.is_key_down(Key::D) || window.is_key_down(Key::Right) {
            rotation_y += rot_speed;
        }

        // Resetear rotación
        if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) {
            rotation_x = 0.0;
            rotation_y = 0.0;
        }

        // Alternar visualización del menú en pantalla
        if window.is_key_pressed(Key::H, minifb::KeyRepeat::No) {
            show_menu = !show_menu;
        }

        // Guardar captura de pantalla actual
        if window.is_key_pressed(Key::P, minifb::KeyRepeat::No) {
            let filename = format!(
                "screenshot_{}.png",
                scenes[current_scene].name.replace(' ', "_").replace('+', "y")
            );
            if let Err(e) = framebuffer.save_to_file(&filename) {
                eprintln!("Error al guardar captura: {}", e);
            } else {
                println!("Captura guardada: {}", filename);
            }
        }

        // Renderizar la geometría 3D y el shader procedural
        render_scene(
            &mut framebuffer,
            &scenes[current_scene],
            rotation_x,
            rotation_y,
            view_scale,
        );

        // Dibujar interfaz y menú informativo sobre el framebuffer
        draw_hud(
            &mut framebuffer,
            scenes[current_scene].name,
            current_scene,
            rotation_x,
            rotation_y,
            show_menu,
        );

        window
            .update_with_buffer(&framebuffer.buffer, WINDOW_WIDTH, WINDOW_HEIGHT)
            .unwrap();
    }
}
