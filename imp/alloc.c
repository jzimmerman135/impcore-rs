#include <stdlib.h>

extern int number() {
    char z = 16;
    return 10 + z;
}

int main() {
    int *x = malloc(sizeof(int));
    int z[2];
    *x = number(); 
    *z = number();
    return 0;
}
