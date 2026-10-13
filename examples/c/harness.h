/* Shared helpers for the C test programs. */
#ifndef HARNESS_H
#define HARNESS_H
#define _GNU_SOURCE 1      /* memmem on glibc */
#define _DARWIN_C_SOURCE 1  /* memmem on macOS under -std=c11 */
#include "capy.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int g_checks = 0, g_failed = 0;
#define CHECK(cond) do { g_checks++; if (!(cond)) { g_failed++; fprintf(stderr, "FAIL %s:%d: %s\n", __FILE__, __LINE__, #cond); } } while (0)
#define U8(s) ((const uint8_t *)(s))
__attribute__((unused)) static int streq(CapyStr a, const char *b) { return a.ptr && a.len == strlen(b) && memcmp(a.ptr, b, a.len) == 0; }
__attribute__((unused)) static int none(CapyStr a) { return a.ptr == NULL && a.len == 0; }

static const char *GRAMMAR = "extension txt\nfunction greet\n  arg literal \"greet\"\n  arg capture who ident\n  write `hi ${who}`\nend\n";

static CapyLibrary *make_lib(void) {
    CapyLibrary *lib = NULL;
    int st = capy_library_new(U8(GRAMMAR), strlen(GRAMMAR), &lib, NULL);
    if (st != CAPY_OK) { fprintf(stderr, "cannot build the test grammar (%d)\n", st); exit(2); }
    return lib;
}
static int finish(const char *name) {
    printf("%s: %d checks, %d failed\n", name, g_checks, g_failed);
    return g_failed ? 1 : 0;
}
#endif
