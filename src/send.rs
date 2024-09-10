use std::net::UdpSocket;
use std::env;

fn main () {
    let socket = UdpSocket::bind ("127.0.0.1:3399").unwrap ();

    let cmd = get_command();

    socket.send_to(cmd.as_bytes(), "127.0.0.1:3400").unwrap();
}

fn get_command() -> String {
    let mut args_iter = env::args();
    let tool_name = args_iter.next().unwrap();
    println!("tool={}", tool_name);
    return args_iter.next().unwrap();
}
