#pragma once

#include <stdint.h>
#include <stdlib.h>

void init_noncegen_avx();
void noncegen_avx(char *cache,
                   const uint64_t numeric_id, const uint64_t local_startnonce,
                   const uint64_t local_nonces,
                   char poc_version);
