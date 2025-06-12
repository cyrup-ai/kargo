sss# Crate Documentation

**Version:** 1.45.1

**Format Version:** 46

# Module `tokio`

A runtime for writing reliable network applications without compromising speed.

Tokio is an event-driven, non-blocking I/O platform for writing asynchronous
applications with the Rust programming language. At a high level, it
provides a few major components:

* Tools for [working with asynchronous tasks][tasks], including
  [synchronization primitives and channels][sync] and [timeouts, sleeps, and
  intervals][time].
* APIs for [performing asynchronous I/O][io], including [TCP and UDP][net] sockets,
  [filesystem][fs] operations, and [process] and [signal] management.
* A [runtime] for executing asynchronous code, including a task scheduler,
  an I/O driver backed by the operating system's event queue (`epoll`, `kqueue`,
  `IOCP`, etc...), and a high performance timer.
s
Guide level documentation is found on the [website].

[tasks]: #working-with-tasks
[sync]: crate::sync
[time]: crate::time
[io]: #asynchronous-io
[net]: crate::net
[fs]: crate::fs
[process]: crate::process
[signal]: crate::signal
[fs]: crate::fs
[runtime]: crate::runtime
[website]: https://tokio.rs/tokio/tutorial

# A Tour of Tokio

Tokio consists of a number of modules that provide a range of functionality
essential for implementing asynchronous applications in Rust. In this
section, we will take a brief tour of Tokio, summarizing the major APIs and
their uses.

The easiest way to get started is to enable all features. Do this by
enabling the `full` feature flag:

```toml
tokio = { version = "1", features = ["full"] }
```

### Authoring applications

Tokio is great for writing applications and most users in this case shouldn't
worry too much about what features they should pick. If you're unsure, we suggest
going with `full` to ensure that you don't run into any road blocks while you're
building your application.

#### Example

This example shows the quickest way to get started with Tokio.

```toml
tokio = { version = "1", features = ["full"] }
```

### Authoring libraries

As a library author your goal should be to provide the lightest weight crate
that is based on Tokio. To achieve this you should ensure that you only enable
the features you need. This allows users to pick up your crate without having
to enable unnecessary features.

#### Example

This example shows how you may want to import features for a library that just
needs to `tokio::spawn` and use a `TcpStream`.

```toml
tokio = { version = "1", features = ["rt", "net"] }
```

## Working With Tasks

Asynchronous programs in Rust are based around lightweight, non-blocking
units of execution called [_tasks_][tasks]. The [`tokio::task`] module provides
important tools for working with tasks:

* The [`spawn`] function and [`JoinHandle`] type, for scheduling a new task
  on the Tokio runtime and awaiting the output of a spawned task, respectively,
* Functions for [running blocking operations][blocking] in an asynchronous
  task context.

The [`tokio::task`] module is present only when the "rt" feature flag
is enabled.

[tasks]: task/index.html#what-are-tasks
[`tokio::task`]: crate::task
[`spawn`]: crate::task::spawn()
[`JoinHandle`]: crate::task::JoinHandle
[blocking]: task/index.html#blocking-and-yielding

The [`tokio::sync`] module contains synchronization primitives to use when
needing to communicate or share data. These include:

* channels ([`oneshot`], [`mpsc`], [`watch`], and [`broadcast`]), for sending values
  between tasks,
* a non-blocking [`Mutex`], for controlling access to a shared, mutable
  value,
* an asynchronous [`Barrier`] type, for multiple tasks to synchronize before
  beginning a computation.

The `tokio::sync` module is present only when the "sync" feature flag is
enabled.

[`tokio::sync`]: crate::sync
[`Mutex`]: crate::sync::Mutex
[`Barrier`]: crate::sync::Barrier
[`oneshot`]: crate::sync::oneshot
[`mpsc`]: crate::sync::mpsc
[`watch`]: crate::sync::watch
[`broadcast`]: crate::sync::broadcast

The [`tokio::time`] module provides utilities for tracking time and
scheduling work. This includes functions for setting [timeouts][timeout] for
tasks, [sleeping][sleep] work to run in the future, or [repeating an operation at an
interval][interval].

In order to use `tokio::time`, the "time" feature flag must be enabled.

[`tokio::time`]: crate::time
[sleep]: crate::time::sleep()
[interval]: crate::time::interval()
[timeout]: crate::time::timeout()

Finally, Tokio provides a _runtime_ for executing asynchronous tasks. Most
applications can use the [`#[tokio::main]`][main] macro to run their code on the
Tokio runtime. However, this macro provides only basic configuration options. As
an alternative, the [`tokio::runtime`] module provides more powerful APIs for configuring
and managing runtimes. You should use that module if the `#[tokio::main]` macro doesn't
provide the functionality you need.

