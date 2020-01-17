; ModuleID = 'bool_operations'
source_filename = "bool_operations"
target datalayout = "e-m:e-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@str_lit = global [5 x i8] c"true\00"
@str_lit.1 = global [6 x i8] c"false\00"
@.str = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@.str.2 = private unnamed_addr constant [15 x i8] c"runtime error\0A\00", align 1
@.str.3 = private unnamed_addr constant [3 x i8] c"%d\00", align 1
@.str.4 = private unnamed_addr constant [7 x i8] c"%1023s\00", align 1

define i32 @main() {
entry:
  %0 = call i1 @t(i32 1)
  br i1 %0, label %lazy_eval_and_lhs_true, label %lazy_eval_and_done

lazy_eval_and_lhs_true:                           ; preds = %entry
  %1 = call i1 @f(i32 2)
  br label %lazy_eval_and_done

lazy_eval_and_done:                               ; preds = %lazy_eval_and_lhs_true, %entry
  %lazy_eval_and_result = phi i1 [ false, %entry ], [ %1, %lazy_eval_and_lhs_true ]
  call void @b(i1 %lazy_eval_and_result)
  %2 = call i1 @t(i32 3)
  br i1 %2, label %lazy_eval_and_lhs_true1, label %lazy_eval_and_done2

lazy_eval_and_lhs_true1:                          ; preds = %lazy_eval_and_done
  %3 = call i1 @t(i32 4)
  br label %lazy_eval_and_done2

lazy_eval_and_done2:                              ; preds = %lazy_eval_and_lhs_true1, %lazy_eval_and_done
  %lazy_eval_and_result3 = phi i1 [ false, %lazy_eval_and_done ], [ %3, %lazy_eval_and_lhs_true1 ]
  call void @b(i1 %lazy_eval_and_result3)
  %4 = call i1 @t(i32 5)
  br i1 %4, label %lazy_eval_or_done, label %lazy_eval_or_lhs_false

lazy_eval_or_lhs_false:                           ; preds = %lazy_eval_and_done2
  %5 = call i1 @t(i32 6)
  br label %lazy_eval_or_done

lazy_eval_or_done:                                ; preds = %lazy_eval_or_lhs_false, %lazy_eval_and_done2
  %lazy_eval_or_result = phi i1 [ true, %lazy_eval_and_done2 ], [ %5, %lazy_eval_or_lhs_false ]
  call void @b(i1 %lazy_eval_or_result)
  %6 = call i1 @f(i32 7)
  br i1 %6, label %lazy_eval_and_lhs_true4, label %lazy_eval_and_done5

lazy_eval_and_lhs_true4:                          ; preds = %lazy_eval_or_done
  %7 = call i1 @t(i32 8)
  br label %lazy_eval_and_done5

lazy_eval_and_done5:                              ; preds = %lazy_eval_and_lhs_true4, %lazy_eval_or_done
  %lazy_eval_and_result6 = phi i1 [ false, %lazy_eval_or_done ], [ %7, %lazy_eval_and_lhs_true4 ]
  call void @b(i1 %lazy_eval_and_result6)
  %8 = call i1 @t(i32 9)
  br i1 %8, label %lazy_eval_and_lhs_true7, label %lazy_eval_and_done8

lazy_eval_and_lhs_true7:                          ; preds = %lazy_eval_and_done5
  %9 = call i1 @t(i32 10)
  br i1 %9, label %lazy_eval_and_lhs_true9, label %lazy_eval_and_done10

lazy_eval_and_done8:                              ; preds = %lazy_eval_and_done10, %lazy_eval_and_done5
  %lazy_eval_and_result12 = phi i1 [ false, %lazy_eval_and_done5 ], [ %lazy_eval_and_result11, %lazy_eval_and_done10 ]
  call void @b(i1 %lazy_eval_and_result12)
  %10 = call i1 @f(i32 12)
  br i1 %10, label %lazy_eval_or_done14, label %lazy_eval_or_lhs_false13

lazy_eval_and_lhs_true9:                          ; preds = %lazy_eval_and_lhs_true7
  %11 = call i1 @t(i32 11)
  br label %lazy_eval_and_done10

lazy_eval_and_done10:                             ; preds = %lazy_eval_and_lhs_true9, %lazy_eval_and_lhs_true7
  %lazy_eval_and_result11 = phi i1 [ false, %lazy_eval_and_lhs_true7 ], [ %11, %lazy_eval_and_lhs_true9 ]
  br label %lazy_eval_and_done8

lazy_eval_or_lhs_false13:                         ; preds = %lazy_eval_and_done8
  %12 = call i1 @f(i32 13)
  br i1 %12, label %lazy_eval_and_lhs_true15, label %lazy_eval_and_done16

lazy_eval_or_done14:                              ; preds = %lazy_eval_and_done16, %lazy_eval_and_done8
  %lazy_eval_or_result18 = phi i1 [ true, %lazy_eval_and_done8 ], [ %lazy_eval_and_result17, %lazy_eval_and_done16 ]
  call void @b(i1 %lazy_eval_or_result18)
  ret i32 0

lazy_eval_and_lhs_true15:                         ; preds = %lazy_eval_or_lhs_false13
  %13 = call i1 @t(i32 14)
  br label %lazy_eval_and_done16

lazy_eval_and_done16:                             ; preds = %lazy_eval_and_lhs_true15, %lazy_eval_or_lhs_false13
  %lazy_eval_and_result17 = phi i1 [ false, %lazy_eval_or_lhs_false13 ], [ %13, %lazy_eval_and_lhs_true15 ]
  br label %lazy_eval_or_done14
}

define i1 @f(i32) {
entry:
  call void @printInt(i32 %0)
  ret i1 false
}

define i1 @t(i32) {
entry:
  %1 = call i1 @f(i32 %0)
  %not = xor i1 %1, true
  ret i1 %not
}

define void @b(i1) {
entry:
  br i1 %0, label %then, label %else

then:                                             ; preds = %entry
  call void @printString(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @str_lit, i32 0, i32 0))
  br label %cont

cont:                                             ; preds = %else, %then
  %a = phi i1 [ %0, %then ], [ %0, %else ]
  ret void

else:                                             ; preds = %entry
  call void @printString(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @str_lit.1, i32 0, i32 0))
  br label %cont
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
