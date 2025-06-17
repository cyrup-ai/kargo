# Module `any`

Dynamic support for any engine
SurrealDB supports various ways of storing and accessing your data. For storing data we support a number of
key value stores. These are SurrealKV, RocksDB, TiKV, FoundationDB and an in-memory store. We call these
local engines. SurrealKV and RocksDB are file-based, single node key value stores. TiKV and FoundationDB are
are distributed stores that can scale horizontally across multiple nodes. The in-memory store does not persist
your data, it only stores it in memory. All these can be embedded in your application, so you don't need to
spin up a SurrealDB server first in order to use them. We also support spinning up a server externally and then
access your database via WebSockets or HTTP. We call these remote engines.
The Rust SDK abstracts away the implementation details of the engines to make them work in a unified way.
All these engines, whether they are local or remote, work exactly the same way using the same API. The only
difference in the API is the endpoint you use to access the engine. Normally you provide the scheme of the engine
you want to use as a type parameter to `Surreal::new`. This allows you detect, at compile, whether the engine
you are trying to use is enabled. If not, your code won't compile. This is awesome but it strongly couples your
application to the engine you are using. In order to change an engine you would need to update your code to
the new scheme and endpoint you need to use and recompile it. This is where the `any` engine comes in. We will
call it `Surreal<Any>` (the type it creates) to avoid confusion with the word any.
`Surreal<Any>` allows you to use any engine as long as it was enabled when compiling. Unlike with the typed scheme,
the choice of the engine is made at runtime depending on the endpoint that you provide as a string. If you use an
environment variable to provide this endpoint string, you won't need to change your code  in order to
switch engines. The downside to this is that you will get a runtime error if you forget to enable the engine you
want to use when compiling your code. On the other hand, this totally decouples your application from the engine
you are using and makes it possible to use whichever engine SurrealDB supports by simply changing the Cargo
features you enable when compiling. This enables some cool workflows.
One of the common use cases we see is using SurrealDB as an embedded database using RocksDB as the local engine.
This is a nice way to boost the performance of your application when all you need is a single node. The downside
of this approach is that RocksDB is not written in Rust so you will need to install some external dependencies
on your development machine in order to successfully compile it. Some of our users have reported that
this is not exactly straight-forward on Windows. Another issue is that RocksDB is very resource intensive to
compile and it takes a long time. Both of these issues can be easily avoided by using `Surreal<Any>`. You can
develop using an in-memory engine but deploy using RocksDB. If you develop on Windows but deploy to Linux then
you completely avoid having to build RocksDB on Windows at all.
# Getting Started
You can start by declaring your `surrealdb` dependency like this in Cargo.toml
```toml
surrealdb = {
version = "1",
# Disables the default features, which are `protocol-ws` and `rustls`.
# Not necessary but can reduce your compile times if you don't need those features.
default-features = false,
# Unconditionally enables the in-memory store.
# Also not necessary but this will make `cargo run` just work.
# Without it, you would need `cargo run --features surrealdb/kv-mem` during development. If you use a build
# tool like `make` or `cargo make`, however, you can put that in your build step and avoid typing it manually.
features = ["kv-mem"],
# Also not necessary but this makes it easy to switch between `stable`, `beta` and `nightly` crates, if need be.
# See https://surrealdb.com/blog/introducing-nightly-and-beta-rust-crates for more information on those crates.
package = "surrealdb"
}
```
You then simply need to instantiate `Surreal<Any>` instead of `Surreal<Db>` or `Surreal<Client>`.
# Examples
```rust
use std::env;
use surrealdb::engine::any;
use surrealdb::engine::any::Any;
use surrealdb::opt::Resource;
use surrealdb::Surreal;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
// Use the endpoint specified in the environment variable or default to `memory`.
// This makes it possible to use the memory engine during development but switch it
// to any other engine for deployment.
let endpoint = env::var("SURREALDB_ENDPOINT").unwrap_or_else(|_| "memory".to_owned());
// Create the Surreal instance. This will create `Surreal<Any>`.
let db = any::connect(endpoint).await?;
// Specify the namespace and database to use
db.use_ns("namespace").use_db("database").await?;
// Use the database like you normally would.
delete_user(&db, "jane").await?;
Ok(())
}
// Deletes a user from the user table in the database
async fn delete_user(db: &Surreal<Any>, username: &str) -> surrealdb::Result<()> {
db.delete(Resource::from(("user", username))).await?;
Ok(())
}
```
By doing something like this, you can use an in-memory database on your development machine and you can just run `cargo run`
without having to specify the environment variable first or spinning up an external server remotely to avoid RocksDB's
compilation cost. You also don't need to install any `C` or `C++` dependencies on your Windows machine. For the production
binary you simply need to build it using something like
```bash
cargo build --features surrealdb/kv-rocksdb --release
```
and export the `SURREALDB_ENDPOINT` environment variable when starting it.
```bash
export SURREALDB_ENDPOINT="rocksdb:/path/to/database/folder"
/path/to/binary
```
The example above shows how you can avoid compiling RocksDB on your development machine, thereby avoiding dependency hell
and paying the compilation cost during development. This is not the only benefit you can derive from using `Surreal<Any>`
though. It's still useful even when your engine isn't expensive to compile. For example, the remote engines use pure Rust
dependencies but you can still benefit from using `Surreal<Any>` by using the in-memory engine for development and deploy
using a remote engine like the WebSocket engine. This way you avoid having to spin up a SurrealDB server first when
developing and testing your application.
For some applications where you allow users to determine the engine they want to use, you can enable multiple engines for
them when building, or even enable them all. To do this you simply need to comma separate the Cargo features.
```bash
cargo build --features surrealdb/protocol-ws,surrealdb/kv-rocksdb,surrealdb/kv-tikv --release
```
In this case, the binary you build will have support for accessing an external server via WebSockets, embedding the database
using RocksDB or using a distributed TiKV cluster.

## Contents

* **Trait** `IntoEndpoint` - A trait for converting inputs to a server address object
* **Struct** `Any` - A dynamic connection that supports any engine and allows you to pick at runtime
* **Function** `connect` - Connects to a local, remote or embedded database...

