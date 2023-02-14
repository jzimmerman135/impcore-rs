#include <stdlib.h>

extern int number() {
    return 10;
}

int main() {
    int i = number();
    int *x = malloc(sizeof(int) * 4);
    if (i == 20) {
        x[1] = 15000;
        x[2] = 14000;
    } else {
        x[3] = 200000;
        x[2] = 14021;
    }
   
    return x[2];
}
