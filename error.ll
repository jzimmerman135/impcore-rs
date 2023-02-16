; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

define i32 @"has-divisor?"(i32 %n, i32 %d) {
"has-divisor?":
  br label %tailrecurse

tailrecurse:                                      ; preds = %else8, %"has-divisor?"
  %d.tr = phi i32 [ %d, %"has-divisor?" ], [ %mul, %else8 ]
  %div = sdiv i32 %n, 2
  %gt = icmp slt i32 %div, %d.tr
  br i1 %gt, label %ifcont, label %else

else:                                             ; preds = %tailrecurse
  %mod = srem i32 %n, %d.tr
  %eq = icmp eq i32 %mod, 0
  br i1 %eq, label %ifcont, label %else8

ifcont:                                           ; preds = %else, %tailrecurse
  %iftmp12 = phi i32 [ 0, %tailrecurse ], [ 1, %else ]
  ret i32 %iftmp12

else8:                                            ; preds = %else
  %mul = add i32 %d.tr, 1
  br label %tailrecurse
}

define i32 @"prime?"(i32 %n) {
"prime?":
  %lt = icmp slt i32 %n, 2
  br i1 %lt, label %ifcont, label %else

else:                                             ; preds = %"prime?"
  %userfn = tail call i32 @"has-divisor?"(i32 %n, i32 2)
  %not = xor i32 %userfn, -1
  br label %ifcont

ifcont:                                           ; preds = %"prime?", %else
  %iftmp = phi i32 [ %not, %else ], [ 0, %"prime?" ]
  ret i32 %iftmp
}

define i32 @next-prime(i32 %p, i32 %n) {
next-prime:
  br label %tailrecurse

tailrecurse:                                      ; preds = %else5, %else, %next-prime
  %p.tr = phi i32 [ %p, %next-prime ], [ %mul12, %else ], [ %mul, %else5 ]
  %n.tr = phi i32 [ %n, %next-prime ], [ %n.tr, %else ], [ %sub, %else5 ]
  %userfn = tail call i32 @"prime?"(i32 %p.tr)
  %ifcond.not = icmp eq i32 %userfn, 0
  br i1 %ifcond.not, label %else, label %then

then:                                             ; preds = %tailrecurse
  %eq = icmp eq i32 %n.tr, 1
  br i1 %eq, label %ifcont, label %else5

else:                                             ; preds = %tailrecurse
  %mul12 = add i32 %p.tr, 1
  br label %tailrecurse

ifcont:                                           ; preds = %then
  ret i32 %p.tr

else5:                                            ; preds = %then
  %mul = add i32 %p.tr, 1
  %sub = add i32 %n.tr, -1
  br label %tailrecurse
}

define i32 @nthprime(i32 %n) {
nthprime:
  %userfn = tail call i32 @next-prime(i32 2, i32 %n)
  ret i32 %userfn
}

define i32 @myF(i32 %n) {
myF:
  %mul = add i32 %n, 2
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
