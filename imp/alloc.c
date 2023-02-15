#include <stdio.h>
#include <stdlib.h>

int *global_var; // declare a mutable global variable of type int pointer

void init_global_var(int initial_value) {
  global_var = (int*) malloc(sizeof(int)); // allocate memory for global_var
  *global_var = initial_value; // set the initial value
}

void update_global_var(int new_value) {
  *global_var = new_value; // update the value
}

int main() {
  init_global_var(42); // initialize the global variable with 42
  printf("global_var = %d\n", *global_var); // prints "global_var = 42"
  update_global_var(123); // update the global variable to 123
  printf("global_var = %d\n", *global_var); // prints "global_var = 123"
  free(global_var); // free the memory allocated for global_var
  return 0;
}