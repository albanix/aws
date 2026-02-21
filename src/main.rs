use runtime::Reactor;
use std::net::{TcpStream, SocketAddr};
use std::io::ErrorKind;
use std::os::fd::{AsFd, AsRawFd};
use std::process::Output;
use std::sync::Arc;
use std::future::Future;
use std::task::Poll;
pub struct AsyncTcpConnect {
    stream: Option<TcpStream>,
    addr: SocketAddr,
    reactor: Arc<Reactor>,
    registered: bool
}

impl AsyncTcpConnect {
    pub fn new(addr: SocketAddr, reactor: Arc<Reactor>) -> Self {
        Self {
            stream: None,
            addr,
            reactor,
            registered: false
        }
    }
}

impl Future for AsyncTcpConnect {
    type Output = std::io::Result<TcpStream>;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if self.stream.is_none() {
            match TcpStream::connect(self.addr) {
                Ok(stream) => {
                    stream.set_nonblocking(true).unwrap();
                    self.stream = Some(stream);
                },
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    let stream = TcpStream::connect(self.addr)?;
                    stream.set_nonblocking(true).unwrap();
                    self.stream = Some(stream);
                },
                Err(e) => return Poll::Ready(Err(e))
            }
        }

        let stream = self.stream.as_ref().unwrap();
        let fd = stream.as_raw_fd();
        if !self.registered {
            self.reactor.register(fd, true, cx.waker().clone());
            self.registered = true;
            return Poll::Pending
        }
        Poll::Ready(Ok(stream.try_clone().unwrap()))
    }
}
fn main() {

}