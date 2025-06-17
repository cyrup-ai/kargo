# Module `net`

TCP/UDP/Unix bindings for `tokio`.
This module contains the TCP/UDP/Unix networking types, similar to the standard
library, which can be used to implement networking protocols.
# Organization
* [`TcpListener`] and [`TcpStream`] provide functionality for communication over TCP
* [`UdpSocket`] provides functionality for communication over UDP
* [`UnixListener`] and [`UnixStream`] provide functionality for communication over a
Unix Domain Stream Socket **(available on Unix only)**
* [`UnixDatagram`] provides functionality for communication
over Unix Domain Datagram Socket **(available on Unix only)**
* [`tokio::net::unix::pipe`] for FIFO pipes **(available on Unix only)**
* [`tokio::net::windows::named_pipe`] for Named Pipes **(available on Windows only)**
For IO resources not available in `tokio::net`, you can use [`AsyncFd`].
[`TcpListener`]: TcpListener
[`TcpStream`]: TcpStream
[`UdpSocket`]: UdpSocket
[`UnixListener`]: UnixListener
[`UnixStream`]: UnixStream
[`UnixDatagram`]: UnixDatagram
[`tokio::net::unix::pipe`]: unix::pipe
[`tokio::net::windows::named_pipe`]: windows::named_pipe
[`AsyncFd`]: crate::io::unix::AsyncFd

## Contents


