use image::{ImageBuffer, Rgb, RgbImage};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub const IMAGE_WIDTH: u32 = 400;
pub const IMAGE_HEIGHT: u32 = 300;
pub const BANNER_WIDTH: u32 = 600;
pub const BANNER_HEIGHT: u32 = 200;

type Color = [u8; 3];

/// Generate an image based on a seed string (deterministic)
pub fn generate_image_from_seed(seed: &str) -> RgbImage {
    // Create a deterministic seed from the string
    let hash = simple_hash(seed);
    let mut rng = ChaCha8Rng::seed_from_u64(hash);
    
    // Choose a generator based on the seed
    let generator_choice = rng.gen_range(0..8);
    
    match generator_choice {
        0 => generate_noise(&mut rng),
        1 => generate_geometric_circles(&mut rng),
        2 => generate_geometric_rectangles(&mut rng),
        3 => generate_gradient(&mut rng),
        4 => generate_mondrian(&mut rng),
        5 => generate_wave_interference(&mut rng),
        6 => generate_pixel_pattern(&mut rng),
        7 => generate_stripes(&mut rng),
        _ => generate_noise(&mut rng),
    }
}

/// Generate a profile avatar based on username (deterministic, square)
pub fn generate_avatar_from_seed(seed: &str) -> RgbImage {
    let hash = simple_hash(seed);
    let mut rng = ChaCha8Rng::seed_from_u64(hash);
    
    // Avatars use a symmetric pattern for a more "icon-like" feel
    generate_symmetric_avatar(&mut rng)
}

/// Generate a profile banner/cover image based on username (deterministic, wide)
pub fn generate_banner_from_seed(seed: &str) -> RgbImage {
    let hash = simple_hash(seed);
    let mut rng = ChaCha8Rng::seed_from_u64(hash);
    
    // Choose a banner style
    let style = rng.gen_range(0..6);
    
    match style {
        0 => generate_banner_gradient(&mut rng),
        1 => generate_banner_waves(&mut rng),
        2 => generate_banner_geometric(&mut rng),
        3 => generate_banner_noise_gradient(&mut rng),
        4 => generate_banner_stripes(&mut rng),
        _ => generate_banner_abstract(&mut rng),
    }
}

/// Gradient banner (horizontal, vertical, or diagonal)
fn generate_banner_gradient<R: Rng>(rng: &mut R) -> RgbImage {
    let mut img = ImageBuffer::new(BANNER_WIDTH, BANNER_HEIGHT);
    
    let color1 = random_saturated_color(rng);
    let color2 = random_saturated_color(rng);
    
    let diagonal = rng.gen_bool(0.5);
    
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let t = if diagonal {
            ((x as f32 / BANNER_WIDTH as f32) + (y as f32 / BANNER_HEIGHT as f32)) / 2.0
        } else {
            x as f32 / BANNER_WIDTH as f32
        };
        
        let color = lerp_color(color1, color2, t);
        *pixel = Rgb(color);
    }
    
    img
}

/// Wave pattern banner
fn generate_banner_waves<R: Rng>(rng: &mut R) -> RgbImage {
    let mut img = ImageBuffer::new(BANNER_WIDTH, BANNER_HEIGHT);
    
    let color1 = random_saturated_color(rng);
    let color2 = random_saturated_color(rng);
    let num_waves = rng.gen_range(3..8);
    let amplitude = rng.gen_range(20.0..50.0);
    let frequency = rng.gen_range(0.01..0.05);
    
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let wave_y = (BANNER_HEIGHT as f32 / 2.0) + 
            amplitude * ((x as f32 * frequency).sin() + (x as f32 * frequency * 2.0).sin() * 0.5);
        
        let dist_from_wave = (y as f32 - wave_y).abs();
        let band = (dist_from_wave / (BANNER_HEIGHT as f32 / num_waves as f32)) as usize;
        
        let t = (band as f32 / num_waves as f32).min(1.0);
        let color = lerp_color(color1, color2, t);
        *pixel = Rgb(color);
    }
    
    img
}

