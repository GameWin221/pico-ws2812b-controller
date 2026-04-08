use std::{fs::File, io::BufReader};

use rand::Rng;

// hue in degrees, saturation and value in range [0.0, 1.0]
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if h < 60.0 {
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

    let r = ((r1 + m) * 255.0) as u8;
    let g = ((g1 + m) * 255.0) as u8;
    let b = ((b1 + m) * 255.0) as u8;

    (r, g, b)
}

fn set_pixel(canvas: &mut [u8; 16*16*3], x: i32, y: i32, r: u8, g: u8, b: u8) {
    if y < 0 || y >= 16 || x < 0 || x >= 16 {
        return;
    }
    
    let idx = y as usize * 16 + x as usize;

    canvas[idx * 3 + 0] = r;
    canvas[idx * 3 + 1] = g;
    canvas[idx * 3 + 2] = b;
}

fn draw_circle_on_orbit(canvas: &mut [u8; 16*16*3], time: f32, shift: f32, distance: f32, r: u8, g: u8, b: u8) {
    let x0 = (((time + shift).cos() * 0.5 + 0.5) * (15.0 - distance * 2.0) + 0.5 + distance) as i32;
    let y0 = (((time + shift).sin() * 0.5 + 0.5) * (15.0 - distance * 2.0) + 0.5 + distance) as i32;

    set_pixel(canvas, x0, y0, r, g, b);
    set_pixel(canvas, x0, y0+1, r, g, b);
    set_pixel(canvas, x0, y0-1, r, g, b);
    set_pixel(canvas, x0+1, y0, r, g, b);
    set_pixel(canvas, x0-1, y0, r, g, b);
}
fn draw_filled_circle(canvas: &mut [u8; 16*16*3], xo: i32, yo: i32, diameter: i32, r: u8, g: u8, b: u8, half: bool) {
    let sy = if half { diameter / 2 } else { 0 };
    for y0 in sy..diameter {
        for x0 in 0..diameter {
            let y = y0 - diameter/2;
            let x = x0 - diameter/2;
            let d = ((x*x + y*y) as f32).sqrt();

            if d <= diameter as f32 / 2.0 {
                set_pixel(canvas, x + xo + diameter/2, y + yo + diameter/2, r, g, b);
            }
        }
    }
}
fn draw_half_square(canvas: &mut [u8; 16*16*3], x0: i32, y0: i32, size: i32, r: u8, g: u8, b: u8, inverted: bool) {
    let x1 = x0 + size - 1;
    let y1 = y0 + size - 1;

    for x in x0..=x1 {
        let dx = if inverted { x1 - x } else { x - x0 };
        
        for y in (y1-dx)..=y1 {
            set_pixel(canvas, x, y, r, g, b);
        }
    }

}
fn draw_line(canvas: &mut [u8; 16*16*3], x0: i32, y0: i32, x1: i32, y1: i32, r: u8, g: u8, b: u8) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = x0;
    let mut y = y0;

    loop {
        set_pixel(canvas, x, y, r, g, b);

        if x == x1 && y == y1 {
            break;
        }

        let err2 = err * 2;

        if err2 > -dy {
            err -= dy;
            x += sx;
        }
        if err2 < dx {
            err += dx;
            y += sy;
        }
    }
}

fn fade_canvas(canvas: &mut [u8; 16*16*3], r_fade: u8, g_fade: u8, b_fade: u8) {
    for i in 0..=255 {
        canvas[i*3+0] = canvas[i*3+0].saturating_sub(r_fade);
        canvas[i*3+1] = canvas[i*3+1].saturating_sub(g_fade);
        canvas[i*3+2] = canvas[i*3+2].saturating_sub(b_fade);
    }
}

pub trait Effect {
    fn process(&mut self, canvas: &mut [u8; 16*16*3], time: f32);
}

