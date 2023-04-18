#include <stdio.h>

int __implib_getc() { return fgetc(stdin); }

int __implib_printstr(int *str) {
  printf("%s", (char *)str);
  return 0;
}