/// Geometric shapes banner
fn generate_banner_geometric<R: Rng>(rng: &mut R) -> RgbImage {
    let bg_color = random_light_color(rng);
    let mut img = ImageBuffer::from_pixel(BANNER_WIDTH, BANNER_HEIGHT, Rgb(bg_color));
    
    let num_shapes = rng.gen_range(8..20);
    
    for _ in 0..num_shapes {
        let shape = rng.gen_range(0..2);
        let color = random_saturated_color(rng);
        
        match shape {
            0 => {
                // Circle
                let cx = rng.gen_range(0..BANNER_WIDTH as i32);
                let cy = rng.gen_range(0..BANNER_HEIGHT as i32);
                let radius = rng.gen_range(20..80);
                draw_circle(&mut img, cx, cy, radius, color, rng.gen_bool(0.6));
            }
            _ => {
                // Rectangle
                let x = rng.gen_range(0..BANNER_WIDTH);
                let y = rng.gen_range(0..BANNER_HEIGHT);
                let w = rng.gen_range(30..150);
                let h = rng.gen_range(20..80);
                draw_rect(&mut img, x, y, w, h, color);
            }
        }
    }
    
    img
}

/// Noisy gradient banner
fn generate_banner_noise_gradient<R: Rng>(rng: &mut R) -> RgbImage {
    let mut img = ImageBuffer::new(BANNER_WIDTH, BANNER_HEIGHT);
    
    let color1 = random_saturated_color(rng);
    let color2 = random_saturated_color(rng);
    let noise_amount = rng.gen_range(10..40);
    
    for (x, _y, pixel) in img.enumerate_pixels_mut() {
        let t = x as f32 / BANNER_WIDTH as f32;
        let base_color = lerp_color(color1, color2, t);
        
        // Add noise
        let noise_r = rng.gen_range(-(noise_amount as i32)..(noise_amount as i32));
        let noise_g = rng.gen_range(-(noise_amount as i32)..(noise_amount as i32));
        let noise_b = rng.gen_range(-(noise_amount as i32)..(noise_amount as i32));
        
        let color = [
            (base_color[0] as i32 + noise_r).clamp(0, 255) as u8,
            (base_color[1] as i32 + noise_g).clamp(0, 255) as u8,
            (base_color[2] as i32 + noise_b).clamp(0, 255) as u8,
        ];
        
        *pixel = Rgb(color);
    }
    
    img
}

/// Striped banner
fn generate_banner_stripes<R: Rng>(rng: &mut R) -> RgbImage {
    let mut img = ImageBuffer::new(BANNER_WIDTH, BANNER_HEIGHT);
    
    let num_colors = rng.gen_range(2..5);
    let colors: Vec<[u8; 3]> = (0..num_colors).map(|_| random_saturated_color(rng)).collect();
    
    let stripe_width = rng.gen_range(20..80);
    let diagonal = rng.gen_bool(0.6);
    
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let pos = if diagonal {
            (x as i32 + y as i32) as u32
        } else {
            x
        };
        
        let stripe_idx = (pos / stripe_width) as usize % colors.len();
        *pixel = Rgb(colors[stripe_idx]);
    }
    
    img
}

/// Abstract layered banner
fn generate_banner_abstract<R: Rng>(rng: &mut R) -> RgbImage {
    let bg_color = random_saturated_color(rng);
    let mut img = ImageBuffer::from_pixel(BANNER_WIDTH, BANNER_HEIGHT, Rgb(bg_color));
    
    // Add overlapping semi-transparent layers
    let num_layers = rng.gen_range(3..7);
    
    for _ in 0..num_layers {
        let color = random_saturated_color(rng);
        let layer_type = rng.gen_range(0..3);
        
        match layer_type {
            0 => {
                // Horizontal band
                let y_start = rng.gen_range(0..BANNER_HEIGHT);
                let height = rng.gen_range(30..100);
                for y in y_start..((y_start + height).min(BANNER_HEIGHT)) {
                    for x in 0..BANNER_WIDTH {
                        let existing = img.get_pixel(x, y).0;
                        let blended = lerp_color(existing, color, 0.5);
                        img.put_pixel(x, y, Rgb(blended));
                    }
                }
            }
            1 => {
                // Diagonal band
                let offset = rng.gen_range(-(BANNER_WIDTH as i32 / 2)..(BANNER_WIDTH as i32));
                let width = rng.gen_range(50..150);
                for y in 0..BANNER_HEIGHT {
                    for x in 0..BANNER_WIDTH {
                        let diag = x as i32 - y as i32;
                        if diag > offset && diag < offset + width as i32 {
                            let existing = img.get_pixel(x, y).0;
                            let blended = lerp_color(existing, color, 0.5);
                            img.put_pixel(x, y, Rgb(blended));
                        }
                    }
                }
            }
            _ => {
                // Large circle overlay
                let cx = rng.gen_range(0..BANNER_WIDTH as i32);
                let cy = rng.gen_range(0..BANNER_HEIGHT as i32);
                let radius = rng.gen_range(50..150);
                
                for y in 0..BANNER_HEIGHT {
                    for x in 0..BANNER_WIDTH {
                        let dx = x as i32 - cx;
                        let dy = y as i32 - cy;
                        if dx * dx + dy * dy < radius * radius {
                            let existing = img.get_pixel(x, y).0;
                            let blended = lerp_color(existing, color, 0.4);
                            img.put_pixel(x, y, Rgb(blended));
                        }
                    }
                }
            }
        }
    }
    
    img
}

fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

/// Random RGB noise
fn generate_noise<R: Rng>(rng: &mut R) -> RgbImage {
    let mut img = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    // Optional: add a color tint
    let use_tint = rng.gen_bool(0.5);
    let tint_color = random_color(rng);

    for pixel in img.pixels_mut() {
        let color = random_color(rng);

        if use_tint {
            // Blend with tint
            let blended = [
                ((color[0] as u16 + tint_color[0] as u16) / 2) as u8,
                ((color[1] as u16 + tint_color[1] as u16) / 2) as u8,
                ((color[2] as u16 + tint_color[2] as u16) / 2) as u8,
            ];
            *pixel = Rgb(blended);
        } else {
            *pixel = Rgb(color);
        }
    }

    img
}

/// Random circles on a background
fn generate_geometric_circles<R: Rng>(rng: &mut R) -> RgbImage {
    let bg_color = random_color(rng);
    let mut img = ImageBuffer::from_pixel(IMAGE_WIDTH, IMAGE_HEIGHT, Rgb(bg_color));
    
    let num_circles = rng.gen_range(5..20);
    
    for _ in 0..num_circles {
        let cx = rng.gen_range(0..IMAGE_WIDTH as i32);
        let cy = rng.gen_range(0..IMAGE_HEIGHT as i32);
        let radius = rng.gen_range(20..100);
        let color = random_color(rng);
        let filled = rng.gen_bool(0.7);
        
        draw_circle(&mut img, cx, cy, radius, color, filled);
    }
    
    img
}

/// Random rectangles (Mondrian-lite)
fn generate_geometric_rectangles<R: Rng>(rng: &mut R) -> RgbImage {
    let bg_color = random_color(rng);
    let mut img = ImageBuffer::from_pixel(IMAGE_WIDTH, IMAGE_HEIGHT, Rgb(bg_color));
    
    let num_rects = rng.gen_range(5..15);
    
    for _ in 0..num_rects {
        let x = rng.gen_range(0..IMAGE_WIDTH);
        let y = rng.gen_range(0..IMAGE_HEIGHT);
        let w = rng.gen_range(30..150);
        let h = rng.gen_range(30..150);
        let color = random_color(rng);
        
        draw_rect(&mut img, x, y, w, h, color);
    }
    
    img
}

/// Multi-stop gradient
fn generate_gradient<R: Rng>(rng: &mut R) -> RgbImage {
    let mut img = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    
    let color1 = random_color(rng);
    let color2 = random_color(rng);
    let color3 = random_color(rng);
    
    let horizontal = rng.gen_bool(0.5);
    let diagonal = rng.gen_bool(0.3);
    
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let t = if diagonal {
            ((x as f32 / IMAGE_WIDTH as f32) + (y as f32 / IMAGE_HEIGHT as f32)) / 2.0
        } else if horizontal {
            x as f32 / IMAGE_WIDTH as f32
        } else {
            y as f32 / IMAGE_HEIGHT as f32
        };
        
        // Three-color gradient
        let color = if t < 0.5 {
            let t2 = t * 2.0;
            lerp_color(color1, color2, t2)
        } else {
            let t2 = (t - 0.5) * 2.0;
            lerp_color(color2, color3, t2)
        };
        
        *pixel = Rgb(color);
    }
    
    img
}

