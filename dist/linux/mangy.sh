out=$($HOME/.mangy/mangy $@)
if [[ $out == \** ]] ; then
    cd "${out#\*}"
else
    echo "$out"
fi
