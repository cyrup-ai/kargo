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

## Contents

* **Module** `error` - Different error types for embedded and remote databases
* **Enum** `Error` - An error originating from the SurrealDB client library

