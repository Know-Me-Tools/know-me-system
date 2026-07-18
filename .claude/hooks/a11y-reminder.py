#!/usr/bin/env python3
# TJ-ARCH-MOB-001 compliant
"""PostToolUse reminder hook backing the a11y-gate skill.

After a Write|Edit touches a UI file (.tsx/.jsx under src/, .dart under lib/), this hook
prints an advisory reminder to run the a11y-gate WCAG 2.2 AA checklist. It is non-blocking
by design — it does not judge or reject the edit, it only ensures the accessibility gate is
not silently skipped.

Wired via .claude/settings.json:
  { "hooks": { "PostToolUse": [ { "matcher": "Write|Edit",
      "command": "python3 .claude/hooks/a11y-reminder.py" } ] } }
"""
import json
import sys

UI_SUFFIXES = (".tsx", ".jsx", ".dart")


def is_ui_file(path: str) -> bool:
    p = path.replace("\\", "/")
    if not p.endswith(UI_SUFFIXES):
        return False
    if p.endswith(".dart"):
        return "/lib/" in p or p.startswith("lib/")
    return "/src/" in p or p.startswith("src/")


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    tool_input = payload.get("tool_input", {}) or {}
    path = tool_input.get("file_path", "") or ""
    if not is_ui_file(path):
        return 0

    print(
        json.dumps(
            {
                "hookSpecificOutput": {
                    "hookEventName": "PostToolUse",
                    "additionalContext": (
                        f"UI file edited ({path}). Before marking done, invoke the "
                        "a11y-gate skill and run the WCAG 2.2 AA checklist "
                        "(contrast in both themes, keyboard reachability, accessible "
                        "names, reduced motion, live-region announcements)."
                    ),
                }
            }
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
