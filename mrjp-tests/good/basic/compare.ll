; ModuleID = 'compare'
source_filename = "compare"
target datalayout = "e-m:e-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@str_lit = global [2 x i8] c"4\00"
@str_lit.1 = global [2 x i8] c"6\00"
@.str = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@.str.2 = private unnamed_addr constant [15 x i8] c"runtime error\0A\00", align 1
@.str.3 = private unnamed_addr constant [3 x i8] c"%d\00", align 1
@.str.4 = private unnamed_addr constant [7 x i8] c"%1023s\00", align 1

define i32 @main() {
entry:
  call void @printString(i8* getelementptr inbounds ([2 x i8], [2 x i8]* @str_lit, i32 0, i32 0))
  call void @printString(i8* getelementptr inbounds ([2 x i8], [2 x i8]* @str_lit, i32 0, i32 0))
  call void @printString(i8* getelementptr inbounds ([2 x i8], [2 x i8]* @str_lit.1, i32 0, i32 0))
  call void @printString(i8* getelementptr inbounds ([2 x i8], [2 x i8]* @str_lit.1, i32 0, i32 0))
  ret i32 0
}

; Function Attrs: nounwind uwtable
define void @printInt(i32) #0 {
  %2 = tail call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @.str, i64 0, i64 0), i32 %0)
  ret void
}

; Function Attrs: nounwind
declare i32 @printf(i8* nocapture readonly, ...) local_unnamed_addr #1

; Function Attrs: nounwind uwtable
define void @printString(i8* nocapture readonly) #0 {
  %2 = tail call i32 @puts(i8* %0)
  ret void
}

; Function Attrs: nounwind
declare i32 @puts(i8* nocapture readonly) local_unnamed_addr #2

; Function Attrs: noreturn nounwind uwtable
define void @error() local_unnamed_addr #3 {
  tail call void (i32, i8*, ...) @errx(i32 1, i8* getelementptr inbounds ([15 x i8], [15 x i8]* @.str.2, i64 0, i64 0)) #7
  unreachable
}

; Function Attrs: noreturn
declare void @errx(i32, i8*, ...) local_unnamed_addr #4

; Function Attrs: nounwind uwtable
define i32 @readInt() #0 {
  %1 = alloca i32, align 4
  %2 = bitcast i32* %1 to i8*
  call void @llvm.lifetime.start.p0i8(i64 4, i8* nonnull %2) #2
  store i32 0, i32* %1, align 4, !tbaa !2
  %3 = call i32 (i8*, ...) @__isoc99_scanf(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @.str.3, i64 0, i64 0), i32* nonnull %1)
  %4 = icmp eq i32 %3, 1
  br i1 %4, label %6, label %5

; <label>:5:                                      ; preds = %0
  call void @error()
  unreachable

; <label>:6:                                      ; preds = %0
  %7 = load i32, i32* %1, align 4, !tbaa !2
  call void @llvm.lifetime.end.p0i8(i64 4, i8* nonnull %2) #2
  ret i32 %7
}

; Function Attrs: argmemonly nounwind
declare void @llvm.lifetime.start.p0i8(i64, i8* nocapture) #5

; Function Attrs: nounwind
declare i32 @__isoc99_scanf(i8* nocapture readonly, ...) local_unnamed_addr #1

; Function Attrs: argmemonly nounwind
declare void @llvm.lifetime.end.p0i8(i64, i8* nocapture) #5

; Function Attrs: nounwind uwtable
define i8* @readString() #0 {
  %1 = alloca [1024 x i8], align 16
  %2 = getelementptr inbounds [1024 x i8], [1024 x i8]* %1, i64 0, i64 0
  call void @llvm.lifetime.start.p0i8(i64 1024, i8* nonnull %2) #2
  %3 = call i32 (i8*, ...) @__isoc99_scanf(i8* getelementptr inbounds ([7 x i8], [7 x i8]* @.str.4, i64 0, i64 0), i8* nonnull %2)
  %4 = icmp eq i32 %3, 1
  br i1 %4, label %6, label %5

; <label>:5:                                      ; preds = %0
  call void @error()
  unreachable

; <label>:6:                                      ; preds = %0
  %7 = call i64 @strlen(i8* nonnull %2) #8
  %8 = shl i64 %7, 32
  %9 = add i64 %8, 4294967296
  %10 = ashr exact i64 %9, 32
  %11 = call noalias i8* @malloc(i64 %10) #2
  %12 = call i8* @strcpy(i8* %11, i8* nonnull %2) #2
  call void @llvm.lifetime.end.p0i8(i64 1024, i8* nonnull %2) #2
  ret i8* %12
}

; Function Attrs: argmemonly nounwind readonly
declare i64 @strlen(i8* nocapture) local_unnamed_addr #6

; Function Attrs: nounwind
declare noalias i8* @malloc(i64) local_unnamed_addr #1

; Function Attrs: nounwind
declare i8* @strcpy(i8*, i8* nocapture readonly) local_unnamed_addr #1

; Function Attrs: nounwind uwtable
define i8* @__latc_concat_str(i8* nocapture readonly, i8* nocapture readonly) #0 {
  %3 = tail call i64 @strlen(i8* %0) #8
  %4 = tail call i64 @strlen(i8* %1) #8
  %5 = add i64 %3, 1
  %6 = add i64 %5, %4
  %7 = shl i64 %6, 32
  %8 = ashr exact i64 %7, 32
  %9 = tail call noalias i8* @malloc(i64 %8) #2
  %10 = tail call i8* @strcpy(i8* %9, i8* %0) #2
  %11 = shl i64 %3, 32
  %12 = ashr exact i64 %11, 32
  %13 = getelementptr inbounds i8, i8* %9, i64 %12
  %14 = tail call i8* @strcpy(i8* %13, i8* %1) #2
  ret i8* %9
}

attributes #0 = { nounwind uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #1 = { nounwind "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #2 = { nounwind }
attributes #3 = { noreturn nounwind uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #4 = { noreturn "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #5 = { argmemonly nounwind }
attributes #6 = { argmemonly nounwind readonly "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="false" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #7 = { noreturn nounwind }
attributes #8 = { nounwind readonly }

!llvm.ident = !{!0}
!llvm.module.flags = !{!1}

!0 = !{!"clang version 6.0.0-1ubuntu2 (tags/RELEASE_600/final)"}
!1 = !{i32 1, !"wchar_size", i32 4}
!2 = !{!3, !3, i64 0}
!3 = !{!"int", !4, i64 0}
!4 = !{!"omnipotent char", !5, i64 0}
!5 = !{!"Simple C/C++ TBAA"}
