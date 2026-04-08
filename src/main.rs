use std::io::prelude::*;
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::time::{Duration, Instant};

mod packet;
mod effects;
use effects::Effect;

const WAIT_TIME: f32 = 0.02;
const USE_FULL_PACKETS: bool = true;

const PICO_PORT: u16 = 4242;
const PICO_TIMEOUT_S: u32 = 8;

fn any_as_bytes<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>()) 
    }
}

fn send_any<T: Sized>(p: &T, stream: &mut TcpStream) {
    let mut ack_buffer = [0u8; 8];

    stream.write_all(any_as_bytes(p)).unwrap(); // Send the data packet
    stream.read(&mut ack_buffer).unwrap(); // Wait until the 'ACK' message is received and read it

    assert!(ack_buffer == [b'A', b'C', b'K', 0, 0, 0, 0, 0]);
}

fn find_pico() -> Result<TcpStream, String> {
    for i in 6u8..=255u8 {
        let addr = SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, i), PICO_PORT);
 
        match TcpStream::connect_timeout(&addr.into(), Duration::from_millis(1500)) {
          Ok(stream) => {
            println!("found pico on {}!", addr.to_string());

            return Ok(stream);
          },
          Err(err) => {
            println!("pico not found on {}, {:?}", addr.to_string(), err);
          } 
        } 
    }

    Err(String::from("failed to find pico!"))
}

fn main() {
    let mut stream = find_pico().unwrap();
    stream.set_read_timeout(Option::from(Duration::from_secs(PICO_TIMEOUT_S as u64))).unwrap();

    let mut canvas = [0u8; 16*16*3];
    let mut render_target = [0u8; 16*16*3];
    let mut time = 0.0f32;

    let mut hearts_effect = effects::Hearts::new();
    let mut image_sequence_effect_tree = effects::Image16x16Sequence::from_image("data/tree.png");
    let mut image_sequence_effect_guy = effects::Image16x16Sequence::from_gif("data/example.gif");
    let mut snowflakes_effect = effects::Snowflakes::new(9.81, 5);
    let mut meteors_effect = effects::Meteors::new();
    let mut orbs_effect = effects::Orbs::new();
    let mut fill_bg_effect = effects::FillCanvas::new(110, 178, 210);
    let mut flickering_stars_effect = effects::FlickeringStars::new(255, 180, 255, 4, 3, 3);
    let mut sliding_rainbow_effect = effects::SlidingRainbow::new();
    let mut sliding_rotating_rainbow_effect = effects::SlidingRotatingRainbow::new();
    let mut pendulum_effect = effects::Pendulum::new(14.0);

    let mut gamma_correction = effects::GammaCorrection::from_value(2.2);

    let mut last_frame_timestamp = Instant::now();
    /*
    let start_flash_write = Instant::now();

    let frame_count = 128;//image_sequence_effect.get_frame_count();
    for i in 0..frame_count {
        fill_bg_effect.process(&mut canvas, 0.0);
        snowflakes_effect.process(&mut canvas, time);
        image_sequence_effect.process(&mut canvas, 0.0);
        gamma_correction.process(&mut canvas, 0.0);

        send_any(&packet::WriteFlash::new(i as u16, &canvas), &mut stream);
        println!("Sent frame {i}");
    }

    send_any(&packet::PlayFlash::new(0, frame_count - 1, 50), &mut stream);

    let elapsed_flash_write = start_flash_write.elapsed().as_secs_f32();

    println!("Flash write took: {}ms", elapsed_flash_write*1000.0);
    
    std::thread::sleep(Duration::from_millis(4000));
 */
    loop {
        // Render loop start
            let ddd = (time / 4.0) as i32 % 7;
            match ddd {
                0 => orbs_effect.process(&mut canvas, time),
                1 => hearts_effect.process(&mut canvas, [3, 5, 7, 5][((time*2.0) as usize) % 4] as f32),
                2 => meteors_effect.process(&mut canvas, time),
                3 => image_sequence_effect_guy.process(&mut canvas, time*4.0-3.0),
                4 => flickering_stars_effect.process(&mut canvas, time),
                5 => sliding_rotating_rainbow_effect.process(&mut canvas, time),
                6 => pendulum_effect.process(&mut canvas, time),
                _ => {}
            }

            //hearts_effect.process(&mut canvas, [3, 5, 7, 5][((time*2.0) as usize) % 4] as f32);
            //orbs_effect.process(&mut canvas, time);
            //meteors_effect.process(&mut canvas, time);
            //image_sequence_effect_guy.process(&mut canvas, time * 1.0);
            //flickering_stars_effect.process(&mut canvas, time);
            //sliding_rotating_rainbow_effect.process(&mut canvas, time);
            //pendulum_effect.process(&mut canvas, time);

            render_target = canvas.clone();
            gamma_correction.process(&mut render_target, time);
        // Render loop end

        let now = Instant::now();
        let frametime_elapsed = (now - last_frame_timestamp).as_secs_f32();

        // Send new frames at WAIT_TIME seconds intervals.
        if frametime_elapsed < WAIT_TIME {
            std::thread::sleep(Duration::from_secs_f32(WAIT_TIME - frametime_elapsed));
        }

        time += WAIT_TIME;
        last_frame_timestamp = now;

        let send_start = Instant::now();

        if USE_FULL_PACKETS { 
            send_any(&packet::Full::new(&mut render_target), &mut stream);
        } else {
            send_any(&packet::Half::new(&mut render_target), &mut stream);
        };

        let send_elapsed = send_start.elapsed().as_secs_f32();
    
        println!("SEND -> ACK elapsed: {}ms", send_elapsed * 1000.0);
    }
    
}