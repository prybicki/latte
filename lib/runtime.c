#include <stdio.h>
#include <string.h>
#include <err.h>
#include <stdlib.h>

void printInt(int val) {
    printf("%d\n", val);
}

void printString(char* str) {
    printf("%s\n", str);
}

void error() {
    errx(1, "runtime error\n");
}

int readInt() {
    int val = 0;
    if (scanf("%d", &val) != 1) {
        error();
    }
    return val;
}

char* readString() {
    char buffer[1024];
    if (scanf("%1023s", buffer) != 1) {
        error();
    }
    int len = 1 + strlen(buffer);
    char* result = malloc(len);
    return strcpy(result, buffer);
}

char* __latc_concat_str(char* a, char* b) {
    int len_a = strlen(a);
    int len_b = strlen(b);
    int len_ab = 1 + len_a + len_b;
    char* result = malloc(len_ab);
    strcpy(result, a);
    strcpy(result + len_a, b);
    return result;
}

#include <stdbool.h>
bool __latc_compare_str(char* a, char* b) {
    return strcmp(a, b) == 0;
}
