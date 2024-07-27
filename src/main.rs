use std::io::prelude::*;
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::time::{Duration, Instant};

mod packet;
mod effects;
use effects::Effect;

const WAIT_TIME: f32 = 0.0333;
const USE_FULL_PACKETS: bool = false;

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
    for i in 2u8..=255u8 {
        let addr = SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, i), PICO_PORT);
 
        match TcpStream::connect_timeout(&addr.into(), Duration::from_millis(1000)) {
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
    let mut time = 0.0f32;

    let mut image_sequence_effect = effects::Image16x16Sequence::from_gif("data/example.gif");
    let mut meteors_effect = effects::Meteors::new();
    let mut orbs_effect = effects::Orbs::new();
    
    let mut last_frame_timestamp = Instant::now();

    let start_flash_write = Instant::now();

    for i in 0..(image_sequence_effect.get_frame_count()) {
        image_sequence_effect.process(&mut canvas, i as f32);
    
        send_any(&packet::WriteFlash::new(i as u16, &canvas), &mut stream);
        println!("Sent frame {i}");
    }
    
    send_any(&packet::PlayFlash::new(0, image_sequence_effect.get_frame_count() - 1, 100), &mut stream);
    
    let elapsed_flash_write = start_flash_write.elapsed().as_secs_f32();
    
    println!("Flash write took: {}ms", elapsed_flash_write*1000.0);

    std::thread::sleep(Duration::from_millis(4000));

    loop {
        //meteors_effect.process(&mut canvas, time);
        //orbs_effect.process(&mut canvas, time);
        image_sequence_effect.process(&mut canvas, time * 1.0);

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
            send_any(&packet::Full::new(&mut canvas), &mut stream);
        } else {
            send_any(&packet::Half::new(&mut canvas), &mut stream);
        };

        let send_elapsed = send_start.elapsed().as_secs_f32();
    
        println!("SEND -> ACK elapsed: {}ms", send_elapsed * 1000.0);
    }
}