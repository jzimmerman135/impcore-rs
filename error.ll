; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

@fmt_ln = private unnamed_addr constant [4 x i8] c"%i\0A\00", align 1
@fmt_i = private unnamed_addr constant [3 x i8] c"%i\00", align 1
@fmt_u = private unnamed_addr constant [3 x i8] c"%u\00", align 1
@fmt_c = private unnamed_addr constant [3 x i8] c"%c\00", align 1
@fmt_str = private unnamed_addr constant [3 x i8] c"%s\00", align 1
@n = global i32* null
@t = global i32* null
@"arr[" = global i32* null
@"swaparr[" = global i32* null
@bits = global i32* null
@"nums[" = global i32* null

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

define i32 @add-ten(i32 %x) {
add-ten:
  %mul = add i32 %x, 10
  ret i32 %mul
}

define i32 @locals(i32 %n, i32 %x) {
locals:
  %gt = icmp sgt i32 %n, 0
  %. = select i1 %gt, i32 14, i32 -13
  %mul = add i32 %., %x
  ret i32 %mul
}

define i32 @set-global-with-loop(i32 %x) {
set-global-with-loop:
  %load1.pre = load i32*, i32** @t, align 8
  %load3.pre = load i32, i32* %load1.pre, align 4
  br label %loop

loop:                                             ; preds = %loop, %set-global-with-loop
  %load3 = phi i32 [ %load5, %loop ], [ %load3.pre, %set-global-with-loop ]
  %load1 = phi i32* [ %load4, %loop ], [ %load1.pre, %set-global-with-loop ]
  %mul = add i32 %load3, 1
  store i32 %mul, i32* %load1, align 4
  %load4 = load i32*, i32** @t, align 8
  %load5 = load i32, i32* %load4, align 4
  %sub = add i32 %load5, -1
  %lt = icmp slt i32 %sub, %x
  br i1 %lt, label %loop, label %afterwhile

afterwhile:                                       ; preds = %loop
  ret i32 %x
}

define i32 @sum-arr(i32 %i, i32 %sum) {
sum-arr:
  br label %tailrecurse

tailrecurse:                                      ; preds = %else, %sum-arr
  %i.tr = phi i32 [ %i, %sum-arr ], [ %decr, %else ]
  %sum.tr = phi i32 [ %sum, %sum-arr ], [ %mul, %else ]
  %eq = icmp eq i32 %i.tr, 0
  br i1 %eq, label %ifcont, label %else

else:                                             ; preds = %tailrecurse
  %decr = add i32 %i.tr, -1
  %load5 = load i32*, i32** @"arr[", align 8
  %0 = sext i32 %i.tr to i64
  %index = getelementptr i32, i32* %load5, i64 %0
  %load7 = load i32, i32* %index, align 4
  %mul = add i32 %load7, %sum.tr
  br label %tailrecurse

ifcont:                                           ; preds = %tailrecurse
  ret i32 %sum.tr
}

define i32 @array-xor-swap(i32* %"A[", i32 %i, i32 %j) {
array-xor-swap:
  %0 = sext i32 %i to i64
  %index = getelementptr i32, i32* %"A[", i64 %0
  %load7 = load i32, i32* %index, align 4
  %1 = sext i32 %j to i64
  %index10 = getelementptr i32, i32* %"A[", i64 %1
  %load11 = load i32, i32* %index10, align 4
  %xor = xor i32 %load11, %load7
  store i32 %xor, i32* %index, align 4
  %load18 = load i32, i32* %index10, align 4
  %xor23 = xor i32 %load18, %xor
  store i32 %xor23, i32* %index10, align 4
  %load30 = load i32, i32* %index, align 4
  %xor35 = xor i32 %load30, %xor23
  store i32 %xor35, i32* %index, align 4
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

define i32 @"#anon"() {
entry:
  %printres = call i32 @println(i32 0)
  ret i32 0
}

define i32 @"#anon.1"() {
entry:
  %printres = call i32 @println(i32 1)
  ret i32 1
}

define i32 @val() {
entry:
  %load = load i32*, i32** @n, align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32))
  %single = bitcast i8* %malloccall to i32*
  store i32 2, i32* %single, align 4
  store i32* %single, i32** @n, align 8
  %printres = call i32 @println(i32 2)
  ret i32 2
}

declare void @free(i8*)

declare noalias i8* @malloc(i32)

define i32 @"#anon.2"() {
entry:
  %userfn = call i32 @add-ten(i32 -7)
  %printres = call i32 @println(i32 %userfn)
  ret i32 %userfn
}

