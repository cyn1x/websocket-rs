# websocket-rs 

Basic implementation of a WebSocket server in Rust.

## Installation

Add the following in Cargo.toml.

```
[dependencies]
websocket-rs = { git = "https://github.com/cyn1x/websocket-rs" }
```

## Usage

```rs
use std::thread;
use websocket_rs as websocket;

fn main() {
    // Message event handler
    let event = websocket::init();

    thread::spawn(|| {
        websocket::serve();
    });

    let thread_join_handle = thread::spawn(move || {
        loop {
            // Waits auntil a message is received
            let recv = event.recv_msg();

            let send = get_msg(); // Client function that responds to received message
            event.send_msg(send.to_string());
        }
    });

    let _res = thread_join_handle.join();
}
```
