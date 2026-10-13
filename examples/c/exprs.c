/* PROP-2026-0008 UC-04 / UC-06: read the typed expression behind a capture, and dump the JSON.
 * Build and run:  examples/c/run.sh exprs */
#define _GNU_SOURCE 1
#define _DARWIN_C_SOURCE 1
#include "capy.h"
#include <stdio.h>
#include <string.h>

static void show(const CapyParse *p, CapyId e, int depth) {
    static const char *names[] = {"?", "number", "str", "bool", "null", "var", "call", "compare", "binary", "not", "unary", "list", "obj"};
    uint8_t k = capy_expr_kind(p, e);
    printf("%*s%s", depth * 2, "", names[k]);
    if (k == CAPY_EXPR_NUMBER) {
        int64_t v;
        if (capy_expr_int(p, e, &v)) printf(" %lld", (long long)v); else printf(" %g", capy_expr_float(p, e));
    } else if (k == CAPY_EXPR_STR || k == CAPY_EXPR_BINARY || k == CAPY_EXPR_COMPARE || k == CAPY_EXPR_UNARY) {
        CapyStr s = capy_expr_str(p, e); printf(" %.*s", (int)s.len, s.ptr);
    } else if (k == CAPY_EXPR_BOOL) {
        printf(" %s", capy_expr_bool(p, e) ? "true" : "false");
    }
    printf("\n");
    if (k == CAPY_EXPR_VAR) {
        for (uint32_t i = 0; i < capy_var_step_count(p, e); i++) {
            if (capy_var_step_is_index(p, e, i)) { printf("%*s[index]\n", depth * 2 + 2, ""); show(p, capy_var_step_index(p, e, i), depth + 2); }
            else { CapyStr f = capy_var_step_field(p, e, i); printf("%*s.%.*s\n", depth * 2 + 2, "", (int)f.len, f.ptr); }
        }
    } else {
        for (uint32_t i = 0; i < capy_expr_child_count(p, e); i++) show(p, capy_expr_child(p, e, i), depth + 1);
    }
}

int main(void) {
    const char *g = "extension txt\nfunction let\n  arg literal \"let\"\n  arg capture name ident\n  arg literal \"=\"\n  arg capture value any\n  write `${name}`\nend\n";
    CapyLibrary *lib = NULL;
    capy_library_new((const uint8_t *)g, strlen(g), &lib, NULL);
    const char *src = "let y = a.b[i + 1] * -2\n";
    int32_t st;
    CapyParse *p = capy_parse(lib, (const uint8_t *)src, strlen(src), &st);

    CapyId root = capy_root(p), n = capy_block_stmt(p, root, 0);
    CapyId value = capy_node_capture(p, n, (const uint8_t *)"value", 5);
    CapyStr text = capy_cap_text(p, value);
    printf("value text: %.*s\n", (int)text.len, text.ptr);
    show(p, capy_cap_expr(p, value), 0);

    uint8_t *buf; size_t len;
    if (capy_parse_to_json(p, CAPY_JSON_EXPR, &buf, &len) == CAPY_OK) {
        printf("json: %zu bytes, has expr: %s\n", len, memmem(buf, len, "\"expr\"", 6) ? "yes" : "no");
        capy_buf_free(buf, len);
    }
    capy_parse_free(p);
    capy_library_free(lib);
    return 0;
}
