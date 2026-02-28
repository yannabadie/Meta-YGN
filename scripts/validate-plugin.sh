#!/bin/bash
set -e
echo "=== MetaYGN Plugin Validation ==="
ERRORS=0
CHECKS=0

check() {
    CHECKS=$((CHECKS+1))
    if [ "$1" = "ok" ]; then
        echo "  OK: $2"
    else
        echo "  FAIL: $2"
        ERRORS=$((ERRORS+1))
    fi
}

# 1. JSON files valid
for f in .claude-plugin/plugin.json hooks/hooks.json settings.json; do
    if python3 -c "import json; json.load(open('$f'))" 2>/dev/null; then
        check ok "$f is valid JSON"
    else
        check fail "$f is invalid or missing"
    fi
done

# 2. All skills have SKILL.md
for skill in skills/*/; do
    if [ -f "${skill}SKILL.md" ]; then
        check ok "${skill}SKILL.md exists"
    else
        check fail "${skill}SKILL.md missing"
    fi
done

# 3. All agents have .md
for agent in agents/*.md; do
    check ok "$agent exists"
done

# 4. TS hooks exist
for hook in session-start user-prompt-submit pre-tool-use post-tool-use post-tool-use-failure stop pre-compact session-end; do
    if [ -f "packages/hooks/src/${hook}.ts" ]; then
        check ok "${hook}.ts exists"
    else
        check fail "${hook}.ts missing"
    fi
done

# 5. Output style exists
if [ -f "output-styles/aletheia-proof.md" ]; then
    check ok "output-styles/aletheia-proof.md exists"
else
    check fail "output-styles/aletheia-proof.md missing"
fi

echo ""
if [ $ERRORS -eq 0 ]; then
    echo "=== ALL $CHECKS VALIDATIONS PASSED ==="
else
    echo "=== $ERRORS VALIDATION(S) FAILED ==="
    exit 1
fi