Using the runtime requires the "rt" or "rt-multi-thread" feature flags, to
enable the current-thread [single-threaded scheduler][rt] and the [multi-thread
scheduler][rt-multi-thread], respectively. See the [`runtime` module
documentation][rt-features] for details. In addition, the "macros" feature
flag enables the `#[tokio::main]` and `#[tokio::test]` attributes.

[main]: attr.main.html
[`tokio::runtime`]: crate::runtime
[`Builder`]: crate::runtime::Builder
[`Runtime`]: crate::runtime::Runtime
[rt]: runtime/index.html#current-thread-scheduler
[rt-multi-thread]: runtime/index.html#multi-thread-scheduler
[rt-features]: runtime/index.html#runtime-scheduler

## CPU-bound tasks and blocking code

Tokio is able to concurrently run many tasks on a few threads by repeatedly
swapping the currently running task on each thread. However, this kind of
swapping can only happen at `.await` points, so code that spends a long time
without reaching an `.await` will prevent other tasks from running. To
combat this, Tokio provides two kinds of threads: Core threads and blocking threads.

The core threads are where all asynchronous code runs, and Tokio will by default
spawn one for each CPU core. You can use the environment variable `TOKIO_WORKER_THREADS`
to override the default value.

The blocking threads are spawned on demand, can be used to run blocking code
that would otherwise block other tasks from running and are kept alive when
not used for a certain amount of time which can be configured with [`thread_keep_alive`].
Since it is not possible for Tokio to swap out blocking tasks, like it
can do with asynchronous code, the upper limit on the number of blocking
threads is very large. These limits can be configured on the [`Builder`].

To spawn a blocking task, you should use the [`spawn_blocking`] function.

[`Builder`]: crate::runtime::Builder
[`spawn_blocking`]: crate::task::spawn_blocking()
[`thread_keep_alive`]: crate::runtime::Builder::thread_keep_alive()

```
#[tokio::main]
async fn main() {
    // This is running on a core thread.

    let blocking_task = tokio::task::spawn_blocking(|| {
        // This is running on a blocking thread.
        // Blocking here is ok.
    });

    // We can wait for the blocking task like this:
    // If the blocking task panics, the unwrap below will propagate the
    // panic.
    blocking_task.await.unwrap();
}
```

If your code is CPU-bound and you wish to limit the number of threads used
to run it, you should use a separate thread pool dedicated to CPU bound tasks.
For example, you could consider using the [rayon] library for CPU-bound
tasks. It is also possible to create an extra Tokio runtime dedicated to
CPU-bound tasks, but if you do this, you should be careful that the extra
runtime runs _only_ CPU-bound tasks, as IO-bound tasks on that runtime
will behave poorly.

Hint: If using rayon, you can use a [`oneshot`] channel to send the result back
to Tokio when the rayon task finishes.

[rayon]: https://docs.rs/rayon
[`oneshot`]: crate::sync::oneshot

## Asynchronous IO

As well as scheduling and running tasks, Tokio provides everything you need
to perform input and output asynchronously.

The [`tokio::io`] module provides Tokio's asynchronous core I/O primitives,
the [`AsyncRead`], [`AsyncWrite`], and [`AsyncBufRead`] traits. In addition,
when the "io-util" feature flag is enabled, it also provides combinators and
functions for working with these traits, forming as an asynchronous
counterpart to [`std::io`].

Tokio also includes APIs for performing various kinds of I/O and interacting
with the operating system asynchronously. These include:

* [`tokio::net`], which contains non-blocking versions of [TCP], [UDP], and
  [Unix Domain Sockets][UDS] (enabled by the "net" feature flag),
* [`tokio::fs`], similar to [`std::fs`] but for performing filesystem I/O
  asynchronously (enabled by the "fs" feature flag),
* [`tokio::signal`], for asynchronously handling Unix and Windows OS signals
  (enabled by the "signal" feature flag),
* [`tokio::process`], for spawning and managing child processes (enabled by
  the "process" feature flag).

[`tokio::io`]: crate::io
[`AsyncRead`]: crate::io::AsyncRead
[`AsyncWrite`]: crate::io::AsyncWrite
[`AsyncBufRead`]: crate::io::AsyncBufRead
[`std::io`]: std::io
[`tokio::net`]: crate::net
[TCP]: crate::net::tcp
[UDP]: crate::net::UdpSocket
[UDS]: crate::net::unix
[`tokio::fs`]: crate::fs
[`std::fs`]: std::fs
[`tokio::signal`]: crate::signal
[`tokio::process`]: crate::process

