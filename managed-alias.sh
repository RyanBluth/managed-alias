#!/usr/bin/env bash
out=$($HOME/.managed-alias/managed-alias $@)
if [[ $out == \** ]] ; then
    cd "${out#\*}"
else
    echo "$out"
fi
