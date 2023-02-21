; ModuleID = 'intstrings.c'
source_filename = "intstrings.c"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"
target triple = "arm64-apple-macosx13.0.0"

@.str = private unnamed_addr constant [11 x i8] c"String:\0A%s\00", align 1

; Function Attrs: noinline nounwind optnone ssp uwtable
define i32 @mkletters(i8 signext %0, i8 signext %1, i8 signext %2, i8 signext %3) #0 {
  %5 = alloca i8, align 1
  %6 = alloca i8, align 1
  %7 = alloca i8, align 1
  %8 = alloca i8, align 1
  %9 = alloca i32, align 4
  store i8 %0, i8* %5, align 1
  store i8 %1, i8* %6, align 1
  store i8 %2, i8* %7, align 1
  store i8 %3, i8* %8, align 1
  %10 = load i8, i8* %5, align 1
  %11 = sext i8 %10 to i32
  store i32 %11, i32* %9, align 4
  %12 = load i8, i8* %6, align 1
  %13 = sext i8 %12 to i32
  %14 = shl i32 %13, 8
  %15 = load i32, i32* %9, align 4
  %16 = or i32 %15, %14
  store i32 %16, i32* %9, align 4
  %17 = load i8, i8* %7, align 1
  %18 = sext i8 %17 to i32
  %19 = shl i32 %18, 16
  %20 = load i32, i32* %9, align 4
  %21 = or i32 %20, %19
  store i32 %21, i32* %9, align 4
  %22 = load i8, i8* %8, align 1
  %23 = sext i8 %22 to i32
  %24 = shl i32 %23, 24
  %25 = load i32, i32* %9, align 4
  %26 = or i32 %25, %24
  store i32 %26, i32* %9, align 4
  %27 = load i32, i32* %9, align 4
  ret i32 %27
}

; Function Attrs: noinline nounwind optnone ssp uwtable
define i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca [4 x i32], align 4
  %3 = alloca i8*, align 8
  store i32 0, i32* %1, align 4
  %4 = getelementptr inbounds [4 x i32], [4 x i32]* %2, i64 0, i64 0
  %5 = call i32 @mkletters(i8 signext 72, i8 signext 101, i8 signext 108, i8 signext 108)
  store i32 %5, i32* %4, align 4
  %6 = getelementptr inbounds i32, i32* %4, i64 1
  %7 = call i32 @mkletters(i8 signext 111, i8 signext 32, i8 signext 87, i8 signext 111)
  store i32 %7, i32* %6, align 4
  %8 = getelementptr inbounds i32, i32* %6, i64 1
  %9 = call i32 @mkletters(i8 signext 114, i8 signext 108, i8 signext 100, i8 signext 33)
  store i32 %9, i32* %8, align 4
  %10 = getelementptr inbounds i32, i32* %8, i64 1
  %11 = call i32 @mkletters(i8 signext 10, i8 signext 0, i8 signext 0, i8 signext 0)
  store i32 %11, i32* %10, align 4
  %12 = bitcast [4 x i32]* %2 to i8*
  store i8* %12, i8** %3, align 8
  %13 = load i8*, i8** %3, align 8
  %14 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([11 x i8], [11 x i8]* @.str, i64 0, i64 0), i8* %13)
  ret i32 0
}

declare i32 @printf(i8*, ...) #1

attributes #0 = { noinline nounwind optnone ssp uwtable "frame-pointer"="non-leaf" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="apple-m1" "target-features"="+aes,+crc,+crypto,+dotprod,+fp-armv8,+fp16fml,+fullfp16,+lse,+neon,+ras,+rcpc,+rdm,+sha2,+v8.5a,+zcm,+zcz" }
attributes #1 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="apple-m1" "target-features"="+aes,+crc,+crypto,+dotprod,+fp-armv8,+fp16fml,+fullfp16,+lse,+neon,+ras,+rcpc,+rdm,+sha2,+v8.5a,+zcm,+zcz" }

!llvm.module.flags = !{!0, !1, !2, !3, !4, !5, !6, !7}
!llvm.ident = !{!8}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 1, !"branch-target-enforcement", i32 0}
!2 = !{i32 1, !"sign-return-address", i32 0}
!3 = !{i32 1, !"sign-return-address-all", i32 0}
!4 = !{i32 1, !"sign-return-address-with-bkey", i32 0}
!5 = !{i32 7, !"PIC Level", i32 2}
!6 = !{i32 7, !"uwtable", i32 1}
!7 = !{i32 7, !"frame-pointer", i32 1}
!8 = !{!"Homebrew clang version 13.0.1"}
