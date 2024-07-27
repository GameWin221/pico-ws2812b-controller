use std::{fs::File, io::BufReader};

use rand::Rng;

fn set_pixel(canvas: &mut [u8; 16*16*3], x: i32, y: i32, r: u8, g: u8, b: u8) {
    if y < 0 || y >= 16 || x < 0 || x >= 16 {
        return;
    }
    
    let idx = y as usize * 16 + x as usize;

    canvas[idx * 3 + 0] = r;
    canvas[idx * 3 + 1] = g;
    canvas[idx * 3 + 2] = b;
}

fn draw_circle(canvas: &mut [u8; 16*16*3], time: f32, shift: f32, distance: f32, r: u8, g: u8, b: u8) {
    let x0 = (((time + shift).cos() * 0.5 + 0.5) * (15.0 - distance * 2.0) + 0.5 + distance) as i32;
    let y0 = (((time + shift).sin() * 0.5 + 0.5) * (15.0 - distance * 2.0) + 0.5 + distance) as i32;

    set_pixel(canvas, x0, y0, r, g, b);
    set_pixel(canvas, x0, y0+1, r, g, b);
    set_pixel(canvas, x0, y0-1, r, g, b);
    set_pixel(canvas, x0+1, y0, r, g, b);
    set_pixel(canvas, x0-1, y0, r, g, b);
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

        draw_circle(canvas, time * 3.14159262      , 3.14159262 * 0.5, 1.0, 0x00, 0xff, 0xff);
        draw_circle(canvas, time * 3.14159262      , 3.14159262 * 1.5, 1.0, 0xff, 0x00, 0xff);
        draw_circle(canvas, time * 3.14159262 * 2.0, 0.0             , 5.0, 0xff, 0x00, 0xaa);
        draw_circle(canvas, time * 3.14159262 * 2.0, 3.14159262      , 5.0, 0xaa, 0x00, 0xff);
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

        let frame_idx = (time as usize) % self.get_frame_count() as usize;

        for i in 0..16*16 {
            let pixel_offset_image = (frame_idx * 16 * 16 + i) * 3;

            canvas[i * 3 + 0] = self.bytes[pixel_offset_image + 0];
            canvas[i * 3 + 1] = self.bytes[pixel_offset_image + 1];
            canvas[i * 3 + 2] = self.bytes[pixel_offset_image + 2];
        }
    }
}