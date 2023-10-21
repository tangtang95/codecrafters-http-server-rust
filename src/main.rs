use nom::bytes::complete::{tag, take_until};
use nom::multi::fold_many0;
use nom::IResult;
use std::collections::HashMap;
use std::io::{BufReader, Read, Write};
use std::net::TcpListener;
use std::str::from_utf8;

use anyhow::anyhow;
use anyhow::Result;

#[derive(Debug)]
struct HttpRequest {
    command: HttpCommand,
    headers: HttpHeaders,
}

#[derive(Debug)]
struct HttpCommand {
    method: String,
    path: String,
    version: String,
}

#[derive(Debug)]
struct HttpHeaders(HashMap<String, String>);

fn http_request(input: &str) -> IResult<&str, HttpRequest> {
    let (input, command) = take_line(input)?;
    let (_, command) = http_command(command)?;

    let (input, header_map) = fold_many0(http_header, HashMap::new, |mut acc, (key, value)| {
        acc.insert(key, value);
        acc
    })(input)?;

    Ok((
        input,
        HttpRequest {
            command,
            headers: HttpHeaders(header_map),
        },
    ))
}

fn http_command(input: &str) -> IResult<&str, HttpCommand> {
    let (input, method) = take_until(" ")(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, path) = take_until(" ")(input)?;
    let (input, _) = tag(" ")(input)?;

    Ok((
        input,
        HttpCommand {
            method: method.to_string(),
            path: path.to_string(),
            version: input.to_string(),
        },
    ))
}

fn http_header(input: &str) -> IResult<&str, (String, String)> {
    let (input, header) = take_line(input)?;
    let (rest, header_key) = take_until(":")(header)?;
    let (header_value, _) = tag(":")(rest)?;

    Ok((input, (header_key.to_string(), header_value.to_string())))
}

fn take_line(input: &str) -> IResult<&str, &str> {
    let (input, line) = take_until("\r\n")(input)?;
    let (input, _) = tag("\r\n")(input)?;
    Ok((input, line))
}

fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    const MAX_BYTES: usize = 8192;
    let listener = TcpListener::bind("127.0.0.1:4221")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut reader = BufReader::new(stream.try_clone()?);
                let mut buf = [0u8; MAX_BYTES];
                reader.read(&mut buf)?;
                let (_, request) = http_request(from_utf8(&buf)?).unwrap();

                eprintln!("request: {:?}", request);
                match request.command.path.as_str() {
                    "/" => write!(stream, "HTTP/1.1 200 OK\r\n\r\n")?,
                    path if path.starts_with("/echo/") => {
                        let echo_text = path
                            .split("/echo/")
                            .last()
                            .ok_or(anyhow!("Could not find echo text"))?;
                        write!(stream, "HTTP/1.1 200 OK\r\n")?;
                        write!(stream, "Content-Type: text/plain\r\n")?;
                        write!(stream, "Content-Length: {}\r\n", echo_text.len())?;
                        write!(stream, "\r\n")?;
                        write!(stream, "{}\r\n", echo_text)?;
                        write!(stream, "\r\n")?;
                    }
                    "/user-agent" => {
                        let user_agent = request
                            .headers
                            .0
                            .get("User-Agent")
                            .ok_or(anyhow!("User-Agent header not found"))?;
                        write!(stream, "HTTP/1.1 200 OK\r\n")?;
                        write!(stream, "Content-Type: text/plain\r\n")?;
                        write!(stream, "Content-Length: {}\r\n", user_agent.len())?;
                        write!(stream, "\r\n")?;
                        write!(stream, "{}\r\n", user_agent)?;
                        write!(stream, "\r\n")?;
                    }
                    _ => write!(stream, "HTTP/1.1 404 Not Found\r\n\r\n")?,
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
