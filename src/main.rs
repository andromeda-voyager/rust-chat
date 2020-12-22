use std::env;
use std::io::prelude::*;
use std::io::stdin;
use std::io::stdout;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::{thread, time};

const SLEEP_LENGTH: time::Duration = time::Duration::from_millis(100);

fn stream_io_thread(mut stream: TcpStream, other_usr: String) -> Sender<std::string::String> {
    let mut reader = BufReader::new(stream.try_clone().expect("failed to clone stream."));
    let (input_sender, input_receiver) = channel::<String>();
    thread::spawn(move || loop {
        let mut line = String::new();
        if let Ok(_err) = reader.read_line(&mut line) {
            if line != "" {
                println!("{} : {}", other_usr, line.trim());
            }
        }
        match input_receiver.try_recv() {
            Ok(usr_input) => {
                match stream.write(usr_input.as_bytes()) {
                    Err(_e) => return, // end thread
                    Ok(f) => f,
                };
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
        thread::sleep(SLEEP_LENGTH);
    });
    return input_sender;
}

fn chat(stream: TcpStream, usr: &str) {
    stream
        .set_nonblocking(true)
        .expect("set_nonblocking call failed");
    let input_sender = stream_io_thread(stream, usr.to_string());
    loop {
        thread::sleep(SLEEP_LENGTH);
        let mut usr_input = String::new();
        stdin()
            .read_line(&mut usr_input)
            .expect("Failed to read from stdin.");
        match input_sender.send(usr_input) {
            Err(_e) => return,
            Ok(f) => f,
        };
    }
}

fn connect(usr_name: String) -> std::io::Result<()> {
    let mut stream = TcpStream::connect("localhost:8080")?;
    stream
        .write(usr_name.as_bytes())
        .expect("Unable to write user name to stream.");
    let mut line = [0; 128];
    stream.read(&mut line).expect("Unable to read stream.");
    chat(stream, str::from_utf8(&line).unwrap());
    Ok(())
}

fn listen(usr_name: String) -> std::io::Result<()> {
    let listener = TcpListener::bind("localhost:8080")?;

    // accept connections and process them serially
    for tcp_result in listener.incoming() {
        let mut stream = tcp_result?;
        stream
            .write(usr_name.as_bytes())
            .expect("Unable to write user name to stream.");
        let mut line = [0; 128];
        stream.read(&mut line).expect("Unable to read stream.");
        chat(stream, str::from_utf8(&line).unwrap());
    }
    Ok(())
}

fn get_usr_name() -> String {
    print!("Enter your name: ");
    stdout().flush().expect("unable to flush stdout");
    let mut usr_name = String::new();
    stdin()
        .read_line(&mut usr_name)
        .expect("Unable to read stdin.");
    return usr_name.trim().to_string();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        if args[1] == "server" {
            println!("I'm a server");
            if let Err(_err) = listen(get_usr_name()) {
                println!("an error occured in listen()")
            }
        } else if args[1] == "client" {
            println!("I'm a client");
            match connect(get_usr_name()) {
                Err(_e) => println!("There is no server listening for connections."),
                Ok(_f) => println!("Connection ended."),
            };
        }
    }
}
