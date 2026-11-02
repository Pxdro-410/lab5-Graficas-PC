pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    pub zbuffer: Vec<f32>,
    pub stars_buffer: Vec<u32>,
    background_color: u32,
    current_color: u32,
}

/// Generador de números pseudoaleatorios
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u32() & 0xFFFFFF) as f32 / 16777216.0
    }
}

#[allow(dead_code)]
impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut stars_buffer = vec![0x02020a; width * height];

        // Generar un campo de estrellas 
        let mut rng = Rng::new(421337);

        // Generar nebulosas de fondo 
        for y in 0..height {
            let ny = y as f32 / height as f32;
            for x in 0..width {
                let nx = x as f32 / width as f32;
                // Gradiente  
                let dist = ((nx - 0.5) * (nx - 0.5) + (ny - 0.5) * (ny - 0.5)).sqrt();
                let nebula = (1.0 - dist * 1.2).max(0.0) * 0.04;
                let r = (nebula * 20.0) as u32;
                let g = (nebula * 15.0) as u32;
                let b = (nebula * 45.0 + 10.0) as u32;
                stars_buffer[y * width + x] = (r << 16) | (g << 8) | b;
            }
        }

        // Sembrar 1200 estrellas de distintos brillos, tipos y tamaños
        let num_stars = 1200;
        for _ in 0..num_stars {
            let x = (rng.next_u32() as usize) % width;
            let y = (rng.next_u32() as usize) % height;
            let roll = rng.next_f32();

            if roll < 0.65 {
                // Estrellas pequeñas y tenues 
                let br = (rng.next_f32() * 80.0 + 80.0) as u32;
                let color = (br << 16) | ((br + 5) << 8) | (br + 15).min(255);
                stars_buffer[y * width + x] = color;
            } else if roll < 0.92 {
                // Estrellas medianas
                let br = (rng.next_f32() * 60.0 + 190.0) as u32;
                let tint_warm = rng.next_f32() > 0.6;
                let color = if tint_warm {
                    (br << 16) | ((br - 10) << 8) | (br - 30) // celeste sutil
                } else {
                    ((br - 15) << 16) | ((br - 5) << 8) | br // azulada
                };
                stars_buffer[y * width + x] = color;
            } else {
                // Estrellas brillantes con destello en cruz
                let color_core = 0xFFFFFF;
                let color_halo = 0x667799;
                stars_buffer[y * width + x] = color_core;
                if x > 0 { stars_buffer[y * width + (x - 1)] = color_halo; }
                if x + 1 < width { stars_buffer[y * width + (x + 1)] = color_halo; }
                if y > 0 { stars_buffer[(y - 1) * width + x] = color_halo; }
                if y + 1 < height { stars_buffer[(y + 1) * width + x] = color_halo; }
            }
        }

        Framebuffer {
            width,
            height,
            buffer: stars_buffer.clone(),
            zbuffer: vec![f32::INFINITY; width * height],
            stars_buffer,
            background_color: 0x02020a,
            current_color: 0xFFFFFF,
        }
    }

    /// Limpia el framebuffer restaurando el fondo estelar y reiniciando el z-buffer
    pub fn clear(&mut self) {
        self.buffer.copy_from_slice(&self.stars_buffer);
        self.zbuffer.fill(f32::INFINITY);
    }

    pub fn point(&mut self, x: i32, y: i32) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.buffer[y as usize * self.width + x as usize] = self.current_color;
        }
    }

    /// Dibuja un punto con test de profundidad o zbuffer
    pub fn point_with_depth(&mut self, x: usize, y: usize, depth: f32, color: u32) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            if depth < self.zbuffer[idx] {
                self.buffer[idx] = color;
                self.zbuffer[idx] = depth;
            }
        }
    }

    pub fn get_color(&self, x: usize, y: usize) -> u32 {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x]
        } else {
            self.background_color
        }
    }

    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }

    pub fn get_background_color(&self) -> u32 {
        self.background_color
    }

    pub fn get_current_color(&self) -> u32 {
        self.current_color
    }

    /// Exporta el contenido actual del framebuffer a un archivo de imagen.
    pub fn save_to_file(&self, path: &str) -> Result<(), image::ImageError> {
        let mut img = image::RgbImage::new(self.width as u32, self.height as u32);
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = self.buffer[y * self.width + x];
                let r = ((pixel >> 16) & 0xFF) as u8;
                let g = ((pixel >> 8) & 0xFF) as u8;
                let b = (pixel & 0xFF) as u8;
                img.put_pixel(x as u32, y as u32, image::Rgb([r, g, b]));
            }
        }
        img.save(path)
    }
}