# Examples

A simple TCP echo server:

```no_run
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
```

# Feature flags

Tokio uses a set of [feature flags] to reduce the amount of compiled code. It
is possible to just enable certain features over others. By default, Tokio
does not enable any features but allows one to enable a subset for their use
case. Below is a list of the available feature flags. You may also notice
above each function, struct and trait there is listed one or more feature flags
that are required for that item to be used. If you are new to Tokio it is
recommended that you use the `full` feature flag which will enable all public APIs.
Beware though that this will pull in many extra dependencies that you may not
need.

- `full`: Enables all features listed below except `test-util` and `tracing`.
- `rt`: Enables `tokio::spawn`, the current-thread scheduler,
        and non-scheduler utilities.
- `rt-multi-thread`: Enables the heavier, multi-threaded, work-stealing scheduler.
- `io-util`: Enables the IO based `Ext` traits.
- `io-std`: Enable `Stdout`, `Stdin` and `Stderr` types.
- `net`: Enables `tokio::net` types such as `TcpStream`, `UnixStream` and
         `UdpSocket`, as well as (on Unix-like systems) `AsyncFd` and (on
         FreeBSD) `PollAio`.
- `time`: Enables `tokio::time` types and allows the schedulers to enable
          the built in timer.
- `process`: Enables `tokio::process` types.
- `macros`: Enables `#[tokio::main]` and `#[tokio::test]` macros.
- `sync`: Enables all `tokio::sync` types.
- `signal`: Enables all `tokio::signal` types.
- `fs`: Enables `tokio::fs` types.
- `test-util`: Enables testing based infrastructure for the Tokio runtime.
- `parking_lot`: As a potential optimization, use the `_parking_lot_` crate's
                 synchronization primitives internally. Also, this
                 dependency is necessary to construct some of our primitives
                 in a `const` context. `MSRV` may increase according to the
                 `_parking_lot_` release in use.

_Note: `AsyncRead` and `AsyncWrite` traits do not require any features and are
always available._

## Unstable features

Some feature flags are only available when specifying the `tokio_unstable` flag:

- `tracing`: Enables tracing events.

Likewise, some parts of the API are only available with the same flag:

- [`task::Builder`]
- Some methods on [`task::JoinSet`]
- [`runtime::RuntimeMetrics`]
- [`runtime::Builder::on_task_spawn`]
- [`runtime::Builder::on_task_terminate`]
- [`runtime::Builder::unhandled_panic`]
- [`runtime::TaskMeta`]

This flag enables **unstable** features. The public API of these features
may break in 1.x releases. To enable these features, the `--cfg
tokio_unstable` argument must be passed to `rustc` when compiling. This
serves to explicitly opt-in to features which may break semver conventions,
since Cargo [does not yet directly support such opt-ins][unstable features].

You can specify it in your project's `.cargo/config.toml` file:

```toml
[build]
rustflags = ["--cfg", "tokio_unstable"]
```

<div class="warning">
The <code>[build]</code> section does <strong>not</strong> go in a
<code>Cargo.toml</code> file. Instead it must be placed in the Cargo config
file <code>.cargo/config.toml</code>.
</div>

Alternatively, you can specify it with an environment variable:

```sh
## Many *nix shells:
export RUSTFLAGS="--cfg tokio_unstable"
cargo build
```

```powershell
## Windows PowerShell:
$Env:RUSTFLAGS="--cfg tokio_unstable"
cargo build
```

[unstable features]: https://internals.rust-lang.org/t/feature-request-unstable-opt-in-non-transitive-crate-features/16193#why-not-a-crate-feature-2
[feature flags]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section

# Supported platforms

Tokio currently guarantees support for the following platforms:

 * Linux
 * Windows
 * Android (API level 21)
 * macOS
 * iOS
 * FreeBSD

Tokio will continue to support these platforms in the future. However,
future releases may change requirements such as the minimum required libc
version on Linux, the API level on Android, or the supported FreeBSD
release.

Beyond the above platforms, Tokio is intended to work on all platforms
supported by the mio crate. You can find a longer list [in mio's
documentation][mio-supported]. However, these additional platforms may
become unsupported in the future.

Note that Wine is considered to be a different platform from Windows. See
mio's documentation for more information on Wine support.

[mio-supported]: https://crates.io/crates/mio#platforms

## `WASM` support

