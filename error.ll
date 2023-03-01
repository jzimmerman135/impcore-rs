; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

@fmt_ln = private unnamed_addr constant [4 x i8] c"%i\0A\00", align 1
@fmt_i = private unnamed_addr constant [3 x i8] c"%i\00", align 1
@fmt_u = private unnamed_addr constant [3 x i8] c"%u\00", align 1
@fmt_c = private unnamed_addr constant [3 x i8] c"%c\00", align 1
@fmt_str = private unnamed_addr constant [3 x i8] c"%s\00", align 1
@x = global i32 0

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

define i32 @val() {
entry:
  store i32 9, i32* @x, align 4
  %printres = call i32 @println(i32 9)
  ret i32 9
}

define i32 @"#anon"() {
entry:
  %load = load i32, i32* @x, align 4
  %alloca = alloca i32, align 4
  switch i32 %load, label %default [
    i32 10, label %case
    i32 11, label %case1
  ]

default:                                          ; preds = %entry
  %load2 = load i32, i32* @x, align 4
  %mul = add i32 %load2, 100
  store i32 %mul, i32* %alloca, align 4
  br label %merge

case:                                             ; preds = %entry
  store i32 11, i32* %alloca, align 4
  br label %merge

case1:                                            ; preds = %entry
  store i32 12, i32* %alloca, align 4
  br label %merge

merge:                                            ; preds = %case1, %case, %default
  %load3 = load i32, i32* %alloca, align 4
  %printres = call i32 @println(i32 %load3)
  ret i32 %load3
}

attributes #0 = { nofree nounwind }
