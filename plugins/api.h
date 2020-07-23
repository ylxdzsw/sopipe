// C header file that defines the middleware API for sopipe.

#include <stdint.h>
#define int int32_t
#define char int8_t

// Notes:
// - sopipe is fully pipelined and concurrent. That means:
//   1. the forward and backward function may be called at the same time on different thread.
//   2. multiple middlewares of the same direction on the same stream may be called at the same time on different thread, processing different buffers.
// - If the middleware need memory allocation, it should dynamically link to the system libc, or choose an allocator that won't conflict with it (use only mmap).

// The API version. This file defines the API v1 and middlewares implementing it should return 1.
int api_version() {
    return 1;
}

// The version string of the middleware. An arbitrary UTF-8 string that is less than 40 bytes.
void version(int *len, char *buffer);

// Initialize the middleware. Several functions are provided to the middleware and can be used later.
// All the functions are thread safe. However, some arguments may be shared with other threads.
void init(
    // get(stream, key_len, key, value_len, value) read the associated value of a key in the stream.
    // value_len should be initialized with the maximum length of value buffer. A negative value will ensure the value buffer not being written.
    // if the value buffer is not long enough, only value_len will be set.
    // value_len will be set to -1 if the key does not exist.
    void (*get)(void*, int, char*, int*, char*),

    // set(stream, key_len, key, value_len, value) set the associated value of a key in the stream.
    // a negative value_len will delete the key.
    void (*set)(void*, int, char*, int, char*),

    // write(stream, len, buffer) write the buffer to the stream.
    void (*write)(void*, int, char*)
);

// a negative len indicates eof. No new call will be made on the same stream. Middlewares should free associated states.
void forward(void *stream, int len, char *buffer);

void backward(void *stream, int len, char *buffer);

#undef int
#undef char
