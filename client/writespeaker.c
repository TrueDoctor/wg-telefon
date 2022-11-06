#include "audio.h"
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <inttypes.h>
#include <fcntl.h>
#include <stdbool.h>

uint64_t buffer_size;
float* buffer_ptr;

ssize_t read_file_to_buffer(int file_des);

int main(void)
{
    // initialize audio context
    cinit(30.);

    // size of audio buffer in samples
    buffer_size = 40000;

    // open/create file "buffer"
    int file_des = open("buffer", O_RDONLY);

    // allocate space for the buffer
    buffer_ptr = malloc(sizeof(float) * buffer_size);

    ssize_t nsamples;
    do {
        nsamples = read_file_to_buffer(file_des);
    } while (nsamples > 0);;

    // free used resources
    free(buffer_ptr);
    cfree();
    return 0;
}

ssize_t read_file_to_buffer(int file_des) {

    // read samples from file to buffer
    ssize_t number_of_read_samples = read(file_des, buffer_ptr, sizeof(float) * buffer_size) / sizeof(float);

    // number of bytes written to the speaker
    int64_t number_written = 0;

    // write bytes to the speaker until all bytes are written
    while (number_written < number_of_read_samples) {
        intptr_t amount_of_written_samples = write_samples(buffer_ptr + number_written, number_of_read_samples - number_written);

        number_written += (int64_t) amount_of_written_samples;
    }
    return number_of_read_samples;
}

