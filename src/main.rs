use coremidi::{Client, Destination, Destinations, EventBuffer, Protocol};
use std::{env, u32};
use std::net::UdpSocket;
use std::str::from_utf8;

static NOTES: [char; 12] = [
    'C',
    '_',
    'D',
    '_',
    'E',
    'F',
    '_',
    'G',
    '_',
    'A',
    '_',
    'B'
];

/**
 * First char: O(on)/X(off)
 * Second char: Note [A,B,C,D,E,F,G]
 * Third char: # or b, increment or decrement by 1
 * Fourth char: octave [0,1,2,3,4,5,6,7,8]
 * Fifth-sixth: velocity in hex
 */
fn generate_note (text: [char;6]) -> EventBuffer {
    let mut note_value: u32 = 0x20800000;
    if text[0] == 'O' {
        note_value += 0x00100000;
    }
    let base_note_number: u32 = NOTES.iter().position(|v| {
        v == &text[1]
    }).unwrap().try_into().unwrap();
    if text [2] == '#' {
        note_value += 0x00000100;
    }
    let base_note_octave: u32 = text[3].to_digit(10).unwrap().try_into().unwrap();
    note_value += 0x00000100 * base_note_number + 0x00000c00 * (base_note_octave + 1);
    note_value += 0x00000010 * text[4].to_digit(16).unwrap();
    note_value += 0x00000001 * text[5].to_digit(16).unwrap();
    return EventBuffer::new(Protocol::Midi10).with_packet(0, &[note_value]);
}

fn main() {
    let destination_index = get_destination_index();
    println!("Destination index: {}", destination_index);

    let destination = Destination::from_index(destination_index).unwrap();
    println!(
        "Destination display name: {}",
        destination.display_name().unwrap()
    );

    let client = Client::new("Example Client").unwrap();
    let output_port = client.output_port("Example Port").unwrap();

    let socket = UdpSocket::bind("0.0.0.0:3400").expect("couldn't bind to address");

    let mut buf: [u8; 1024] = [0; 1024];

    loop {
        socket.recv(&mut buf).unwrap();
        let mut strs = from_utf8(&buf).unwrap().split(".");
        let mut curr:[char; 6] = strs.next().unwrap().chars().take(6).collect::<Vec<char>>().try_into().unwrap();
        while curr.len() > 0 && curr[0] == 'O' {
            output_port.send(&destination, generate_note(curr)).unwrap();
            curr = strs.next().unwrap().chars().take(6).collect::<Vec<char>>().try_into().unwrap();
        }
        buf.fill(0);
    }
}

fn get_destination_index() -> usize {
    let mut args_iter = env::args();
    let tool_name = args_iter
        .next()
        .and_then(|path| {
            path.split(std::path::MAIN_SEPARATOR)
                .last()
                .map(|v| v.to_string())
        })
        .unwrap_or_else(|| "send".to_string());

    match args_iter.next() {
        Some(arg) => match arg.parse::<usize>() {
            Ok(index) => {
                if index >= Destinations::count() {
                    println!("Destination index out of range: {}", index);
                    std::process::exit(-1);
                }
                index
            }
            Err(_) => {
                println!("Wrong destination index: {}", arg);
                std::process::exit(-1);
            }
        },
        None => {
            println!("Usage: {} <destination-index>", tool_name);
            println!();
            println!("Available Destinations:");
            print_destinations();
            std::process::exit(-1);
        }
    }
}

fn print_destinations() {
    for (i, destination) in Destinations.into_iter().enumerate() {
        if let Some(display_name) = destination.display_name() {
            println!("[{}] {}", i, display_name)
        }
    }
}
