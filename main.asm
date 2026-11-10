    mov V0, 1
    mov V1, 65535

loop:
    outn V0
    out "\r"
    add V0, 1
    cmp V0, V1
    jnz loop

    halt