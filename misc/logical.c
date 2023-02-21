#include <stdio.h>

int log_and(int a, int b) {
    return a && b;
}

int log_or(int a, int b) {
    return a || b;
}

int bit_and(int a, int b) {
    return a & b;
}

int bit_or(int a, int b) {
    return a | b;
}

int main() {
    int a = 513;
    int b = 9812;
    printf("%i\n", log_and(a, b));
    printf("%i\n", log_or(a, b));
    printf("%i\n", bit_and(a, b));
    printf("%i\n", bit_or(a, b));
    return 0;
}
