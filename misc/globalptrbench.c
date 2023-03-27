#include <stdio.h>
#include <stdlib.h>
#include <time.h>

int *global_ptr;

int load_store_global(int n_ops, int len) {
  int x = 0;
  int sum = 12;
  for (int i = 0; i < n_ops; i++) {
    x += global_ptr[i % len];
    global_ptr[i % len] = x;
  }
  return x;
}

int load_store_local(int *local_ptr, int n_ops, int len) {
  int x = 0;
  int sum = 12;
  for (int i = 0; i < n_ops; i++) {
    x += local_ptr[i % len];
    local_ptr[i % len] = x;
  }
  return x;
}

int main() {
  int len = 1000000;
  int nop = 100000000;
  global_ptr = malloc(len);

  load_store_global(nop, len);
  load_store_global(nop, len);

  clock_t global_time_start = clock();
  load_store_global(nop, len);
  clock_t global_time_elapsed =
      100000 * (clock() - global_time_start) / CLOCKS_PER_SEC;
  printf("time taken %lu\n", global_time_elapsed);

  clock_t local_time_start = clock();
  load_store_local(global_ptr, nop, len);
  clock_t local_time_elapsed =
      100000 * (clock() - local_time_start) / CLOCKS_PER_SEC;
  printf("time taken %lu\n", global_time_elapsed);

  free(global_ptr);
  global_ptr = NULL;

  return 0;
}
