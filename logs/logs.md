# Development logs
> The chronological order of updates for the nocutra component: Axion---
# 📚 Documentation - V-1-0-0

**Time**: 05/09/2026 - 01:08 PM
**Status**: 🌕 Complete

## Summary
This is the official repository and update logs for Axion, Nocturas desktop compositor. This log has started after basic functions were implemented

## Notice
Because this project is underdeveloped expect a lot of archetectural changes

## Things that were added/changed
```
src/  -  🟢 Added
Cargo.toml  -  🟡 Changed

```
---
# 🐛 Bug fix - V-1-1-0

**Time**: 13/09/2026 - 11:12 PM
**Status**: 🌘 Detected

## Summary
This minor version of Axion is going to focus on making Axion more safer and less crash-prone
This update just made the redraw logic a bit more robust and crash-free
This update also made the winit window resize work properly

## Notice
There will be less .unwrap() ussage, and perference towards error handling
I'm still considering if we should include logging in this version or the next,
next patch note will answer this question

## Things that were added/changed
```
compositor_imps.rs  -  🟡 Changed
basicUtils.rs - 🟢 Added

```
---
# ⭐ New Feature - V-1-1-1

**Time**: 16/09/2026 - 09:26 PM
**Status**: 🌗 Partially Complete

## Summary
Added cursor to the compositor
Made certain things more safer

## Notice
The NocturaCursor(check cursor_impls.rs/state.rs) is a implemented a bit less tranditionally
In NocturaState, the key "cs" (stands for cursor image state), is shared with NocturaCursor
(check implementation in event loop handle at compositor_impls, start_win)

## Things that were added/changed
```
cursor_impls.rs  -  🟢 Added
compositor_impls.rs  -  🟡 Changed

```
