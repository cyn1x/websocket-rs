use base_rs as base;
use event::Event;
use sha_rs as sha;
use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

pub mod event;

pub fn init() -> Arc<Event> {
    let event = Event::instance();
    event
}

pub fn serve() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        // Connection established
        // TODO: Logging

        thread::spawn(|| {
            accept_connection(stream);
        });
    }
}

fn generate_accept_key(key: &str) -> String {
    let magic_string = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let combined = format!("{}{}", key, magic_string);
    let hash = sha::sha1(combined.as_bytes());
    base::base64_encode(&hash)
}

fn accept_connection(mut stream: TcpStream) {
    // Determine size of the incoming stream in bytes
    let mut peek_buffer: [u8; 1024] = [0; 1024];
    let incoming_bytes: usize = stream
        .peek(&mut peek_buffer)
        .expect("Error peeking incoming bytes");
    let mut buffer = vec![0; incoming_bytes];

    // Read the client's handshake request
    let bytes_read = stream.read(&mut buffer).expect("Failed to read stream");
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);

    // Extract Sec-WebSocket-Key
    let key_line = request
        .lines()
        .find(|line| line.starts_with("Sec-WebSocket-Key:"))
        .expect("Invalid WebSocket handshake request");

    let key = key_line
        .split(": ")
        .nth(1)
        .expect("Malformed Sec-WebSocket-Key");

    let accept_key = generate_accept_key(key.trim());

    // Send handshake response
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {}\r\n\r\n",
        accept_key
    );

    stream
        .write_all(response.as_bytes())
        .expect("Failed to write handshake response");
    // WebSocket handshake completed!
    // TODO: Logging

    handle_connection(stream);
}

fn handle_connection(mut stream: TcpStream) {
    let event = Event::instance();

    loop {
        let mut frame_header = [0; 2];
        if stream.read_exact(&mut frame_header).is_err() {
            // Client disconnected
            // TODO: Logging
            break;
        }

        let payload_len = (frame_header[1] & 0x7F) as usize;
        let mut mask_key = [0; 4];
        stream
            .read_exact(&mut mask_key)
            .expect("Failed to read mask key");

        let mut payload = vec![0; payload_len];
        stream
            .read_exact(&mut payload)
            .expect("Failed to read payload");

        // Unmask payload
        for i in 0..payload_len {
            payload[i] ^= mask_key[i % 4];
        }

        let recv_msg = String::from_utf8_lossy(&payload);

        // Send client message to WebSocket implementer
        event.send_msg(recv_msg.to_string());
        // Wait for implementer to respond to the client
        let send_msg = event.recv_msg();

        // Echo response (frame format: FIN=1, Text Frame=0x1, no masking)
        let mut response_frame = vec![0x81, send_msg.len() as u8];
        response_frame.extend_from_slice(&send_msg.as_bytes());
        stream
            .write_all(&response_frame)
            .expect("Failed to send response");
    }
}
