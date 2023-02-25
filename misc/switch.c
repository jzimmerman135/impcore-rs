#include <stdio.h>
int main() {
    int scrut = fgetc(stdin);

    int res = -1;

    switch (scrut) {
        case 'a':
            res = 0;
            break;
        case 'b':
            res = 1;
            break;
        default:
            res = scrut;
            break;
    }

    printf("got res %i\n", res);

    return 0;
}
