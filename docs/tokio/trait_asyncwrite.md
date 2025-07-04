# Trait `AsyncWrite`

Writes bytes asynchronously.
The trait inherits from [`std::io::Write`] and indicates that an I/O object is
**nonblocking**. All non-blocking I/O objects must return an error when
bytes cannot be written instead of blocking the current thread.
Specifically, this means that the [`poll_write`] function will return one of
the following:
* `Poll::Ready(Ok(n))` means that `n` bytes of data was immediately
written.
* `Poll::Pending` means that no data was written from the buffer
provided. The I/O object is not currently writable but may become writable
in the future. Most importantly, **the current future's task is scheduled
to get unparked when the object is writable**. This means that like
`Future::poll` you'll receive a notification when the I/O object is
writable again.
* `Poll::Ready(Err(e))` for other errors are standard I/O errors coming from the
underlying object.
This trait importantly means that the [`write`][stdwrite] method only works in
the context of a future's task. The object may panic if used outside of a task.
Note that this trait also represents that the  [`Write::flush`][stdflush] method
works very similarly to the `write` method, notably that `Ok(())` means that the
writer has successfully been flushed, a "would block" error means that the
current task is ready to receive a notification when flushing can make more
progress, and otherwise normal errors can happen as well.
Utilities for working with `AsyncWrite` values are provided by
[`AsyncWriteExt`].
[`std::io::Write`]: std::io::Write
[`poll_write`]: AsyncWrite::poll_write()
[stdwrite]: std::io::Write::write()
[stdflush]: std::io::Write::flush()
[`AsyncWriteExt`]: crate::io::AsyncWriteExt

## Associated Items

### Method `poll_write`

Attempt to write bytes from `buf` into the object.
On success, returns `Poll::Ready(Ok(num_bytes_written))`. If successful,
then it must be guaranteed that `n <= buf.len()`. A return value of `0`
typically means that the underlying object is no longer able to accept
bytes and will likely not be able to in the future as well, or that the
buffer provided is empty.
If the object is not ready for writing, the method returns
`Poll::Pending` and arranges for the current task (via
`cx.waker()`) to receive a notification when the object becomes
writable or is closed.

### Method `poll_flush`

Attempts to flush the object, ensuring that any buffered data reach
their destination.
On success, returns `Poll::Ready(Ok(()))`.
If flushing cannot immediately complete, this method returns
`Poll::Pending` and arranges for the current task (via
`cx.waker()`) to receive a notification when the object can make
progress towards flushing.

### Method `poll_shutdown`

Initiates or attempts to shut down this writer, returning success when
the I/O connection has completely shut down.
This method is intended to be used for asynchronous shutdown of I/O
connections. For example this is suitable for implementing shutdown of a
TLS connection or calling `TcpStream::shutdown` on a proxied connection.
Protocols sometimes need to flush out final pieces of data or otherwise
perform a graceful shutdown handshake, reading/writing more data as
appropriate. This method is the hook for such protocols to implement the
graceful shutdown logic.
This `shutdown` method is required by implementers of the
`AsyncWrite` trait. Wrappers typically just want to proxy this call
through to the wrapped type, and base types will typically implement
shutdown logic here or just return `Ok(().into())`. Note that if you're
wrapping an underlying `AsyncWrite` a call to `shutdown` implies that
transitively the entire stream has been shut down. After your wrapper's
shutdown logic has been executed you should shut down the underlying
stream.
Invocation of a `shutdown` implies an invocation of `flush`. Once this
method returns `Ready` it implies that a flush successfully happened
before the shutdown happened. That is, callers don't need to call
`flush` before calling `shutdown`. They can rely that by calling
`shutdown` any pending buffered data will be written out.
# Return value
This function returns a `Poll<io::Result<()>>` classified as such:
* `Poll::Ready(Ok(()))` - indicates that the connection was
successfully shut down and is now safe to deallocate/drop/close
resources associated with it. This method means that the current task
will no longer receive any notifications due to this method and the
I/O object itself is likely no longer usable.
* `Poll::Pending` - indicates that shutdown is initiated but could
not complete just yet. This may mean that more I/O needs to happen to
continue this shutdown operation. The current task is scheduled to
receive a notification when it's otherwise ready to continue the
shutdown operation. When woken up this method should be called again.
* `Poll::Ready(Err(e))` - indicates a fatal error has happened with shutdown,
indicating that the shutdown operation did not complete successfully.
This typically means that the I/O object is no longer usable.
# Errors
This function can return normal I/O errors through `Err`, described
above. Additionally this method may also render the underlying
`Write::write` method no longer usable (e.g. will return errors in the
future). It's recommended that once `shutdown` is called the
`write` method is no longer called.
# Panics
This function will panic if not called within the context of a future's
task.

### Method `poll_write_vectored`

Like [`poll_write`], except that it writes from a slice of buffers.
Data is copied from each buffer in order, with the final buffer
read from possibly being only partially consumed. This method must
behave as a call to [`write`] with the buffers concatenated would.
The default implementation calls [`poll_write`] with either the first nonempty
buffer provided, or an empty one if none exists.
On success, returns `Poll::Ready(Ok(num_bytes_written))`.
If the object is not ready for writing, the method returns
`Poll::Pending` and arranges for the current task (via
`cx.waker()`) to receive a notification when the object becomes
writable or is closed.
# Note
This should be implemented as a single "atomic" write action. If any
data has been partially written, it is wrong to return an error or
pending.
[`poll_write`]: AsyncWrite::poll_write

### Method `is_write_vectored`

Determines if this writer has an efficient [`poll_write_vectored`]
implementation.
If a writer does not override the default [`poll_write_vectored`]
implementation, code using it may want to avoid the method all together
and coalesce writes into a single buffer for higher performance.
The default implementation returns `false`.
[`poll_write_vectored`]: AsyncWrite::poll_write_vectored