define i32 @"#anon.3"() {
entry:
  %userfn = call i32 @locals(i32 8, i32 -10)
  %printres = call i32 @println(i32 %userfn)
  ret i32 %userfn
}

define i32 @"#anon.4"() {
entry:
  %userfn = call i32 @locals(i32 -8, i32 18)
  %printres = call i32 @println(i32 %userfn)
  ret i32 %userfn
}

define i32 @"#anon.5"() {
entry:
  %load = load i32*, i32** @n, align 8
  %load1 = load i32, i32* %load, align 4
  %mul = add i32 %load1, 4
  %printres = call i32 @println(i32 %mul)
  ret i32 %mul
}

define i32 @val.6() {
entry:
  %load = load i32*, i32** @n, align 8
  store i32 7, i32* %load, align 4
  %load1 = load i32*, i32** @t, align 8
  %0 = bitcast i32* %load1 to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32))
  %single = bitcast i8* %malloccall to i32*
  store i32 7, i32* %single, align 4
  store i32* %single, i32** @t, align 8
  %printres = call i32 @println(i32 7)
  ret i32 7
}

define i32 @"#anon.7"() {
entry:
  %userfn = call i32 @set-global-with-loop(i32 8)
  %printres = call i32 @println(i32 %userfn)
  ret i32 %userfn
}

define i32 @"#anon.8"() {
entry:
  %load = load i32*, i32** @t, align 8
  %load1 = load i32, i32* %load, align 4
  %printres = call i32 @println(i32 %load1)
  ret i32 %load1
}

define i32 @"#anon.9"() {
entry:
  %load = load i32*, i32** @n, align 8
  %load1 = load i32, i32* %load, align 4
  %mul = add i32 %load1, 3
  %printres = call i32 @println(i32 %mul)
  ret i32 %mul
}

define i32 @val.10() {
entry:
  %load = load i32*, i32** @t, align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32))
  %single = bitcast i8* %malloccall to i32*
  store i32 11, i32* %single, align 4
  store i32* %single, i32** @t, align 8
  %printres = call i32 @println(i32 11)
  ret i32 11
}

define i32 @"#anon.11"() {
entry:
  %load = load i32*, i32** @t, align 8
  %load1 = load i32, i32* %load, align 4
  %mul = add i32 %load1, 1
  %printres = call i32 @println(i32 %mul)
  ret i32 %mul
}

define i32 @val.12() {
entry:
  %load = load i32*, i32** @"arr[", align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 mul (i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32), i32 13))
  %array = bitcast i8* %malloccall to i32*
  %1 = bitcast i32* %array to i8*
  call void @llvm.memset.p0i8.i32(i8* align 4 %1, i8 0, i32 mul (i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32), i32 13), i1 false)
  store i32* %array, i32** @"arr[", align 8
  %printres = call i32 @println(i32 13)
  ret i32 13
}

; Function Attrs: argmemonly nofree nounwind willreturn writeonly
declare void @llvm.memset.p0i8.i32(i8* nocapture writeonly, i8, i32, i1 immarg) #1

define i32 @"#anon.13"() {
entry:
  %load = load i32*, i32** @"arr[", align 8
  %index = getelementptr i32, i32* %load, i32 3
  store i32 14, i32* %index, align 4
  %printres = call i32 @println(i32 14)
  ret i32 14
}

define i32 @"#anon.14"() {
entry:
  %load = load i32*, i32** @"arr[", align 8
  %index = getelementptr i32, i32* %load, i32 9
  store i32 15, i32* %index, align 4
  %printres = call i32 @println(i32 15)
  ret i32 15
}

define i32 @"#anon.15"() {
entry:
  %load = load i32*, i32** @"arr[", align 8
  %index = getelementptr i32, i32* %load, i32 3
  %load1 = load i32, i32* %index, align 4
  %mul = add i32 2, %load1
  %printres = call i32 @println(i32 %mul)
  ret i32 %mul
}

define i32 @"#anon.16"() {
entry:
  %load = load i32*, i32** @"arr[", align 8
  %index = getelementptr i32, i32* %load, i32 9
  %load1 = load i32, i32* %index, align 4
  %mul = add i32 2, %load1
  %printres = call i32 @println(i32 %mul)
  ret i32 %mul
}

define i32 @"#anon.17"() {
entry:
  %load = load i32*, i32** @"arr[", align 8
  %index = getelementptr i32, i32* %load, i32 3
  store i32 18, i32* %index, align 4
  %printres = call i32 @println(i32 18)
  ret i32 18
}

