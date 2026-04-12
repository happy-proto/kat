let g:kat_theme = 'dracula'
let s:palette = {'accent': '#bd93f9'}

function! s:RenderTheme(name) abort
  if a:name ==# 'dracula'
    echohl Title
    echomsg printf('kat theme: %s', a:name)
    echohl None
  endif
endfunction

autocmd BufReadPost *.vim call s:RenderTheme(g:kat_theme)
lua << EOF
print("embedded lua from vim")
EOF
