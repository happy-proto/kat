#!/usr/bin/env tcsh
# C shell has list variables, backtick substitutions, and its own control flow.
set sources = (src/*.c)
set builddir = "$cwd/build"
mkdir -p "$builddir"
alias report 'echo "[build] \!:*"'

foreach file ($sources)
    if (-f "$file") then
        set target = "$builddir/$file:t:r.o"
        report "$file"
        cc -c "$file" -o "$target" >>&! "$builddir/build.log"
    endif
end

switch ($status)
case 0:
    echo "Built $#sources source files on `hostname`"
    breaksw
default:
    echo 'Build failed; inspect the log.'
    exit 1
endsw
