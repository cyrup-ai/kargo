# Module `stream`

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

