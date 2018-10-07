#!/usr/bin/env bash
cargo build
cp target/debug/managed-alias ./dist/linux/managed-alias
echo "Copying to $MA_HOME/managed-alias"
cp target/debug/managed-alias $MA_HOME/managed-alias