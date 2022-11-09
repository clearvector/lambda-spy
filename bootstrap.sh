#!/bin/bash

# Main bootstrap shell script to execute the main extension binary
BASENAME="$(basename $0 .sh)"

set -euo pipefail

if [[ $(uname -a) == *"aarch64"* ]]; then
    BIN="/opt/$BASENAME/arm64/$BASENAME"
else
    BIN="/opt/$BASENAME/x86_64/$BASENAME"
fi

echo "Launching $BIN"
exec $BIN