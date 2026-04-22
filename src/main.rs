use anyhow::Context;
use std::{
    io::{BufRead as _, BufReader, Write},
    os::unix::{
        ffi::OsStrExt as _,
        net::{UnixListener, UnixStream},
        process::CommandExt,
    },
    path::PathBuf,
    process::Command,
    thread,
};

fn handle_connection(conn: UnixStream, uid: u32) {
    let mut buf = BufReader::new(conn);
    loop {
        let buf_len = buf.buffer().len();
        let eol_reached = {
            let buffer = buf.fill_buf();
            match buffer {
                Ok(b) => b.contains(&b'\n'),
                Err(e) => {
                    eprintln!("failed to read from socket: {e:?}");
                    return;
                }
            }
        };
        if eol_reached {
            break;
        }
        if buf_len == buf.buffer().len() {
            // nothing was read break
            return;
        }
    }
    let output = {
        let buf = buf.buffer();
        let mut cmd = buf[0..buf.iter().position(|c| *c == b'\n').unwrap()]
            .split(|c| *c == b' ')
            .map(std::ffi::OsStr::from_bytes);
        let program = cmd.next().unwrap();
        let mut command = Command::new(program);
        command.args(cmd);
        command.uid(uid);
        println!("running: {} {:?}", program.display(), command.get_args());
        command.output()
    };
    let mut conn = buf.into_inner();
    let reply = match output {
        Ok(output) => {
            let code = output
                .status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| String::from("null"));
            let escape = |s: &str| s.replace("\n", r#"\n"#).replace("\"", r#"\""#);
            format!(
                r#"{{"status": "success", "exit": {code}, "stdout": "{}", "stderr": "{}"}}"#,
                escape(&String::from_utf8_lossy(&output.stdout)),
                escape(&String::from_utf8_lossy(&output.stderr)),
            )
        }
        Err(e) => format!(r#"{{"status":"failure": "error": {e}}}"#),
    };
    println!("replying: {reply}");
    if let Err(e) = conn.write_all(reply.as_bytes()) {
        eprintln!("failed to reply: {e:?}");
    }
}

fn main() -> anyhow::Result<()> {
    let mut socket_path = PathBuf::from("/var/lib/hass-bridge");
    std::fs::create_dir_all(&socket_path)
        .with_context(|| format!("failed to create dir {}", socket_path.display()))?;
    socket_path.push("socket");
    let _ = std::fs::remove_file(&socket_path);
    let socket = UnixListener::bind(&socket_path)
        .with_context(|| format!("binding socket {}", socket_path.display()))?;

    let uid = std::env::var("UID")
        .ok()
        .and_then(|i| i.parse().ok())
        .unwrap_or(1000);
    for conn in socket.incoming() {
        let conn = conn.context("failed to accept connection")?;
        thread::spawn(move || handle_connection(conn, uid));
    }

    Ok(())
}