define i32 @"#anon.18"() {
entry:
  %load = load i32*, i32** @"arr[", align 8
  %index = getelementptr i32, i32* %load, i32 3
  %load1 = load i32, i32* %index, align 4
  %mul = add i32 %load1, 1
  %printres = call i32 @println(i32 %mul)
  ret i32 %mul
}

define i32 @"#anon.19"() {
entry:
  %userfn = call i32 @sum-arr(i32 12, i32 0)
  %sub = sub i32 %userfn, 13
  %printres = call i32 @println(i32 %sub)
  ret i32 %sub
}

define i32 @val.20() {
entry:
  %load = load i32*, i32** @"swaparr[", align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 mul (i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32), i32 21))
  %array = bitcast i8* %malloccall to i32*
  %1 = bitcast i32* %array to i8*
  call void @llvm.memset.p0i8.i32(i8* align 4 %1, i8 0, i32 mul (i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32), i32 21), i1 false)
  store i32* %array, i32** @"swaparr[", align 8
  %printres = call i32 @println(i32 21)
  ret i32 21
}

define i32 @"#anon.21"() {
entry:
  %load = load i32*, i32** @"swaparr[", align 8
  %index = getelementptr i32, i32* %load, i32 0
  store i32 22, i32* %index, align 4
  %printres = call i32 @println(i32 22)
  ret i32 22
}

define i32 @"#anon.22"() {
entry:
  %load = load i32*, i32** @"swaparr[", align 8
  %index = getelementptr i32, i32* %load, i32 6
  store i32 23, i32* %index, align 4
  %printres = call i32 @println(i32 23)
  ret i32 23
}

define i32 @"#anon.23"() {
entry:
  %load = load i32*, i32** @"swaparr[", align 8
  %userfn = call i32 @array-xor-swap(i32* %load, i32 0, i32 6)
  %mul = add i32 24, %userfn
  %printres = call i32 @println(i32 %mul)
  ret i32 %mul
}

define i32 @"#anon.24"() {
entry:
  %load = load i32*, i32** @"swaparr[", align 8
  %index = getelementptr i32, i32* %load, i32 6
  %load1 = load i32, i32* %index, align 4
  %mul = add i32 3, %load1
  %printres = call i32 @println(i32 %mul)
  ret i32 %mul
}

define i32 @"#anon.25"() {
entry:
  %load = load i32*, i32** @"swaparr[", align 8
  %index = getelementptr i32, i32* %load, i32 0
  %load1 = load i32, i32* %index, align 4
  %mul = add i32 3, %load1
  %printres = call i32 @println(i32 %mul)
  ret i32 %mul
}

define i32 @val.26() {
entry:
  %load = load i32*, i32** @bits, align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32))
  %single = bitcast i8* %malloccall to i32*
  store i32 27, i32* %single, align 4
  store i32* %single, i32** @bits, align 8
  %printres = call i32 @println(i32 27)
  ret i32 27
}

define i32 @"#anon.27"() {
entry:
  %load = load i32*, i32** @bits, align 8
  %load1 = load i32, i32* %load, align 4
  %bitand = and i32 %load1, 24
  %bitor = or i32 %bitand, 4
  %printres = call i32 @println(i32 %bitor)
  ret i32 %bitor
}

define i32 @val.28() {
entry:
  %load = load i32*, i32** @"nums[", align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 mul (i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32), i32 29))
  %array = bitcast i8* %malloccall to i32*
  %1 = bitcast i32* %array to i8*
  call void @llvm.memset.p0i8.i32(i8* align 4 %1, i8 0, i32 mul (i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32), i32 29), i1 false)
  store i32* %array, i32** @"nums[", align 8
  %printres = call i32 @println(i32 29)
  ret i32 29
}

define i32 @"#anon.29"() {
entry:
  %load = load i32*, i32** @"nums[", align 8
  %index = getelementptr i32, i32* %load, i32 0
  %userfn = call i32 @word(i32 51, i32 48, i32 10, i32 51)
  store i32 %userfn, i32* %index, align 4
  %load1 = load i32*, i32** @"nums[", align 8
  %index2 = getelementptr i32, i32* %load1, i32 1
  %userfn3 = call i32 @word(i32 49, i32 10, i32 0, i32 0)
  store i32 %userfn3, i32* %index2, align 4
  %load4 = load i32*, i32** @"nums[", align 8
  %userfn5 = call i32 @printstr(i32* %load4)
  %printres = call i32 @println(i32 32)
  ret i32 32
}

define i32 @"#anon.30"() {
entry:
  %print = call i32 @print(i32 3)
  %printres = call i32 @println(i32 %print)
  ret i32 %print
}

attributes #0 = { nofree nounwind }
attributes #1 = { argmemonly nofree nounwind willreturn writeonly }
