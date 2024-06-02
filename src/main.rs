use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Duration;

use rand::Rng;

const DATA_TYPE_FULL: u8 = 0x01;
const DATA_TYPE_HALF: u8 = 0x02;

const WAIT_TIME: f32 = 0.0333;

// Enter the correct IP of your Raspberry Pi Pico W.
// You can check it either by looking at your router's admin page or
// by looking at the debug printf messages coming from your Pico via USB
// during the start-up sequence.

const PICO_W_ADDR: &str = "192.168.1.14:4242";

#[repr(C)]
struct Packet {
    data_type: u8,
    data: [u8; 1023]
}

impl Packet {
    fn from_data(data_type: u8, data: &[u8]) -> Packet {
        assert!(data.len() <= 1023);
        
        let mut packet = Packet{
            data_type,
            data: [0u8; 1023]
        };

        for i in 0..data.len() {
            packet.data[i] = data[i];
        }

        packet
    }

    fn get_data_size(&self) -> usize {
        if self.data_type == DATA_TYPE_FULL {
            16*16*3
        } else if self.data_type == DATA_TYPE_HALF {
            16*16*3/2
        } else {
            panic!("invalid packet data_type!")
        }
    }

    fn to_bytes(&self) -> &[u8] {
        unsafe {
            ::core::slice::from_raw_parts(
                (self as *const Packet) as *const u8,
                1 + self.get_data_size(),
            ) 
        }
    }
}

fn make_full(canvas: &[u8; 16*16*3]) -> Packet {
    Packet::from_data(DATA_TYPE_FULL, canvas)
}

// No visible difference for bright colors (top 4 bits), (bottom 4 bits are lost)
fn make_half(canvas: &[u8; 16*16*3]) -> Packet {
    let mut half_canvas = [0u8; 16*16*3/2];
    
    for y in 0..16 {
        for x in 0..16 {
            let idx = y as usize * 16 + x as usize;

            // Trim to 4 bits
            let r = canvas[idx * 3 + 0] / 16; 
            let g = canvas[idx * 3 + 1] / 16; 
            let b = canvas[idx * 3 + 2] / 16;

            // Pack full canvas to the half_canvas
            if idx % 2 == 0 {
                half_canvas[idx * 3 / 2 + 0] = (half_canvas[idx * 3 / 2 + 0] & 0x0f) | (r << 4);
                half_canvas[idx * 3 / 2 + 0] = (half_canvas[idx * 3 / 2 + 0] & 0xf0) | (g);
                half_canvas[idx * 3 / 2 + 1] = (half_canvas[idx * 3 / 2 + 1] & 0x0f) | (b << 4);
            } else {
                half_canvas[idx * 3 / 2 + 0] = (half_canvas[idx * 3 / 2 + 0] & 0xf0) | (r);
                half_canvas[idx * 3 / 2 + 1] = (half_canvas[idx * 3 / 2 + 1] & 0x0f) | (g << 4);
                half_canvas[idx * 3 / 2 + 1] = (half_canvas[idx * 3 / 2 + 1] & 0xf0) | (b);
            } 
        }
    }
    
    Packet::from_data(DATA_TYPE_HALF, &half_canvas)
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

fn send_packet(packet: Packet, stream: &mut TcpStream) {
    let mut ack_buffer = [0u8; 8];

    let start = std::time::Instant::now();

    stream.write_all(packet.to_bytes()).unwrap(); // Send the data packet
    stream.read(&mut ack_buffer).unwrap(); // Wait until the 'ACK' message is received and read it

    assert!(ack_buffer == [b'A', b'C', b'K', 0, 0, 0, 0, 0]);

    let elapsed = start.elapsed().as_secs_f32();
    
    println!("SEND => ACK elapsed: {}ms", elapsed * 1000.0);

    if WAIT_TIME > elapsed {
        // Make more accurate frame pacing
        // It should be based on the total "frame" time

        std::thread::sleep(Duration::from_secs_f32(WAIT_TIME - elapsed));
    } else {
        println!("frame timeout");
    }
}

fn main() {
    let mut stream = TcpStream::connect(PICO_W_ADDR).unwrap();
    stream.set_read_timeout(Option::from(Duration::from_millis(4000))).unwrap();

    let mut canvas = [0u8; 16*16*3];
    let mut time = 0.0f32;

    let mut rng = rand::thread_rng();

    let mut particles = [(0.0, 0.0); 12];

    for i in 0..particles.len() {
        particles[i].0 = rng.gen_range(0.0..16.0);
        particles[i].1 = rng.gen_range(0.0..16.0);
    }

    loop {
        fade_canvas(&mut canvas, 12, 24, 12);

        // "Orbiting circles" example
        //draw_circle(&mut canvas, time * 3.14159262      , 3.14159262 * 0.5, 1.0, 0x00, 0xff, 0xff);
        //draw_circle(&mut canvas, time * 3.14159262      , 3.14159262 * 1.5, 1.0, 0xff, 0x00, 0xff);
        //draw_circle(&mut canvas, time * 3.14159262 * 2.0, 0.0             , 5.0, 0xff, 0x00, 0xaa);
        //draw_circle(&mut canvas, time * 3.14159262 * 2.0, 3.14159262      , 5.0, 0xaa, 0x00, 0xff);

        // "Asteroids" example
        for (x, y) in &mut particles {
            set_pixel(&mut canvas, *x as i32, *y as i32, 0xff, 0xa0, 0x00);

            *y -= rng.gen_range(0.2..=1.0);

            if *y < 0.0 {
                *y = 15.999;

                *x -= rng.gen_range(-2.0..=2.0);

                if *x < 0.0 {
                    *x = 16.0 + *x;
                } else if *x >= 16.0 {
                    *x = *x - 16.0;
                }
            }
        } 

        // Uncomment one 
        let pack = make_full(&mut canvas);
        //let pack = make_half(&mut canvas);
        
        send_packet(pack, &mut stream);

        time += WAIT_TIME;
    }
}