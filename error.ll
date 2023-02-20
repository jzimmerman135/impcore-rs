; ModuleID = 'tmp'
source_filename = "tmp"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"

@"arr[" = global i32* null

define i32 @val() {
entry:
  %load = load i32*, i32** @"arr[", align 8
  %0 = bitcast i32* %load to i8*
  tail call void @free(i8* %0)
  %malloccall = tail call i8* @malloc(i32 mul (i32 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i32), i32 15))
  %array = bitcast i8* %malloccall to i32*
  %1 = bitcast i32* %array to i8*
  call void @llvm.memset.p0i8.i32(i8* align 4 %1, i8 0, i32 15, i1 false)
  store i32* %array, i32** @"arr[", align 8
  ret i32 15
}

declare void @free(i8*)

declare noalias i8* @malloc(i32)

; Function Attrs: argmemonly nofree nounwind willreturn writeonly
declare void @llvm.memset.p0i8.i32(i8* nocapture writeonly, i8, i32, i1 immarg) #0

define i32 @"#anon"() {
entry:
  %load = load i32*, i32** @"arr[", align 8
  %index = getelementptr i32, i32* %load, i32 8
  store i32 2001, i32* %index, align 4
  ret i32 2001
}

define i32 @array-index(i32* %"x[", i32 %i) {
array-index:
  %0 = sext i32 %i to i64
  %index = getelementptr i32, i32* %"x[", i64 %0
  %load3 = load i32, i32* %index, align 4
  ret i32 %load3
}

define i32 @"#anon.1"() {
entry:
  %load = load i32*, i32** @"arr[", align 8
  %userfn = call i32 @array-index(i32* %load, i32 8)
  ret i32 %userfn
}

define i32 @array-set-index-to(i32* %"x[", i32 %i, i32 %v) {
array-set-index-to:
  %0 = sext i32 %i to i64
  %index = getelementptr i32, i32* %"x[", i64 %0
  store i32 %v, i32* %index, align 4
  ret i32 %v
}

define i32 @"#anon.2"() {
entry:
  %load = load i32*, i32** @"arr[", align 8
  %userfn = call i32 @array-set-index-to(i32* %load, i32 8, i32 2005)
  ret i32 %userfn
}

define i32 @"#anon.3"() {
entry:
  %load = load i32*, i32** @"arr[", align 8
  %index = getelementptr i32, i32* %load, i32 8
  %load1 = load i32, i32* %index, align 4
  ret i32 %load1
}

attributes #0 = { argmemonly nofree nounwind willreturn writeonly }
