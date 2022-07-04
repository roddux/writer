use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::time::Duration;

#[derive(Debug)]
enum RequestType {
    Get(String),
    Post(String),
    Invalid,
}

#[derive(PartialEq)]
enum Action {
    ShouldStop,
    Nothing,
}

#[allow(clippy::unused_io_amount)]
fn get_req_from_stream(stream: &mut TcpStream) -> Result<String, std::io::Error> {
    let mut buf: [u8; 4096] = [0; 4096];
    let mut data: Vec<u8> = Vec::new();

    loop {
        println!("reading...");
        let read = stream.read(&mut buf);
        if read.is_err() {
            match read {
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    println!("read timed out");
                    break;
                }
                _ => {
                    println!("error: read socket - {:?}", read);
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "read"));
                }
            }
        }
        let read = read.unwrap();

        if read == 0 {
            break;
        }
        data.extend(&buf[..read]);
    }

    println!(
        "read {} bytes:\n'{:?}'\n'{}'\n",
        data.len(),
        data,
        std::str::from_utf8(&data).unwrap()
    );

    Ok(String::from_utf8(data).unwrap())
}

fn parse_req(request: String) -> RequestType {
    let split: Vec<&str> = request.split(' ').collect();
    match split[0..3] {
        ["GET", path, _] => {
            /*println!("got GET req to '{}'", path);*/
            RequestType::Get(path.to_string())
        }
        ["POST", path, _] => {
            /*println!("got POST req to '{}'", path);*/
            RequestType::Post(path.to_string())
        }
        _ => {
            println!("unhandled: {:?}", split);
            RequestType::Invalid
        }
    }
}

#[allow(clippy::unused_io_amount)]
fn process_req(req: &RequestType, stream: &mut TcpStream) -> Action {
    let mut act = Action::Nothing;

    match req {
        RequestType::Invalid => {
            stream
                .write("HTTP/1.1 400 Bad Request\n\n".as_bytes())
                .unwrap();
        }
        _ => {
            /*println!("processing request");*/
            stream
                .write("HTTP/1.1 200 OK\n\nhello, world\n".as_bytes())
                .unwrap();
        }
    }

    match req {
        RequestType::Get(path) if path.eq("/abc") => {
            stream.write("d, e, f!\n".as_bytes()).unwrap();
        }
        RequestType::Post(path) if path.eq("/bye") => {
            stream.write("bye!\n".as_bytes()).unwrap();
            act = Action::ShouldStop;
        }
        _ => {
            stream.write("no cookie for you\n".as_bytes()).unwrap();
        }
    }
    stream.shutdown(Shutdown::Both).ok();
    act
}

fn handle_connection(stream: &mut TcpStream) -> Action {
    stream.set_read_timeout(Some(Duration::new(1, 0))).unwrap();
    stream.set_nonblocking(false).unwrap();

    let req_str = get_req_from_stream(stream);
    if req_str.is_err() {
        return Action::Nothing;
    };

    let req = parse_req(req_str.unwrap());
    process_req(&req, stream)
}

fn main() {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:9999").unwrap();
    for (req_count, s) in listener.incoming().enumerate() {
        let act = handle_connection(&mut s.unwrap());
        println!("req # {}", req_count);
        if act == Action::ShouldStop {
            println!("got call to stop");
            return;
        }
    }
}
