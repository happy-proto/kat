#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Included after parser.c by the build-time exporter. These tables are generated
// C data; the exporter runs on the same architecture as the release compiler.
typedef struct {
  uint16_t value;
  uint16_t count;
  uint16_t first_symbol;
  bool nonterminal;
  int head;
  int tail;
} CompactGroup;

static void write_u16(FILE *file, uint16_t value) {
  if (fwrite(&value, sizeof(value), 1, file) != 1) abort();
}

static void write_u32(FILE *file, uint32_t value) {
  if (fwrite(&value, sizeof(value), 1, file) != 1) abort();
}

static int compare_groups(const void *left, const void *right) {
  const CompactGroup *a = left;
  const CompactGroup *b = right;
  if (a->count != b->count) return (int)a->count - (int)b->count;
  if (a->nonterminal != b->nonterminal) return (int)a->nonterminal - (int)b->nonterminal;
  if (a->value != b->value) return (int)a->value - (int)b->value;
  return (int)a->first_symbol - (int)b->first_symbol;
}

int main(int argc, char **argv) {
  if (argc != 3 || STATE_COUNT < 2 || LARGE_STATE_COUNT < 2) return 2;
  FILE *direct = fopen(argv[1], "wb");
  FILE *compact = fopen(argv[2], "wb");
  if (!direct || !compact) return 3;
  if (fwrite(ts_parse_table, sizeof(uint16_t), 2 * SYMBOL_COUNT, direct) != 2 * SYMBOL_COUNT) return 4;

  CompactGroup *groups = calloc(SYMBOL_COUNT, sizeof(*groups));
  int *next = malloc(SYMBOL_COUNT * sizeof(*next));
  int *terminal_index = malloc((UINT16_MAX + 1u) * sizeof(*terminal_index));
  int *nonterminal_index = malloc((UINT16_MAX + 1u) * sizeof(*nonterminal_index));
  uint32_t *map = calloc(STATE_COUNT - 2, sizeof(*map));
  if (!groups || !next || !terminal_index || !nonterminal_index || !map) abort();
  for (size_t i = 0; i <= UINT16_MAX; i++) terminal_index[i] = nonterminal_index[i] = -1;

  uint32_t offset = 0;
  for (uint32_t state = 2; state < LARGE_STATE_COUNT; state++) {
    map[state - 2] = offset;
    uint16_t group_count = 0;
    for (uint32_t symbol = 0; symbol < SYMBOL_COUNT; symbol++) {
      uint16_t value = ts_parse_table[state][symbol];
      if (!value) continue;
      bool nonterminal = symbol >= TOKEN_COUNT;
      int *index = nonterminal ? &nonterminal_index[value] : &terminal_index[value];
      if (*index < 0) {
        *index = group_count++;
        groups[*index] = (CompactGroup){.value = value, .first_symbol = symbol,
                                        .nonterminal = nonterminal, .head = -1, .tail = -1};
      }
      CompactGroup *group = &groups[*index];
      next[symbol] = -1;
      if (group->tail >= 0) next[group->tail] = symbol;
      else group->head = symbol;
      group->tail = symbol;
      group->count++;
    }
    for (uint32_t i = 0; i < group_count; i++) {
      if (groups[i].nonterminal) nonterminal_index[groups[i].value] = -1;
      else terminal_index[groups[i].value] = -1;
    }
    qsort(groups, group_count, sizeof(*groups), compare_groups);
    write_u16(compact, group_count);
    offset++;
    for (uint32_t i = 0; i < group_count; i++) {
      CompactGroup *group = &groups[i];
      write_u16(compact, group->value);
      write_u16(compact, group->count);
      offset += 2;
      for (int symbol = group->head; symbol >= 0; symbol = next[symbol]) {
        write_u16(compact, symbol);
        offset++;
      }
    }
  }

  if (fwrite(ts_small_parse_table, 1, sizeof(ts_small_parse_table), compact) != sizeof(ts_small_parse_table)) return 5;
  uint32_t map_offset = (offset * 2 + sizeof(ts_small_parse_table) + 3) & ~UINT32_C(3);
  uint8_t padding[3] = {0};
  uint32_t padding_size = map_offset - (offset * 2 + sizeof(ts_small_parse_table));
  if (fwrite(padding, 1, padding_size, compact) != padding_size) return 6;
  for (uint32_t state = 2; state < LARGE_STATE_COUNT; state++) write_u32(compact, map[state - 2]);
  for (uint32_t state = LARGE_STATE_COUNT; state < STATE_COUNT; state++) {
    write_u32(compact, offset + ts_small_parse_table_map[state - LARGE_STATE_COUNT]);
  }
  if (fclose(direct) || fclose(compact)) return 7;
  printf("%u\n", map_offset);
  return 0;
}
