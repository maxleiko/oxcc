#include "oxcc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char *argv[]) {
  if (argc != 2) {
    fprintf(stderr, "Usage: %s <path_to_file>\n", argv[0]);
    return 1;
  }

  const char *filepath = argv[1];
  size_t filepath_len = strlen(filepath);

  const char *code;
  size_t code_len = 0;
  oxcc_transpile_result res =
      oxcc__transpile(filepath, filepath_len, &code, &code_len);
  if (res != 0) {
    return 1;
  }

  printf("%.*s", (int)code_len, code);
  oxcc__free(code);

  return 0;
}
