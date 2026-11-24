# Engineering contract

- Treat `docs/PRODUCT_CONCEPT.md` as product authority. World simulation is the product; football is an optional bounded sector.
- Separate sourced historical facts, scholarly interpretations, derived estimates, model assumptions and generated simulation outcomes.
- Never present generated output as verified history or current real-world data.
- Keep domain rules in pure Rust, not HTTP handlers or UI. PostgreSQL is authoritative; browser state is presentation-only.
- Material changes record tick, actor, cause, affected entities, provenance and ruleset version.
- Commands validate before mutation. Failed operations must leave state unchanged.
- Preserve deterministic ordering and test replay for the same ruleset, seed and commands.
- Inspect before editing; define acceptance criteria; run checks on exact candidate SHA and inspect outputs.
- Never claim tests, deployments, agent sessions or data imports without direct evidence. No secrets in source control.
- Do not merge while required checks fail or remain unverified.

## Roles
Systems architect; world-systems researcher; Rust domain engineer; persistence engineer; independent simulation QA; product integrator. Role definitions are not evidence that autonomous agent sessions are running.
