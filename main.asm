; ========================================================
; main.asm: iteration until "Hello world!" using stack ops
; ========================================================

start:
    mov V0, target_str      ; V0 = pointer to target string
    mov V1, buffer          ; V1 = pointer to buffer

char_loop:
    loadb V2, [V0]          ; Read target char from memory into V2
    cmp V2, 0               ; End of string (\0)?
    je done

    mov V3, 32              ; V3 = ASCII candidate starting from 32 (' ')

guess_loop:
    ; 1. Store candidate into buffer
    storeb [V1], V3

    ; 2. Temporarily write null terminator after candidate
    mov V4, V1
    add V4, 1
    mov V5, 0
    storeb [V4], V5

    ; 3. Print current buffer state using subroutine
    call print_buffer_sub

    ; Return cursor to beginning of line (carriage return, \r)
    mov V6, 13
    out V6

    ; 4. Call delay subroutine
    call delay_sub

    ; 5. Check if guessed
    cmp V3, V2
    je next_char

    ; Next candidate
    add V3, 1
    jmp guess_loop

next_char:
    add V0, 1
    add V1, 1
    jmp char_loop

done:
    ; Final clean output
    call print_buffer_sub
    nl                      
    halt

; --------------------------------------------------------
; Subroutine: print_buffer_sub
; Saves registers to stack, prints null-terminated buffer
; --------------------------------------------------------
print_buffer_sub:
    push V5
    push V6
    mov V5, buffer

pb_loop:
    loadb V6, [V5]
    cmp V6, 0
    je pb_done
    out V6
    add V5, 1
    jmp pb_loop

pb_done:
    pop V6
    pop V5
    ret

; --------------------------------------------------------
; Subroutine: delay_sub
; Simple busy-wait delay loop
; --------------------------------------------------------
delay_sub:
    push V7
    mov V7, 65535
delay_loop:
    sub V7, 1
    jne delay_loop
    pop V7
    ret

target_str:
    db "Hello world!", 0

buffer:
    db 0