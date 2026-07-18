#!/usr/bin/env python3
# TJ-ARCH-MOB-001 compliant
"""UserPromptSubmit activation hook for TJ-ARCH-MOB-001 project-local UI/UX skills.

Prompt-word matching raises skill-activation reliability from ~50% to ~84-100%
(per the assessment's cited studies). This hook reads the user's prompt from stdin,
matches it against each project-local skill's trigger words, and appends a directive
reminder to invoke the matched skill(s) via additionalContext. It is additive and
non-blocking — it never rejects a prompt, only nudges skill activation.

Wired via .claude/settings.json:
  { "hooks": { "UserPromptSubmit": [ { "command": "python3 .claude/hooks/skill-activation.py" } ] } }
"""
import json
import sys

# skill name -> trigger words (lowercased substring match against the prompt)
SKILLS = {
    "reference-ui-fidelity": [
        "reference app", "reference ui", "standalone html", "prototype",
        "mood board", "moodboard", "screenshot", "attached image",
        "match this design", "visual fidelity", "pixel parity", "redesign",
        "product shell", "all screens", "design spec",
    ],
    "content-block-ui": [
        "contentblock", "content block", "chat message", "streaming block",
        "a2ui", "render message", "block variant", "toolresult", "tooluse",
        "thinking block", "citation block", "artifact block",
    ],
    "hybrid-design-tokens": [
        "design token", "theme", "palette", "color", "spacing", "typography",
        "dark mode", "light mode", "tailwind", "shadcn theme", "css variable",
        "themedata", "brand color", "theme-factory",
    ],
    "tauri-ui-review": [
        "tauri ui", "desktop ui", "react component", "screen", "page", "layout",
        "responsive", "breakpoint", "screenshot", "visual check", "ui review",
        "web ui", "does it look",
    ],
    "tauri-custom-titlebar": [
        "titlebar", "title bar", "window chrome", "window decorations",
        "decorations false", "startdragging", "data-tauri-drag-region",
        "traffic lights", "window controls", "minimize", "maximize", "close button",
        "frameless", "drag region", "custom header",
    ],
    "mobile-navigation": [
        "navigation", "nav bar", "navigation bar", "bottom nav", "tab bar", "tabs",
        "navigationbar", "navigationrail", "bottomnavigationbar", "tabbar", "rail",
        "sidebar", "app shell", "destinations", "shellroute", "router layout",
        "pwa navigation", "mobile layout", "responsive nav",
        # The wrong turn this skill exists to prevent: branching nav on OS rather
        # than width. Catch the check itself, not just the nav vocabulary.
        "platform.isios", "platform.isandroid", "useragent", "user agent",
    ],
    "flutter-golden-ui": [
        "flutter widget", "flutter screen", "golden test", "golden file",
        "widget test", "visual regression", "shadcn_flutter", "consumerwidget",
        "flutter ui", "mobile ui", "pumpwidget", "matchesgoldenfile",
    ],
    "a11y-gate": [
        "accessibility", "a11y", "wcag", "screen reader", "keyboard navigation",
        "focus", "aria", "semantics", "contrast", "alt text", "reduced motion",
        "tab order", "accessible name", "focus trap",
    ],
    "hybrid-runtime-verification": [
        "working app", "application works", "app works", "actually runs",
        "runnable", "runtime verification", "verify runtime", "real launch",
        "production build", "clean checkout", "fresh clone", "smoke test",
        "end-to-end", "e2e", "ready to ship", "shippable", "release gate",
        "working example", "launch the app", "launch ios", "launch tauri",
    ],
    "deploy-hybrid-agentic-stack": [
        "axum", "web server", "dockerfile", "docker compose", "docker-compose",
        "kubernetes", "kustomize", "flint forge", "flint fabric", "flint gate",
        "ory kratos", "byok", "liter-llm", "realtime sync", "web deployment",
        "agentic stack", "embedded web assets",
    ],
    "karpathy-progress-memory": [
        "karpathy", "running llm wiki", "progress memory", "session log",
        "phase boundary", "continuous improvement", "prometheus wiki",
        "private wiki", "record the lesson", "record our path", "handoff",
    ],
}


def matched_skills(prompt: str) -> list[str]:
    p = prompt.lower()
    return [name for name, words in SKILLS.items() if any(w in p for w in words)]


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0  # never block on malformed input

    prompt = payload.get("prompt", "") or ""
    hits = matched_skills(prompt)
    if not hits:
        return 0

    lines = [
        "TJ-ARCH-MOB-001 project skills relevant to this prompt "
        "— invoke each via the Skill tool before doing the matching work:",
    ]
    lines += [f"  - {name}" for name in hits]

    print(
        json.dumps(
            {
                "hookSpecificOutput": {
                    "hookEventName": "UserPromptSubmit",
                    "additionalContext": "\n".join(lines),
                }
            }
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
