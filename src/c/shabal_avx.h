#pragma once

#include <stdint.h>
#include <stdlib.h>

void init_shabal_avx();

void find_best_deadline_avx(char *scoops, uint64_t nonce_count, char *gensig,
                            uint64_t *best_deadline, uint64_t *best_offset);
