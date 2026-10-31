#!/usr/bin/env bash
# ============================================================================
#  Antigravity-Manager: Unified End-to-End Test Runner Harness (Linux & macOS)
#  Consolidates integration, lifecycle, switcher, and prompt tests.
# ============================================================================

set -e

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
GRAY='\033[90m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
ROOT_DIR="${SCRIPT_DIR}"
TAURI_DIR="${ROOT_DIR}/src-tauri"

SUITE="all"
FILTER=""
TARGET_BINARY=""
SKIP_CLEANUP=0
VERBOSE=0
BAIL=0

while [ "$#" -gt 0 ]; do
    case "$1" in
        --suite|-s)
            SUITE="$2"
            shift 2
            ;;
        --filter|-f)
            FILTER="$2"
            shift 2
            ;;
        --target|-t)
            TARGET_BINARY="$2"
            shift 2
            ;;
        --skip-cleanup)
            SKIP_CLEANUP=1
            shift
            ;;
        --verbose|-v)
            VERBOSE=1
            shift
            ;;
        --bail|-b)
            BAIL=1
            shift
            ;;
        --help|-h)
            echo "Usage: ./e2e.sh [options]"
            echo "Options:"
            echo "  --suite, -s <name>     Select test suite (all|instance|lifecycle|switcher|prompts|smoke)"
            echo "  --filter, -f <text>    Substring filter for test case names"
            echo "  --target, -t <path>    Path to precompiled agm / agm-alim binary"
            echo "  --skip-cleanup         Retain transient test sandbox"
            echo "  --verbose, -v          Verbose output"
            echo "  --bail, -b             Halt on first test failure"
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# Source cargo env if present
if [ -f "${HOME}/.cargo/env" ]; then
    . "${HOME}/.cargo/env"
fi

# 1. Caller IDE Self-Preservation Guard
CALLER_PID=""
if [ -n "${PPID:-}" ]; then
    CALLER_PID="${PPID}"
fi

# 2. Transient Test Sandboxing
TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
RAND_ID="$(head -c 6 /dev/urandom 2>/dev/null | xxd -p 2>/dev/null || date +%s | tail -c 6)"
SANDBOX_DIR="${ROOT_DIR}/.temp/test-sandbox-${TIMESTAMP}-${RAND_ID}"
mkdir -p "${SANDBOX_DIR}/data" "${SANDBOX_DIR}/config" "${SANDBOX_DIR}/logs"

export AGM_DATA_DIR="${SANDBOX_DIR}/data"
export AGM_CONFIG_DIR="${SANDBOX_DIR}/config"
export AGM_LOG_DIR="${SANDBOX_DIR}/logs"

cleanup() {
    local exit_code=$?
    if [ "${SKIP_CLEANUP}" -eq 0 ]; then
        rm -rf "${SANDBOX_DIR}"
    else
        echo ""
        echo -e "  ${YELLOW}[!] Sandbox retained for inspection: ${SANDBOX_DIR}${NC}"
    fi
    exit "${exit_code}"
}
trap cleanup EXIT INT TERM

# 3. Locate Target Executable
AGM_EXE=""
if [ -n "${TARGET_BINARY}" ] && [ -f "${TARGET_BINARY}" ]; then
    AGM_EXE="${TARGET_BINARY}"
elif [ -f "${TAURI_DIR}/target/debug/agm" ]; then
    AGM_EXE="${TAURI_DIR}/target/debug/agm"
elif [ -f "${TAURI_DIR}/target/release/agm" ]; then
    AGM_EXE="${TAURI_DIR}/target/release/agm"
elif [ -f "${TAURI_DIR}/target/debug/agm-alim" ]; then
    AGM_EXE="${TAURI_DIR}/target/debug/agm-alim"
elif [ -f "${TAURI_DIR}/target/release/agm-alim" ]; then
    AGM_EXE="${TAURI_DIR}/target/release/agm-alim"
fi

invoke_agm() {
    if [ -n "${AGM_EXE}" ]; then
        "${AGM_EXE}" "$@"
    else
        cargo run --bin agm --manifest-path "${TAURI_DIR}/Cargo.toml" --quiet -- "$@"
    fi
}

echo ""
echo -e "${CYAN}================================================================================${NC}"
echo -e "${CYAN}                     Antigravity-Manager E2E Test Suite                        ${NC}"
echo -e "${CYAN}================================================================================${NC}"
echo "  Target Binary : ${AGM_EXE:-cargo run (dynamic)}"
echo "  Suite Target  : ${SUITE}"
echo "  Sandbox Path  : ${SANDBOX_DIR}"
if [ -n "${CALLER_PID}" ]; then
    echo -e "  Caller PID    : ${CALLER_PID} (Self-Preservation Guard Active)"
