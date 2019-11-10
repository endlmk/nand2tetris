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

// Put your code here.
(MAIN)
    @24576
    D=M
    @DRAWW
    D;JEQ
    @DRAWB
    D;JNE
    @MAIN
    0;JMP
(DRAWB)
    @wb
    M=-1
    @DRAW
    0;JMP
(DRAWW)
    @wb
    M=0
    @DRAW
    0;JMP
(DRAW)
    @i
    M=1
    @SCREEN
    D=A
    @adr
    M=D
(LOOP)
    @i
    D=M
    @8192
    D=D-A
    @MAIN
    D;JGT
    @wb
    D=M
    @adr
    A=M
    M=D
    @adr
    M=M+1
    @i
    M=M+1
    @LOOP
    0;JMP