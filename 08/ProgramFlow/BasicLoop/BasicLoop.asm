@0
D=A
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@0
D=D+A
@13
M=D
@SP
M=M-1
@SP
A=M
D=M
@13
A=M
M=D
(LOOP_START)
@ARG
D=M
@0
D=D+A
@13
M=D
@13
A=M
D=M
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@0
D=D+A
@13
M=D
@13
A=M
D=M
@SP
A=M
M=D
@SP
M=M+1
@SP
M=M-1
@SP
A=M
D=M
@SP
M=M-1
@SP
A=M
D=M+D
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@0
D=D+A
@13
M=D
@SP
M=M-1
@SP
A=M
D=M
@13
A=M
M=D
@ARG
D=M
@0
D=D+A
@13
M=D
@13
A=M
D=M
@SP
A=M
M=D
@SP
M=M+1
@1
D=A
@SP
A=M
M=D
@SP
M=M+1
@SP
M=M-1
@SP
A=M
D=M
@SP
M=M-1
@SP
A=M
D=M-D
@SP
A=M
M=D
@SP
M=M+1
@ARG
D=M
@0
D=D+A
@13
M=D
@SP
M=M-1
@SP
A=M
D=M
@13
A=M
M=D
@ARG
D=M
@0
D=D+A
@13
M=D
@13
A=M
D=M
@SP
A=M
M=D
@SP
M=M+1
@SP
M=M-1
@SP
A=M
D=M
@LOOP_START
D;JNE
@LCL
D=M
@0
D=D+A
@13
M=D
@13
A=M
D=M
@SP
A=M
M=D
@SP
M=M+1
