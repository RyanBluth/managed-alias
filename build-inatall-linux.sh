#!/usr/bin/env bash
cargo build
echo "Copying to $MA_HOME/managed-alias"
cp target/debug/managed-alias $MA_HOME/managed-alias