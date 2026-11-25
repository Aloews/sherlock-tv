#!/usr/bin/env python3
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
errors = []

def check(path: Path, expected=None):
    text = path.read_text(encoding="utf-8")
    if not text.endswith("\n"):
        errors.append(f"{path.relative_to(ROOT)}: missing final newline")
    if expected is None:
        return
    match = re.match(r"^---\n(?P<meta>.*?)\n---\n", text, re.DOTALL)
    if not match:
        errors.append(f"{path.relative_to(ROOT)}: missing frontmatter")
        return
    meta = match.group("meta")
    name = re.search(r"^name:\s*([a-z0-9-]+)\s*$", meta, re.MULTILINE)
    desc = re.search(r"^description:\s*\S.*$", meta, re.MULTILINE)
    if not name or name.group(1) != expected:
        errors.append(f"{path.relative_to(ROOT)}: invalid name")
    if not desc:
        errors.append(f"{path.relative_to(ROOT)}: missing description")

agents = sorted((ROOT / ".claude/agents").glob("*.md"))
skills = sorted((ROOT / ".claude/skills").glob("*/SKILL.md"))
commands = sorted((ROOT / ".claude/commands").glob("*.md"))
if len(agents) < 9: errors.append(f"expected >=9 agents, found {len(agents)}")
if len(skills) < 3: errors.append(f"expected >=3 skills, found {len(skills)}")
if len(commands) < 3: errors.append(f"expected >=3 commands, found {len(commands)}")
for p in agents: check(p, p.stem)
for p in skills: check(p, p.parent.name)
for p in commands: check(p)
for p in ("AGENTS.md", "CLAUDE.md"):
    if not (ROOT / p).is_file(): errors.append(f"missing {p}")
if errors:
    print("\n".join(f"ERROR: {e}" for e in errors))
    sys.exit(1)
print(f"OK: {len(agents)} agents, {len(skills)} skills, {len(commands)} commands.")
