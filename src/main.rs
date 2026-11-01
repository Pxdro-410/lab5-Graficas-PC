mod framebuffer;
mod model;
mod triangle;

use framebuffer::Framebuffer;
use minifb::{Key, Window, WindowOptions};
use model::Model;
use nalgebra_glm::{rotate_x, rotate_y, rotate_z, vec3, vec4, Mat4, Vec3};
use triangle::{draw_triangle_filled, draw_triangle_wireframe};

const WINDOW_WIDTH: usize = 900;
const WINDOW_HEIGHT: usize = 700;

const COLOR_FONDO: u32 = 0x0a0a25;
const COLOR_BORDE: u32 = 0xFF0000;
const COLOR_RELLENO: u32 = 0x501010;

#[derive(Copy, Clone, PartialEq)]
enum RenderMode {
    SolidoConBordes,
    MallaDeAlambre,
    Solido,
}

/// Transforma los vértices del modelo al espacio de pantalla y dibuja según el modo elegido.
fn render(
    framebuffer: &mut Framebuffer,
    model: &Model,
    rotation: &Vec3,
    scale: f32,
    mode: RenderMode,
    fill_color: u32,
    edge_color: u32,
) {
    let center = model.center();
    let screen_w = framebuffer.width as f32;
    let screen_h = framebuffer.height as f32;

    let mut rot_matrix = Mat4::identity();
    rot_matrix = rotate_x(&rot_matrix, rotation.x);
    rot_matrix = rotate_y(&rot_matrix, rotation.y);
    rot_matrix = rotate_z(&rot_matrix, rotation.z);

    let transformed_vertices: Vec<Vec3> = model
        .vertices
        .iter()
        .map(|v| {
            let centered = v - center;
            let v4 = vec4(centered.x, centered.y, centered.z, 1.0);
            let rotated = rot_matrix * v4;

            // Proyectar y escalar al centro de la pantalla
            let screen_x = rotated.x * scale + (screen_w / 2.0);
            let screen_y = -rotated.y * scale + (screen_h / 2.0);
            let screen_z = rotated.z;

            vec3(screen_x, screen_y, screen_z)
        })
        .collect();

    if mode == RenderMode::Solido || mode == RenderMode::SolidoConBordes {
        framebuffer.set_current_color(fill_color);
        for face in &model.faces {
            let v0 = &transformed_vertices[face[0]];
            let v1 = &transformed_vertices[face[1]];
            let v2 = &transformed_vertices[face[2]];
            draw_triangle_filled(framebuffer, v0, v1, v2);
        }
    }

    if mode == RenderMode::MallaDeAlambre || mode == RenderMode::SolidoConBordes {
        framebuffer.set_current_color(edge_color);
        for face in &model.faces {
            let v0 = &transformed_vertices[face[0]];
            let v1 = &transformed_vertices[face[1]];
            let v2 = &transformed_vertices[face[2]];
            draw_triangle_wireframe(framebuffer, v0, v1, v2);
        }
    }
}

fn main() {
    println!("Renderizador OBJ con color diferenciado de borde y relleno");

    let obj_path = "nave-lab3-PC.obj";
    let model = match Model::load(obj_path) {
        Ok(m) => {
            println!("Modelo cargado exitosamente:");
            println!("* Vértices: {}", m.vertices.len());
            println!("* Caras: {}", m.faces.len());
            println!("* Centro: {:?}", m.center());
            println!("* Extensión máxima: {:.4}", m.max_extent());
            m
        }
        Err(e) => {
            eprintln!("Error al cargar el archivo OBJ '{}': {}", obj_path, e);
            return;
        }
    };

    let mut window = Window::new(
        "Laboratorio 4 - Renderizador OBJ - Pedro Caso",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("No se pudo crear la ventana de minifb: {}", e);
    });

    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    framebuffer.set_background_color(COLOR_FONDO);

    let target_screen_size = (WINDOW_HEIGHT as f32).min(WINDOW_WIDTH as f32) * 0.70;
    let base_scale = target_screen_size / model.max_extent();
    let mut current_scale = base_scale;

    let mut rotation = vec3(-0.35, 0.70, 0.0);
    let mut current_mode = RenderMode::SolidoConBordes;

    println!("\nControles de la ventana");
    println!("* Flechas o teclas A, D, W, S: Rotar la nave en 3D");
    println!("* Q o E: Zoom para alejar o acercar");
    println!("* M: Alternar modo entre bordes y relleno, solo alambre, solo relleno");
    println!("* P: Guardar captura de pantalla en render.png");
    println!("* Espacio: Restablecer vista inicial");
    println!("* ESC: Salir");


    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_down(Key::Left) || window.is_key_down(Key::A) {
            rotation.y -= 0.04;
        }
        if window.is_key_down(Key::Right) || window.is_key_down(Key::D) {
            rotation.y += 0.04;
        }
        if window.is_key_down(Key::Up) || window.is_key_down(Key::W) {
            rotation.x -= 0.04;
        }
        if window.is_key_down(Key::Down) || window.is_key_down(Key::S) {
            rotation.x += 0.04;
        }

        if window.is_key_down(Key::E) {
            current_scale *= 1.02;
        }
        if window.is_key_down(Key::Q) {
            current_scale *= 0.98;
        }

        if window.is_key_down(Key::Space) {
            rotation = vec3(-0.35, 0.70, 0.0);
            current_scale = base_scale;
        }

        if window.is_key_pressed(Key::M, minifb::KeyRepeat::No) {
            current_mode = match current_mode {
                RenderMode::SolidoConBordes => RenderMode::MallaDeAlambre,
                RenderMode::MallaDeAlambre => RenderMode::Solido,
                RenderMode::Solido => RenderMode::SolidoConBordes,
            };
            let nombre_modo = match current_mode {
                RenderMode::SolidoConBordes => "Solido con bordes visibles",
                RenderMode::MallaDeAlambre => "Solo malla de alambre",
                RenderMode::Solido => "Solo relleno solido",
            };
            println!("Modo cambiado a: {}", nombre_modo);
        }

        if window.is_key_pressed(Key::P, minifb::KeyRepeat::No) {
            if let Err(e) = framebuffer.save_to_file("render.png") {
                eprintln!("Error al guardar captura: {}", e);
            } else {
                println!("Nueva captura guardada como render.png");
            }
        }

        framebuffer.clear();

        render(
            &mut framebuffer,
            &model,
            &rotation,
            current_scale,
            current_mode,
            COLOR_RELLENO,
            COLOR_BORDE,
        );

        window
            .update_with_buffer(&framebuffer.buffer, WINDOW_WIDTH, WINDOW_HEIGHT)
            .unwrap();
    }
}