; ModuleID = 'simplest'
source_filename = "simplest"

define i32 @f(i32) {
entry:
	  ret i32 42
}

define i32 @g() {
entry:
	  ret i32 32
}

define i32 @main() {
entry:
	  %0 = call i32 @g()
	    %1 = call i32 @f(i32 %0)
	      ret i32 %1
}

