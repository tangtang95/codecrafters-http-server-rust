use nom::bytes::complete::{tag, take_until};
use nom::multi::fold_many0;
use nom::IResult;
use std::collections::HashMap;
use std::str::from_utf8;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

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
    let (rest, header_key) = take_until(": ")(header)?;
    let (header_value, _) = tag(": ")(rest)?;

    Ok((input, (header_key.to_string(), header_value.to_string())))
}

fn take_line(input: &str) -> IResult<&str, &str> {
    let (input, line) = take_until("\r\n")(input)?;
    let (input, _) = tag("\r\n")(input)?;
    Ok((input, line))
}

const MAX_BYTES: usize = 8192;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => match handle_connection(&mut stream).await {
                Ok(_) => println!("connection handled properly!"),
                Err(err) => println!("connection failed due to {}", err),
            },
            Err(err) => println!("error: {}", err),
        }
    }
}

async fn handle_connection(stream: &mut TcpStream) -> Result<()> {
    println!("accepted new connection");
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut buf = [0u8; MAX_BYTES];
    reader.read(&mut buf).await?;
    let (_, request) = http_request(from_utf8(&buf)?).map_err(|_| anyhow!("Http Request Parsing Error"))?;

    eprintln!("request: {:?}", request);
    match request.command.path.as_str() {
        "/" => writer.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).await?,
        path if path.starts_with("/echo/") => {
            let echo_text = path
                .split("/echo/")
                .last()
                .ok_or(anyhow!("Could not find echo text"))?;
            let response = format!(
                "HTTP/1.1 200 OK \r\n\
                Content-Type: text/plain\r\n\
                Content-Length: {}\r\n\
                \r\n\
                {}\r\n\r\n",
                echo_text.len(),
                echo_text
            );
            writer.write_all(response.as_bytes()).await?;
        }
        "/user-agent" => {
            let user_agent = request
                .headers
                .0
                .get("User-Agent")
                .ok_or(anyhow!("User-Agent header not found"))?;
            let response = format!(
                "HTTP/1.1 200 OK \r\n\
                Content-Type: text/plain\r\n\
                Content-Length: {}\r\n\
                \r\n\
                {}\r\n\r\n",
                user_agent.len(),
                user_agent
            );
            writer.write_all(response.as_bytes()).await?;
        }
        _ => writer.write_all("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).await?,
    }
    Ok(())
}
