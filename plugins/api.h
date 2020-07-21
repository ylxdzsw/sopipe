// C header file that defines the middleware API for sopipe.

#include <stdint.h>
#define int int32_t
#define char int8_t

// The API version. This file defines the API v1 and middlewares implementing it should return 1.
int api_version() {
    return 1;
}

// The version string of the middleware. An arbitrary UTF-8 string that is less than 40 bytes.
// buffer: a buffer that is at least 40 bytes.
// returns: the actual length.
int version(char *buffer);
