
start:

# Initialize stack pointer to point to the first address of RAM
ldi 0x0400
mov SP, A

# Store 0xf0f0 on top of the stack
ldi 0x00f0
sli 8
ori 0x00f0
mov Addr, SP
st

# Increment stack pointer
mov A, SP
addi 1
mov SP, A

# Store 0x0f0f on top of the stack
ldi 0x000f
sli 8
ori 0x000f
mov Addr, SP
st

# Jump to subroutine
ldi subroutine
jmp A

# The subroutine jumps here when it is done
return:

# Repeat the whole program
ldi start
jmp A

subroutine:
    # Load value on top of the stack
    mov Addr, SP
    ld

    # Put it aside
    mov B, A

    # Output the value on the GPIO port
    ldi 0x0600
    mov Addr, A
    mov A, B
    st

    # Load value at SP - 1
    ldi 1
    mov B, A
    mov A, SP
    sub
    mov Addr, A
    ld

    # Put it aside
    mov B, A

    # Output the value on the GPIO port
    ldi 0x0600
    mov Addr, A
    mov A, B
    st

    # Return
    ldi return
    jmp A
