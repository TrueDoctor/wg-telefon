#include "audio.h"
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <inttypes.h>
#include <fcntl.h>
#include <stdbool.h>

uint64_t buffer_size;
float* buffer_ptr;

void write_buffer_to_file(int file_des);

int main(void)
{
    // initialize audio context
    cinit(30.);

    // size of audio buffer in samples
    buffer_size = 40000;

    // open/create file "buffer"
    int file_des = open("buffer", O_RDWR | O_CREAT | O_TRUNC, 00660);

    // allocate space for the buffer
    buffer_ptr = malloc(sizeof(float) * buffer_size);

    while(true) {
        write_buffer_to_file(file_des);
    }

    // free used resources
    free(buffer_ptr);
    cfree();
    return 0;
}

// read samples from mic and write them to the specified file
void write_buffer_to_file(int file_des) {
    // read samples from the microphone
    intptr_t number_of_read_samples = read_samples(buffer_ptr, buffer_size);

    // if any samples have been read to the buffer, save them to the file
    if (number_of_read_samples > 0) {
        write(file_des, buffer_ptr, sizeof(float) * number_of_read_samples);

    }
}

