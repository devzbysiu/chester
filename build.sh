#!/usr/bin/env bash

# This script builds core and/or client when file changes are found.
# It reads the changes from git, then checks if paths start with
# `client/` or `core/` respectively. If changed files are not visible
# in git or do not start with `core/` or `client/` then nothing happens.

set -o errexit
set -o pipefail
set -o nounset

# ============================ [ CONFIGURATION ] ============================ 

DEBUG=0 # enable debug logs

CHANGES=1 # no changes

# colors

BOLD="\e[1";
ENDCOLOR="\e[0m";

# font color
CYAN="36";

# bg color
BG_DEFAULT="49";

# ============================ [ DECLARATION ] ============================ 

function debug() {
  if [[ "${DEBUG}" -eq 0 ]]; then
    local -r message=$1
    echo -e "${BOLD};${BG_DEFAULT};${CYAN}m [DEBUG] ${ENDCOLOR}${message}"
  fi
}

function read_changes() {
  local -r changed_files=$(git status --porcelain=v1 2>/dev/null | wc -l)
  debug "# of changed_files = ${changed_files}"
  if [[ ${changed_files} -gt 0 ]]; then
    CHANGES=0
  fi
}

function code_changed() {
  debug "Checking code changed..."
  return ${CHANGES}
}

function build() {
  debug "Building code..."
  cargo make clip 
  cargo make test
}

function main() {
  read_changes
  if code_changed; then
    build
  fi
  exit 0
}

# ============================ [ EXECUTION ] ============================ 

main
