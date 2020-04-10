// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Fill.asm

// Runs an infinite loop that listens to the keyboard input.
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel;
// the screen should remain fully black as long as the key is pressed. 
// When no key is pressed, the program clears the screen, i.e. writes
// "white" in every pixel;
// the screen should remain fully clear as long as no key is pressed.

// Constants
    @SCREEN
    D=A
    @8192
    D=A+D
    @end_of_screen
    M=D             // Set end_of_screen 16384 + 8192 = 24576

(KBD_CHECK)
    @SCREEN
    D=A         
    @screen_address
    M=D             // Reset screen_address to the base memory address of the screen

    @KBD
    D=M
    @CLEAR
    D;JEQ           // If no keyboard input (0), goto CLEAR
    @FILL
    D;JGT           // If keyboard input (>0), goto FILL

(CLEAR)
    @screen_address
    A=M
    M=0             // Clear 16 pixels at M[screen_address]

    @screen_address
    M=M+1           // Increment screen_address by 1

    @screen_address
    D=M
    @end_of_screen
    D=D-M
    @CLEAR
    D;JNE           // If screen_address != end_of_screen goto CLEAR

    @KBD_CHECK
    0;JMP           // Return to KBD_CHECK

(FILL)
    @screen_address
    A=M
    M=-1            // Fill 16 pixels at M[screen_address]

    @screen_address
    M=M+1           // Increment screen_address by 1

    @screen_address
    D=M
    @end_of_screen
    D=D-M
    @FILL 
    D;JNE           // If screen_address != end_of_screen goto CLEAR

    @KBD_CHECK
    0;JMP           // Return to KBD_CHECK

(END)
    @END
    0;JMP
