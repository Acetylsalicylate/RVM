; ========================================================
; main.asm: iteration until "Hello world!"
; ========================================================

start:
    mov V0, target_str      ; V0 = pointer to target string
    mov V1, buffer          ; V1 = pointer to buffer

char_loop:
    loadb V2, [V0]          ; Читаем целевой символ из памяти в V2
    cmp V2, 0               ; end of line (\0)?
    je done

    mov V3, 32              ; V3 = ASCII candidate from 32 (' ')

guess_loop:
    ; 1. load to memory
    storeb [V1], V3

    ; 2. temporarily place a null terminator after the candidate to ensure safe output
    mov V4, V1
    add V4, 1
    mov V5, 0
    storeb [V4], V5

    ; 3. output the string from memory (from the beginning of the buffer up to the current character)
    mov V5, buffer
print_buf:
    loadb V6, [V5]
    cmp V6, 0
    je print_cr
    out V6
    add V5, 1
    jmp print_buf

print_cr:
    ; return cursor to beginning of line
    mov V6, 13
    out V6

    ; 4. small delay
    mov V7, 65535
delay:
    sub V7, 1
    jne delay

    ; 5. guessed?
    cmp V3, V2
    je next_char

    ; if not take next ASCII char and repeat
    add V3, 1
    jmp guess_loop

next_char:
    ; found
    add V0, 1
    add V1, 1
    jmp char_loop

done:
    ; final output
    mov V5, buffer

final_print:
    loadb V6, [V5]
    cmp V6, 0
    je finish
    out V6
    add V5, 1
    jmp final_print

finish:
    nl                      
    halt

target_str:
    db "Hello world!", 0

buffer:
    db 0