pub struct Meteors {
    particles: [(f32, f32); 12],
    rng: rand::rngs::ThreadRng
}
impl Meteors {
    pub fn new() -> Meteors {
        let mut particles = [(0.0, 0.0); 12];
        let mut rng = rand::thread_rng();

        for i in 0..particles.len() {
            particles[i].0 = rng.gen_range(0.0..16.0);
            particles[i].1 = rng.gen_range(0.0..16.0);
        }

        Meteors {
            particles,
            rng
        }
    }
}
impl Effect for Meteors {
    fn process(&mut self, canvas: &mut [u8; 16*16*3], time: f32) {
        fade_canvas(canvas, 12, 24, 12);

        for (x, y) in &mut self.particles {
            set_pixel(canvas, *x as i32, *y as i32, 0xff, 0xa0, 0x00);

            *y -= self.rng.gen_range(0.2..=1.0);

            if *y < 0.0 {
                *y = 15.999;

                *x -= self.rng.gen_range(-2.0..=2.0);

                if *x < 0.0 {
                    *x = 16.0 + *x;
                } else if *x >= 16.0 {
                    *x = *x - 16.0;
                }
            }
        } 
    }
}


pub struct Hearts {

}
impl Hearts {
    pub fn new() -> Hearts {
        Hearts { }
    }
}

impl Effect for Hearts {
    // time acts as the heart's radius
    fn process(&mut self, canvas: &mut [u8; 16*16*3], time: f32) {
        let radius = (time as i32).clamp(0, 8);

        fade_canvas(canvas, 255, 255, 255);

        draw_filled_circle(canvas, 8-radius, 8 - radius/2, radius, 0xff, 0x00, 0x00, true);
        draw_filled_circle(canvas, 8, 8 - radius/2, radius, 0xff, 0x00, 0x00, true);
        draw_half_square(canvas, 8-radius, 8-radius, radius, 0xff, 0x00, 0x00, false);
        draw_half_square(canvas, 8, 8-radius, radius, 0xff, 0x00, 0x00, true);
    }
}


pub struct Orbs {

}
impl Orbs {
    pub fn new() -> Orbs {
        Orbs { }
    }
}

impl Effect for Orbs {
    fn process(&mut self, canvas: &mut [u8; 16*16*3], time: f32) {
        fade_canvas(canvas, 12, 24, 12);

        draw_circle_on_orbit(canvas, time * 3.14159262      , 3.14159262 * 0.5, 1.0, 0x00, 0xff, 0xff);
        draw_circle_on_orbit(canvas, time * 3.14159262      , 3.14159262 * 1.5, 1.0, 0xff, 0x00, 0xff);
        draw_circle_on_orbit(canvas, time * 3.14159262 * 2.0, 0.0             , 5.0, 0xff, 0x00, 0xaa);
        draw_circle_on_orbit(canvas, time * 3.14159262 * 2.0, 3.14159262      , 5.0, 0xaa, 0x00, 0xff);
    }
}

pub struct Image16x16Sequence {
    bytes: Vec<u8>,
}
impl Image16x16Sequence {
    pub fn from_gif(path: &str) -> Image16x16Sequence {
        let file = File::open(path).unwrap();

        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);

        let mut decoder = options.read_info(file).unwrap();

        if decoder.width() != 16 || decoder.height() != 16 {
            panic!("Failed to import \"{}\". The imported gif must be exactly 16x16", path)
        }

        let frames_count = decoder.next_frame_info().into_iter().count();

        let mut data = Vec::<u8>::with_capacity(frames_count*16*16*3);
        
        while let Some(frame) = &mut decoder.read_next_frame().unwrap() { 
            for y in (0..16).rev() {
                for x in 0..16 {
                    let i = y * 16 + x;
                    let alpha = frame.buffer[i * 4 + 3];
    
                    data.push(frame.buffer[i * 4 + 0] & alpha);
                    data.push(frame.buffer[i * 4 + 1] & alpha);
                    data.push(frame.buffer[i * 4 + 2] & alpha);
                }
            }
        }

