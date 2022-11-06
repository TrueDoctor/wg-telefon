#include <stdint.h>

// Initialize audio context
int cinit(float buffer_ms);

// Free used ressources
void cfree(void);

// Read samples from mic and write them to the buffer
intptr_t read_samples(float *buffer, uintptr_t buffer_len);

// Write samples from the provided buffer to the speaker
intptr_t write_samples(float *buffer, uintptr_t buffer_len);