/// Mondrian-style grid
fn generate_mondrian<R: Rng>(rng: &mut R) -> RgbImage {
    let mut img = ImageBuffer::from_pixel(IMAGE_WIDTH, IMAGE_HEIGHT, Rgb([255, 255, 255]));
    
    // Mondrian colors: red, blue, yellow, white, black
    let colors = [
        [255, 255, 255], // white
        [255, 0, 0],     // red
        [0, 0, 255],     // blue
        [255, 255, 0],   // yellow
        [0, 0, 0],       // black (rare)
    ];
    
    // Generate grid divisions
    let mut x_divs: Vec<u32> = vec![0];
    let mut x = 0;
    while x < IMAGE_WIDTH {
        x += rng.gen_range(50..150);
        if x < IMAGE_WIDTH {
            x_divs.push(x);
        }
    }
    x_divs.push(IMAGE_WIDTH);
    
    let mut y_divs: Vec<u32> = vec![0];
    let mut y = 0;
    while y < IMAGE_HEIGHT {
        y += rng.gen_range(40..120);
        if y < IMAGE_HEIGHT {
            y_divs.push(y);
        }
    }
    y_divs.push(IMAGE_HEIGHT);
    
    // Fill rectangles
    for i in 0..x_divs.len() - 1 {
        for j in 0..y_divs.len() - 1 {
            let color_idx = if rng.gen_bool(0.6) { 0 } else { rng.gen_range(0..colors.len()) };
            let color = colors[color_idx];
            
            draw_rect(
                &mut img,
                x_divs[i],
                y_divs[j],
                x_divs[i + 1] - x_divs[i],
                y_divs[j + 1] - y_divs[j],
                color,
            );
        }
    }
    
    // Draw black grid lines
    let line_width = 4;
    for &x in &x_divs[1..x_divs.len() - 1] {
        for y in 0..IMAGE_HEIGHT {
            for dx in 0..line_width {
                if x + dx < IMAGE_WIDTH {
                    img.put_pixel(x + dx, y, Rgb([0, 0, 0]));
                }
            }
        }
    }
    for &y in &y_divs[1..y_divs.len() - 1] {
        for x in 0..IMAGE_WIDTH {
            for dy in 0..line_width {
                if y + dy < IMAGE_HEIGHT {
                    img.put_pixel(x, y + dy, Rgb([0, 0, 0]));
                }
            }
        }
    }
    
    img
}

/// Wave interference pattern
fn generate_wave_interference<R: Rng>(rng: &mut R) -> RgbImage {
    let mut img = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    
    let num_waves = rng.gen_range(2..5);
    let mut wave_params: Vec<(f32, f32, f32, f32)> = Vec::new();
    
    for _ in 0..num_waves {
        wave_params.push((
            rng.gen_range(0.0..IMAGE_WIDTH as f32),   // center x
            rng.gen_range(0.0..IMAGE_HEIGHT as f32),  // center y
            rng.gen_range(0.02..0.1),                 // frequency
            rng.gen_range(0.0..std::f32::consts::TAU), // phase
        ));
    }
    
    let color1 = random_color(rng);
    let color2 = random_color(rng);
    
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let mut sum = 0.0;
        
        for (cx, cy, freq, phase) in &wave_params {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            sum += (dist * freq + phase).sin();
        }
        
        let t = (sum / num_waves as f32 + 1.0) / 2.0;
        let color = lerp_color(color1, color2, t);
        *pixel = Rgb(color);
    }
    
    img
}

/// Symmetric pixel pattern (like an identicon)
fn generate_pixel_pattern<R: Rng>(rng: &mut R) -> RgbImage {
    let bg_color = random_light_color(rng);
    let fg_color = random_color(rng);
    let mut img = ImageBuffer::from_pixel(IMAGE_WIDTH, IMAGE_HEIGHT, Rgb(bg_color));
    
    let block_size = rng.gen_range(15..40);
    let cols = (IMAGE_WIDTH / block_size) as usize;
    let rows = (IMAGE_HEIGHT / block_size) as usize;
    
    // Generate half the pattern, then mirror
    let half_cols = (cols + 1) / 2;
    
    for row in 0..rows {
        for col in 0..half_cols {
            if rng.gen_bool(0.5) {
                // Draw block on left side
                let x = col as u32 * block_size;
                let y = row as u32 * block_size;
                draw_rect(&mut img, x, y, block_size, block_size, fg_color);
                
                // Mirror on right side
                let mirror_col = cols - 1 - col;
                let mirror_x = mirror_col as u32 * block_size;
                draw_rect(&mut img, mirror_x, y, block_size, block_size, fg_color);
            }
        }
    }
    
    img
}

