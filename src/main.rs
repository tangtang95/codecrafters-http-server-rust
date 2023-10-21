use std::net::TcpListener;
use std::io::{Write, BufReader, BufRead};
use nom::IResult;
use nom::bytes::complete::{tag, take_until};

use anyhow::Result;
use anyhow::anyhow;

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

    Ok((input, HttpCommand {
        method: method.to_string(),
        path: path.to_string(),
        version: input.to_string()
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
                   "/"  => write!(stream, "HTTP/1.1 200 OK\r\n\r\n")?,
                   path if path.starts_with("/echo/") => {
                        let echo_text = path.split("/echo/").last().ok_or(anyhow!("Could not find echo text"))?;
                        write!(stream, "HTTP/1.1 200 OK\r\n")?;
                        write!(stream, "Content-Type: text/plain\r\n")?;
                        write!(stream, "Content-Length: {}\r\n", echo_text.len())?;
                        write!(stream, "\r\n")?;
                        write!(stream, "{}\r\n", echo_text)?;
                        write!(stream, "\r\n")?;
                   },
                   _  => write!(stream, "HTTP/1.1 404 Not Found\r\n\r\n")?,
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
