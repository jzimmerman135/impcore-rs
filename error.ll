; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

@fmt_ln = private unnamed_addr constant [4 x i8] c"%i\0A\00", align 1
@fmt_i = private unnamed_addr constant [3 x i8] c"%i\00", align 1
@fmt_u = private unnamed_addr constant [3 x i8] c"%u\00", align 1
@fmt_str = private unnamed_addr constant [3 x i8] c"%s\00", align 1
@"message[" = global i32* null

declare i32 @main()

declare i32 @printf(i8*, ...)

define i32 @println(i32 %0) {
entry:
  %printfcall = tail call i32 (i8*, ...) @printf(i8* noundef nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @fmt_ln, i64 0, i64 0), i32 %0)
  ret i32 %0
}

define i32 @print(i32 %0) {
entry:
  %printfcall = tail call i32 (i8*, ...) @printf(i8* noundef nonnull dereferenceable(1) getelementptr inbounds ([3 x i8], [3 x i8]* @fmt_i, i64 0, i64 0), i32 %0)
  ret i32 %0
}

define i32 @printu(i32 %0) {
entry:
  %printfcall = tail call i32 (i8*, ...) @printf(i8* noundef nonnull dereferenceable(1) getelementptr inbounds ([3 x i8], [3 x i8]* @fmt_u, i64 0, i64 0), i32 %0)
  ret i32 %0
}

define i32 @printstr(i32* %0) {
entry:
  %cast = bitcast i32* %0 to i8*
  %printfcall = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @fmt_str, i32 0, i32 0), i8* %cast)
  ret i32 0
}

define i32 @char(i32 %x) {
char:
  %bitand = and i32 %x, 255
  ret i32 %bitand
}

define i32 @word(i32 %a, i32 %b, i32 %c, i32 %d) {
word:
  %userfn = tail call i32 @char(i32 %a)
  %userfn5 = tail call i32 @char(i32 %b)
  %shiftl = shl i32 %userfn5, 8
  %userfn7 = tail call i32 @char(i32 %c)
  %shiftl8 = shl i32 %userfn7, 16
  %userfn10 = tail call i32 @char(i32 %d)
  %shiftl11 = shl i32 %userfn10, 24
  %bitor = or i32 %shiftl, %userfn
  %bitor12 = or i32 %bitor, %shiftl8
  %bitor13 = or i32 %bitor12, %shiftl11
  ret i32 %bitor13
}

define i32 @val() {
entry:
  %load = load i32*, i32** @"message[", align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 mul (i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32), i32 4))
  %array = bitcast i8* %malloccall to i32*
  %1 = bitcast i32* %array to i8*
  call void @llvm.memset.p0i8.i32(i8* align 4 %1, i8 0, i32 4, i1 false)
  store i32* %array, i32** @"message[", align 8
  ret i32 4
}

declare void @free(i8*)

declare noalias i8* @malloc(i32)

; Function Attrs: argmemonly nofree nounwind willreturn writeonly
declare void @llvm.memset.p0i8.i32(i8* nocapture writeonly, i8, i32, i1 immarg) #0

define i32 @"#anon"() {
entry:
  %load = load i32*, i32** @"message[", align 8
  %index = getelementptr i32, i32* %load, i32 0
  %userfn = call i32 @word(i32 72, i32 101, i32 108, i32 108)
  store i32 %userfn, i32* %index, align 4
  ret i32 %userfn
}

define i32 @"#anon.1"() {
entry:
  %load = load i32*, i32** @"message[", align 8
  %index = getelementptr i32, i32* %load, i32 1
  %userfn = call i32 @word(i32 111, i32 32, i32 87, i32 111)
  store i32 %userfn, i32* %index, align 4
  ret i32 %userfn
}

define i32 @"#anon.2"() {
entry:
  %load = load i32*, i32** @"message[", align 8
  %index = getelementptr i32, i32* %load, i32 2
  %userfn = call i32 @word(i32 114, i32 108, i32 100, i32 33)
  store i32 %userfn, i32* %index, align 4
  ret i32 %userfn
}

define i32 @"#anon.3"() {
entry:
  %load = load i32*, i32** @"message[", align 8
  %index = getelementptr i32, i32* %load, i32 3
  %userfn = call i32 @word(i32 10, i32 0, i32 0, i32 0)
  store i32 %userfn, i32* %index, align 4
  ret i32 %userfn
}

define i32 @"#anon.4"() {
entry:
  %load = load i32*, i32** @"message[", align 8
  %userfn = call i32 @printstr(i32* %load)
  ret i32 %userfn
}

attributes #0 = { argmemonly nofree nounwind willreturn writeonly }