Tokio has some limited support for the `WASM` platform. Without the
`tokio_unstable` flag, the following features are supported:

 * `sync`
 * `macros`
 * `io-util`
 * `rt`
 * `time`

Enabling any other feature (including `full`) will cause a compilation
failure.

The `time` module will only work on `WASM` platforms that have support for
timers (e.g. wasm32-wasi). The timing functions will panic if used on a `WASM`
platform that does not support timers.

Note also that if the runtime becomes indefinitely idle, it will panic
immediately instead of blocking forever. On platforms that don't support
time, this means that the runtime can never be idle in any way.

## Unstable `WASM` support

Tokio also has unstable support for some additional `WASM` features. This
requires the use of the `tokio_unstable` flag.

Using this flag enables the use of `tokio::net` on the wasm32-wasi target.
However, not all methods are available on the networking types as `WASI`
currently does not support the creation of new sockets from within `WASM`.
Because of this, sockets must currently be created via the `FromRawFd`
trait.

## Modules

## Module `io`

**Attributes:**

- `#[<cfg_attr>(not(all(feature = "rt", feature = "net")),
allow(dead_code, unused_imports))]`
- `#[allow(dead_code, unused_imports)]`

Traits, helpers, and type definitions for asynchronous I/O functionality.

This module is the asynchronous version of `std::io`. Primarily, it
defines two traits, [`AsyncRead`] and [`AsyncWrite`], which are asynchronous
versions of the [`Read`] and [`Write`] traits in the standard library.

# `AsyncRead` and `AsyncWrite`

Like the standard library's [`Read`] and [`Write`] traits, [`AsyncRead`] and
[`AsyncWrite`] provide the most general interface for reading and writing
input and output. Unlike the standard library's traits, however, they are
_asynchronous_ &mdash; meaning that reading from or writing to a `tokio::io`
type will _yield_ to the Tokio scheduler when IO is not ready, rather than
blocking. This allows other tasks to run while waiting on IO.

Another difference is that `AsyncRead` and `AsyncWrite` only contain
core methods needed to provide asynchronous reading and writing
functionality. Instead, utility methods are defined in the [`AsyncReadExt`]
and [`AsyncWriteExt`] extension traits. These traits are automatically
implemented for all values that implement `AsyncRead` and `AsyncWrite`
respectively.

End users will rarely interact directly with `AsyncRead` and
`AsyncWrite`. Instead, they will use the async functions defined in the
extension traits. Library authors are expected to implement `AsyncRead`
and `AsyncWrite` in order to provide types that behave like byte streams.

Even with these differences, Tokio's `AsyncRead` and `AsyncWrite` traits
can be used in almost exactly the same manner as the standard library's
`Read` and `Write`. Most types in the standard library that implement `Read`
and `Write` have asynchronous equivalents in `tokio` that implement
`AsyncRead` and `AsyncWrite`, such as [`File`] and [`TcpStream`].

For example, the standard library documentation introduces `Read` by
[demonstrating][std_example] reading some bytes from a [`std::fs::File`]. We
can do the same with [`tokio::fs::File`][`File`]:

```no_run
use tokio::io::{self, AsyncReadExt};
use tokio::fs::File;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut f = File::open("foo.txt").await?;
    let mut buffer = [0; 10];

    // read up to 10 bytes
    let n = f.read(&mut buffer).await?;

    println!("The bytes: {:?}", &buffer[..n]);
    Ok(())
}
```

[`File`]: crate::fs::File
[`TcpStream`]: crate::net::TcpStream
[`std::fs::File`]: std::fs::File
[std_example]: std::io#read-and-write

## Buffered Readers and Writers

Byte-based interfaces are unwieldy and can be inefficient, as we'd need to be
making near-constant calls to the operating system. To help with this,
`std::io` comes with [support for _buffered_ readers and writers][stdbuf],
and therefore, `tokio::io` does as well.

Tokio provides an async version of the [`std::io::BufRead`] trait,
[`AsyncBufRead`]; and async [`BufReader`] and [`BufWriter`] structs, which
wrap readers and writers. These wrappers use a buffer, reducing the number
of calls and providing nicer methods for accessing exactly what you want.

For example, [`BufReader`] works with the [`AsyncBufRead`] trait to add
extra methods to any async reader:

```no_run
use tokio::io::{self, BufReader, AsyncBufReadExt};
use tokio::fs::File;

#[tokio::main]
async fn main() -> io::Result<()> {
    let f = File::open("foo.txt").await?;
    let mut reader = BufReader::new(f);
    let mut buffer = String::new();

    // read a line into buffer
    reader.read_line(&mut buffer).await?;

    println!("{}", buffer);
    Ok(())
}
```