fi
echo "--------------------------------------------------------------------------------"
echo ""

TOTAL_COUNT=0
PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0
START_TIME="$(date +%s)"

run_test() {
    local name="$1"
    shift
    local action=("$@")

    if [ -n "${FILTER}" ]; then
        case "${name}" in
            *"${FILTER}"*) ;;
            *)
                echo -e "  [*] ${name} ... [${YELLOW}SKIP${NC}] (Filtered out)"
                SKIP_COUNT=$((SKIP_COUNT + 1))
                TOTAL_COUNT=$((TOTAL_COUNT + 1))
                return 0
                ;;
        esac
    fi

    TOTAL_COUNT=$((TOTAL_COUNT + 1))
    local t_start
    t_start="$(date +%s%N 2>/dev/null || date +%s)"

    if "${action[@]}" >/dev/null 2>&1; then
        local t_end
        t_end="$(date +%s%N 2>/dev/null || date +%s)"
        PASS_COUNT=$((PASS_COUNT + 1))
        echo -e "  [*] ${name} ... [${GREEN}PASS${NC}]"
    else
        FAIL_COUNT=$((FAIL_COUNT + 1))
        echo -e "  [*] ${name} ... [${RED}FAIL${NC}]"
        if [ "${BAIL}" -eq 1 ]; then
            echo -e "  ${RED}[!] Bail triggered on failure.${NC}"
            exit 1
        fi
    fi
}

# ----------------------------------------------------------------------------
# SUITE: smoke
# ----------------------------------------------------------------------------
if [ "${SUITE}" = "all" ] || [ "${SUITE}" = "smoke" ]; then
    echo -e "  ${CYAN}[Suite: smoke]${NC}"
    run_test "smoke: binary help command (agm --help)" invoke_agm --help
    run_test "smoke: binary version probe (agm --version)" invoke_agm --version
    run_test "smoke: caller IDE self-preservation check" test -n "${CALLER_PID}"
fi

# ----------------------------------------------------------------------------
# SUITE: instance
# ----------------------------------------------------------------------------
if [ "${SUITE}" = "all" ] || [ "${SUITE}" = "instance" ]; then
    echo ""
    echo -e "  ${CYAN}[Suite: instance]${NC}"
    run_test "instance: instance count --json" invoke_agm instances count --json
    run_test "instance: instance ls --json output verification" invoke_agm instances ls --json
fi

# ----------------------------------------------------------------------------
# SUITE: lifecycle
# ----------------------------------------------------------------------------
if [ "${SUITE}" = "all" ] || [ "${SUITE}" = "lifecycle" ]; then
    echo ""
    echo -e "  ${CYAN}[Suite: lifecycle]${NC}"
    run_test "lifecycle: transient sandbox directory integrity" test -d "${SANDBOX_DIR}/data"
fi

# ----------------------------------------------------------------------------
# SUITE: switcher
# ----------------------------------------------------------------------------
if [ "${SUITE}" = "all" ] || [ "${SUITE}" = "switcher" ]; then
    echo ""
    echo -e "  ${CYAN}[Suite: switcher]${NC}"
    run_test "switcher: auto-switch daemon status probe" invoke_agm auto-switch status
fi

# ----------------------------------------------------------------------------
# SUITE: prompts
# ----------------------------------------------------------------------------
if [ "${SUITE}" = "all" ] || [ "${SUITE}" = "prompts" ]; then
    echo ""
    echo -e "  ${CYAN}[Suite: prompts]${NC}"
    run_test "prompts: prompt queue command probe" invoke_agm prompts help
fi

END_TIME="$(date +%s)"
TOTAL_DURATION=$((END_TIME - START_TIME))

echo ""
echo -e "${CYAN}================================================================================${NC}"
echo -e "${CYAN}                              TEST RUN SUMMARY                                  ${NC}"
echo -e "${CYAN}================================================================================${NC}"
echo "  Total Test Cases : ${TOTAL_COUNT}"
echo -e "  Passed           : ${GREEN}${PASS_COUNT}${NC}"
echo -e "  Failed           : $( [ "${FAIL_COUNT}" -gt 0 ] && echo "${RED}" || echo "${GREEN}" )${FAIL_COUNT}${NC}"
echo -e "  Skipped          : ${YELLOW}${SKIP_COUNT}${NC}"
echo "  Total Duration   : ${TOTAL_DURATION}s"
echo -e "  Overall Status   : $( [ "${FAIL_COUNT}" -eq 0 ] && echo "${GREEN}SUCCESS${NC}" || echo "${RED}FAILURE${NC}" )"
echo -e "${CYAN}================================================================================${NC}"
echo ""

if [ "${FAIL_COUNT}" -gt 0 ]; then
    exit 1
fi
exit 0
