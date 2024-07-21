use std::io::prelude::*;
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::str::FromStr;
use std::time::{Duration, Instant};

mod effects;
use effects::Effect;

const DATA_TYPE_FULL: u8 = 0x01;
const DATA_TYPE_HALF: u8 = 0x02;

const WAIT_TIME: f32 = 0.0333;
const USE_FULL_PACKETS: bool = true;

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
            ::core::slice::from_raw_parts((self as *const Packet) as *const u8, 1 + self.get_data_size()) 
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
            let r = canvas[idx * 3 + 0] >> 4; 
            let g = canvas[idx * 3 + 1] >> 4; 
            let b = canvas[idx * 3 + 2] >> 4;

            // Pack full canvas to the half_canvas, offset between individual pixels is 3/2 bytes (12 bits)
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

fn send_packet(packet: Packet, stream: &mut TcpStream) {
    let mut ack_buffer = [0u8; 8];

    stream.write_all(packet.to_bytes()).unwrap(); // Send the data packet
    stream.read(&mut ack_buffer).unwrap(); // Wait until the 'ACK' message is received and read it

    assert!(ack_buffer == [b'A', b'C', b'K', 0, 0, 0, 0, 0]);
}

fn construct_local_ip(last_num: u8) -> String {
    let mut str = String::from("192.168.1.");
    str.push_str(last_num.to_string().as_str());
    str.push_str(":4242");
    str
}

fn find_pico() -> Result<TcpStream, String> {
    for i in 2u8..=255u8 {
        let ip = construct_local_ip(i);
        let addr = SocketAddr::from_str(ip.as_str()).unwrap();

        match TcpStream::connect_timeout(&addr, Duration::from_millis(1000)) {
          Ok(stream) => {
            println!("found pico on {}!", ip);

            return Ok(stream);
          },
          Err(err) => {
            println!("pico not found on {}, {:?}", ip, err);
          } 
        } 
    }

    Err(String::from("failed to find pico!"))
}

fn main() {
    //let mut stream = TcpStream::connect("192.168.1.9:4242").unwrap();
    
    let mut stream = find_pico().unwrap();
    stream.set_read_timeout(Option::from(Duration::from_millis(8000))).unwrap();

    let mut canvas = [0u8; 16*16*3];
    let mut time = 0.0f32;

    let mut image_sequence_effect = effects::Image16x16Sequence::from_gif("data/example.gif");
    let mut meteors_effect = effects::Meteors::new();
    let mut orbs_effect = effects::Orbs::new();

    loop {
        //meteors_effect.process(&mut canvas, time);
        //orbs_effect.process(&mut canvas, time);
        image_sequence_effect.process(&mut canvas, time * 8.0);

        let packet = if USE_FULL_PACKETS { 
            make_full(&mut canvas)
        } else {
            make_half(&mut canvas)
        };
        
        let send_start = Instant::now();

        send_packet(packet, &mut stream);

        let send_elapsed = send_start.elapsed().as_secs_f32();
    
        println!("SEND -> ACK elapsed: {}ms", send_elapsed * 1000.0);
        
        if send_elapsed < WAIT_TIME {
            // TODO: Make more accurate frame pacing.
            // It should be based on the total "frame" time,
            // not only on the SEND -> ACK elapsed time.
        
            std::thread::sleep(Duration::from_secs_f32(WAIT_TIME - send_elapsed));
        } else {
            println!("frame timeout");
        }

        time += WAIT_TIME;
    }
}