#pragma once

#include <stdint.h>
#include <stdlib.h>

void init_noncegen_avx();
void noncegen_avx(char *cache, const size_t cache_size,
                   const uint64_t numeric_id, const uint64_t local_startnonce,
                   const uint64_t local_nonces,
                   char poc_version);
