use std::net::TcpListener;
use std::io::{Write, Read, BufReader, BufRead};
use nom::IResult;
use nom::bytes::complete::{tag, take_until};

use anyhow::Result;

#[derive(Debug)]
struct HttpRequest {
    command: HttpCommand
}

#[derive(Debug)]
struct HttpCommand {
    method: String,
    path: String,
    version: String,
}

fn http_request(input: &str) -> IResult<&str, HttpRequest> {
    let (input, command) = take_until("\r\n")(input)?;
    let (_, command) = http_command(command)?;
    let (input, _) = tag("\r\n")(input)?;
    Ok((input, HttpRequest {
        command
    }))
}

fn http_command(input: &str) -> IResult<&str, HttpCommand> {
    let (input, method) = take_until(" ")(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, path) = take_until(" ")(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, version) = take_until("\r\n")(input)?;

    Ok((input, HttpCommand {
        method: method.to_string(),
        path: path.to_string(),
        version: version.to_string()
    }))
}

fn main() -> Result<()>{
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221")?;
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                
                let mut reader = BufReader::new(stream.try_clone()?);
                let mut buf = String::new();
                reader.read_line(&mut buf)?;
                let (_, request) = http_request(&buf).unwrap();
                
                eprintln!("request: {:?}", request);
                match request.command.path.as_str() {
                   "/"  => {
                        let response = "HTTP/1.1 200 OK\r\n\r\n";
                        
                        stream.write(response.as_bytes())?;
                        stream.flush()?;
                   },
                   _ => {
                        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                        
                        stream.write(response.as_bytes())?;
                        stream.flush()?;
                   }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
