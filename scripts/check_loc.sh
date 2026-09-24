#!/usr/bin/env bash
# scripts/check_loc.sh
# Asserts that no source file under crates/, src/, or qa/unit/ exceeds 256 physical lines of code.

set -euo pipefail

MAX_LINES=256
SCAN_DIRS=()
VIOLATIONS=0
CHECKED=0

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --limit)
            MAX_LINES="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [--limit <lines>] [directories...]"
            echo "Default limit: 256 lines"
            exit 0
            ;;
        *)
            SCAN_DIRS+=("$1")
            shift
            ;;
    esac
done

if [[ ${#SCAN_DIRS[@]} -eq 0 ]]; then
    for dir in "${REPO_ROOT}/crates" "${REPO_ROOT}/src" "${REPO_ROOT}/qa/unit"; do
        if [[ -d "$dir" ]]; then
            SCAN_DIRS+=("$dir")
        fi
    done
fi

if [[ ${#SCAN_DIRS[@]} -eq 0 ]]; then
    echo "Notice: No crates/, src/, or qa/unit/ directory found to check."
    exit 0
fi

echo "========================================================"
echo " Running Strict LOC Check (Limit: <= ${MAX_LINES} lines per file)"
echo " Scanning: ${SCAN_DIRS[*]}"
echo "========================================================"

FAILED_FILES=()

while IFS= read -r -d '' file; do
    lines=$(awk 'END {print NR}' "$file" 2>/dev/null || echo 0)
    CHECKED=$((CHECKED + 1))
    if [[ "$lines" -gt "$MAX_LINES" ]]; then
        VIOLATIONS=$((VIOLATIONS + 1))
        FAILED_FILES+=("${lines}\t${file#${REPO_ROOT}/}")
    fi
done < <(find "${SCAN_DIRS[@]}" -type f -name "*.rs" -not -path "*/target/*" -print0 2>/dev/null)

if [[ $VIOLATIONS -gt 0 ]]; then
    echo -e "\n[FAILURE] Found ${VIOLATIONS} file(s) exceeding ${MAX_LINES} physical LOC:"
    echo -e "Lines\tFile"
    echo -e "-----\t----"
    for item in "${FAILED_FILES[@]}"; do
        echo -e "$item"
    done
    echo -e "\nStrict LOC invariant VIOLATED. Please modularize exceeding files."
    exit 1
fi

echo -e "\n[SUCCESS] Verified ${CHECKED} source file(s). All files comply with <= ${MAX_LINES} LOC."
exit 0
