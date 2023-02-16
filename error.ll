; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

@n = global i32* null
@myGlobal = global i32* null
@otherGlobal = global i32* null

define i32 @"#anon"() {
entry:
  ret i32 20
}

define i32 @add-ten(i32 %x) {
add-ten:
  %mul = add i32 %x, 10
  ret i32 %mul
}

define i32 @"#anon.1"() {
entry:
  %userfn = call i32 @add-ten(i32 75)
  ret i32 %userfn
}

define i32 @val() {
entry:
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32))
  %array = bitcast i8* %malloccall to i32*
  store i32* %array, i32** @n, align 8
  store i32 888888, i32* %array, align 4
  ret i32 888888
}

declare noalias i8* @malloc(i32)

define i32 @locals(i32 %n, i32 %x) {
locals:
  %gt.inv = icmp slt i32 %n, 1
  %. = select i1 %gt.inv, i32 3, i32 1
  %mul = add i32 %., %x
  ret i32 %mul
}

define i32 @"#anon.2"() {
entry:
  %userfn = call i32 @locals(i32 8, i32 12)
  ret i32 %userfn
}

define i32 @"#anon.3"() {
entry:
  %userfn = call i32 @locals(i32 -10, i32 12)
  ret i32 %userfn
}

define i32 @"#anon.4"() {
entry:
  %load = load i32*, i32** @n, align 8
  %load1 = load i32, i32* %load, align 4
  ret i32 %load1
}

define i32 @val.5() {
entry:
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32))
  %array = bitcast i8* %malloccall to i32*
  store i32* %array, i32** @myGlobal, align 8
  store i32 6, i32* %array, align 4
  ret i32 6
}

define i32 @val.6() {
entry:
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32))
  %array = bitcast i8* %malloccall to i32*
  store i32* %array, i32** @otherGlobal, align 8
  store i32 18, i32* %array, align 4
  ret i32 18
}

define i32 @"#anon.7"() {
entry:
  %load = load i32*, i32** @myGlobal, align 8
  %load1 = load i32, i32* %load, align 4
  ret i32 %load1
}

define i32 @"#anon.8"() {
entry:
  %load = load i32*, i32** @myGlobal, align 8
  store i32 9, i32* %load, align 4
  ret i32 9
}

define i32 @"#anon.9"() {
entry:
  %load = load i32*, i32** @myGlobal, align 8
  %load1 = load i32, i32* %load, align 4
  ret i32 %load1
}

define i32 @"#anon.10"() {
entry:
  %load = load i32*, i32** @otherGlobal, align 8
  %load1 = load i32, i32* %load, align 4
  ret i32 %load1
}

define i32 @val.11() {
entry:
  %malloccall = tail call i8* @malloc(i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32))
  %array = bitcast i8* %malloccall to i32*
  store i32* %array, i32** @otherGlobal, align 8
  store i32 12, i32* %array, align 4
  ret i32 12
}

define i32 @"#anon.12"() {
entry:
  %load = load i32*, i32** @otherGlobal, align 8
  %load1 = load i32, i32* %load, align 4
  ret i32 %load1
}

define i32 @side-effect(i32 %n) {
side-effect:
  %load = load i32*, i32** @myGlobal, align 8
  store i32 %n, i32* %load, align 4
  ret i32 %n
}

define i32 @"#anon.13"() {
entry:
  %userfn = call i32 @side-effect(i32 2579)
  ret i32 %userfn
}

define i32 @"#anon.14"() {
entry:
  %load = load i32*, i32** @myGlobal, align 8
  %load1 = load i32, i32* %load, align 4
  ret i32 %load1
}
