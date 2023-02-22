#include <stdio.h>

void *__stdin = NULL;

void init_stdin() {
    void **fp = &__stdin;
    *fp = fdopen(0, "r");
}

int impgetc() {
    return fgetc(__stdin);
}

int main() {
    init_stdin();
    int i = impgetc();
    printf("%c", i);
    return 0;
}
