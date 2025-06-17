# Functions

## `advance`

Advances the size of the filled region of the buffer....

## `assume_init`

Asserts that the first `n` unfilled bytes of the buffer are initialized....

## `borrow`

## `borrow_mut`

## `capacity`

Returns the total capacity of the buffer.

## `clear`

Clears the buffer, resetting the filled region to empty.

## `consume`

Tells this buffer that `amt` bytes have been consumed from the buffer, so they should no longer be returned in calls to [`poll_read`]....

## `consume`

## `consume`

## `consume`

## `consume`

## `consume`

## `filled`

Returns a shared reference to the filled portion of the buffer.

## `filled_mut`

Returns a mutable reference to the filled portion of the buffer.

## `fmt`

## `from`

Returns the argument unchanged.

## `initialize_unfilled`

Returns a mutable reference to the unfilled part of the buffer, ensuring it is fully initialized....

## `initialize_unfilled_to`

Returns a mutable reference to the first `n` bytes of the unfilled part of the buffer, ensuring it is fully initialized.

## `initialized`

Returns a shared reference to the initialized portion of the buffer.

## `initialized_mut`

Returns a mutable reference to the initialized portion of the buffer.

## `inner_mut`

Returns a mutable reference to the entire buffer, without ensuring that it has been fully initialized....

## `into`

Calls `U::from(self)`.

## `is_write_vectored`

## `is_write_vectored`

## `is_write_vectored`

## `is_write_vectored`

## `is_write_vectored`

## `is_write_vectored`

## `is_write_vectored`

## `is_write_vectored`

Determines if this writer has an efficient [`poll_write_vectored`] implementation....

## `is_write_vectored`

## `new`

Creates a new `ReadBuf` from a fully initialized buffer.

## `poll_complete`

## `poll_complete`

## `poll_complete`

## `poll_complete`

Waits for a seek operation to complete....

## `poll_complete`

## `poll_fill_buf`

## `poll_fill_buf`

## `poll_fill_buf`

## `poll_fill_buf`

Attempts to return the contents of the internal buffer, filling it with more data from the inner reader if it is empty....

## `poll_fill_buf`

## `poll_fill_buf`

## `poll_flush`

Attempts to flush the object, ensuring that any buffered data reach their destination....

## `poll_flush`

## `poll_flush`

## `poll_flush`

## `poll_flush`

## `poll_flush`

## `poll_flush`

## `poll_flush`

## `poll_flush`

## `poll_read`

## `poll_read`

## `poll_read`

Attempts to read from the `AsyncRead` into `buf`....

## `poll_read`

## `poll_read`

## `poll_read`

## `poll_shutdown`

## `poll_shutdown`

## `poll_shutdown`

## `poll_shutdown`

## `poll_shutdown`

## `poll_shutdown`

## `poll_shutdown`

Initiates or attempts to shut down this writer, returning success when the I/O connection has completely shut down....

## `poll_shutdown`

## `poll_shutdown`

## `poll_write`

## `poll_write`

## `poll_write`

## `poll_write`

## `poll_write`

## `poll_write`

## `poll_write`

## `poll_write`

## `poll_write`

Attempt to write bytes from `buf` into the object....

## `poll_write_vectored`

Like [`poll_write`], except that it writes from a slice of buffers....

## `poll_write_vectored`

## `poll_write_vectored`

## `poll_write_vectored`

## `poll_write_vectored`

## `poll_write_vectored`

## `poll_write_vectored`

## `poll_write_vectored`

## `poll_write_vectored`

## `put_slice`

Appends data to the buffer, advancing the written position and possibly also the initialized position.

## `remaining`

Returns the number of bytes at the end of the slice that have not yet been filled.

## `set_filled`

Sets the size of the filled region of the buffer....

## `start_seek`

## `start_seek`

## `start_seek`

Attempts to seek to an offset, in bytes, in a stream....

## `start_seek`

## `start_seek`

## `take`

Returns a new `ReadBuf` comprised of the unfilled section up to `n`.

## `try_from`

## `try_into`

## `type_id`

## `unfilled_mut`

Returns a mutable reference to the unfilled part of the buffer without ensuring that it has been fully initialized....

## `uninit`

Creates a new `ReadBuf` from a buffer that may be uninitialized....

