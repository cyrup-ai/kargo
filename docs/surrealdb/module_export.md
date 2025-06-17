# Module `export`

Dumps the database contents to a file
# Support
Currently only supported by HTTP and the local engines. *Not* supported on WebAssembly.
# Examples
```no_run
# use futures::StreamExt;
# #[tokio::main]
# async fn main() -> surrealdb::Result<()> {
# let db = surrealdb::engine::any::connect("mem://").await?;
// Select the namespace/database to use
db.use_ns("namespace").use_db("database").await?;
// Export to a file
db.export("backup.sql").await?;
// Export to a stream of bytes
let mut backup = db.export(()).await?;
while let Some(result) = backup.next().await {
match result {
Ok(bytes) => {
// Do something with the bytes received...
}
Err(error) => {
// Handle the export error
}
}
}
# Ok(())
# }
```

## Contents

* **Enum** `ExportDestination`
* **Trait** `IntoExportDestination` - A trait for converting inputs into database export locations

