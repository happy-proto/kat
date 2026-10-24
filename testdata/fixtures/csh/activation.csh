#!/bin/csh
# Synthetic virtual environment activation; source this file.
alias deactivate 'setenv PATH "$_OLD_PATH:q"; test "\!:*" != "keep" && unalias deactivate'
setenv VIRTUAL_ENV '/opt/example/.venv'
set _OLD_PATH="$PATH:q"
setenv PATH "$VIRTUAL_ENV:q/bin:$PATH:q"
if ( $?prompt ) then
    set _OLD_PROMPT="$prompt:q"
    set prompt = '('"$VIRTUAL_ENV:t:q"') '"$prompt:q"
endif
alias pydoc python -m pydoc
rehash
