use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use std::string::String;
use json::object;
use ws::{listen, Message};
use ws::{connect, CloseCode};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode;
    let file_path;
    if args.len() >= 2 {
        mode = &args[1];
    } else {
        println!("No mode specified");
        exit(0)
    }

    if mode == "receive" {
        if args.len() >= 3 {
            receive(&args[2])
        } else {
            receive("6780")
        }
    } else if mode == "send" {
        if args.len() >= 4  {
            file_path = &args[2];
            let addr = &args[3];
            println!("Reading file {}", file_path);

            let contents = fs::read(file_path)
                .expect("Should have been able to read the file");
            println!("Size: {} Byte(s)", contents.len());
            let payload = base64::encode(contents);
            let file_name = Path::new(file_path).file_name().unwrap().to_str().unwrap();
            let json_data = object!{
                "name": file_name,
                "payload": payload,
            };

            send(json_data.dump(), addr)
        }
    }
}

fn send(content: String, addr: &str) {
    let str_addr;
    if !addr.chars().any(|c| matches!(c, ':')) {
        str_addr = addr.to_string() + ":6780";
    } else {
        str_addr = addr.to_string();
    }
    println!("Target: {}", &str_addr);
    if let Err(error) = connect("ws://".to_owned() + &str_addr, |out| {

        if out.send(&*content).is_err() {
            println!("Websocket cannot initialize message query!")
        } else {
            println!("Client transmitting file")
        }

        move |msg| {
            println!("Client received '{}'. ", msg);
            out.close(CloseCode::Normal)
        }
    }) {
        println!("Failed to create WebSocket due to: {:?}", error);
    }
}

fn receive(port: &str) {
    println!("Opening server on {}", "0.0.0.0:".to_owned() + port);
    if let Err(error) = listen("0.0.0.0:".to_owned() + port, |out| {
        move |msg: Message| {
            let json_data = json::parse(&String::from_utf8_lossy(&msg.into_data()).to_string());
            let data = json_data.unwrap();
            let mut file = fs::File::create(data["name"].to_string()).expect("create failed");
            let payload = base64::decode(&data["payload"].to_string()).unwrap();
            println!("Size: {} Byte(s)", payload.len());
            file.write_all(&payload).expect("Write failed");
            println!("Data written to file" );

            out.send("OK")
        }
    }) {
        println!("Failed to create websocket: {:?}", error);
    }
}