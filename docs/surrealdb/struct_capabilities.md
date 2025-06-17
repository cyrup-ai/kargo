# Struct `Capabilities`

Capabilities are used to limit what users are allowed to do using queries.
Capabilities are split into categories:
- Scripting: Whether or not users can execute scripts
- Guest access: Whether or not unauthenticated users can execute queries
- Functions: Whether or not users can execute certain functions
- Network: Whether or not users can connect to certain network addresses
Capabilities are configured globally. By default, capabilities are configured as:
- Scripting: false
- Guest access: false
- Functions: All functions are allowed
- Network: No network address is allowed, all are impliticly denied
The capabilities are defined using allow/deny lists for fine-grained control.
# Filtering functions and net-targets.
The filtering of net targets and functions is done with an allow/deny list.
These list can either match everything, nothing or a given list.
By default every function and net-target is disallowed. For a function or net target to be
allowed it must match the allow-list and not match the deny-list. This means that if for
example a function is both in the allow-list and in the deny-list it will be disallowed.
With the combination of both these lists you can filter subgroups. For example:
```
# use surrealdb::opt::capabilities::Capabilities;
# fn cap() -> surrealdb::Result<Capabilities>{
# let cap =
Capabilities::none()
.with_allow_function("http::*")?
.with_deny_function("http::post")?
# ;
# Ok(cap)
# }
```
Will allow all and only all `http::*` functions except the function `http::post`.
Examples:
- Allow all functions: `--allow-funcs`
- Allow all functions except `http.*`: `--allow-funcs --deny-funcs 'http.*'`
- Allow all network addresses except AWS metadata endpoint: `--allow-net --deny-net='169.254.169.254'`
# Examples
Create a new instance, and allow all capabilities
```ignore
# use surrealdb::opt::capabilities::Capabilities;
# use surrealdb::opt::Config;
# use surrealdb::Surreal;
# use surrealdb::engine::local::File;
# #[tokio::main]
# async fn main() -> surrealdb::Result<()> {
let capabilities = Capabilities::all();
let config = Config::default().capabilities(capabilities);
let db = Surreal::new::<File>(("temp.db", config)).await?;
# Ok(())
# }
```
Create a new instance, and allow certain functions
```ignore
# use std::str::FromStr;
# use surrealdb::engine::local::File;
# use surrealdb::opt::capabilities::Capabilities;
# use surrealdb::opt::Config;
# use surrealdb::Surreal;
# #[tokio::main]
# async fn main() -> surrealdb::Result<()> {
let capabilities = Capabilities::default()
.with_deny_function("http::*")?;
let config = Config::default().capabilities(capabilities);
let db = Surreal::new::<File>(("temp.db", config)).await?;
# Ok(())
# }
```

## Fields

Field information will be available in a future version.

