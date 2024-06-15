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