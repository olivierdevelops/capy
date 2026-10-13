/* PROP-2026-0008 sample: compile a grammar once, parse, walk, report.
 * Build and run:  examples/c/run.sh walk   (expected output: greet(who=world)) */
#include "capy.h"
#include <stdio.h>
#include <string.h>

int main(void) {
    if (capy_abi_version() != CAPY_ABI_VERSION) { fputs("capy: ABI mismatch\n", stderr); return 2; }

    const char *grammar = "extension txt\nfunction greet\n  arg literal \"greet\"\n  arg capture who ident\n  write `hi ${who}`\nend\n";
    CapyLibrary *lib = NULL; CapyError *err = NULL;
    if (capy_library_new((const uint8_t *)grammar, strlen(grammar), &lib, &err) != CAPY_OK) {
        CapyStr m = capy_error_msg(err), h = capy_error_hint(err);
        fprintf(stderr, "grammar:%u:%u: %.*s\n  hint: %.*s\n", capy_error_line(err), capy_error_col(err),
                (int)m.len, m.ptr, (int)h.len, h.ptr);
        capy_error_free(err);
        return 1;
    }

    const char *src = "greet world\n";
    int32_t st = 0;
    CapyParse *p = capy_parse(lib, (const uint8_t *)src, strlen(src), &st);
    if (st != CAPY_OK) { capy_library_free(lib); return 1; }

    if (!capy_is_clean(p)) {
        for (uint32_t i = 0; i < capy_diag_count(p); i++) {
            CapyStr c = capy_diag_code(p, i), m = capy_diag_message(p, i);
            CapySpan sp; capy_diag_primary(p, i, &sp);
            printf("%.*s at %u:%u: %.*s\n", (int)c.len, c.ptr, sp.start_line, sp.start_col, (int)m.len, m.ptr);
        }
    } else {
        CapyId root = capy_root(p);
        for (uint32_t i = 0; i < capy_block_len(p, root); i++) {
            CapyId n = capy_block_stmt(p, root, i);
            CapyStr f = capy_node_func(p, n);
            CapyId who = capy_node_capture(p, n, (const uint8_t *)"who", 3);
            CapyStr t = capy_cap_text(p, who);
            printf("%.*s(who=%.*s)\n", (int)f.len, f.ptr, (int)t.len, t.ptr);   /* greet(who=world) */
        }
    }
    capy_parse_free(p);          /* f.ptr and t.ptr are dead from here */
    capy_library_free(lib);
    return 0;
}
