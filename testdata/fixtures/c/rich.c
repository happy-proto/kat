#include <stdio.h>

typedef struct Theme {
  const char *name;
  int enabled;
} Theme;

static int preview_count(const Theme *theme) {
  return theme->enabled ? 3 : 0;
}

int main(void) {
  Theme theme = {.name = "Dracula", .enabled = 1};
  printf("%s %d\n", theme.name, preview_count(&theme));
  return 0;
}
