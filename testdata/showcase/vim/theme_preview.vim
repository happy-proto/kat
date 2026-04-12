let g:kat_theme = 'dracula'
let s:palette = {
      \ 'accent': '#bd93f9',
      \ 'comment': '#6272a4',
      \ }

function! s:RenderTheme(name) abort
  if a:name =~# 'drac'
    echohl Title
    echomsg printf('Theme => %s', a:name)
    echohl None
  else
    echoerr 'Unsupported theme'
  endif
endfunction

command! -nargs=1 ThemePreview call s:RenderTheme(<f-args>)
autocmd BufReadPost *.vim call s:RenderTheme(g:kat_theme)

lua << EOF
print("lua from vim")
EOF

python3 << EOF
print("python from vim")
EOF
