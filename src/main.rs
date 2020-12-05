use std::env;
use std::io::prelude::*;
use std::io::stdin;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::{thread, time};

const SLEEP_LENGTH: time::Duration = time::Duration::from_millis(100);

fn stream_io_thread(mut stream: TcpStream) -> Sender<std::string::String> {
    let mut reader = BufReader::new(stream.try_clone().expect("failed to clone stream."));
    let (input_sender, input_receiver) = channel::<String>();

    thread::spawn(move || loop {
        let mut line = String::new();
        if let Ok(_err) = reader.read_line(&mut line) {
            if line != "" {
                println!("{}", line.trim());
            }
        }
        match input_receiver.try_recv() {
            Ok(usr_input) => {
                stream
                    .write(usr_input.as_bytes())
                    .expect("Failed to write to stream");
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
        thread::sleep(SLEEP_LENGTH);
    });
    return input_sender;
}

fn chat(stream: TcpStream) {
    stream
        .set_nonblocking(true)
        .expect("set_nonblocking call failed");
    let input_sender = stream_io_thread(stream);
    loop {
        thread::sleep(SLEEP_LENGTH);
        let mut usr_input = String::new();
        stdin()
            .read_line(&mut usr_input)
            .expect("Failed to read from stdin.");
        input_sender
            .send(usr_input)
            .expect("Failed to send user input.")
    }
}

fn connect() {
    let stream = TcpStream::connect("localhost:8080").expect("Couldn't connect to the server.");
    chat(stream);
}

fn listen() -> std::io::Result<()> {
    let listener = TcpListener::bind("localhost:8080").expect("Couldn't connect to the server.");

    // accept connections and process them serially
    for stream in listener.incoming() {
        chat(stream?);
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        if args[1] == "server" {
            println!("I'm a server");
            if let Err(_err) = listen() {
                println!("an error occured in listen()")
            }
        } else if args[1] == "client" {
            println!("I'm a client");
            connect();
        }
    }
}