        Image16x16Sequence {
            bytes: data
        }
    }

    pub fn from_images(paths: &Vec<&str>) -> Image16x16Sequence {
        let mut bytes = Vec::with_capacity(paths.len() * 16*16*3);
    
        if paths.len() == 0 {
            panic!("You cannot create an empty Image16x16Sequence. Specify at least one image")
        }

        for &path in paths {
            if path.ends_with(".png") {
                let file = File::open(path).unwrap();
                let mut decoder = png::Decoder::new(file);
                decoder.set_transformations(png::Transformations::normalize_to_color8());

                let mut reader = decoder.read_info().unwrap();
                let mut img_data = vec![0; reader.output_buffer_size()];
                let info = reader.next_frame(&mut img_data).unwrap();

                if info.width != 16 || info.height != 16 {
                    panic!("Failed to import \"{}\". The imported png must be exactly 16x16", path)
                }

                match info.color_type {
                    png::ColorType::Rgb => {
                        for y in (0..16).rev() {
                            for x in 0..16 {
                                let i = (y * 16 + x) * 3;

                                bytes.push(img_data[i+0]);
                                bytes.push(img_data[i+1]);
                                bytes.push(img_data[i+2]);
                            }
                        }
                    },
                    png::ColorType::Rgba => {
                        for y in (0..16).rev() {
                            for x in 0..16 {
                                let i = (y * 16 + x) * 4;
                                let a = img_data[i+3];

                                bytes.push(img_data[i+0] & a);
                                bytes.push(img_data[i+1] & a);
                                bytes.push(img_data[i+2] & a);
                            }
                        }
                    },
                    _ => panic!("Failed to import \"{}\". The imported png can only be either RGB or RGBA", path),
                };

            } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
                let file = File::open(path).unwrap();
                let mut decoder = jpeg_decoder::Decoder::new(BufReader::new(file));
                let pixels = decoder.decode().unwrap();
                let metadata = decoder.info().unwrap();
                
                if metadata.width != 16 || metadata.height != 16 {
                    panic!("Failed to import \"{}\". The imported jpeg must be exactly 16x16", path)
                }

                for y in (0..16).rev() {
                    for x in 0..16 {
                        let i = (y * 16 + x) * 3;
        
                        bytes.push(pixels[i+0]);
                        bytes.push(pixels[i+1]);
                        bytes.push(pixels[i+2]);
                    }
                }
            } else if path.ends_with(".bmp") {
                let img = bmp::open(path).unwrap();

                if img.get_width() != 16 || img.get_height() != 16 {
                    panic!("Failed to import \"{}\". The imported bitmap must be exactly 16x16", path)
                }

                for y in (0..16).rev() {
                    for x in 0..16 {
                        let pixel = img.get_pixel(x as u32, y as u32);

                        bytes.push(pixel.r);
                        bytes.push(pixel.g);
                        bytes.push(pixel.b);
                    }
                }
            } else {
                panic!("Failed to import \"{}\". Use only .png / .jpg / .jpeg / .bmp", path)
            }
        }

        Image16x16Sequence {
            bytes: Vec::from(bytes)
        }
    }

    pub fn from_image(path: &str) -> Image16x16Sequence {
        Image16x16Sequence::from_images(&vec![path])
    }

    pub fn from_bytes_rgb(bytes: &[u8]) -> Image16x16Sequence {
        if bytes.len() == 0 {
            panic!("You cannot create an empty Image16x16Sequence. bytes.len() must not be equal 0")
        }

        if (bytes.len() % (16*16*3)) != 0 {
            panic!("bytes.len() must be a multiple of (16*16*3) - that is the size of a single frame")
        }

        Image16x16Sequence {
            bytes: Vec::from(bytes)
        }
    }

    pub fn get_frame_count(&self) -> u16 {
        self.bytes.len() as u16 / (16*16*3)
    }
}