[`BufWriter`] doesn't add any new ways of writing; it just buffers every call
to [`write`](crate::io::AsyncWriteExt::write). However, you **must** flush
[`BufWriter`] to ensure that any buffered data is written.

```no_run
use tokio::io::{self, BufWriter, AsyncWriteExt};
use tokio::fs::File;

#[tokio::main]
async fn main() -> io::Result<()> {
    let f = File::create("foo.txt").await?;
    {
        let mut writer = BufWriter::new(f);

        // Write a byte to the buffer.
        writer.write(&[42u8]).await?;

        // Flush the buffer before it goes out of scope.
        writer.flush().await?;

    } // Unless flushed or shut down, the contents of the buffer is discarded on drop.

    Ok(())
}
```

[stdbuf]: std::io#bufreader-and-bufwriter
[`std::io::BufRead`]: std::io::BufRead
[`AsyncBufRead`]: crate::io::AsyncBufRead
[`BufReader`]: crate::io::BufReader
[`BufWriter`]: crate::io::BufWriter

## Implementing `AsyncRead` and `AsyncWrite`

Because they are traits, we can implement [`AsyncRead`] and [`AsyncWrite`] for
our own types, as well. Note that these traits must only be implemented for
non-blocking I/O types that integrate with the futures type system. In
other words, these types must never block the thread, and instead the
current task is notified when the I/O resource is ready.

## Conversion to and from Stream/Sink

It is often convenient to encapsulate the reading and writing of bytes in a
[`Stream`] or [`Sink`] of data.

Tokio provides simple wrappers for converting [`AsyncRead`] to [`Stream`]
and vice-versa in the [tokio-util] crate, see [`ReaderStream`] and
[`StreamReader`].

There are also utility traits that abstract the asynchronous buffering
necessary to write your own adaptors for encoding and decoding bytes to/from
your structured data, allowing to transform something that implements
[`AsyncRead`]/[`AsyncWrite`] into a [`Stream`]/[`Sink`], see [`Decoder`] and
[`Encoder`] in the [tokio-util::codec] module.

[tokio-util]: https://docs.rs/tokio-util
[tokio-util::codec]: https://docs.rs/tokio-util/latest/tokio_util/codec/index.html

# Standard input and output

Tokio provides asynchronous APIs to standard [input], [output], and [error].
These APIs are very similar to the ones provided by `std`, but they also
implement [`AsyncRead`] and [`AsyncWrite`].

Note that the standard input / output APIs  **must** be used from the
context of the Tokio runtime, as they require Tokio-specific features to
function. Calling these functions outside of a Tokio runtime will panic.

[input]: fn@stdin
[output]: fn@stdout
[error]: fn@stderr

# `std` re-exports

Additionally, [`Error`], [`ErrorKind`], [`Result`], and [`SeekFrom`] are
re-exported from `std::io` for ease of use.

[`AsyncRead`]: trait@AsyncRead
[`AsyncWrite`]: trait@AsyncWrite
[`AsyncReadExt`]: trait@AsyncReadExt
[`AsyncWriteExt`]: trait@AsyncWriteExt
["codec"]: https://docs.rs/tokio-util/latest/tokio_util/codec/index.html
[`Encoder`]: https://docs.rs/tokio-util/latest/tokio_util/codec/trait.Encoder.html
[`Decoder`]: https://docs.rs/tokio-util/latest/tokio_util/codec/trait.Decoder.html
[`ReaderStream`]: https://docs.rs/tokio-util/latest/tokio_util/io/struct.ReaderStream.html
[`StreamReader`]: https://docs.rs/tokio-util/latest/tokio_util/io/struct.StreamReader.html
[`Error`]: struct@Error
[`ErrorKind`]: enum@ErrorKind
[`Result`]: type@Result
[`Read`]: std::io::Read
[`SeekFrom`]: enum@SeekFrom
[`Sink`]: https://docs.rs/futures/0.3/futures/sink/trait.Sink.html
[`Stream`]: https://docs.rs/futures/0.3/futures/stream/trait.Stream.html
[`Write`]: std::io::Write

```rust
pub mod io { /* ... */ }
```

#### Other Items

##### Unnamed Item

```rust
pub /* unnamed item */
```

##### Unnamed Item

```rust
pub /* unnamed item */
```

##### Unnamed Item

```rust
pub /* unnamed item */
```

##### Unnamed Item

```rust
pub /* unnamed item */
```

##### Unnamed Item

```rust
pub /* unnamed item */
```

