- Dead-code elimination:
    - Does not recognize while(true) that returns somewhere is returning always.
        ./mrjp-tests/good/basic/while_true.lat
    - Trivially true branch not treated as return
        ./mrjp-tests/good/basic/void_return.lat

FIXED:
- Something wrong with the comments?:
    ./mrjp-tests/good/basic/escaped_string.lat
    ./mrjp-tests/good/basic/fibonacci.lat
    
- Declaration in if/else/while statment.

- Unable to parse --1
    ./mrjp-tests/good/basic/negation.lat

WON'T FIX:

- Does not handle escapes like this: printString("\\a\\n\n\tb\"");
    ./mrjp-tests/good/basic/print_complicated_string.lat