impl Effect for Image16x16Sequence {
    fn process(&mut self, canvas: &mut [u8; 16*16*3], time: f32) {
        if time < 0.0 {
            panic!("time must not be negative")
        }

        fade_canvas(canvas, 255, 255, 255);

        let frame_idx = (time as usize) % self.get_frame_count() as usize;

        for i in 0..16*16 {
            let pixel_offset_image = (frame_idx * 16 * 16 + i) * 3;
            if self.bytes[pixel_offset_image + 0] == 0x00 && 
               self.bytes[pixel_offset_image + 0] == self.bytes[pixel_offset_image + 1] && 
               self.bytes[pixel_offset_image + 1] == self.bytes[pixel_offset_image + 2] {
                continue;
            }

            canvas[i * 3 + 0] = self.bytes[pixel_offset_image + 0];
            canvas[i * 3 + 1] = self.bytes[pixel_offset_image + 1];
            canvas[i * 3 + 2] = self.bytes[pixel_offset_image + 2];
        }
    }
}

pub struct GammaCorrection {
    r_gamma: f32,
    g_gamma: f32,
    b_gamma: f32,
}
impl GammaCorrection {
    pub fn from_value(gamma: f32) -> GammaCorrection {
        GammaCorrection {
            r_gamma: gamma,
            g_gamma: gamma,
            b_gamma: gamma,
        }
    }
    pub fn from_separate(r_gamma: f32, g_gamma: f32, b_gamma: f32) -> GammaCorrection {
        GammaCorrection {
            r_gamma,
            g_gamma,
            b_gamma,
        }
    }
}
impl Effect for GammaCorrection {
    fn process(&mut self, canvas: &mut [u8; 16*16*3], time: f32) {
        for i in 0..16*16 {
            canvas[i*3+0] = (255.0 * (canvas[i*3+0] as f32 / 255.0).powf(self.r_gamma)) as u8;
            canvas[i*3+1] = (255.0 * (canvas[i*3+1] as f32 / 255.0).powf(self.g_gamma)) as u8;
            canvas[i*3+2] = (255.0 * (canvas[i*3+2] as f32 / 255.0).powf(self.b_gamma)) as u8;
        }
    }
}

pub struct Snowflakes {
    gravity: f32, 
    snowflakes: Vec<(f32, f32)>,
    rng: rand::rngs::ThreadRng
}
impl Snowflakes {
    pub fn new(gravity: f32, count: usize) -> Snowflakes {
        let mut snowflakes: Vec<(f32, f32)> = Vec::with_capacity(count);
        let mut rng = rand::thread_rng();

        for _ in 0..count {
            snowflakes.push((rng.gen_range(0.0..16.0), rng.gen_range(0.0..16.0)));
        }

        Snowflakes {
            gravity,
            snowflakes,
            rng
        }
    }
}
impl Effect for Snowflakes {
    fn process(&mut self, canvas: &mut [u8; 16*16*3], time: f32) {
        for (i, (x, y)) in &mut self.snowflakes.iter_mut().enumerate() {
            set_pixel(canvas, *x as i32, *y as i32, 0xff, 0xff, 0xff);
            set_pixel(canvas, *x as i32+1, *y as i32, 0xff, 0xff, 0xff);
            set_pixel(canvas, *x as i32-1, *y as i32, 0xff, 0xff, 0xff);
            set_pixel(canvas, *x as i32, *y as i32+1, 0xff, 0xff, 0xff);
            set_pixel(canvas, *x as i32, *y as i32-1, 0xff, 0xff, 0xff);

            *y -= 4.0/16.0 * ((((time / 2.0) as i32 + i as i32)) % 2 + 1) as f32;

            if *y < 0.0 {
                *y = 15.999;

                *x += 4.0/16.0;

                if *x < 0.0 {
                    *x = 16.0 + *x;
                } else if *x >= 16.0 {
                    *x = *x - 16.0;
                }
            }
        } 
    }
}

