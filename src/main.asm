    mov V0, 1

loop:
    outn V0
    out "\r"
    add V0, 1
    cmp V0, 100
    jnz loop

    halt