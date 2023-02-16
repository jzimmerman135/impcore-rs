; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

define i32 @"has-divisor?"(i32 %n, i32 %d) {
"has-divisor?":
  %alloca = alloca i32, align 4
  store i32 %n, i32* %alloca, align 4
  %alloca1 = alloca i32, align 4
  store i32 %d, i32* %alloca1, align 4
  %load = load i32, i32* %alloca1, align 4
  %load2 = load i32, i32* %alloca, align 4
  %div = sdiv i32 %load2, 2
  %gt = icmp sgt i32 %load, %div
  %cast = sext i1 %gt to i32
  %ifcond = icmp ne i32 %cast, 0
  br i1 %ifcond, label %then, label %else

then:                                             ; preds = %"has-divisor?"
  br label %ifcont

else:                                             ; preds = %"has-divisor?"
  %load3 = load i32, i32* %alloca, align 4
  %load4 = load i32, i32* %alloca1, align 4
  %mod = srem i32 %load3, %load4
  %eq = icmp eq i32 0, %mod
  %cast5 = sext i1 %eq to i32
  %ifcond6 = icmp ne i32 %cast5, 0
  br i1 %ifcond6, label %then7, label %else8

ifcont:                                           ; preds = %ifcont9, %then
  %iftmp12 = phi i32 [ 0, %then ], [ %iftmp, %ifcont9 ]
  ret i32 %iftmp12

then7:                                            ; preds = %else
  br label %ifcont9

else8:                                            ; preds = %else
  %load10 = load i32, i32* %alloca, align 4
  %load11 = load i32, i32* %alloca1, align 4
  %mul = add i32 %load11, 1
  %userfn = call i32 @"has-divisor?"(i32 %load10, i32 %mul)
  br label %ifcont9

ifcont9:                                          ; preds = %else8, %then7
  %iftmp = phi i32 [ 1, %then7 ], [ %userfn, %else8 ]
  br label %ifcont
}

define i32 @"prime?"(i32 %n) {
"prime?":
  %alloca = alloca i32, align 4
  store i32 %n, i32* %alloca, align 4
  %load = load i32, i32* %alloca, align 4
  %lt = icmp slt i32 %load, 2
  %cast = sext i1 %lt to i32
  %ifcond = icmp ne i32 %cast, 0
  br i1 %ifcond, label %then, label %else

then:                                             ; preds = %"prime?"
  br label %ifcont

else:                                             ; preds = %"prime?"
  %load1 = load i32, i32* %alloca, align 4
  %userfn = call i32 @"has-divisor?"(i32 %load1, i32 2)
  %not = xor i32 %userfn, -1
  br label %ifcont

ifcont:                                           ; preds = %else, %then
  %iftmp = phi i32 [ 0, %then ], [ %not, %else ]
  ret i32 %iftmp
}

define i32 @next-prime(i32 %p, i32 %n) {
next-prime:
  %alloca = alloca i32, align 4
  store i32 %p, i32* %alloca, align 4
  %alloca1 = alloca i32, align 4
  store i32 %n, i32* %alloca1, align 4
  %load = load i32, i32* %alloca, align 4
  %userfn = call i32 @"prime?"(i32 %load)
  %ifcond = icmp ne i32 %userfn, 0
  br i1 %ifcond, label %then, label %else

then:                                             ; preds = %next-prime
  %load2 = load i32, i32* %alloca1, align 4
  %eq = icmp eq i32 %load2, 1
  %cast = sext i1 %eq to i32
  %ifcond3 = icmp ne i32 %cast, 0
  br i1 %ifcond3, label %then4, label %else5

else:                                             ; preds = %next-prime
  %load11 = load i32, i32* %alloca, align 4
  %mul12 = add i32 %load11, 1
  %load13 = load i32, i32* %alloca1, align 4
  %userfn14 = call i32 @next-prime(i32 %mul12, i32 %load13)
  br label %ifcont

ifcont:                                           ; preds = %else, %ifcont6
  %iftmp15 = phi i32 [ %iftmp, %ifcont6 ], [ %userfn14, %else ]
  ret i32 %iftmp15

then4:                                            ; preds = %then
  %load7 = load i32, i32* %alloca, align 4
  br label %ifcont6

else5:                                            ; preds = %then
  %load8 = load i32, i32* %alloca, align 4
  %mul = add i32 %load8, 1
  %load9 = load i32, i32* %alloca1, align 4
  %sub = sub i32 %load9, 1
  %userfn10 = call i32 @next-prime(i32 %mul, i32 %sub)
  br label %ifcont6

ifcont6:                                          ; preds = %else5, %then4
  %iftmp = phi i32 [ %load7, %then4 ], [ %userfn10, %else5 ]
  br label %ifcont
}

define i32 @nthprime(i32 %n) {
nthprime:
  %alloca = alloca i32, align 4
  store i32 %n, i32* %alloca, align 4
  %load = load i32, i32* %alloca, align 4
  %userfn = call i32 @next-prime(i32 2, i32 %load)
  ret i32 %userfn
}

define i32 @myF(i32 %n) {
myF:
  %alloca = alloca i32, align 4
  store i32 %n, i32* %alloca, align 4
  %load = load i32, i32* %alloca, align 4
  %mul = add i32 %load, 2
  ret i32 %mul
}

define i32 @"#anon"() {
entry:
  %userfn = call i32 @nthprime(i32 100)
  ret i32 %userfn
}

define i32 @"#anon.1"() {
entry:
  %userfn = call i32 @"prime?"(i32 2147483647)
  ret i32 %userfn
}

define i32 @"#anon.2"() {
entry:
  %userfn = call i32 @"prime?"(i32 2147483646)
  %not = xor i32 %userfn, -1
  ret i32 %not
}

define i32 @cleanup() {
entry:
  ret i32 0
}