##### Unnamed Item

**Attributes:**

- `#[doc(no_inline)]`

```rust
pub /* unnamed item */
```

##### Unnamed Item

**Attributes:**

- `#[doc(no_inline)]`

```rust
pub /* unnamed item */
```

##### Unnamed Item

**Attributes:**

- `#[doc(no_inline)]`

```rust
pub /* unnamed item */
```

##### Unnamed Item

**Attributes:**

- `#[doc(no_inline)]`

```rust
pub /* unnamed item */
```

## Module `net`

**Attributes:**

- `#[<cfg>(not(loom))]`

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

```rust
pub mod net { /* ... */ }
```

#### Other Items

##### Unnamed Item

```rust
pub /* unnamed item */
```

## Module `task`

Asynchronous green-threads.

## What are Tasks?

A _task_ is a light weight, non-blocking unit of execution. A task is similar
to an OS thread, but rather than being managed by the OS scheduler, they are
managed by the [Tokio runtime][rt]. Another name for this general pattern is
[green threads]. If you are familiar with [Go's goroutines], [Kotlin's
coroutines], or [Erlang's processes], you can think of Tokio's tasks as
something similar.

Key points about tasks include:

* Tasks are **light weight**. Because tasks are scheduled by the Tokio
  runtime rather than the operating system, creating new tasks or switching
  between tasks does not require a context switch and has fairly low
  overhead. Creating, running, and destroying large numbers of tasks is
  quite cheap, especially compared to OS threads.

* Tasks are scheduled **cooperatively**. Most operating systems implement
  _preemptive multitasking_. This is a scheduling technique where the
  operating system allows each thread to run for a period of time, and then
  _preempts_ it, temporarily pausing that thread and switching to another.
  Tasks, on the other hand, implement _cooperative multitasking_. In
  cooperative multitasking, a task is allowed to run until it _yields_,
  indicating to the Tokio runtime's scheduler that it cannot currently
  continue executing. When a task yields, the Tokio runtime switches to
  executing the next task.

* Tasks are **non-blocking**. Typically, when an OS thread performs I/O or
  must synchronize with another thread, it _blocks_, allowing the OS to
  schedule another thread. When a task cannot continue executing, it must
  yield instead, allowing the Tokio runtime to schedule another task. Tasks
  should generally not perform system calls or other operations that could
  block a thread, as this would prevent other tasks running on the same
  thread from executing as well. Instead, this module provides APIs for
  running blocking operations in an asynchronous context.

[rt]: crate::runtime
[green threads]: https://en.wikipedia.org/wiki/Green_threads
[Go's goroutines]: https://tour.golang.org/concurrency/1
[Kotlin's coroutines]: https://kotlinlang.org/docs/reference/coroutines-overview.html
[Erlang's processes]: http://erlang.org/doc/getting_started/conc_prog.html#processes

## Working with Tasks

This module provides the following APIs for working with tasks:

### Spawning

Perhaps the most important function in this module is [`task::spawn`]. This
function can be thought of as an async equivalent to the standard library's
[`thread::spawn`][`std::thread::spawn`]. It takes an `async` block or other
[future], and creates a new task to run that work concurrently:

```
use tokio::task;

# async fn doc() {
task::spawn(async {
    // perform some work here...
});
# }
```

Like [`std::thread::spawn`], `task::spawn` returns a [`JoinHandle`] struct.
A `JoinHandle` is itself a future which may be used to await the output of
the spawned task. For example:

```
use tokio::task;

# #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
let join = task::spawn(async {
    // ...
    "hello world!"
});

// ...

// Await the result of the spawned task.
let result = join.await?;
assert_eq!(result, "hello world!");
# Ok(())
# }
```

Again, like `std::thread`'s [`JoinHandle` type][thread_join], if the spawned
task panics, awaiting its `JoinHandle` will return a [`JoinError`]. For
example:

```
use tokio::task;

# #[tokio::main] async fn main() {
let join = task::spawn(async {
    panic!("something bad happened!")
});

// The returned result indicates that the task failed.
assert!(join.await.is_err());
# }
```

`spawn`, `JoinHandle`, and `JoinError` are present when the "rt"
feature flag is enabled.

[`task::spawn`]: crate::task::spawn()
[future]: std::future::Future
[`std::thread::spawn`]: std::thread::spawn
[`JoinHandle`]: crate::task::JoinHandle
[thread_join]: std::thread::JoinHandle
[`JoinError`]: crate::task::JoinError

#### Cancellation

Spawned tasks may be cancelled using the [`JoinHandle::abort`] or
[`AbortHandle::abort`] methods. When one of these methods are called, the
task is signalled to shut down next time it yields at an `.await` point. If
the task is already idle, then it will be shut down as soon as possible
without running again before being shut down. Additionally, shutting down a
Tokio runtime (e.g. by returning from `#[tokio::main]`) immediately cancels
all tasks on it.

When tasks are shut down, it will stop running at whichever `.await` it has
yielded at. All local variables are destroyed by running their destructor.
Once shutdown has completed, awaiting the [`JoinHandle`] will fail with a
[cancelled error](crate::task::JoinError::is_cancelled).

Note that aborting a task does not guarantee that it fails with a cancelled
error, since it may complete normally first. For example, if the task does
not yield to the runtime at any point between the call to `abort` and the
end of the task, then the [`JoinHandle`] will instead report that the task
exited normally.

Be aware that tasks spawned using [`spawn_blocking`] cannot be aborted
because they are not async. If you call `abort` on a `spawn_blocking`
task, then this *will not have any effect*, and the task will continue
running normally. The exception is if the task has not started running
yet; in that case, calling `abort` may prevent the task from starting.

Be aware that calls to [`JoinHandle::abort`] just schedule the task for
cancellation, and will return before the cancellation has completed. To wait
for cancellation to complete, wait for the task to finish by awaiting the
[`JoinHandle`]. Similarly, the [`JoinHandle::is_finished`] method does not
return `true` until the cancellation has finished.

Calling [`JoinHandle::abort`] multiple times has the same effect as calling
it once.

Tokio also provides an [`AbortHandle`], which is like the [`JoinHandle`],
except that it does not provide a mechanism to wait for the task to finish.
Each task can only have one [`JoinHandle`], but it can have more than one
[`AbortHandle`].

[`JoinHandle::abort`]: crate::task::JoinHandle::abort
[`AbortHandle::abort`]: crate::task::AbortHandle::abort
[`AbortHandle`]: crate::task::AbortHandle
[`JoinHandle::is_finished`]: crate::task::JoinHandle::is_finished

### Blocking and Yielding

As we discussed above, code running in asynchronous tasks should not perform
operations that can block. A blocking operation performed in a task running
on a thread that is also running other tasks would block the entire thread,
preventing other tasks from running.

Instead, Tokio provides two APIs for running blocking operations in an
asynchronous context: [`task::spawn_blocking`] and [`task::block_in_place`].

Be aware that if you call a non-async method from async code, that non-async
method is still inside the asynchronous context, so you should also avoid
blocking operations there. This includes destructors of objects destroyed in
async code.

#### `spawn_blocking`

The `task::spawn_blocking` function is similar to the `task::spawn` function
discussed in the previous section, but rather than spawning an
_non-blocking_ future on the Tokio runtime, it instead spawns a
_blocking_ function on a dedicated thread pool for blocking tasks. For
example:

```
use tokio::task;

# async fn docs() {
task::spawn_blocking(|| {
    // do some compute-heavy work or call synchronous code
});
# }
```

Just like `task::spawn`, `task::spawn_blocking` returns a `JoinHandle`
which we can use to await the result of the blocking operation:

```rust
# use tokio::task;
# async fn docs() -> Result<(), Box<dyn std::error::Error>>{
let join = task::spawn_blocking(|| {
    // do some compute-heavy work or call synchronous code
    "blocking completed"
});

let result = join.await?;
assert_eq!(result, "blocking completed");
# Ok(())
# }
```

#### `block_in_place`

When using the [multi-threaded runtime][rt-multi-thread], the [`task::block_in_place`]
function is also available. Like `task::spawn_blocking`, this function
allows running a blocking operation from an asynchronous context. Unlike
`spawn_blocking`, however, `block_in_place` works by transitioning the
_current_ worker thread to a blocking thread, moving other tasks running on
that thread to another worker thread. This can improve performance by avoiding
context switches.

For example:

```
use tokio::task;

# async fn docs() {
let result = task::block_in_place(|| {
    // do some compute-heavy work or call synchronous code
    "blocking completed"
});

assert_eq!(result, "blocking completed");
# }
```

#### `yield_now`

In addition, this module provides a [`task::yield_now`] async function
that is analogous to the standard library's [`thread::yield_now`]. Calling
and `await`ing this function will cause the current task to yield to the
Tokio runtime's scheduler, allowing other tasks to be
scheduled. Eventually, the yielding task will be polled again, allowing it
to execute. For example:

```rust
use tokio::task;

# #[tokio::main] async fn main() {
async {
    task::spawn(async {
        // ...
        println!("spawned task done!")
    });

    // Yield, allowing the newly-spawned task to execute first.
    task::yield_now().await;
    println!("main task done!");
}
# .await;
# }
```

[`task::spawn_blocking`]: crate::task::spawn_blocking
[`task::block_in_place`]: crate::task::block_in_place
[rt-multi-thread]: ../runtime/index.html#threaded-scheduler
[`task::yield_now`]: crate::task::yield_now()
[`thread::yield_now`]: std::thread::yield_now

```rust
pub mod task { /* ... */ }
```

## Module `stream`

Due to the `Stream` trait's inclusion in `std` landing later than Tokio's 1.0
release, most of the Tokio stream utilities have been moved into the [`tokio-stream`]
crate.

# Why was `Stream` not included in Tokio 1.0?

Originally, we had planned to ship Tokio 1.0 with a stable `Stream` type
but unfortunately the [RFC] had not been merged in time for `Stream` to
reach `std` on a stable compiler in time for the 1.0 release of Tokio. For
this reason, the team has decided to move all `Stream` based utilities to
the [`tokio-stream`] crate. While this is not ideal, once `Stream` has made
it into the standard library and the `MSRV` period has passed, we will implement
stream for our different types.

While this may seem unfortunate, not all is lost as you can get much of the
`Stream` support with `async/await` and `while let` loops. It is also possible
to create a `impl Stream` from `async fn` using the [`async-stream`] crate.

[`tokio-stream`]: https://docs.rs/tokio-stream
[`async-stream`]: https://docs.rs/async-stream
[RFC]: https://github.com/rust-lang/rfcs/pull/2996

# Example

Convert a [`sync::mpsc::Receiver`] to an `impl Stream`.

```rust,no_run
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel::<usize>(16);

let stream = async_stream::stream! {
    while let Some(item) = rx.recv().await {
        yield item;
    }
};
```

```rust
pub mod stream { /* ... */ }
```

## Macros

### Macro `pin`

**Attributes:**

- `#[macro_export]`

Pins a value on the stack.

Calls to `async fn` return anonymous [`Future`] values that are `!Unpin`.
These values must be pinned before they can be polled. Calling `.await` will
handle this, but consumes the future. If it is required to call `.await` on
a `&mut _` reference, the caller is responsible for pinning the future.

Pinning may be done by allocating with [`Box::pin`] or by using the stack
with the `pin!` macro.

The following will **fail to compile**:

```compile_fail
async fn my_async_fn() {
    // async logic here
}

#[tokio::main]
async fn main() {
    let mut future = my_async_fn();
    (&mut future).await;
}
```

To make this work requires pinning:

```
use tokio::pin;

async fn my_async_fn() {
    // async logic here
}

#[tokio::main]
async fn main() {
    let future = my_async_fn();
    pin!(future);

    (&mut future).await;
}
```

Pinning is useful when using `select!` and stream operators that require `T:
Stream + Unpin`.

[`Future`]: trait@std::future::Future
[`Box::pin`]: std::boxed::Box::pin

# Usage

The `pin!` macro takes **identifiers** as arguments. It does **not** work
with expressions.

The following does not compile as an expression is passed to `pin!`.

```compile_fail
async fn my_async_fn() {
    // async logic here
}

#[tokio::main]
async fn main() {
    let mut future = pin!(my_async_fn());
    (&mut future).await;
}
```

# Examples

Using with select:

```
use tokio::{pin, select};
use tokio_stream::{self as stream, StreamExt};

async fn my_async_fn() {
    // async logic here
}

#[tokio::main]
async fn main() {
    let mut stream = stream::iter(vec![1, 2, 3, 4]);

    let future = my_async_fn();
    pin!(future);

    loop {
        select! {
            _ = &mut future => {
                // Stop looping `future` will be polled after completion
                break;
            }
            Some(val) = stream.next() => {
                println!("got value = {}", val);
            }
        }
    }
}
```

Because assigning to a variable followed by pinning is common, there is also
a variant of the macro that supports doing both in one go.

```
use tokio::{pin, select};

async fn my_async_fn() {
    // async logic here
}

#[tokio::main]
async fn main() {
    pin! {
        let future1 = my_async_fn();
        let future2 = my_async_fn();
    }

    select! {
        _ = &mut future1 => {}
        _ = &mut future2 => {}
    }
}
```

```rust
pub macro_rules! pin {
    /* macro_rules! pin {
    ($($x:ident),*) => { ... };
    ($(
            let $x:ident = $init:expr;
    )*) => { ... };
} */
}
```

