#!/usr/bin/env bash
# scripts/check_deps.sh
# Asserts that binaries link ZERO dynamic C libraries (libsystemd, libdbus, libssl, etc.)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

FORBIDDEN_REGEX="libsystemd|libdbus|libssl|libcrypto|libzstd|liblz4"
TARGET_BINARIES=()

if [[ $# -gt 0 ]]; then
    TARGET_BINARIES+=("$@")
else
    for search_dir in "${REPO_ROOT}/target/release" "${REPO_ROOT}/target/debug"; do
        if [[ -d "$search_dir" ]]; then
            while IFS= read -r -d '' bin; do
                if [[ -x "$bin" && ! -d "$bin" ]] && file "$bin" 2>/dev/null | grep -q "ELF"; then
                    case "$(basename "$bin")" in
                        build-script-*|*.so|*.rlib|*.d) ;;
                        *) TARGET_BINARIES+=("$bin") ;;
                    esac
                fi
            done < <(find "$search_dir" -maxdepth 2 -type f -print0 2>/dev/null)
        fi
    done
fi

if [[ ${#TARGET_BINARIES[@]} -eq 0 ]]; then
    echo "[INFO] No compiled ELF binaries found in target/. Run 'cargo build' or 'cargo test' first."
    exit 0
fi

echo "========================================================"
echo " Running Pure-Rust Zero-C Dynamic Dependency Check"
echo " Forbidden patterns: ${FORBIDDEN_REGEX}"
echo "========================================================"

OVERALL_STATUS=0

for binary in "${TARGET_BINARIES[@]}"; do
    echo -n "Checking $(basename "$binary")... "

    direct_needed=""
    if command -v readelf &>/dev/null; then
        direct_needed=$(readelf -d "$binary" 2>/dev/null | grep "(NEEDED)" || true)
    fi

    transitive_deps=""
    if command -v ldd &>/dev/null; then
        transitive_deps=$(ldd "$binary" 2>/dev/null || true)
    fi

    combined_output="${direct_needed}
${transitive_deps}"
    forbidden_matches=$(echo -e "$combined_output" | grep -E "$FORBIDDEN_REGEX" || true)

    if [[ -n "$forbidden_matches" ]]; then
        echo -e "\n[FAILURE] Forbidden dynamic C libraries detected in $binary:"
        echo "$forbidden_matches" | sed 's/^/  -> /'
        OVERALL_STATUS=1
    else
        echo "OK (Pure Rust)"
    fi
done

if [[ $OVERALL_STATUS -ne 0 ]]; then
    echo -e "\n[ERROR] One or more binaries violate the Zero-C Dynamic Library invariant."
    exit 1
fi

echo -e "\n[SUCCESS] All checked binaries verified. Zero forbidden dynamic C libraries linked."
exit 0
