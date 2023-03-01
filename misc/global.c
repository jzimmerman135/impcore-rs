#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int global_number;
int *global_array = NULL;

void set_global() { global_number = 15; }

void val_array() {
  free(global_array);
  global_array = malloc(16);
  memset(global_array, 0, 16);
}

int main() {
  set_global();
  printf("number is: %i\n", global_number);
  val_array();
  printf("number is: %i\n", global_array[2]);

  return 0;
}
