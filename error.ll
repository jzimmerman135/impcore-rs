; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

%FILE = type opaque

@fmt_ln = private unnamed_addr constant [4 x i8] c"%i\0A\00", align 1
@fmt_i = private unnamed_addr constant [3 x i8] c"%i\00", align 1
@fmt_u = private unnamed_addr constant [3 x i8] c"%u\00", align 1
@fmt_c = private unnamed_addr constant [3 x i8] c"%c\00", align 1
@fmt_str = private unnamed_addr constant [3 x i8] c"%s\00", align 1
@"DEBUGMSG[" = global i32* null
@bufsize = global i32* null
@"mybuf[" = global i32* null
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

define i32 @char(i32 %i) {
char:
  %bitand = and i32 %i, 255
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
  %load = load i32*, i32** @"DEBUGMSG[", align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 mul (i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32), i32 2))
  %array = bitcast i8* %malloccall to i32*
  %1 = bitcast i32* %array to i8*
  call void @llvm.memset.p0i8.i32(i8* align 4 %1, i8 0, i32 mul (i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32), i32 2), i1 false)
  store i32* %array, i32** @"DEBUGMSG[", align 8
  ret i32 2
}

declare void @free(i8*)

declare noalias i8* @malloc(i32)

; Function Attrs: argmemonly nofree nounwind willreturn writeonly
declare void @llvm.memset.p0i8.i32(i8* nocapture writeonly, i8, i32, i1 immarg) #1

define i32 @"#anon"() {
entry:
  %load = load i32*, i32** @"DEBUGMSG[", align 8
  %index = getelementptr i32, i32* %load, i32 0
  %userfn = call i32 @word(i32 68, i32 69, i32 66, i32 85)
  store i32 %userfn, i32* %index, align 4
  ret i32 %userfn
}

define i32 @"#anon.1"() {
entry:
  %load = load i32*, i32** @"DEBUGMSG[", align 8
  %index = getelementptr i32, i32* %load, i32 1
  %userfn = call i32 @word(i32 71, i32 10, i32 0, i32 0)
  store i32 %userfn, i32* %index, align 4
  ret i32 %userfn
}

define i32 @DEBUG(i32 %x) {
DEBUG:
  %load = load i32*, i32** @"DEBUGMSG[", align 8
  %userfn = tail call i32 @printstr(i32* %load)
  ret i32 %x
}

define i32 @sgetc(i32 %i) {
sgetc:
  %userfn = tail call i32 @getc()
  %eq = icmp eq i32 %userfn, -1
  %.userfn = select i1 %eq, i32 0, i32 %userfn
  ret i32 %.userfn
}

define i32 @readstr(i32* %"buffer[", i32 %bufsize, i32 %i) {
readstr:
  br label %tailrecurse

tailrecurse:                                      ; preds = %else, %readstr
  %i.tr = phi i32 [ %i, %readstr ], [ %mul, %else ]
  %ge.not = icmp slt i32 %i.tr, %bufsize
  br i1 %ge.not, label %else, label %ifcont

else:                                             ; preds = %tailrecurse
  %0 = sext i32 %i.tr to i64
  %index = getelementptr i32, i32* %"buffer[", i64 %0
  %userfn = tail call i32 @sgetc(i32 0)
  %userfn6 = tail call i32 @sgetc(i32 0)
  %userfn7 = tail call i32 @sgetc(i32 0)
  %userfn8 = tail call i32 @sgetc(i32 0)
  %userfn9 = tail call i32 @word(i32 %userfn, i32 %userfn6, i32 %userfn7, i32 %userfn8)
  store i32 %userfn9, i32* %index, align 4
  %mul = add i32 %i.tr, 1
  br label %tailrecurse

ifcont:                                           ; preds = %tailrecurse
  ret i32 256
}

define i32 @add-newline(i32* %"buffer[", i32 %bufsize) {
add-newline:
  %sub = add i32 %bufsize, -1
  %0 = sext i32 %sub to i64
  %index = getelementptr i32, i32* %"buffer[", i64 %0
  %userfn = tail call i32 @word(i32 0, i32 0, i32 10, i32 0)
  %load7 = load i32, i32* %index, align 4
  %userfn8 = tail call i32 @word(i32 255, i32 255, i32 0, i32 0)
  %bitand = and i32 %userfn8, %load7
  %bitor = or i32 %bitand, %userfn
  store i32 %bitor, i32* %index, align 4
  ret i32 %bitor
}

define i32 @val.2() {
entry:
  %load = load i32*, i32** @bufsize, align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32))
  %single = bitcast i8* %malloccall to i32*
  store i32 256, i32* %single, align 4
  store i32* %single, i32** @bufsize, align 8
  ret i32 256
}

define i32 @val.3() {
entry:
  %load = load i32*, i32** @bufsize, align 8
  %load1 = load i32, i32* %load, align 4
  %load2 = load i32*, i32** @"mybuf[", align 8
  %0 = bitcast i32* %load2 to i8*
  tail call void @free(i8* %0)
  %bytes = mul i32 %load1, ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32)
  %mallocsize = mul i32 %load1, ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32)
  %malloccall = tail call i8* @malloc(i32 %mallocsize)
  %array = bitcast i8* %malloccall to i32*
  %1 = bitcast i32* %array to i8*
  call void @llvm.memset.p0i8.i32(i8* align 4 %1, i8 0, i32 %bytes, i1 false)
  store i32* %array, i32** @"mybuf[", align 8
  ret i32 %load1
}

define i32 @"#anon.4"() {
entry:
  %load = load i32*, i32** @"mybuf[", align 8
  %load1 = load i32*, i32** @bufsize, align 8
  %load2 = load i32, i32* %load1, align 4
  %userfn = call i32 @readstr(i32* %load, i32 %load2, i32 0)
  ret i32 %userfn
}

define i32 @"#anon.5"() {
entry:
  %load = load i32*, i32** @"mybuf[", align 8
  %userfn = call i32 @printstr(i32* %load)
  ret i32 %userfn
}

attributes #0 = { nofree nounwind }
attributes #1 = { argmemonly nofree nounwind willreturn writeonly }
