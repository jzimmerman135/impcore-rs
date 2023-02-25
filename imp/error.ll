; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

%FILE = type opaque

@fmt_ln = private unnamed_addr constant [4 x i8] c"%i\0A\00", align 1
@fmt_i = private unnamed_addr constant [3 x i8] c"%i\00", align 1
@fmt_u = private unnamed_addr constant [3 x i8] c"%u\00", align 1
@fmt_c = private unnamed_addr constant [3 x i8] c"%c\00", align 1
@fmt_str = private unnamed_addr constant [3 x i8] c"%s\00", align 1
@bufsize = global i32* null
@"buffer[" = global i32* null
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

define i32 @word(i32 %a, i32 %b, i32 %c, i32 %d) {
word:
  %bitand = and i32 %a, 255
  %bitand5 = shl i32 %b, 8
  %shiftl = and i32 %bitand5, 65280
  %bitand7 = shl i32 %c, 16
  %shiftl8 = and i32 %bitand7, 16711680
  %shiftl11 = shl i32 %d, 24
  %bitor = add nuw nsw i32 %shiftl8, %shiftl11
  %bitor12 = add nuw nsw i32 %bitor, %shiftl
  %bitor13 = add nuw nsw i32 %bitor12, %bitand
  ret i32 %bitor13
}

define i32 @addhello(i32* %"buffer[", i32 %i) {
addhello:
  %0 = sext i32 %i to i64
  %index = getelementptr i32, i32* %"buffer[", i64 %0
  %userfn = tail call i32 @word(i32 104, i32 101, i32 108, i32 108)
  store i32 %userfn, i32* %index, align 4
  %mul5 = add i32 %i, 1
  %1 = sext i32 %mul5 to i64
  %index6 = getelementptr i32, i32* %"buffer[", i64 %1
  %userfn7 = tail call i32 @word(i32 111, i32 32, i32 119, i32 111)
  store i32 %userfn7, i32* %index6, align 4
  %mul10 = add i32 %i, 2
  %2 = sext i32 %mul10 to i64
  %index11 = getelementptr i32, i32* %"buffer[", i64 %2
  %userfn12 = tail call i32 @word(i32 114, i32 108, i32 100, i32 33)
  store i32 %userfn12, i32* %index11, align 4
  %mul15 = add i32 %i, 3
  %3 = sext i32 %mul15 to i64
  %index16 = getelementptr i32, i32* %"buffer[", i64 %3
  %userfn17 = tail call i32 @word(i32 10, i32 0, i32 0, i32 0)
  store i32 %userfn17, i32* %index16, align 4
  ret i32 %userfn17
}

define i32 @val() {
entry:
  %load = load i32*, i32** @bufsize, align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32))
  %single = bitcast i8* %malloccall to i32*
  store i32 81, i32* %single, align 4
  store i32* %single, i32** @bufsize, align 8
  %printres = call i32 @println(i32 81)
  ret i32 81
}

declare void @free(i8*)

declare noalias i8* @malloc(i32)

define i32 @val.1() {
entry:
  %load = load i32*, i32** @bufsize, align 8
  %load1 = load i32, i32* %load, align 4
  %load2 = load i32*, i32** @"buffer[", align 8
  %0 = bitcast i32* %load2 to i8*
  tail call void @free(i8* %0)
  %bytes = mul i32 %load1, ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32)
  %mallocsize = mul i32 %load1, ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32)
  %malloccall = tail call i8* @malloc(i32 %mallocsize)
  %array = bitcast i8* %malloccall to i32*
  %1 = bitcast i32* %array to i8*
  call void @llvm.memset.p0i8.i32(i8* align 4 %1, i8 0, i32 %bytes, i1 false)
  store i32* %array, i32** @"buffer[", align 8
  %printres = call i32 @println(i32 %load1)
  ret i32 %load1
}

; Function Attrs: argmemonly nofree nounwind willreturn writeonly
declare void @llvm.memset.p0i8.i32(i8* nocapture writeonly, i8, i32, i1 immarg) #1

define i32 @"#anon"() {
entry:
  %load = load i32*, i32** @"buffer[", align 8
  %userfn = call i32 @addhello(i32* %load, i32 0)
  %printres = call i32 @println(i32 %userfn)
  ret i32 %userfn
}

define i32 @"#anon.2"() {
entry:
  %load = load i32*, i32** @"buffer[", align 8
  %userfn = call i32 @printstr(i32* %load)
  %printres = call i32 @println(i32 %userfn)
  ret i32 %userfn
}

attributes #0 = { nofree nounwind }
attributes #1 = { argmemonly nofree nounwind willreturn writeonly }
