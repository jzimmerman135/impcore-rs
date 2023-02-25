; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

%FILE = type opaque

@fmt_ln = private unnamed_addr constant [4 x i8] c"%i\0A\00", align 1
@fmt_i = private unnamed_addr constant [3 x i8] c"%i\00", align 1
@fmt_u = private unnamed_addr constant [3 x i8] c"%u\00", align 1
@fmt_c = private unnamed_addr constant [3 x i8] c"%c\00", align 1
@fmt_str = private unnamed_addr constant [3 x i8] c"%s\00", align 1
@res = global i32* null
@a = global i32* null
@x = global i32* null
@__stdin = global i8* null
@__fdopen_arg_read = private unnamed_addr constant [2 x i8] c"r\00", align 1

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

define i32 @printc(i32 %0) {
entry:
  %putchar = tail call i32 @putchar(i32 %0)
  ret i32 %0
}

; Function Attrs: nofree nounwind
declare noundef i32 @putchar(i32 noundef) #0

define i32 @printstr(i32* %0) {
entry:
  %printfcall = tail call i32 (i8*, ...) @printf(i8* noundef nonnull dereferenceable(1) getelementptr inbounds ([3 x i8], [3 x i8]* @fmt_str, i64 0, i64 0), i32* %0)
  ret i32 0
}

declare %FILE* @fdopen(i32, i8*)

declare i32 @fgetc(%FILE*)

define void @__init_stdin() {
entry:
  %fp = alloca i8**, align 8
  store i8** @__stdin, i8*** %fp, align 8
  %fdopen = call %FILE* @fdopen(i32 0, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @__fdopen_arg_read, i32 0, i32 0))
  %voidcast = bitcast %FILE* %fdopen to i8*
  %load = load i8**, i8*** %fp, align 8
  store i8* %voidcast, i8** %load, align 8
  ret void
}

define i32 @getc() {
entry:
  %stdin = load i8*, i8** @__stdin, align 8
  %fp = bitcast i8* %stdin to %FILE*
  %call = call i32 @fgetc(%FILE* %fp)
  ret i32 %call
}

define i32 @val() {
entry:
  %load = load i32*, i32** @res, align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32))
  %single = bitcast i8* %malloccall to i32*
  store i32 -1, i32* %single, align 4
  store i32* %single, i32** @res, align 8
  ret i32 -1
}

declare void @free(i8*)

declare noalias i8* @malloc(i32)

define i32 @val.1() {
entry:
  %load = load i32*, i32** @a, align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32))
  %single = bitcast i8* %malloccall to i32*
  store i32 97, i32* %single, align 4
  store i32* %single, i32** @a, align 8
  ret i32 97
}

define i32 @val.2() {
entry:
  %userfn = call i32 @getc()
  %load = load i32*, i32** @x, align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32))
  %single = bitcast i8* %malloccall to i32*
  store i32 %userfn, i32* %single, align 4
  store i32* %single, i32** @x, align 8
  ret i32 %userfn
}

define i32 @"#anon"() {
entry:
  %load = load i32*, i32** @res, align 8
  %load1 = load i32*, i32** @x, align 8
  %load2 = load i32, i32* %load1, align 4
  %alloca = alloca i32, align 4
  switch i32 %load2, label %default [
    i32 97, label %case
    i32 98, label %case3
  ]

default:                                          ; preds = %entry
  store i32 %load2, i32* %alloca, align 4
  br label %merge

case:                                             ; preds = %entry
  store i32 0, i32* %alloca, align 4
  br label %merge

case3:                                            ; preds = %entry
  store i32 1, i32* %alloca, align 4
  br label %merge

merge:                                            ; preds = %case3, %case, %default
  %load4 = load i32, i32* %alloca, align 4
  store i32 %load4, i32* %load, align 4
  ret i32 %load4
}

define i32 @"#anon.3"() {
entry:
  %load = load i32*, i32** @res, align 8
  %load1 = load i32, i32* %load, align 4
  %print = call i32 @print(i32 %load1)
  ret i32 %print
}

attributes #0 = { nofree nounwind }
