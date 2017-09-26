out= $($HOME/.mangy/mangy $@)
if [ "$1" == "g" ] || [ "$1" == "go" ] ; then 
    cd "$out"
else
    echo "$out"
fi