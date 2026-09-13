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
