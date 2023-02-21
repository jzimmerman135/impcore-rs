; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

declare i32 @main()

declare i32 @printf(i8*, ...)

define i32 @println(i32 %0) {
entry:
  %alloca = alloca [4 x i8], align 1
  %alloca.repack = getelementptr inbounds [4 x i8], [4 x i8]* %alloca, i64 0, i64 0
  store i8 37, i8* %alloca.repack, align 1
  %alloca.repack1 = getelementptr inbounds [4 x i8], [4 x i8]* %alloca, i64 0, i64 1
  store i8 105, i8* %alloca.repack1, align 1
  %alloca.repack2 = getelementptr inbounds [4 x i8], [4 x i8]* %alloca, i64 0, i64 2
  store i8 10, i8* %alloca.repack2, align 1
  %alloca.repack3 = getelementptr inbounds [4 x i8], [4 x i8]* %alloca, i64 0, i64 3
  store i8 0, i8* %alloca.repack3, align 1
  %printfcall = call i32 (i8*, ...) @printf(i8* noundef nonnull %alloca.repack, i32 %0)
  ret i32 %0
}

define i32 @print(i32 %0) {
entry:
  %alloca = alloca [3 x i8], align 1
  %alloca.repack = getelementptr inbounds [3 x i8], [3 x i8]* %alloca, i64 0, i64 0
  store i8 37, i8* %alloca.repack, align 1
  %alloca.repack1 = getelementptr inbounds [3 x i8], [3 x i8]* %alloca, i64 0, i64 1
  store i8 105, i8* %alloca.repack1, align 1
  %alloca.repack2 = getelementptr inbounds [3 x i8], [3 x i8]* %alloca, i64 0, i64 2
  store i8 0, i8* %alloca.repack2, align 1
  %printfcall = call i32 (i8*, ...) @printf(i8* noundef nonnull %alloca.repack, i32 %0)
  ret i32 %0
}

define i32 @printu(i32 %0) {
entry:
  %alloca = alloca [3 x i8], align 1
  %alloca.repack = getelementptr inbounds [3 x i8], [3 x i8]* %alloca, i64 0, i64 0
  store i8 37, i8* %alloca.repack, align 1
  %alloca.repack1 = getelementptr inbounds [3 x i8], [3 x i8]* %alloca, i64 0, i64 1
  store i8 117, i8* %alloca.repack1, align 1
  %alloca.repack2 = getelementptr inbounds [3 x i8], [3 x i8]* %alloca, i64 0, i64 2
  store i8 0, i8* %alloca.repack2, align 1
  %printfcall = call i32 (i8*, ...) @printf(i8* noundef nonnull %alloca.repack, i32 %0)
  ret i32 %0
}

define i32 @"#anon"() {
entry:
  %print = call i32 @println(i32 -6)
  ret i32 %print
}

define i32 @"#anon.1"() {
entry:
  %print = call i32 @print(i32 6)
  ret i32 %print
}

define i32 @"#anon.2"() {
entry:
  %print = call i32 @printu(i32 -5)
  ret i32 %print
}
