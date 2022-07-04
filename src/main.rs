use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::time::Duration;

/*
TODO
- buffered reads/writes on stream
- less syscalls; keep a local hashmap of post data and sync it to disc periodically
- split up handle_connection
- remove .unwrap()s and handle errors nicer
*/

/*
#[derive(Debug)]
enum RequestType {
    Get,
    Post,
    Invalid,
}

#[derive(Debug)]
struct Request {
    request_type: RequestType,
    path: String,
    data: Vec<u8>,
}
*/

fn die(msg: &str, stream: &mut TcpStream) {
    println!("sent '{}'", msg);
    stream.write(msg.as_bytes()).ok();
    stream.shutdown(Shutdown::Both).ok();
}

const MAX_DATA_LEN: usize = 16384; // 16Kib

// we only handle GET. format is /save?data=<stuff>
fn handle_connection(stream: &mut TcpStream) {
    stream
        .set_read_timeout(Some(Duration::new(0, 100 * 1000000))) // 100ms
        .ok();
    let mut buf = [0_u8; 4096]; // 4Kib on stack
    let mut data = Vec::<u8>::new(); // heap to store the data, can grow to 16Kib
    loop {
        // read all headers
        if std::str::from_utf8(&data).unwrap().contains("\r\n\r\n") {
            // println!("got message");
            break;
        }

        let bytes_read = stream.read(&mut buf);
        if bytes_read.is_err() {
            // println!("bytes_read returned err, breaking");
            break;
        }
        let bytes_read = bytes_read.unwrap();
        if bytes_read == 0 {
            // println!("bytes_read == 0, breaking");
            break;
        }
        data.extend_from_slice(&buf[..bytes_read]); // grow heap buffer and add what we read to stack
        if data.len() >= MAX_DATA_LEN {
            die("HTTP/1.1 400 Bad Request\nrequest too long", stream);
        }
        // println!("extended. new data:\n{:?}\n{}", data, std::str::from_utf8(&data).unwrap());
    }

    let headers = std::str::from_utf8(&data).unwrap();
    let headers: Vec<&str> = headers.split("\r\n").collect();

    if !headers[0].starts_with("GET") {
        die("HTTP/1.1 400 Bad Request\nbad method", stream);
    }
/*
    for header in &headers {
        println!("got header '{}'", header);
    }
*/
    let dat: Vec<&str> = headers[0].split(' ').collect();
/*
    for part in &dat {
        println!("got part '{}'", part);
    }
*/

    if dat[1] == "/" {
        let index_page = include_bytes!("../index.html");
        // let index_page = std::fs::read("/Users/roddux/src/play1/index.html").unwrap(); // dev mode for fucking with HTML
        stream.write(b"HTTP/1.1 200 OK\n").ok();
        stream.write(&index_page).ok();
        stream.shutdown(Shutdown::Both).ok();
        return;
    }

    println!("{:?}", dat[1].split('?').collect::<Vec<&str>>());

    match dat[1].split('?').collect::<Vec<&str>>()[..] {
        ["/save", x] => {
            println!("x: {}", &x[5..]);
            stream.write(b"HTTP/1.1 200 OK\nsaved").ok();
            let fname = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string();
            let mut path = "/tmp/posts/".to_string();
            path.push_str(&fname);
            std::fs::write(path, &x[5..]).ok(); // trim data=
            stream.shutdown(Shutdown::Both).ok();
        }
        ["/list"] => {
            stream.write(b"HTTP/1.1 200 OK\nContent-Type: application/json\r\n\r\n{\"posts\":{").ok();
            let mut dirs = std::fs::read_dir("/tmp/posts").unwrap().peekable();

            while let Some(path) = dirs.next() {
                let path = path.unwrap();
                println!("got path {:?}", path.file_name());
                let post_data = std::fs::read(&path.path()).unwrap();
                stream.write( b"\"" );
                stream.write(&path.file_name().into_string().unwrap().as_bytes() );
                stream.write(b"\": \"");
                stream.write(&post_data );
                
                if !dirs.peek().is_none() {
                    stream.write(b"\",");
                } else {
                    stream.write(b"\"");
                }
            }
            stream.write(b"}}\n");
            stream.shutdown(Shutdown::Both).ok();
        }
        _ => {
            die("HTTP/1.1 400 Bad Request\nbad req", stream);
        }
    }
}

fn main() {
    let listener: TcpListener = TcpListener::bind("0.0.0.0:9999").unwrap();
    for (req_count, s) in listener.incoming().enumerate() {
        handle_connection(&mut s.unwrap());
        println!("req # {}", req_count);
    }
}