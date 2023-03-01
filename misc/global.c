#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int global_number;
int *global_array = NULL;

void set_global(int i) { global_number = i; }

void val_array(int size) {
  free(global_array);
  int bytes = size * sizeof(int);
  global_array = malloc(bytes);
  memset(global_array, 0, bytes);
}

int main() {
  set_global(15);
  printf("number is: %i\n", global_number);
  val_array(10);

  return 0;
}
