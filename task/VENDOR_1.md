# VENDOR_1: Fix Async/Await Consistency in Vendoring

**Area:** Vendor Manager
**Priority:** HIGH (Critical architectural flaw)
**Estimated Effort:** 15 minutes

---

## CRITICAL ISSUE

The vendoring implementation uses **blocking I/O operations inside async functions**, violating async/await best practices and the tokio runtime execution model. This causes the tokio thread pool to block on filesystem operations, degrading performance and potentially causing runtime starvation under load.

**Location:** [`kargo-cli/src/vendor.rs`](../kargo-cli/src/vendor.rs)

## ANALYSIS: WHAT'S ALREADY CORRECT

The codebase **already demonstrates correct async patterns** in multiple places:

1. ✅ **`copy_dir_recursive()` (lines 90-117)** - Already fully async:
   ```rust
   async fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
       let mut entries = tokio::fs::read_dir(src).await?;  // ✅ Correct
       while let Some(entry) = entries.next_entry().await? {
           let file_type = entry.file_type().await?;        // ✅ Correct
           // ...
       }
   }
   ```

2. ✅ **[`host_functions.rs:101`](../kargo-cli/src/plugins/host_functions.rs#L101)** - Uses async file I/O:
   ```rust
   let res = tokio::fs::read_to_string(&path).await;  // ✅ Correct pattern
   ```

3. ✅ All functional requirements met: checksum generation, directory format, error handling

## BLOCKING OPERATIONS TO FIX

### 1. `find_package_source()` - Lines 10-39

**Current Implementation (BLOCKING):**
```rust
fn find_package_source(pkg: &Package) -> Result<PathBuf> {
    // ... CARGO_HOME logic ...

    for entry in std::fs::read_dir(&registry_src)?  {  // ❌ BLOCKING
        let index_dir = entry?.path();
        if !index_dir.is_dir() {                       // ❌ BLOCKING (metadata check)
            continue;
        }
        let pkg_dir = index_dir.join(format!("{}-{}", pkg.name, pkg.version));
        if pkg_dir.exists() {                          // ❌ BLOCKING
            return Ok(pkg_dir);
        }
    }
    // ...
}
```

**Fixed Implementation (ASYNC):**
```rust
async fn find_package_source(pkg: &Package) -> Result<PathBuf> {
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| {
        dirs::home_dir()
            .map(|p| p.join(".cargo").to_string_lossy().to_string())
            .unwrap_or_else(|| ".cargo".to_string())
    });

    let registry_src = PathBuf::from(cargo_home).join("registry").join("src");

    // ✅ Use tokio::fs::read_dir for async iteration
    let mut entries = tokio::fs::read_dir(&registry_src)
        .await
        .with_context(|| format!("Failed to read registry src directory: {:?}", registry_src))?;

    while let Some(entry) = entries.next_entry().await? {
        let index_dir = entry.path();

        // ✅ Use tokio::fs::metadata for async metadata check
        let metadata = tokio::fs::metadata(&index_dir).await;
        if metadata.is_err() || !metadata?.is_dir() {
            continue;
        }

        let pkg_dir = index_dir.join(format!("{}-{}", pkg.name, pkg.version));

        // ✅ Use tokio::fs::try_exists for async existence check
        if tokio::fs::try_exists(&pkg_dir).await.unwrap_or(false) {
            return Ok(pkg_dir);
        }
    }

    anyhow::bail!(
        "Package source not found: {}-{}\nMake sure the package is downloaded (run `cargo fetch` first)",
        pkg.name,
        pkg.version
    );
}
```

**Changes Required:**
- Change function signature from `fn` to `async fn`
- Replace `std::fs::read_dir()` with `tokio::fs::read_dir().await`
- Replace `for entry in` with `while let Some(entry) = entries.next_entry().await?`
- Replace `index_dir.is_dir()` with `tokio::fs::metadata(&index_dir).await?.is_dir()`
- Replace `pkg_dir.exists()` with `tokio::fs::try_exists(&pkg_dir).await.unwrap_or(false)`

---

### 2. `find_crate_file()` - Lines 55-87

**Current Implementation (BLOCKING):**
```rust
fn find_crate_file(pkg: &Package) -> Result<PathBuf> {
    // ... CARGO_HOME logic ...

    for entry in std::fs::read_dir(&registry_cache)? {  // ❌ BLOCKING
        let index_dir = entry?.path();
        if !index_dir.is_dir() {                        // ❌ BLOCKING
            continue;
        }
        let crate_file = index_dir.join(format!("{}-{}.crate", pkg.name, pkg.version));
        if crate_file.exists() {                        // ❌ BLOCKING
            return Ok(crate_file);
        }
    }
    // ...
}
```

**Fixed Implementation (ASYNC):**
```rust
async fn find_crate_file(pkg: &Package) -> Result<PathBuf> {
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| {
        dirs::home_dir()
            .map(|p| p.join(".cargo").to_string_lossy().to_string())
            .unwrap_or_else(|| ".cargo".to_string())
    });

    let registry_cache = PathBuf::from(cargo_home).join("registry").join("cache");

    // ✅ Use tokio::fs::read_dir for async iteration
    let mut entries = tokio::fs::read_dir(&registry_cache)
        .await
        .with_context(|| {
            format!(
                "Failed to read registry cache directory: {:?}",
                registry_cache
            )
        })?;

    while let Some(entry) = entries.next_entry().await? {
        let index_dir = entry.path();

        // ✅ Use tokio::fs::metadata for async metadata check
        let metadata = tokio::fs::metadata(&index_dir).await;
        if metadata.is_err() || !metadata?.is_dir() {
            continue;
        }

        let crate_file = index_dir.join(format!("{}-{}.crate", pkg.name, pkg.version));

        // ✅ Use tokio::fs::try_exists for async existence check
        if tokio::fs::try_exists(&crate_file).await.unwrap_or(false) {
            return Ok(crate_file);
        }
    }

    anyhow::bail!(
        "Crate file not found: {}-{}.crate\nMake sure the package is downloaded (run `cargo fetch` first)",
        pkg.name,
        pkg.version
    );
}
```

**Changes Required:**
- Change function signature from `fn` to `async fn`
- Replace `std::fs::read_dir()` with `tokio::fs::read_dir().await`
- Replace `for entry in` with `while let Some(entry) = entries.next_entry().await?`
- Replace `index_dir.is_dir()` with `tokio::fs::metadata(&index_dir).await?.is_dir()`
- Replace `crate_file.exists()` with `tokio::fs::try_exists(&crate_file).await.unwrap_or(false)`

---

### 3. `compute_sha256()` - Lines 42-52

**Current Implementation (BLOCKING):**
```rust
fn compute_sha256(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)?;        // ❌ BLOCKING
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;                // ❌ BLOCKING
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}
```

**Fixed Implementation (ASYNC):**
```rust
async fn compute_sha256(path: &Path) -> Result<String> {
    use tokio::io::AsyncReadExt;

    // ✅ Use tokio::fs::File for async file operations
    let mut file = tokio::fs::File::open(path)
        .await
        .with_context(|| format!("Failed to open file for hashing: {:?}", path))?;

    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192];  // 8KB buffer for streaming reads

    // ✅ Manual async read loop instead of io::copy
    loop {
        let n = file
            .read(&mut buffer)
            .await
            .with_context(|| format!("Failed to read file for hashing: {:?}", path))?;

        if n == 0 {
            break;  // EOF reached
        }

        hasher.update(&buffer[..n]);
    }

    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}
```

**Changes Required:**
- Change function signature from `fn` to `async fn`
- Add import: `use tokio::io::AsyncReadExt;` at function scope or module level
- Replace `std::fs::File::open()` with `tokio::fs::File::open().await`
- Replace `io::copy()` with manual async read loop using `file.read().await`
- Use 8KB buffer for efficient streaming reads

**Why manual loop instead of tokio::io::copy?**
`tokio::io::copy()` requires a writer implementing `AsyncWrite`. The `Sha256` hasher only has synchronous `update()` method, so we read chunks asynchronously and update the hasher synchronously (which is fine since `update()` is CPU-bound, not I/O-bound).

---

### 4. Update `vendor_package()` Call Sites - Lines 181, 193, 196

**Current Implementation (INCORRECT):**
```rust
async fn vendor_package(&self, pkg: &Package) -> Result<()> {
    let source_dir = find_package_source(pkg)?;        // ❌ Missing .await
    // ...
    let crate_file = find_crate_file(pkg)?;            // ❌ Missing .await
    let checksum = compute_sha256(&crate_file)?;       // ❌ Missing .await
    // ...
}
```

**Fixed Implementation:**
```rust
async fn vendor_package(&self, pkg: &Package) -> Result<()> {
    // 1. Find the source directory in cargo's registry cache
    let source_dir = find_package_source(pkg).await?;  // ✅ Add .await

    // 2. Determine vendor destination (correct format: {name}-{version})
    let pkg_name = format!("{}-{}", pkg.name, pkg.version);
    let dest_dir = self.vendor_path.join(&pkg_name);

    // 3. Copy source files to vendor directory
    copy_dir_recursive(&source_dir, &dest_dir)
        .await
        .with_context(|| format!("Failed to vendor package: {}", pkg_name))?;

    // 4. Find the .crate file for checksum computation
    let crate_file = find_crate_file(pkg).await?;      // ✅ Add .await

    // 5. Compute SHA256 of .crate file
    let checksum = compute_sha256(&crate_file)
        .await                                          // ✅ Add .await
        .with_context(|| format!("Failed to compute checksum for: {}", pkg_name))?;

    // 6. Generate .cargo-checksum.json
    let checksum_data = serde_json::json!({
        "files": {},
        "package": checksum
    });

    let checksum_path = dest_dir.join(".cargo-checksum.json");
    tokio::fs::write(
        &checksum_path,
        serde_json::to_string_pretty(&checksum_data)?,
    )
    .await
    .with_context(|| format!("Failed to write checksum file: {:?}", checksum_path))?;

    log::info!("Vendored: {} -> {:?}", pkg_name, dest_dir);

    Ok(())
}
```

**Changes Required:**
- Line 181: Add `.await` after `find_package_source(pkg)`
- Line 193: Add `.await` after `find_crate_file(pkg)`
- Line 196-197: Add `.await` after `compute_sha256(&crate_file)`

---

## IMPORT CHANGES

**Current imports (lines 1-7):**
```rust
use crate::events::{Event, EventBus};
use anyhow::{Context, Result};
use cargo_metadata::{MetadataCommand, Package};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io;  // ❌ Only used for blocking io::copy - will be removed
use std::path::{Path, PathBuf};
```

**Updated imports:**
```rust
use crate::events::{Event, EventBus};
use anyhow::{Context, Result};
use cargo_metadata::{MetadataCommand, Package};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::AsyncReadExt;  // ✅ Add for async file reading
```

**Changes Required:**
- Remove: `use std::io;` (no longer needed)
- Add: `use tokio::io::AsyncReadExt;` (for `.read()` method on async files)

---

## IMPLEMENTATION CHECKLIST

Transform the three blocking helper functions and update their call sites:

### Step 1: Update imports
- [ ] Remove `use std::io;` from line 6
- [ ] Add `use tokio::io::AsyncReadExt;` to imports

### Step 2: Fix `find_package_source()` (lines 10-39)
- [ ] Change signature: `fn` → `async fn`
- [ ] Replace `std::fs::read_dir()` → `tokio::fs::read_dir().await`
- [ ] Replace `for entry in` → `while let Some(entry) = entries.next_entry().await?`
- [ ] Replace `index_dir.is_dir()` → `tokio::fs::metadata(&index_dir).await?.is_dir()`
- [ ] Replace `pkg_dir.exists()` → `tokio::fs::try_exists(&pkg_dir).await.unwrap_or(false)`

### Step 3: Fix `find_crate_file()` (lines 55-87)
- [ ] Change signature: `fn` → `async fn`
- [ ] Replace `std::fs::read_dir()` → `tokio::fs::read_dir().await`
- [ ] Replace `for entry in` → `while let Some(entry) = entries.next_entry().await?`
- [ ] Replace `index_dir.is_dir()` → `tokio::fs::metadata(&index_dir).await?.is_dir()`
- [ ] Replace `crate_file.exists()` → `tokio::fs::try_exists(&crate_file).await.unwrap_or(false)`

### Step 4: Fix `compute_sha256()` (lines 42-52)
- [ ] Change signature: `fn` → `async fn`
- [ ] Replace `std::fs::File::open()` → `tokio::fs::File::open().await`
- [ ] Replace `io::copy()` with manual async read loop:
  ```rust
  let mut buffer = vec![0u8; 8192];
  loop {
      let n = file.read(&mut buffer).await?;
      if n == 0 { break; }
      hasher.update(&buffer[..n]);
  }
  ```

### Step 5: Update `vendor_package()` call sites (lines 181, 193, 196)
- [ ] Line 181: Add `.await` after `find_package_source(pkg)`
- [ ] Line 193: Add `.await` after `find_crate_file(pkg)`
- [ ] Line 196: Add `.await` after `compute_sha256(&crate_file)`

### Step 6: Verify compilation
- [ ] Run `cargo build --package kargo-cli`
- [ ] Confirm no warnings or errors

---

## DEFINITION OF DONE

- [ ] `find_package_source()` is async and uses only `tokio::fs` functions
- [ ] `find_crate_file()` is async and uses only `tokio::fs` functions
- [ ] `compute_sha256()` is async and uses `tokio::fs::File` + `AsyncReadExt::read()`
- [ ] All three functions called with `.await` in `vendor_package()`
- [ ] `use std::io;` removed from imports
- [ ] `use tokio::io::AsyncReadExt;` added to imports
- [ ] Code compiles without warnings: `cargo build --package kargo-cli`
- [ ] No blocking I/O operations remain in async functions
- [ ] All existing functionality preserved (checksum, directory format, error messages)

---

## CONSTRAINTS

- **NO TESTS:** Do not write unit tests or integration tests
- **NO BENCHMARKS:** Do not write benchmark code
- **NO DOCS:** Do not add module-level documentation or README updates
- **PRESERVE FUNCTIONALITY:** All existing behavior must remain identical
- **MINIMAL SCOPE:** Only fix async/await consistency - do not refactor other code

---

## TECHNICAL REFERENCES

### Tokio Documentation
- [tokio::fs module](https://docs.rs/tokio/latest/tokio/fs/) - Async filesystem operations
- [tokio::fs::read_dir](https://docs.rs/tokio/latest/tokio/fs/fn.read_dir.html) - Async directory iteration
- [tokio::fs::try_exists](https://docs.rs/tokio/latest/tokio/fs/fn.try_exists.html) - Async path existence check
- [tokio::fs::metadata](https://docs.rs/tokio/latest/tokio/fs/fn.metadata.html) - Async file metadata
- [tokio::io::AsyncReadExt](https://docs.rs/tokio/latest/tokio/io/trait.AsyncReadExt.html) - Async read utilities

### Codebase References
- Current file: [`kargo-cli/src/vendor.rs`](../kargo-cli/src/vendor.rs)
- Async pattern example: [`kargo-cli/src/plugins/host_functions.rs:101`](../kargo-cli/src/plugins/host_functions.rs#L101)
- Existing async directory traversal: [`kargo-cli/src/vendor.rs:90-117`](../kargo-cli/src/vendor.rs#L90-L117)

### Dependencies (Already Available)
```toml
# From workspace Cargo.toml
tokio = { version = "1.45", features = ["full"] }  # Includes io-util feature
sha2 = "0.10"
anyhow = "1"
```

---

**Last Updated:** 2025-09-30
**Status:** ⚠️ Critical fix required
**QA Rating:** 7/10 (blocking I/O in async context)
