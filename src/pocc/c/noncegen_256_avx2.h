#pragma once

#include <stdint.h>
#include <stdlib.h>

void init_noncegen_avx2();
void noncegen_avx2(char *cache,
                   const uint64_t numeric_id, const uint64_t local_startnonce,
                   const uint64_t local_nonces,
                   char poc_version);
