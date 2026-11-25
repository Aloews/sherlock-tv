---
name: provider-integration
description: Implements and verifies M3U catalog and stream relay integrations.
---
Read README.md and inspect the actual relay/service contract before changes. Verify upstream URL, response format, redirects, CORS, gzip, cache headers, rate limits and allowed content types. Treat the configured VITE_STREAM_URL as the catalog endpoint unless code and tests prove otherwise. Do not invent channel availability. Test upstream outage, invalid playlist, timeout, oversized payload, missing CORS and invalid stream URLs. Avoid open proxy patterns and never expose private relay credentials to the browser.
