; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

@n = global i32* null
@t = global i32* null
@arr = global i32* null

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
  %load5 = load i32*, i32** @arr, align 8
  %0 = sext i32 %i.tr to i64
  %index = getelementptr i32, i32* %load5, i64 %0
  %load7 = load i32, i32* %index, align 4
  %mul = add i32 %load7, %sum.tr
  br label %tailrecurse

ifcont:                                           ; preds = %tailrecurse
  ret i32 %sum.tr
}

define i32 @"#anon"() {
entry:
  ret i32 0
}

define i32 @"#anon.1"() {
entry:
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
  ret i32 2
}

declare void @free(i8*)

declare noalias i8* @malloc(i32)

define i32 @"#anon.2"() {
entry:
  %userfn = call i32 @add-ten(i32 -7)
  ret i32 %userfn
}

define i32 @"#anon.3"() {
entry:
  %userfn = call i32 @locals(i32 8, i32 -10)
  ret i32 %userfn
}

define i32 @"#anon.4"() {
entry:
  %userfn = call i32 @locals(i32 -8, i32 18)
  ret i32 %userfn
}

define i32 @"#anon.5"() {
entry:
  %load = load i32*, i32** @n, align 8
  %load1 = load i32, i32* %load, align 4
  %mul = add i32 %load1, 4
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
  ret i32 7
}

define i32 @"#anon.7"() {
entry:
  %userfn = call i32 @set-global-with-loop(i32 8)
  ret i32 %userfn
}

define i32 @"#anon.8"() {
entry:
  %load = load i32*, i32** @t, align 8
  %load1 = load i32, i32* %load, align 4
  ret i32 %load1
}

define i32 @"#anon.9"() {
entry:
  %load = load i32*, i32** @n, align 8
  %load1 = load i32, i32* %load, align 4
  %mul = add i32 %load1, 3
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
  ret i32 11
}

define i32 @"#anon.11"() {
entry:
  %load = load i32*, i32** @t, align 8
  %load1 = load i32, i32* %load, align 4
  %mul = add i32 %load1, 1
  ret i32 %mul
}

define i32 @val.12() {
entry:
  %load = load i32*, i32** @arr, align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 mul (i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32), i32 13))
  %array = bitcast i8* %malloccall to i32*
  %1 = bitcast i32* %array to i8*
  call void @llvm.memset.p0i8.i32(i8* align 4 %1, i8 0, i32 13, i1 false)
  store i32* %array, i32** @arr, align 8
  ret i32 13
}

; Function Attrs: argmemonly nofree nounwind willreturn writeonly
declare void @llvm.memset.p0i8.i32(i8* nocapture writeonly, i8, i32, i1 immarg) #0

define i32 @"#anon.13"() {
entry:
  %load = load i32*, i32** @arr, align 8
  %index = getelementptr i32, i32* %load, i32 3
  store i32 14, i32* %index, align 4
  ret i32 14
}

define i32 @"#anon.14"() {
entry:
  %load = load i32*, i32** @arr, align 8
  %index = getelementptr i32, i32* %load, i32 9
  store i32 15, i32* %index, align 4
  ret i32 15
}

define i32 @"#anon.15"() {
entry:
  %load = load i32*, i32** @arr, align 8
  %index = getelementptr i32, i32* %load, i32 3
  %load1 = load i32, i32* %index, align 4
  %mul = add i32 2, %load1
  ret i32 %mul
}

define i32 @"#anon.16"() {
entry:
  %load = load i32*, i32** @arr, align 8
  %index = getelementptr i32, i32* %load, i32 9
  %load1 = load i32, i32* %index, align 4
  %mul = add i32 2, %load1
  ret i32 %mul
}

define i32 @"#anon.17"() {
entry:
  %load = load i32*, i32** @arr, align 8
  %index = getelementptr i32, i32* %load, i32 3
  store i32 18, i32* %index, align 4
  ret i32 18
}

define i32 @"#anon.18"() {
entry:
  %load = load i32*, i32** @arr, align 8
  %index = getelementptr i32, i32* %load, i32 3
  %load1 = load i32, i32* %index, align 4
  %mul = add i32 %load1, 1
  ret i32 %mul
}

define i32 @"#anon.19"() {
entry:
  %userfn = call i32 @sum-arr(i32 12, i32 0)
  %sub = sub i32 %userfn, 13
  ret i32 %sub
}

attributes #0 = { argmemonly nofree nounwind willreturn writeonly }
