use super::proto::{MofosRequest, MofosResponse};

use std::io::Error;
use std::net::{SocketAddr, UdpSocket};
use std::process::Command;

pub struct Client {
    socket: UdpSocket,
}

impl Client {
    pub fn new(dest: SocketAddr) -> Result<Client, Error> {
        match UdpSocket::bind(dest) {
            Ok(socket) => Ok(Client { socket }),
            Err(e) => Err(e),
        }
    }

    pub fn send_req(&mut self, req: MofosRequest) -> Result<MofosResponse, Error> {
        panic!("send_req unimplemented");
    }
}

pub fn spawn_remote_server(
    addr: &SocketAddr,
    server: String,
    port: u16,
    dir: String,
) -> Result<(), Error> {
    debug!("spawning remote server using ssh");

    // FIXME: parametric server spawning
    let result = Command::new("ssh")
        .arg(server)
        .arg(format!("-p {}", port))
        .arg("mofos-server -p 6000 -t /tmp")
        .output()?;

    if result.status.success() {
        Ok(())
    } else {
        Err(Error {})
    }
}
