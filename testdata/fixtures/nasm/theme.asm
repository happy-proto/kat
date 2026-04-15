global theme_preview

section .text

theme_preview:
    mov rax, 60
    mov rdi, 0
    syscall
