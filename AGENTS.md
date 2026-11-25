# Agent operating instructions

Before changing this repository, read the portable [Reality-First Autonomous Engineering skill](.agents/skills/reality-first-autonomous-engineering/SKILL.md) and its security/cost reference.

This repository contains MatchLive, a standalone live sports-streaming catalogue and player. Preserve the separation from other products and services. Read the README and inspect the current stream catalogue, CORS/mixed-content constraints, player source selection, and any rights-related boundaries before changing streaming behavior. Do not assume a catalogue URL is itself a playable media manifest.

The skill defines the execution process; it does not grant permissions, override product constraints, or authorize paid resources, destructive changes, production data changes, or releases. Verify the real player journey and the relevant network behavior; a successful frontend build alone is not sufficient.

Do not send routine progress narration. Escalate only a material decision, genuine blocker, security/cost/rights risk, or final evidence-based handoff.
