; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

@n = global i32* null
@t = global i32* null

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
  %array = bitcast i8* %malloccall to i32*
  store i32* %array, i32** @n, align 8
  store i32 2, i32* %array, align 4
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
  %array = bitcast i8* %malloccall to i32*
  store i32* %array, i32** @t, align 8
  store i32 7, i32* %array, align 4
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
  %array = bitcast i8* %malloccall to i32*
  store i32* %array, i32** @t, align 8
  store i32 11, i32* %array, align 4
  ret i32 11
}

define i32 @"#anon.11"() {
entry:
  %load = load i32*, i32** @t, align 8
  %load1 = load i32, i32* %load, align 4
  %mul = add i32 %load1, 1
  ret i32 %mul
}
