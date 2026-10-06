#include <stdarg.h>
#include <stdio.h>

void phicade_libretro_log(unsigned level, const char *fmt, ...) {
    (void)level;
    if (!fmt) {
        return;
    }

    va_list args;
    va_start(args, fmt);
    vfprintf(stderr, fmt, args);
    va_end(args);
}
