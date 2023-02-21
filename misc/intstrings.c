#include <stdio.h>
#include <stdint.h>

int mkletters(char a, char b, char c, char d) {
    int i = a;
    i |= ((int)b) << 8;
    i |= ((int)c) << 16;
    i |= ((int)d) << 24;
    return i;
}

int main()
{
    int letters[] = {
        mkletters('H', 'e', 'l', 'l'),
        mkletters('o', ' ', 'W', 'o'),
        mkletters('r', 'l', 'd', '!'),
        mkletters('\n', 0, 0, 0),
    };

    char *s = (char*) &letters;
    printf("String:\n%s", s);
    return 0;
}
