#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void *__stdin = NULL;

void init_stdin() {
    void **fp = &__stdin;
    *fp = fdopen(0, "r");
}

int impgetc() {
    return fgetc(__stdin);
}

int word(char a, char b, char c, char d) {
    int r = a;
    r |= b << 8;
    r |= c << 16;
    r |= d << 24;
    return r;
}

int main() {
    init_stdin();
    int bufsize = 81;
    int *buffer = malloc(bufsize * sizeof(int));
    memset(buffer, 0, bufsize * sizeof(int));

    buffer[0] = word(97, 98, 99, 100);
    buffer[1] = word(97, 98, 99, 100);
    buffer[2] = word(97, 98, 99, 100);
    buffer[3] = word(97, 98, 99, 10);
    printf("%s", (char*)buffer);
    free(buffer);
    return 0;
}
