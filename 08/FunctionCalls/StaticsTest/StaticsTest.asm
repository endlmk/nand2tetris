@256
D=A
@SP
M=D
@Sys.init$RETURN_ADDR.0
D=A
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@SP
A=M
M=D
@SP
M=M+1
@ARG
D=M
@SP
A=M
M=D
@SP
M=M+1
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1
@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1
@SP
D=M
@0
D=D-A
@5
D=D-A
@ARG
M=D
@SP
D=M
@LCL
M=D
@Sys.init
0;JMP
(Sys.init$RETURN_ADDR.0)
(Sys.init)
@0
D=A
@14
M=D
(Sys.init$LOOP_START)
@14
D=M
@Sys.init$LOOP_END
D;JEQ
@0
D=A
@SP
A=M
M=D
@SP
M=M+1
@14
M=M-1
@Sys.init$LOOP_START
0;JMP
(Sys.init$LOOP_END)
@6
D=A
@SP
A=M
M=D
@SP
M=M+1
@8
D=A
@SP
A=M
M=D
@SP
M=M+1
@Class1.set$RETURN_ADDR.1
D=A
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@SP
A=M
M=D
@SP
M=M+1
@ARG
D=M
@SP
A=M
M=D
@SP
M=M+1
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1
@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1
@SP
D=M
@2
D=D-A
@5
D=D-A
@ARG
M=D
@SP
D=M
@LCL
M=D
@Class1.set
0;JMP
(Class1.set$RETURN_ADDR.1)
@SP
M=M-1
@SP
A=M
D=M
@5
M=D
@23
D=A
@SP
A=M
M=D
@SP
M=M+1
@15
D=A
@SP
A=M
M=D
@SP
M=M+1
@Class2.set$RETURN_ADDR.2
D=A
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@SP
A=M
M=D
@SP
M=M+1
@ARG
D=M
@SP
A=M
M=D
@SP
M=M+1
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1
@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1
@SP
D=M
@2
D=D-A
@5
D=D-A
@ARG
M=D
@SP
D=M
@LCL
M=D
@Class2.set
0;JMP
(Class2.set$RETURN_ADDR.2)
@SP
M=M-1
@SP
A=M
D=M
@5
M=D
@Class1.get$RETURN_ADDR.3
D=A
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@SP
A=M
M=D
@SP
M=M+1
@ARG
D=M
@SP
A=M
M=D
@SP
M=M+1
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1
@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1
@SP
D=M
@0
D=D-A
@5
D=D-A
@ARG
M=D
@SP
D=M
@LCL
M=D
@Class1.get
0;JMP
(Class1.get$RETURN_ADDR.3)
@Class2.get$RETURN_ADDR.4
D=A
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@SP
A=M
M=D
@SP
M=M+1
@ARG
D=M
@SP
A=M
M=D
@SP
M=M+1
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1
@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1
@SP
D=M
@0
D=D-A
@5
D=D-A
@ARG
M=D
@SP
D=M
@LCL
M=D
@Class2.get
0;JMP
(Class2.get$RETURN_ADDR.4)
(WHILE)
@WHILE
0;JMP
(Class1.set)
@0
D=A
@14
M=D
(Class1.set$LOOP_START)
@14
D=M
@Class1.set$LOOP_END
D;JEQ
@0
D=A
@SP
A=M
M=D
@SP
M=M+1
@14
M=M-1
@Class1.set$LOOP_START
0;JMP
(Class1.set$LOOP_END)
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
@Class1.0
M=D
@ARG
D=M
@1
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
@Class1.1
M=D
@0
D=A
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@13
M=D
@5
D=D-A
A=D
D=M
@14
M=D
@SP
M=M-1
@SP
A=M
D=M
@ARG
A=M
M=D
@ARG
D=M+1
@SP
M=D
@13
D=M
@1
D=D-A
A=D
D=M
@THAT
M=D
@13
D=M
@2
D=D-A
A=D
D=M
@THIS
M=D
@13
D=M
@3
D=D-A
A=D
D=M
@ARG
M=D
@13
D=M
@4
D=D-A
A=D
D=M
@LCL
M=D
@14
A=M
0;JMP
(Class1.get)
@0
D=A
@14
M=D
(Class1.get$LOOP_START)
@14
D=M
@Class1.get$LOOP_END
D;JEQ
@0
D=A
@SP
A=M
M=D
@SP
M=M+1
@14
M=M-1
@Class1.get$LOOP_START
0;JMP
(Class1.get$LOOP_END)
@Class1.0
D=M
@SP
A=M
M=D
@SP
M=M+1
@Class1.1
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
D=M-D
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@13
M=D
@5
D=D-A
A=D
D=M
@14
M=D
@SP
M=M-1
@SP
A=M
D=M
@ARG
A=M
M=D
@ARG
D=M+1
@SP
M=D
@13
D=M
@1
D=D-A
A=D
D=M
@THAT
M=D
@13
D=M
@2
D=D-A
A=D
D=M
@THIS
M=D
@13
D=M
@3
D=D-A
A=D
D=M
@ARG
M=D
@13
D=M
@4
D=D-A
A=D
D=M
@LCL
M=D
@14
A=M
0;JMP
(Class2.set)
@0
D=A
@14
M=D
(Class2.set$LOOP_START)
@14
D=M
@Class2.set$LOOP_END
D;JEQ
@0
D=A
@SP
A=M
M=D
@SP
M=M+1
@14
M=M-1
@Class2.set$LOOP_START
0;JMP
(Class2.set$LOOP_END)
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
@Class2.0
M=D
@ARG
D=M
@1
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
@Class2.1
M=D
@0
D=A
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@13
M=D
@5
D=D-A
A=D
D=M
@14
M=D
@SP
M=M-1
@SP
A=M
D=M
@ARG
A=M
M=D
@ARG
D=M+1
@SP
M=D
@13
D=M
@1
D=D-A
A=D
D=M
@THAT
M=D
@13
D=M
@2
D=D-A
A=D
D=M
@THIS
M=D
@13
D=M
@3
D=D-A
A=D
D=M
@ARG
M=D
@13
D=M
@4
D=D-A
A=D
D=M
@LCL
M=D
@14
A=M
0;JMP
(Class2.get)
@0
D=A
@14
M=D
(Class2.get$LOOP_START)
@14
D=M
@Class2.get$LOOP_END
D;JEQ
@0
D=A
@SP
A=M
M=D
@SP
M=M+1
@14
M=M-1
@Class2.get$LOOP_START
0;JMP
(Class2.get$LOOP_END)
@Class2.0
D=M
@SP
A=M
M=D
@SP
M=M+1
@Class2.1
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
D=M-D
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@13
M=D
@5
D=D-A
A=D
D=M
@14
M=D
@SP
M=M-1
@SP
A=M
D=M
@ARG
A=M
M=D
@ARG
D=M+1
@SP
M=D
@13
D=M
@1
D=D-A
A=D
D=M
@THAT
M=D
@13
D=M
@2
D=D-A
A=D
D=M
@THIS
M=D
@13
D=M
@3
D=D-A
A=D
D=M
@ARG
M=D
@13
D=M
@4
D=D-A
A=D
D=M
@LCL
M=D
@14
A=M
0;JMP
