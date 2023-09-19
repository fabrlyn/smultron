#include <stdint.h>

void uint32ToBe(uint32_t from, uint8_t* to)
{
  to[0] = (from >> 24) & 0xFF;
  to[1] = (from >> 16) & 0xFF;
  to[2] = (from >> 8) & 0xFF;
  to[3] = from & 0xFF;
}

