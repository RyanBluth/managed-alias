#!/usr/bin/env bash
if ! [[ -n "$MA_HOME" ]] ; then
    echo -e "\nexport MA_HOME=\$HOME/.managed-alias" | tee -a $HOME/.bashrc
    echo -e '\nPATH="$MA_HOME:$PATH"' | tee -a $HOME/.bashrc
    export MA_HOME=$HOME/.managed-alias
fi

if ! [ -n "$(type -t ma)" ] && ! [ "$(type -t ma)" = function ] ; then
echo -e "\nfunction ma() {\n\
  . \$MA_HOME/managed-alias.sh $@\n\
}\n\
export -f ma" | tee -a $HOME/.bashrc
fi

mkdir $MA_HOME

cp ./../../managed-alias.sh $MA_HOME/managed-alias.sh
cp ./managed-alias $MA_HOME/managed-alias