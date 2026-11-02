/// Implementación de ruido Perlin 3D y funciones derivadas
/// para generar patrones procedurales en los shaders de planetas.

// Tabla de permutación para ruido Perlin
const PERM: [u8; 512] = {
    const P: [u8; 256] = [
        151,160,137,91,90,15,131,13,201,95,96,53,194,233,7,225,
        140,36,103,30,69,142,8,99,37,240,21,10,23,190,6,148,
        247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,
        57,177,33,88,237,149,56,87,174,20,125,136,171,168,68,175,
        74,165,71,134,139,48,27,166,77,146,158,231,83,111,229,122,
        60,211,133,230,220,105,92,41,55,46,245,40,244,102,143,54,
        65,25,63,161,1,216,80,73,209,76,132,187,208,89,18,169,
        200,196,135,130,116,188,159,86,164,100,109,198,173,186,3,64,
        52,217,226,250,124,123,5,202,38,147,118,126,255,82,85,212,
        207,206,59,227,47,16,58,17,182,189,28,42,223,183,170,213,
        119,248,152,2,44,154,163,70,221,153,101,155,167,43,172,9,
        129,22,39,253,19,98,108,110,79,113,224,232,178,185,112,104,
        218,246,97,228,251,34,242,193,238,210,144,12,191,179,162,241,
        81,51,145,235,249,14,239,107,49,192,214,31,181,199,106,157,
        184,84,204,176,115,121,50,45,127,4,150,254,138,236,205,93,
        222,114,67,29,24,72,243,141,128,195,78,66,215,61,156,180,
    ];
    let mut table = [0u8; 512];
    let mut i = 0;
    while i < 512 {
        table[i] = P[i & 255];
        i += 1;
    }
    table
};

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp_f(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

fn grad3d(hash: u8, x: f32, y: f32, z: f32) -> f32 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 { y } else if h == 12 || h == 14 { x } else { z };
    let a = if h & 1 == 0 { u } else { -u };
    let b = if h & 2 == 0 { v } else { -v };
    a + b
}

/// Ruido Perlin 3D clásico
pub fn perlin_noise_3d(x: f32, y: f32, z: f32) -> f32 {
    let xi = (x.floor() as i32 & 255) as usize;
    let yi = (y.floor() as i32 & 255) as usize;
    let zi = (z.floor() as i32 & 255) as usize;

    let xf = x - x.floor();
    let yf = y - y.floor();
    let zf = z - z.floor();

    let u = fade(xf);
    let v = fade(yf);
    let w = fade(zf);

    let aaa = PERM[PERM[PERM[xi] as usize + yi] as usize + zi];
    let aba = PERM[PERM[PERM[xi] as usize + yi + 1] as usize + zi];
    let aab = PERM[PERM[PERM[xi] as usize + yi] as usize + zi + 1];
    let abb = PERM[PERM[PERM[xi] as usize + yi + 1] as usize + zi + 1];
    let baa = PERM[PERM[PERM[xi + 1] as usize + yi] as usize + zi];
    let bba = PERM[PERM[PERM[xi + 1] as usize + yi + 1] as usize + zi];
    let bab = PERM[PERM[PERM[xi + 1] as usize + yi] as usize + zi + 1];
    let bbb = PERM[PERM[PERM[xi + 1] as usize + yi + 1] as usize + zi + 1];

    let x1 = lerp_f(u, grad3d(aaa, xf, yf, zf), grad3d(baa, xf - 1.0, yf, zf));
    let x2 = lerp_f(u, grad3d(aba, xf, yf - 1.0, zf), grad3d(bba, xf - 1.0, yf - 1.0, zf));
    let y1 = lerp_f(v, x1, x2);

    let x1 = lerp_f(u, grad3d(aab, xf, yf, zf - 1.0), grad3d(bab, xf - 1.0, yf, zf - 1.0));
    let x2 = lerp_f(u, grad3d(abb, xf, yf - 1.0, zf - 1.0), grad3d(bbb, xf - 1.0, yf - 1.0, zf - 1.0));
    let y2 = lerp_f(v, x1, x2);

    lerp_f(w, y1, y2)
}

/// Ruido Perlin 2D
#[allow(dead_code)]
pub fn perlin_noise_2d(x: f32, y: f32) -> f32 {
    perlin_noise_3d(x, y, 0.0)
}

/// Fractional Brownian Motion que superpone múltiples octavas de ruido Perlin
/// para crear patrones con detalle a múltiples escalas.
#[allow(dead_code)]
pub fn fbm(x: f32, y: f32, z: f32, octaves: u32, lacunarity: f32, gain: f32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += amplitude * perlin_noise_3d(x * frequency, y * frequency, z * frequency);
        max_value += amplitude;
        amplitude *= gain;
        frequency *= lacunarity;
    }

    value / max_value
}

/// Turbulencia usa valores absolutos del ruido
/// para crear patrones más irregulares
#[allow(dead_code)]
pub fn turbulence(x: f32, y: f32, z: f32, octaves: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += amplitude * perlin_noise_3d(x * frequency, y * frequency, z * frequency).abs();
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    value / max_value
}

/// para crear patrones orgánicos más interesantes.
#[allow(dead_code)]
pub fn domain_warp(x: f32, y: f32, z: f32, scale: f32, warp_amount: f32) -> f32 {
    let wx = x + warp_amount * perlin_noise_3d(x * scale, y * scale, z * scale);
    let wy = y + warp_amount * perlin_noise_3d(x * scale + 5.2, y * scale + 1.3, z * scale + 2.7);
    let wz = z + warp_amount * perlin_noise_3d(x * scale + 9.1, y * scale + 4.6, z * scale + 7.8);
    perlin_noise_3d(wx * scale, wy * scale, wz * scale)
}

