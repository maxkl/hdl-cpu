
# This program waits until the button connected to the first GPIO pin is pressed. It then counts from 0 to 9 and
# displays the current value using the LEDs connected to the GPIO. Afterwards it restarts and waits until the button is
# pressed again.

start:

# Clear display
ldi 0x0600
mov Addr, A
ldi 0
st

# Busy wait until button pressed
wait_button:
    # Load address of input peripheral
    ldi 0x0601
    mov Addr, A
    # Read buttons
    ld
    # Mask first button
    andi 0x0001
    # Repeat if button is not pressed
    ldi wait_button
    jmp.z A

# Light up the whole display
ldi 0x0600
mov Addr, A
ldi 0xff
sli 8
ori 0xff
st

# Set the counter in B to 0
ldi 0
mov B, A

# Count from 0 to 9
loop:
    # Load address of display peripheral
    ldi 0x0600
    mov Addr, A
    # Copy counter from B
    mov A, B
    # Display counter
    st
    # Increment A
    addi 1
    # Compare A to 10
    cmpi 10
    # Store counter in B
    mov B, A
    # Load loop start address
    ldi loop
    # If A < 10, repeat
    jmp.lt A

# Repeat
ldi start
jmp A
