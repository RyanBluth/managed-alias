
if ! [[ -n "$MANGY_HOME" ]] ; then
    echo -e "\nexport MANGY_HOME=\$HOME/.mangy" | tee -a $HOME/.bashrc
    echo -e '\nPATH="$MANGY_HOME:$PATH"' | tee -a $HOME/.bashrc
fi

if ! [ -n "$(type -t mangy)" ] && ! [ "$(type -t mangy)" = function ] ; then
echo -e "\nfunction mangy() {\n\
  . \$MANGY_HOME/mangy.sh $@\n\
}\n\
export -f mangy" | tee -a $HOME/.bashrc
fi

