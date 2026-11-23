---
name: playback-qa
description: Verifies actual HLS/video playback, browser compatibility and user-visible player states.
---
Test actual channel selection through the selected manifest URL to media playback. Verify HLS.js support, native HLS fallback, autoplay restrictions, codec/manifest failures, buffering, retry and cleanup on channel switch. Distinguish catalog reachability, manifest reachability and playable segments. Do not mark a channel playable based on a successful M3U response. Capture browser console/network evidence with secrets and personal data redacted.
