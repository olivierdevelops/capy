/* PROP-2026-0008 UC-01 / UC-05 negative paths: a grammar that does not compile, and a script that does not parse.
 * Build and run:  examples/c/run.sh diagnose */
#include "capy.h"
#include <stdio.h>
#include <string.h>

int main(void) {
    /* 1. a grammar that fails to compile -> CAPY_E_LOAD and a CapyError */
    const char *bad = "function broken\n  arg capture x nonsense_type\nend\n";
    CapyLibrary *lib = NULL; CapyError *err = NULL;
    int st = capy_library_new((const uint8_t *)bad, strlen(bad), &lib, &err);
    CapyStr m = capy_error_msg(err);
    printf("load: status=%d lib=%s\n", st, lib ? "set" : "NULL");
    printf("load: %.*s\n", (int)m.len, m.ptr);
    capy_error_free(err);

    /* 2. a good grammar, a script with a mistake -> a handle, not clean, diagnostics */
    const char *g = "extension txt\nfunction greet\n  arg literal \"greet\"\n  arg capture who ident\n  write `hi ${who}`\nend\n";
    capy_library_new((const uint8_t *)g, strlen(g), &lib, NULL);
    const char *src = "greet world\nhello there\n";
    CapyParse *p = capy_parse(lib, (const uint8_t *)src, strlen(src), &st);
    printf("parse: status=%d clean=%d diagnostics=%u\n", st, capy_is_clean(p), capy_diag_count(p));
    for (uint32_t i = 0; i < capy_diag_count(p); i++) {
        CapyStr c = capy_diag_code(p, i), d = capy_diag_message(p, i);
        CapySpan sp; capy_diag_primary(p, i, &sp);
        printf("  %.*s %u:%u %.*s\n", (int)c.len, c.ptr, sp.start_line, sp.start_col, (int)d.len, d.ptr);
    }
    /* the statement that did parse is still there */
    CapyId root = capy_root(p);
    printf("statements=%u error_regions=%u\n", capy_block_len(p, root), capy_block_error_count(p, root));

    /* 3. misuse: an id that does not exist reads as nothing */
    CapyStr none = capy_node_func(p, 123456);
    printf("bad id: ptr=%s len=%zu\n", none.ptr ? "set" : "NULL", none.len);
    capy_parse_free(p);
    capy_library_free(lib);
    return 0;
}
