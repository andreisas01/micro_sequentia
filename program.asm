ADD (R6),R0
XOR R1, 2
LSR 14(R0)
CMP 2, R3
PUSH PC # comm
BVC eticheta

eticheta: PUSH PC ; final boss