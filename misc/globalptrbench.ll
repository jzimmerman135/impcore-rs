; ModuleID = 'globalptrbench.c'
source_filename = "globalptrbench.c"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"
target triple = "arm64-apple-macosx13.0.0"

@global_ptr = global ptr null, align 8
@.str = private unnamed_addr constant [16 x i8] c"time taken %lu\0A\00", align 1

; Function Attrs: noinline nounwind optnone ssp uwtable
define i32 @load_store_global(i32 noundef %0, i32 noundef %1) #0 {
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store i32 %0, ptr %3, align 4
  store i32 %1, ptr %4, align 4
  store i32 0, ptr %5, align 4
  store i32 12, ptr %6, align 4
  store i32 0, ptr %7, align 4
  br label %8

8:                                                ; preds = %29, %2
  %9 = load i32, ptr %7, align 4
  %10 = load i32, ptr %3, align 4
  %11 = icmp slt i32 %9, %10
  br i1 %11, label %12, label %32

12:                                               ; preds = %8
  %13 = load ptr, ptr @global_ptr, align 8
  %14 = load i32, ptr %7, align 4
  %15 = load i32, ptr %4, align 4
  %16 = srem i32 %14, %15
  %17 = sext i32 %16 to i64
  %18 = getelementptr inbounds i32, ptr %13, i64 %17
  %19 = load i32, ptr %18, align 4
  %20 = load i32, ptr %5, align 4
  %21 = add nsw i32 %20, %19
  store i32 %21, ptr %5, align 4
  %22 = load i32, ptr %5, align 4
  %23 = load ptr, ptr @global_ptr, align 8
  %24 = load i32, ptr %7, align 4
  %25 = load i32, ptr %4, align 4
  %26 = srem i32 %24, %25
  %27 = sext i32 %26 to i64
  %28 = getelementptr inbounds i32, ptr %23, i64 %27
  store i32 %22, ptr %28, align 4
  br label %29

29:                                               ; preds = %12
  %30 = load i32, ptr %7, align 4
  %31 = add nsw i32 %30, 1
  store i32 %31, ptr %7, align 4
  br label %8, !llvm.loop !5

32:                                               ; preds = %8
  %33 = load i32, ptr %5, align 4
  ret i32 %33
}

; Function Attrs: noinline nounwind optnone ssp uwtable
define i32 @load_store_local(ptr noundef %0, i32 noundef %1, i32 noundef %2) #0 {
  %4 = alloca ptr, align 8
  %5 = alloca i32, align 4
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  store ptr %0, ptr %4, align 8
  store i32 %1, ptr %5, align 4
  store i32 %2, ptr %6, align 4
  store i32 0, ptr %7, align 4
  store i32 12, ptr %8, align 4
  store i32 0, ptr %9, align 4
  br label %10

10:                                               ; preds = %31, %3
  %11 = load i32, ptr %9, align 4
  %12 = load i32, ptr %5, align 4
  %13 = icmp slt i32 %11, %12
  br i1 %13, label %14, label %34

14:                                               ; preds = %10
  %15 = load ptr, ptr %4, align 8
  %16 = load i32, ptr %9, align 4
  %17 = load i32, ptr %6, align 4
  %18 = srem i32 %16, %17
  %19 = sext i32 %18 to i64
  %20 = getelementptr inbounds i32, ptr %15, i64 %19
  %21 = load i32, ptr %20, align 4
  %22 = load i32, ptr %7, align 4
  %23 = add nsw i32 %22, %21
  store i32 %23, ptr %7, align 4
  %24 = load i32, ptr %7, align 4
  %25 = load ptr, ptr %4, align 8
  %26 = load i32, ptr %9, align 4
  %27 = load i32, ptr %6, align 4
  %28 = srem i32 %26, %27
  %29 = sext i32 %28 to i64
  %30 = getelementptr inbounds i32, ptr %25, i64 %29
  store i32 %24, ptr %30, align 4
  br label %31

31:                                               ; preds = %14
  %32 = load i32, ptr %9, align 4
  %33 = add nsw i32 %32, 1
  store i32 %33, ptr %9, align 4
  br label %10, !llvm.loop !7

34:                                               ; preds = %10
  %35 = load i32, ptr %7, align 4
  ret i32 %35
}

