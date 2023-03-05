; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

@fmt_ln = private unnamed_addr constant [4 x i8] c"%i\0A\00", align 1
@fmt_i = private unnamed_addr constant [3 x i8] c"%i\00", align 1
@fmt_u = private unnamed_addr constant [3 x i8] c"%u\00", align 1
@fmt_c = private unnamed_addr constant [3 x i8] c"%c\00", align 1
@fmt_str = private unnamed_addr constant [3 x i8] c"%s\00", align 1
@"one[" = global i32* null

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

define i32 @fill-arr(i32* %"r[", i32 %i, i32 %v) {
fill-arr:
  %0 = sext i32 %i to i64
  %index = getelementptr i32, i32* %"r[", i64 %0
  store i32 %v, i32* %index, align 4
  ret i32 %v
}

define i32 @val() {
entry:
  %load = load i32*, i32** @"one[", align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 mul (i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32), i32 6))
  %array = bitcast i8* %malloccall to i32*
  %1 = bitcast i32* %array to i8*
  call void @llvm.memset.p0i8.i32(i8* align 4 %1, i8 0, i32 mul (i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32), i32 6), i1 false)
  store i32* %array, i32** @"one[", align 8
  %printres = call i32 @println(i32 6)
  ret i32 6
}

declare void @free(i8*)

declare noalias i8* @malloc(i32)

; Function Attrs: argmemonly nofree nounwind willreturn writeonly
declare void @llvm.memset.p0i8.i32(i8* nocapture writeonly, i8, i32, i1 immarg) #1

define i32 @add-one(i32 %x) {
add-one:
  %load1 = load i32*, i32** @"one[", align 8
  %load2 = load i32, i32* %load1, align 4
  %mul = add i32 %load2, %x
  ret i32 %mul
}

define i32 @add-two(i32 %x) {
add-two:
  %userfn = tail call i32 @add-one(i32 %x)
  %userfn1 = tail call i32 @add-one(i32 %userfn)
  ret i32 %userfn1
}

define i32 @add-three(i32 %x) {
add-three:
  %userfn = tail call i32 @add-two(i32 %x)
  %userfn1 = tail call i32 @add-one(i32 %userfn)
  ret i32 %userfn1
}

define i32 @"#anon"() {
entry:
  %load = load i32*, i32** @"one[", align 8
  %userfn = call i32 @fill-arr(i32* %load, i32 0, i32 1)
  %printres = call i32 @println(i32 %userfn)
  ret i32 %userfn
}

define i32 @"#anon.1"() {
entry:
  %userfn = call i32 @add-three(i32 2)
  %printres = call i32 @println(i32 %userfn)
  ret i32 %userfn
}

define i32 @"#anon.2"() {
entry:
  %userfn = call i32 @add-three(i32 9)
  %userfn1 = call i32 @add-one(i32 %userfn)
  %printres = call i32 @println(i32 %userfn1)
  ret i32 %userfn1
}

attributes #0 = { nofree nounwind }
attributes #1 = { argmemonly nofree nounwind willreturn writeonly }