pub struct FillCanvas {
    r: u8,
    g: u8,
    b: u8,
}
impl FillCanvas {
    pub fn new(r: u8, g: u8, b: u8) -> FillCanvas {
        FillCanvas {
            r, g, b
        }
    }
}
impl Effect for FillCanvas {
    fn process(&mut self, canvas: &mut [u8; 16*16*3], time: f32) {
        for i in 0..16*16 {
            canvas[i*3+0] = self.r;
            canvas[i*3+1] = self.g;
            canvas[i*3+2] = self.b;
        } 
    }
}

pub struct FlickeringStars {
    rng: rand::rngs::ThreadRng,
    fade_r: u8,
    fade_g: u8,
    fade_b: u8,
    color_r: u8,
    color_g: u8,
    color_b: u8,
}
impl FlickeringStars {
    pub fn new(color_r: u8, color_g: u8, color_b: u8, fade_r: u8, fade_g: u8, fade_b: u8) -> FlickeringStars {
        FlickeringStars {
            rng: rand::thread_rng(),
            fade_r,
            fade_g,
            fade_b,
            color_r,
            color_g,
            color_b
        }
    }
}
impl Effect for FlickeringStars {
    fn process(&mut self, canvas: &mut [u8; 16*16*3], time: f32) {
        fade_canvas(canvas, self.fade_r, self.fade_g, self.fade_b);
       
        let idx = self.rng.gen_range(0..(16*16));

        canvas[idx*3+0] = self.color_r;
        canvas[idx*3+1] = self.color_g;
        canvas[idx*3+2] = self.color_b;
    }
}


pub struct SlidingRainbow {

}
impl SlidingRainbow {
    pub fn new() -> SlidingRainbow {
        SlidingRainbow {

        }
    }
}
impl Effect for SlidingRainbow {
    fn process(&mut self, canvas: &mut [u8; 16*16*3], time: f32) {
        for y in 0..16 {
            for x in 0..16 {
                let h = (x as f32 / 16.0 * 360.0 + time * 60.0) % 360.0;
                let (r, g, b) = hsv_to_rgb(h, 1.0, 1.0);

                set_pixel(canvas, x, y, r, g, b);
            }
        }
    }
}

pub struct SlidingRotatingRainbow {

}
impl SlidingRotatingRainbow {
    pub fn new() -> SlidingRotatingRainbow {
        SlidingRotatingRainbow {

        }
    }
}
impl Effect for SlidingRotatingRainbow {
    fn process(&mut self, canvas: &mut [u8; 16*16*3], time: f32) {
        for y0 in 0..16 {
            for x0 in 0..16 {
                let rot_matrix = [
                    [time.cos(), -time.sin()],
                    [time.sin(),  time.cos()]
                ];

                let x= (rot_matrix[0][0] * x0 as f32 + rot_matrix[0][1] * y0 as f32) + 8.0;
                //let y = (rot_matrix[1][0] * x0 as f32 + rot_matrix[1][1] * y0 as f32) + 8.0;

                let h = (x as f32 / 16.0 * 360.0 + time * 60.0) % 360.0;
                let (r, g, b) = hsv_to_rgb(h, 1.0, 1.0);
                
                set_pixel(canvas, x0, y0, r, g, b);
            }
        }
    }
}


pub struct Pendulum {
    radius: f32,
}
impl Pendulum {
    pub fn new(radius: f32) -> Pendulum {
        Pendulum {
            radius
        }
    }
}
impl Effect for Pendulum {
    fn process(&mut self, canvas: &mut [u8; 16*16*3], time: f32) {
        let y0 = 15;
        let x0 = 7;

        let angle = ((time * 3.14159265).sin() * 40.0).to_radians();

        let y1 = y0 - (angle.cos() * self.radius) as i32;
        let x1 = (angle.sin() * self.radius) as i32 + x0;

        fade_canvas(canvas, 255,255,255);
        draw_line(canvas, x0, y0, x1, y1, 255, 255, 255);
        draw_filled_circle(canvas, x1 - 1, y1 - 1, 3, 255, 255, 255, false);
    }
}