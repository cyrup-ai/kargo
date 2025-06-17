# Trait `AsyncSeek`

Seek bytes asynchronously.
This trait is analogous to the [`std::io::Seek`] trait, but integrates
with the asynchronous task system. In particular, the `start_seek`
method, unlike [`Seek::seek`], will not block the calling thread.
Utilities for working with `AsyncSeek` values are provided by
[`AsyncSeekExt`].
[`std::io::Seek`]: std::io::Seek
[`Seek::seek`]: std::io::Seek::seek()
[`AsyncSeekExt`]: crate::io::AsyncSeekExt

## Associated Items

### Method `start_seek`

Attempts to seek to an offset, in bytes, in a stream.
A seek beyond the end of a stream is allowed, but behavior is defined
by the implementation.
If this function returns successfully, then the job has been submitted.
To find out when it completes, call `poll_complete`.
# Errors
This function can return [`io::ErrorKind::Other`] in case there is
another seek in progress. To avoid this, it is advisable that any call
to `start_seek` is preceded by a call to `poll_complete` to ensure all
pending seeks have completed.

### Method `poll_complete`

Waits for a seek operation to complete.
If the seek operation completed successfully,
this method returns the new position from the start of the stream.
That position can be used later with [`SeekFrom::Start`]. Repeatedly
calling this function without calling `start_seek` might return the
same result.
# Errors
Seeking to a negative offset is considered an error.

