; ModuleID = 'core001.lat'
source_filename = "core001.lat"
target datalayout = "e-m:e-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@str_lit = global [2 x i8] c"=\00"
@str_lit.1 = global [9 x i8] c"hello */\00"
@str_lit.2 = global [9 x i8] c"/* world\00"
@str_lit.3 = global [1 x i8] zeroinitializer
@.str = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@.str.2 = private unnamed_addr constant [15 x i8] c"runtime error\0A\00", align 1
@.str.3 = private unnamed_addr constant [3 x i8] c"%d\00", align 1
@.str.4 = private unnamed_addr constant [7 x i8] c"%1023s\00", align 1

define i32 @main() {
entry:
  %0 = call i32 @fac(i32 10)
  call void @printInt(i32 %0)
  %1 = call i32 @rfac(i32 10)
  call void @printInt(i32 %1)
  %2 = call i32 @mfac(i32 10)
  call void @printInt(i32 %2)
  %3 = call i32 @ifac(i32 10)
  call void @printInt(i32 %3)
  br label %loop_cond

loop_cond:                                        ; preds = %loop_body, %entry
  %n = phi i32 [ 10, %entry ], [ %4, %loop_body ]
  %r = phi i32 [ 1, %entry ], [ %r1, %loop_body ]
  %gt = icmp sgt i32 %n, 0
  br i1 %gt, label %loop_body, label %loop_cont

loop_body:                                        ; preds = %loop_cond
  %r1 = mul i32 %r, %n
  %4 = sub i32 %n, 1
  br label %loop_cond

loop_cont:                                        ; preds = %loop_cond
  call void @printInt(i32 %r)
  %5 = call i8* @repStr(i8* getelementptr inbounds ([2 x i8], [2 x i8]* @str_lit, i32 0, i32 0), i32 60)
  call void @printString(i8* %5)
  call void @printString(i8* getelementptr inbounds ([9 x i8], [9 x i8]* @str_lit.1, i32 0, i32 0))
  call void @printString(i8* getelementptr inbounds ([9 x i8], [9 x i8]* @str_lit.2, i32 0, i32 0))
  ret i32 0
}

define i32 @fac(i32 %n) {
entry:
  br label %loop_cond

loop_cond:                                        ; preds = %loop_body, %entry
  %r = phi i32 [ 1, %entry ], [ %r2, %loop_body ]
  %n1 = phi i32 [ %n, %entry ], [ %n3, %loop_body ]
  %a = phi i32 [ %n, %entry ], [ %a, %loop_body ]
  %gt = icmp sgt i32 %n1, 0
  br i1 %gt, label %loop_body, label %loop_cont

loop_body:                                        ; preds = %loop_cond
  %r2 = mul i32 %r, %n1
  %n3 = sub i32 %n1, 1
  br label %loop_cond

loop_cont:                                        ; preds = %loop_cond
  ret i32 %r
}

define i32 @rfac(i32) {
entry:
  %int_eq = icmp eq i32 %0, 0
  br i1 %int_eq, label %then, label %else

then:                                             ; preds = %entry
  ret i32 1

else:                                             ; preds = %entry
  %sub = sub i32 %0, 1
  %1 = call i32 @rfac(i32 %sub)
  %mul = mul i32 %0, %1
  ret i32 %mul
}

define i32 @mfac(i32) {
entry:
  %int_eq = icmp eq i32 %0, 0
  br i1 %int_eq, label %then, label %else

then:                                             ; preds = %entry
  ret i32 1

else:                                             ; preds = %entry
  %sub = sub i32 %0, 1
  %1 = call i32 @nfac(i32 %sub)
  %mul = mul i32 %0, %1
  ret i32 %mul
}

define i32 @nfac(i32) {
entry:
  %int_neq = icmp ne i32 %0, 0
  br i1 %int_neq, label %then, label %else

then:                                             ; preds = %entry
  %sub = sub i32 %0, 1
  %1 = call i32 @mfac(i32 %sub)
  %mul = mul i32 %1, %0
  ret i32 %mul

else:                                             ; preds = %entry
  ret i32 1
}

define i32 @ifac(i32) {
entry:
  %1 = call i32 @ifac2f(i32 1, i32 %0)
  ret i32 %1
}

define i32 @ifac2f(i32, i32) {
entry:
  %int_eq = icmp eq i32 %0, %1
  br i1 %int_eq, label %then, label %cont

then:                                             ; preds = %entry
  ret i32 %0

cont:                                             ; preds = %entry
  %l = phi i32 [ %0, %entry ]
  %h = phi i32 [ %1, %entry ]
  %gt = icmp sgt i32 %l, %h
  br i1 %gt, label %then1, label %cont2

then1:                                            ; preds = %cont
  ret i32 1

cont2:                                            ; preds = %cont
  %l4 = phi i32 [ %l, %cont ]
  %h5 = phi i32 [ %h, %cont ]
  %add = add i32 %l4, %h5
  %m = sdiv i32 %add, 2
  %2 = call i32 @ifac2f(i32 %l4, i32 %m)
  %add6 = add i32 %m, 1
  %3 = call i32 @ifac2f(i32 %add6, i32 %h5)
  %mul = mul i32 %2, %3
  ret i32 %mul
}

define i8* @repStr(i8*, i32) {
entry:
  br label %loop_cond

loop_cond:                                        ; preds = %loop_body, %entry
  %r = phi i8* [ getelementptr inbounds ([1 x i8], [1 x i8]* @str_lit.3, i32 0, i32 0), %entry ], [ %r1, %loop_body ]
  %s = phi i8* [ %0, %entry ], [ %s, %loop_body ]
  %n = phi i32 [ %1, %entry ], [ %n, %loop_body ]
  %i = phi i32 [ 0, %entry ], [ %2, %loop_body ]
  %lt = icmp slt i32 %i, %n
  br i1 %lt, label %loop_body, label %loop_cont

loop_body:                                        ; preds = %loop_cond
  %r1 = call i8* @__latc_concat_str(i8* %r, i8* %s)
  %2 = add i32 %i, 1
  br label %loop_cond

loop_cont:                                        ; preds = %loop_cond
  ret i8* %r
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
