/* R3 / T-05 — hostile inputs return a handle (never a crash) and a JSON dump that can be released. */
#include "harness.h"

static void one(CapyLibrary *lib, const char *name, const uint8_t *src, size_t n) {
    int32_t st = -1;
    CapyParse *p = capy_parse(lib, src, n, &st);
    CHECK(p != NULL && st == CAPY_OK);
    if (!p) { fprintf(stderr, "  input: %s\n", name); return; }
    /* walk everything reachable at the top level, whatever the verdict */
    CapyId root = capy_root(p);
    for (uint32_t i = 0; i < capy_block_len(p, root); i++) (void)capy_node_func(p, capy_block_stmt(p, root, i));
    for (uint32_t i = 0; i < capy_diag_count(p); i++) { CapySpan s; (void)capy_diag_primary(p, i, &s); (void)capy_diag_message(p, i); }
    uint8_t *buf = NULL; size_t len = 0;
    CHECK(capy_parse_to_json(p, CAPY_JSON_EXPR, &buf, &len) == CAPY_OK && len > 0);
    capy_buf_free(buf, len);
    CHECK(capy_parse_is_poisoned(p) == 0);
    capy_parse_free(p);
}

int main(void) {
    CapyLibrary *lib = make_lib();
    one(lib, "empty", U8(""), 0);
    one(lib, "only newlines", U8("\n\n\n\n"), 4);
    one(lib, "NUL bytes", U8("greet a\0greet b\0\n"), 16);
    one(lib, "invalid UTF-8", U8("\xff\xfe\xfd greet \x80\n"), 13);
    one(lib, "lone quote", U8("greet \"unterminated\n"), 20);

    size_t big = 1 << 20;                               /* one 1 MiB line */
    char *line = malloc(big + 8);
    memcpy(line, "greet ", 6); memset(line + 6, 'a', big); line[6 + big] = '\n';
    one(lib, "1 MiB line", U8(line), big + 7);
    free(line);

    size_t deep = 10000;                                /* 10 000 unmatched openers */
    char *open = malloc(deep + 2);
    memset(open, '(', deep); open[deep] = '\n';
    one(lib, "10 000 openers", U8(open), deep + 1);
    memset(open, '{', deep);
    one(lib, "10 000 braces", U8(open), deep + 1);
    free(open);

    size_t lines = 50000;                               /* 50 000 statements */
    char *many = malloc(lines * 8 + 1), *w = many;
    for (size_t i = 0; i < lines; i++) { memcpy(w, "greet a\n", 8); w += 8; }
    one(lib, "50 000 statements", U8(many), lines * 8);
    free(many);

    unsigned x = 12345;                                 /* 64 KiB of pseudo-random bytes */
    uint8_t *rnd = malloc(65536);
    for (size_t i = 0; i < 65536; i++) { x = x * 1103515245u + 12345u; rnd[i] = (uint8_t)(x >> 16); }
    one(lib, "random bytes", rnd, 65536);
    free(rnd);

    capy_library_free(lib);
    return finish("hostile");
}
