#!/usr/bin/env bash
out=$($MA_HOME/managed-alias $@)
if [[ $out == \** ]] ; then
    cd "${out#\*}"
else
    echo "$out"
fi
