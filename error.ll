; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

%FILE = type opaque

@fmt_ln = private unnamed_addr constant [4 x i8] c"%i\0A\00", align 1
@fmt_i = private unnamed_addr constant [3 x i8] c"%i\00", align 1
@fmt_u = private unnamed_addr constant [3 x i8] c"%u\00", align 1
@fmt_str = private unnamed_addr constant [3 x i8] c"%s\00", align 1
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

define i32 @"#anon"() {
entry:
  %printres = call i32 @println(i32 1)
  ret i32 1
}

define i32 @"#anon.1"() {
entry:
  %printres = call i32 @println(i32 16)
  ret i32 16
}

define i32 @no-args() {
no-args:
  ret i32 45
}

define i32 @"#anon.2"() {
entry:
  %userfn = call i32 @no-args()
  %printres = call i32 @println(i32 %userfn)
  ret i32 %userfn
}

define i32 @val() {
entry:
  %load = load i32*, i32** @x, align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32))
  %single = bitcast i8* %malloccall to i32*
  store i32 10, i32* %single, align 4
  store i32* %single, i32** @x, align 8
  %printres = call i32 @println(i32 10)
  ret i32 10
}

declare void @free(i8*)

declare noalias i8* @malloc(i32)

define i32 @"#anon.3"() {
entry:
  %userfn = call i32 @getc()
  %printres = call i32 @println(i32 %userfn)
  ret i32 %userfn
}

define i32 @"#anon.4"() {
entry:
  %load = load i32*, i32** @x, align 8
  %load1 = load i32, i32* %load, align 4
  %printres = call i32 @println(i32 %load1)
  ret i32 %load1
}
