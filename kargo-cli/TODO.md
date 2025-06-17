# TODO: Fix kargo mddoc Documentation Generation

## Problem Statement
`kargo mddoc` only generates module structure and overview text, not the actual API items (functions, structs, traits, etc.)

## Root Cause
- `process_module_details` (line 1256) calls `process_items` (line 1263) 
- Items ARE found but NOT shown because heading levels exceed 6 (markdown limit)
- Module at level 2, submodule at 3, items at 4, nested deeper = invisible

## Investigation Steps

### [x] 1. Add Debug Logging
- [x] Add log after line 135 in `process_items` to confirm functions are found
- [x] Add log at line 227 in `process_item` to confirm function processing
- [x] Run `RUST_LOG=debug kargo mddoc tokio` and verify functions are found

**DISCOVERY**: Functions ARE being processed but appear as "##### Unnamed Item" at level 5. This confirms the heading level issue. Zero "Function `name`" entries found.

### [ ] 2. Fix Heading Level Issue - REVISED APPROACH
- [x] WRONG APPROACH: Fixed level 3 - doesn't handle recursive depth
- [ ] CORRECT APPROACH: Remove the 6-level cap (line 77) or handle it differently
- [ ] Functions/types should be visible even at level 7+ 
- [ ] OR: Flatten the structure so modules don't nest so deeply
- [ ] ANALYSIS NEEDED: What's the actual depth in tokio modules?

### [ ] 3. Test with Simple Crate
- [ ] Create test crate with module containing one function
- [ ] Run mddoc and verify function appears
- [ ] Check heading structure is correct

### [ ] 4. Fix Re-exports
- [ ] Identify how re-exports appear in "other_items"
- [ ] Add handler in `process_item` for re-export patterns
- [ ] Show re-exported items with their targets

### [ ] 5. Test with Tokio
- [ ] Run on tokio
- [ ] Verify `spawn` function appears
- [ ] Verify `JoinHandle` type appears
- [ ] Verify re-exports show correctly

## Success Criteria
- All public API items are documented
- Heading levels are correct (max depth 6)
- Re-exports are clearly shown
- tokio::spawn is visible in output

## CLEANUP TASK (DO AFTER ALL DEBUGGING COMPLETE)
- [ ] UNCOMMENT the JSON deletion code in `/Volumes/samsung_t9/projects/y/kargo/kargo-mddoc/src/plugin.rs` lines 134-139