; Function Attrs: noinline nounwind optnone ssp uwtable
define i32 @main() #0 {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  store i32 0, ptr %1, align 4
  store i32 1000000, ptr %2, align 4
  store i32 100000000, ptr %3, align 4
  %8 = load i32, ptr %2, align 4
  %9 = sext i32 %8 to i64
  %10 = call ptr @malloc(i64 noundef %9) #3
  store ptr %10, ptr @global_ptr, align 8
  %11 = load i32, ptr %3, align 4
  %12 = load i32, ptr %2, align 4
  %13 = call i32 @load_store_global(i32 noundef %11, i32 noundef %12)
  %14 = load i32, ptr %3, align 4
  %15 = load i32, ptr %2, align 4
  %16 = call i32 @load_store_global(i32 noundef %14, i32 noundef %15)
  %17 = call i64 @"\01_clock"()
  store i64 %17, ptr %4, align 8
  %18 = load i32, ptr %3, align 4
  %19 = load i32, ptr %2, align 4
  %20 = call i32 @load_store_global(i32 noundef %18, i32 noundef %19)
  %21 = call i64 @"\01_clock"()
  %22 = load i64, ptr %4, align 8
  %23 = sub i64 %21, %22
  %24 = mul i64 100000, %23
  %25 = udiv i64 %24, 1000000
  store i64 %25, ptr %5, align 8
  %26 = load i64, ptr %5, align 8
  %27 = call i32 (ptr, ...) @printf(ptr noundef @.str, i64 noundef %26)
  %28 = call i64 @"\01_clock"()
  store i64 %28, ptr %6, align 8
  %29 = load ptr, ptr @global_ptr, align 8
  %30 = load i32, ptr %3, align 4
  %31 = load i32, ptr %2, align 4
  %32 = call i32 @load_store_local(ptr noundef %29, i32 noundef %30, i32 noundef %31)
  %33 = call i64 @"\01_clock"()
  %34 = load i64, ptr %6, align 8
  %35 = sub i64 %33, %34
  %36 = mul i64 100000, %35
  %37 = udiv i64 %36, 1000000
  store i64 %37, ptr %7, align 8
  %38 = load i64, ptr %5, align 8
  %39 = call i32 (ptr, ...) @printf(ptr noundef @.str, i64 noundef %38)
  %40 = load ptr, ptr @global_ptr, align 8
  call void @free(ptr noundef %40)
  store ptr null, ptr @global_ptr, align 8
  ret i32 0
}

; Function Attrs: allocsize(0)
declare ptr @malloc(i64 noundef) #1

declare i64 @"\01_clock"() #2

declare i32 @printf(ptr noundef, ...) #2

declare void @free(ptr noundef) #2

attributes #0 = { noinline nounwind optnone ssp uwtable "frame-pointer"="non-leaf" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="apple-m1" "target-features"="+aes,+crc,+crypto,+dotprod,+fp-armv8,+fp16fml,+fullfp16,+lse,+neon,+ras,+rcpc,+rdm,+sha2,+sha3,+sm4,+v8.5a,+zcm,+zcz" }
attributes #1 = { allocsize(0) "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="apple-m1" "target-features"="+aes,+crc,+crypto,+dotprod,+fp-armv8,+fp16fml,+fullfp16,+lse,+neon,+ras,+rcpc,+rdm,+sha2,+sha3,+sm4,+v8.5a,+zcm,+zcz" }
attributes #2 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="apple-m1" "target-features"="+aes,+crc,+crypto,+dotprod,+fp-armv8,+fp16fml,+fullfp16,+lse,+neon,+ras,+rcpc,+rdm,+sha2,+sha3,+sm4,+v8.5a,+zcm,+zcz" }
attributes #3 = { allocsize(0) }

!llvm.module.flags = !{!0, !1, !2, !3}
!llvm.ident = !{!4}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{i32 7, !"PIC Level", i32 2}
!2 = !{i32 7, !"uwtable", i32 2}
!3 = !{i32 7, !"frame-pointer", i32 1}
!4 = !{!"Homebrew clang version 15.0.7"}
!5 = distinct !{!5, !6}
!6 = !{!"llvm.loop.mustprogress"}
!7 = distinct !{!7, !6}
