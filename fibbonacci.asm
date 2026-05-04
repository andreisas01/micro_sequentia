br main

; calculate n-th Fibonacci number
; input in R0
; output at memory address 0x0100
;----------------------------------------------------------
; e.g. for n = 24, the expected output is 0x6FF1
;----------------------------------------------------------
fibonacci_proc:
    mov R0, 24 ; input (n)

    mov R1, 0 ; fib1
    mov R2, 1 ; fib2
    xor R3, R3 ; fib3

    sub R0, 2

    loop:
        mov R3, R1
        add R3, R2
        mov R1, R2
        mov R2, R3
        dec R0
    bne loop  ; counter != 0

    xor R4, R4 ; start address
    mov 256(R4), R3 ; 0 + 256 offset = 0x0100

    ret

main:
    mov R10, 2 ; addr of fibonacci_proc
    call R10

    halt
