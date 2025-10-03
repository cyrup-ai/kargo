# TODO: Fix All Errors and Warnings

## Status Summary
- **Total Errors:** 0
- **Total Warnings:** 1
- **Completion:** 0/1 tasks complete

---

## Warnings to Fix

### 1. WARNING: Unused field `orphan_detector` in kargo-turd
- **File:** `plugins/native/kargo-turd/src/executor.rs:19`
- **Issue:** Field `orphan_detector` is never read
- **Root Cause:** The `analyze_file()` method is currently a stub (line 76 comment confirms)
- **Planned Implementation:** EXEC_3.md task shows this field should be used via `self.orphan_detector.lock().unwrap()`
- **Blocker:** The planned implementation uses `Mutex::lock()` which is blocking/locking code
- **Action Required:** ASK David Maple for permission to use blocking/locking code OR redesign to use async-compatible primitives
- **Status:** ⏸️ BLOCKED - Awaiting David's input

### 2. QA: Review orphan_detector implementation approach
- **Status:** ⏸️ PENDING - Cannot QA until implementation is approved and completed
- **Criteria:** Rate quality 1-10, must score 9+ or re-do

---

## Questions for David Maple

### Question 1: Permission for Mutex-based locking in kargo-turd
The `orphan_detector` field is designed to use `Arc<Mutex<OrphanDetector>>` for thread-safe accumulation across rayon parallel iterators. The planned implementation (per EXEC_3.md) uses:

```rust
let mut detector = self.orphan_detector.lock().unwrap();
```

This is blocking/locking code. Per your instructions:
- "NEVER use blocking code at all ... never never never UNLESS David Maple has specifically OKd it"
- "ALWAYS ASK for permission on any blocking or locking code"

**Options:**
1. **Approve Mutex usage:** If approved, I'll annotate with "APPROVED BY DAVID MAPLE on 2025-10-02"
2. **Redesign to async:** Use `tokio::sync::Mutex` or lock-free structures
3. **Different approach:** Use a different pattern for cross-thread accumulation

Please advise on your preferred approach.

---

## Implementation Notes

### If Mutex is approved:
- Will implement per EXEC_3.md specification
- Will add approval annotation to all lock sites
- Will complete analyze_file() stub with full analyzer integration

### If redesign required:
- Will need to redesign OrphanDetector accumulation strategy
- May impact rayon parallel processing approach
- Will require sequential thinking to plan alternative architecture
