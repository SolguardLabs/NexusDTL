#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

run_cargo() {
  if command -v cargo >/dev/null 2>&1; then
    cargo "$@"
  elif is_windows_bash && command -v cargo.exe >/dev/null 2>&1; then
    cargo.exe "$@"
  elif is_windows_bash && command -v cmd.exe >/dev/null 2>&1; then
    cmd.exe /d /c cargo "$@"
  else
    echo "cargo no esta disponible en PATH" >&2
    return 127
  fi
}

run_node() {
  if command -v node >/dev/null 2>&1; then
    node "$@"
  elif is_windows_bash && command -v node.exe >/dev/null 2>&1; then
    node.exe "$@"
  elif is_windows_bash && command -v cmd.exe >/dev/null 2>&1; then
    cmd.exe /d /c node "$@"
  else
    echo "node no esta disponible en PATH" >&2
    return 127
  fi
}

is_windows_bash() {
  case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN*) return 0 ;;
    *) return 1 ;;
  esac
}

run_cargo test --locked
run_node --test tests/node/*.test.js
