# Crate Documentation

**Version:** 2.3.3

**Format Version:** 46

# Module `surrealdb`

This library provides a low-level database library implementation, a remote client
and a query language definition, for [SurrealDB](https://surrealdb.com), the ultimate cloud database for
tomorrow's applications. SurrealDB is a scalable, distributed, collaborative, document-graph
database for the realtime web.

This library can be used to start an embedded in-memory datastore, an embedded datastore
persisted to disk, a browser-based embedded datastore backed by IndexedDB, or for connecting
to a distributed [TiKV](https://tikv.org) key-value store.

It also enables simple and advanced querying of a remote SurrealDB server from
server-side or client-side code. All connections to SurrealDB are made over WebSockets by default,
and automatically reconnect when the connection is terminated.

# Examples

```no_run
use std::borrow::Cow;
use serde::{Serialize, Deserialize};
use serde_json::json;
use surrealdb::{Error, Surreal};
use surrealdb::opt::auth::Root;
use surrealdb::engine::remote::ws::Ws;

#[derive(Serialize, Deserialize)]
struct Person {
    title: String,
    name: Name,
    marketing: bool,
}

// Pro tip: Replace String with Cow<'static, str> to
// avoid unnecessary heap allocations when inserting

#[derive(Serialize, Deserialize)]
struct Name {
    first: Cow<'static, str>,
    last: Cow<'static, str>,
}

// Install at https://surrealdb.com/install
// and use `surreal start --user root --pass root`
// to start a working database to take the following queries
// See the results via `surreal sql --ns namespace --db database --pretty`
// or https://surrealist.app/
// followed by the query `SELECT * FROM person;`
#[tokio::main]
async fn main() -> Result<(), Error> {
    let db = Surreal::new::<Ws>("localhost:8000").await?;

    // Signin as a namespace, database, or root user
    db.signin(Root {
        username: "root",
        password: "root",
    }).await?;

    // Select a specific namespace / database
    db.use_ns("namespace").use_db("database").await?;

    // Create a new person with a random ID
    let created: Option<Person> = db.create("person")
        .content(Person {
            title: "Founder & CEO".into(),
            name: Name {
                first: "Tobie".into(),
                last: "Morgan Hitchcock".into(),
            },
            marketing: true,
        })
        .await?;

    // Create a new person with a specific ID
    let created: Option<Person> = db.create(("person", "jaime"))
        .content(Person {
            title: "Founder & COO".into(),
            name: Name {
                first: "Jaime".into(),
                last: "Morgan Hitchcock".into(),
            },
            marketing: false,
        })
        .await?;

    // Update a person record with a specific ID
    let updated: Option<Person> = db.update(("person", "jaime"))
        .merge(json!({"marketing": true}))
        .await?;

    // Select all people records
    let people: Vec<Person> = db.select("person").await?;

    // Perform a custom advanced query
    let query = r#"
        SELECT marketing, count()
        FROM type::table($table)
        GROUP BY marketing
    "#;

    let groups = db.query(query)
        .bind(("table", "person"))
        .await?;

    Ok(())
}
```

## Modules

## Module `error`

Different error types for embedded and remote databases

```rust
pub mod error { /* ... */ }
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

## Types

### Enum `Error`

An error originating from the SurrealDB client library

```rust
pub enum Error {
    Db(crate::error::Db),
    Api(crate::error::Api),
}
```

#### Variants

##### `Db`

An error with an embedded storage engine

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `crate::error::Db` |  |

##### `Api`

An error with a remote database instance

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `crate::error::Api` |  |

#### Implementations

##### Trait Implementations

- **Instrument**
- **Error**
  - ```rust
    fn source(self: &Self) -> ::core::option::Option<&dyn std::error::Error + ''static> { /* ... */ }
    ```

- **Same**
- **Display**
  - ```rust
    fn fmt(self: &Self, __formatter: &mut ::core::fmt::Formatter<''_>) -> ::core::fmt::Result { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **WithSubscriber**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

  - ```rust
    fn from(_: Infallible) -> Self { /* ... */ }
    ```

  - ```rust
    fn from(e: ParseNetTargetError) -> Self { /* ... */ }
    ```

  - ```rust
    fn from(e: ParseFuncTargetError) -> Self { /* ... */ }
    ```

  - ```rust
    fn from(error: tokio_tungstenite::tungstenite::Error) -> Self { /* ... */ }
    ```

  - ```rust
    fn from(error: async_channel::SendError<T>) -> Self { /* ... */ }
    ```

  - ```rust
    fn from(error: async_channel::RecvError) -> Self { /* ... */ }
    ```

  - ```rust
    fn from(error: url::ParseError) -> Self { /* ... */ }
    ```

  - ```rust
    fn from(source: crate::error::Db) -> Self { /* ... */ }
    ```

  - ```rust
    fn from(source: crate::error::Api) -> Self { /* ... */ }
    ```

- **Pointable**
  - ```rust
    unsafe fn init(init: <T as Pointable>::Init) -> usize { /* ... */ }
    ```

  - ```rust
    unsafe fn deref<''a>(ptr: usize) -> &''a T { /* ... */ }
    ```

  - ```rust
    unsafe fn deref_mut<''a>(ptr: usize) -> &''a mut T { /* ... */ }
    ```

  - ```rust
    unsafe fn drop(ptr: usize) { /* ... */ }
    ```

- **ToString**
  - ```rust
    fn to_string(self: &Self) -> String { /* ... */ }
    ```

- **Sync**
- **Freeze**
- **UnwindSafe**
- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **ErasedDestructor**
- **Within**
  - ```rust
    fn is_within(self: &Self, b: &G2) -> bool { /* ... */ }
    ```

- **IntoEither**
- **ToSmolStr**
  - ```rust
    fn to_smolstr(self: &Self) -> SmolStr { /* ... */ }
    ```

- **Send**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Unpin**
- **RefUnwindSafe**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

## Other Items

### Unnamed Item

**⚠️ Deprecated since 2.3.0**

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

### Unnamed Item

**Attributes:**

- `#[doc(inline)]`

```rust
pub /* unnamed item */
```

