# Trait `AsyncRead`

Reads bytes from a source.
This trait is analogous to the [`std::io::Read`] trait, but integrates with
the asynchronous task system. In particular, the [`poll_read`] method,
unlike [`Read::read`], will automatically queue the current task for wakeup
and return if data is not yet available, rather than blocking the calling
thread.
Specifically, this means that the `poll_read` function will return one of
the following:
* `Poll::Ready(Ok(()))` means that data was immediately read and placed into
the output buffer. The amount of data read can be determined by the
increase in the length of the slice returned by `ReadBuf::filled`. If the
difference is 0, either EOF has been reached, or the output buffer had zero
capacity (i.e. `buf.remaining()` == 0).
* `Poll::Pending` means that no data was read into the buffer
provided. The I/O object is not currently readable but may become readable
in the future. Most importantly, **the current future's task is scheduled
to get unparked when the object is readable**. This means that like
`Future::poll` you'll receive a notification when the I/O object is
readable again.
* `Poll::Ready(Err(e))` for other errors are standard I/O errors coming from the
underlying object.
This trait importantly means that the `read` method only works in the
context of a future's task. The object may panic if used outside of a task.
Utilities for working with `AsyncRead` values are provided by
[`AsyncReadExt`].
[`poll_read`]: AsyncRead::poll_read
[`std::io::Read`]: std::io::Read
[`Read::read`]: std::io::Read::read
[`AsyncReadExt`]: crate::io::AsyncReadExt

## Associated Items

### Method `poll_read`

Attempts to read from the `AsyncRead` into `buf`.
On success, returns `Poll::Ready(Ok(()))` and places data in the
unfilled portion of `buf`. If no data was read (`buf.filled().len()` is
unchanged), it implies that EOF has been reached.
If no data is available for reading, the method returns `Poll::Pending`
and arranges for the current task (via `cx.waker()`) to receive a
notification when the object becomes readable or is closed.