/// Striped pattern
fn generate_stripes<R: Rng>(rng: &mut R) -> RgbImage {
    let mut img = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    
    let num_colors = rng.gen_range(2..5);
    let colors: Vec<[u8; 3]> = (0..num_colors).map(|_| random_color(rng)).collect();
    
    let stripe_width = rng.gen_range(10..50);
    let horizontal = rng.gen_bool(0.5);
    let diagonal = rng.gen_bool(0.3);
    
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let pos = if diagonal {
            (x as i32 + y as i32) as u32
        } else if horizontal {
            y
        } else {
            x
        };
        
        let stripe_idx = (pos / stripe_width) as usize % colors.len();
        *pixel = Rgb(colors[stripe_idx]);
    }
    
    img
}

/// Generate symmetric avatar (for profile pictures)
fn generate_symmetric_avatar<R: Rng>(rng: &mut R) -> RgbImage {
    let size: u32 = 128;
    let bg_color = random_light_color(rng);
    let fg_color = random_saturated_color(rng);
    let mut img = ImageBuffer::from_pixel(size, size, Rgb(bg_color));
    
    let block_size = 16u32;
    let blocks = size / block_size;
    let half_blocks = (blocks + 1) / 2;
    
    for row in 0..blocks {
        for col in 0..half_blocks {
            if rng.gen_bool(0.55) {
                let x = col * block_size;
                let y = row * block_size;
                draw_rect(&mut img, x, y, block_size, block_size, fg_color);
                
                // Mirror
                let mirror_x = (blocks - 1 - col) * block_size;
                draw_rect(&mut img, mirror_x, y, block_size, block_size, fg_color);
            }
        }
    }
    
    img
}

// Helper functions

fn random_color<R: Rng>(rng: &mut R) -> Color {
    [
        rng.gen_range(0u8..=255u8),
        rng.gen_range(0u8..=255u8),
        rng.gen_range(0u8..=255u8),
    ]
}

fn random_light_color<R: Rng>(rng: &mut R) -> Color {
    [
        rng.gen_range(200u8..255u8),
        rng.gen_range(200u8..255u8),
        rng.gen_range(200u8..255u8),
    ]
}

fn random_saturated_color<R: Rng>(rng: &mut R) -> Color {
    // Generate a more saturated/vibrant color
    let hue = rng.gen_range(0.0..360.0);
    hsv_to_rgb(hue, 0.7, 0.8)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    [
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    ]
}

fn lerp_color(c1: Color, c2: Color, t: f32) -> Color {
    [
        (c1[0] as f32 + (c2[0] as f32 - c1[0] as f32) * t) as u8,
        (c1[1] as f32 + (c2[1] as f32 - c1[1] as f32) * t) as u8,
        (c1[2] as f32 + (c2[2] as f32 - c1[2] as f32) * t) as u8,
    ]
}

fn draw_rect(img: &mut RgbImage, x: u32, y: u32, w: u32, h: u32, color: Color) {
    for dy in 0..h {
        for dx in 0..w {
            let px = x + dx;
            let py = y + dy;
            if px < img.width() && py < img.height() {
                img.put_pixel(px, py, Rgb(color));
            }
        }
    }
}

fn draw_circle(img: &mut RgbImage, cx: i32, cy: i32, radius: i32, color: Color, filled: bool) {
    for y in (cy - radius)..=(cy + radius) {
        for x in (cx - radius)..=(cx + radius) {
            if x >= 0 && y >= 0 && x < img.width() as i32 && y < img.height() as i32 {
                let dx = x - cx;
                let dy = y - cy;
                let dist_sq = dx * dx + dy * dy;
                let radius_sq = radius * radius;
                
                if filled {
                    if dist_sq <= radius_sq {
                        img.put_pixel(x as u32, y as u32, Rgb(color));
                    }
                } else {
                    // Ring with some thickness
                    let inner_radius_sq = (radius - 3) * (radius - 3);
                    if dist_sq <= radius_sq && dist_sq >= inner_radius_sq {
                        img.put_pixel(x as u32, y as u32, Rgb(color));
                    }
                }
            }
        }
    }
}
