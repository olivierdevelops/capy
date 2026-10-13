/* R8 / T-07 — every kind of misuse returns a status or a zero value. Run under ASan + UBSan. */
#include "harness.h"
#include <stdint.h>

int main(void) {
    CapyLibrary *lib = make_lib();
    int32_t st;

    /* --- version and build ------------------------------------------------------------------------ */
    CHECK(capy_abi_version() == CAPY_ABI_VERSION);
    CHECK(capy_panics_are_caught() == 1);              /* run.sh builds with --profile ffi */

    /* --- library loading --------------------------------------------------------------------------- */
    CapyLibrary *l2 = NULL; CapyError *e2 = NULL;
    CHECK(capy_library_new(NULL, 5, &l2, &e2) == CAPY_E_NULL);
    CHECK(capy_library_new(U8(GRAMMAR), strlen(GRAMMAR), NULL, &e2) == CAPY_E_NULL);
    CHECK(capy_library_new(U8("\xff\xfe"), 2, &l2, &e2) == CAPY_E_UTF8);
    const char *bad = "function broken\n  arg capture x nonsense_type\nend\n";
    CHECK(capy_library_new(U8(bad), strlen(bad), &l2, &e2) == CAPY_E_LOAD);
    CHECK(l2 == NULL && e2 != NULL);
    CHECK(capy_error_msg(e2).len > 0);
    CHECK(capy_error_file(e2).ptr != NULL);                         /* present, possibly empty */
    capy_error_free(e2); e2 = NULL;
    CHECK(capy_library_new(U8(bad), strlen(bad), &l2, NULL) == CAPY_E_LOAD);          /* err out-param is optional */
    CHECK(capy_library_from_file(U8("/no/such/file.capy"), 18, &l2, &e2) == CAPY_E_LOAD);
    CHECK(e2 != NULL); capy_error_free(e2); e2 = NULL;
    CHECK(capy_library_from_file(NULL, 3, &l2, &e2) == CAPY_E_NULL);
    CHECK(none(capy_error_msg(NULL)) && none(capy_error_hint(NULL)) && none(capy_error_file(NULL)));
    CHECK(capy_error_line(NULL) == 0 && capy_error_col(NULL) == 0);
    capy_error_free(NULL); capy_library_free(NULL); capy_parse_free(NULL); capy_buf_free(NULL, 0);

    /* --- parse arguments --------------------------------------------------------------------------- */
    CHECK(capy_parse(NULL, U8("x"), 1, &st) == NULL && st == CAPY_E_NULL);
    CHECK(capy_parse(lib, NULL, 5, &st) == NULL && st == CAPY_E_NULL);
    CapyParse *empty = capy_parse(lib, NULL, 0, &st);                                    /* empty source */
    CHECK(empty != NULL && st == CAPY_OK && capy_is_clean(empty) == 1 && capy_block_len(empty, capy_root(empty)) == 0);
    capy_parse_free(empty);
    CapyParse *noststatus = capy_parse(lib, U8("greet a\n"), 8, NULL);                   /* status is optional */
    CHECK(noststatus != NULL); capy_parse_free(noststatus);
    CapyParse *u = capy_parse(lib, U8("greet \xff\xfe\n"), 9, &st);                      /* invalid UTF-8 is replaced */
    CHECK(u != NULL && st == CAPY_OK); capy_parse_free(u);

    /* --- a NULL parse handle ----------------------------------------------------------------------- */
    CHECK(capy_is_clean(NULL) == 0 && capy_root(NULL) == 0 && capy_diag_count(NULL) == 0);
    CHECK(capy_block_len(NULL, 1) == 0 && none(capy_node_func(NULL, 1)) && capy_expr_kind(NULL, 1) == 0);
    CHECK(capy_parse_is_poisoned(NULL) == 0);

    /* --- a good parse, then bad ids ---------------------------------------------------------------- */
    CapyParse *p = capy_parse(lib, U8("greet world\n"), 12, &st);
    CHECK(p && st == CAPY_OK && capy_is_clean(p) == 1);
    CapyId root = capy_root(p), n = capy_block_stmt(p, root, 0);
    CHECK(root != 0 && n != 0 && capy_block_len(p, root) == 1);
    CHECK(streq(capy_node_func(p, n), "greet"));
    CHECK(none(capy_node_func(p, 0)));                                  /* id 0 */
    CHECK(none(capy_node_func(p, 0xFFFFFFFFu)));                        /* absurd id */
    CHECK(none(capy_node_func(p, root)));                               /* a block id is not a statement */
    CHECK(capy_block_len(p, n) == 0);                                   /* a statement id is not a block */
    CHECK(capy_block_stmt(p, root, 99) == 0);                           /* index out of range */
    CHECK(capy_node_capture_count(p, n) == 1);
    CHECK(streq(capy_node_capture_name(p, n, 0), "who") && none(capy_node_capture_name(p, n, 99)));
    CapyId who = capy_node_capture(p, n, U8("who"), 3);
    CHECK(who != 0 && streq(capy_cap_text(p, who), "world"));
    CHECK(capy_node_capture(p, n, U8("nope"), 4) == 0);                 /* missing name */
    CHECK(capy_node_capture(p, n, NULL, 3) == 0);                       /* NULL name with a length */
    CHECK(capy_node_capture(p, n, U8(""), 0) == 0);                     /* empty name */
    CHECK(none(capy_cap_text(p, n)));                                   /* a statement id is not a capture */
    CHECK(none(capy_node_func(p, who)));                                /* a capture id is not a statement */
    CHECK(capy_node_capture_count(p, who) == 0 && capy_node_line(p, who) == 0 && capy_node_comment_count(p, who) == 0);
    CHECK(capy_block_len(p, who) == 0 && capy_block_error_count(p, who) == 0);
    CHECK(capy_expr_kind(p, who) == 0 && capy_cap_sub_count(p, n) == 0);
    CHECK(capy_cap_expr(p, who) == 0 && capy_cap_is_expr(p, who) == 0); /* ident capture: no expression */
    CHECK(capy_cap_sub_count(p, who) == 0 && capy_cap_sub(p, who, 0) == 0);
    CapySpan sp = {7, 7, 7, 7};
    CHECK(capy_node_span(p, root, &sp) == CAPY_E_KIND && sp.start_line == 0);   /* wrong kind: zeroed span */
    CHECK(capy_node_span(p, n, NULL) == CAPY_OK);                               /* out is optional */
    CHECK(capy_node_span(p, n, &sp) == CAPY_OK && sp.start_line == 1 && sp.start_col == 1);
    CHECK(capy_node_comment_span(p, n, 0, &sp) == CAPY_E_ARG);
    CHECK(capy_node_body(p, n) == 0 && capy_node_closer(p, n) == 0 && capy_node_section_count(p, n) == 0);
    CHECK(capy_node_section(p, n, U8("else"), 4) == 0 && none(capy_node_section_name(p, n, 0)));
    CapyStr vt = {(const uint8_t *)1, 1};
    CHECK(capy_block_verbatim(p, root, &vt) == 0 && none(vt));
    CHECK(capy_expr_kind(p, root) == 0 && capy_expr_child_count(p, root) == 0 && none(capy_expr_str(p, root)));
    int64_t iv = 99;
    CHECK(capy_expr_int(p, root, &iv) == 0 && iv == 99);
    CHECK(capy_expr_int(p, root, NULL) == 0);

    /* --- an id from another parse ------------------------------------------------------------------ */
    CapyParse *q = capy_parse(lib, U8("greet world\n"), 12, &st);
    CHECK(none(capy_node_func(q, n)));                                  /* n belongs to p */
    CHECK(streq(capy_node_func(q, capy_block_stmt(q, capy_root(q), 0)), "greet"));
    capy_parse_free(q);

    /* --- diagnostics ------------------------------------------------------------------------------- */
    CHECK(capy_diag_count(p) == 0 && none(capy_diag_code(p, 0)) && capy_diag_severity(p, 0) == 2);
    CHECK(capy_diag_primary(p, 0, &sp) == CAPY_E_ARG);
    CHECK(capy_diag_context(p, 0, 0, NULL, NULL, NULL) == CAPY_E_ARG);
    CHECK(capy_diag_label_count(p, 0) == 0 && none(capy_diag_label(p, 0, 0, &sp)));
    capy_parse_free(p);

    CapyParse *bp = capy_parse(lib, U8("nonsense here\n"), 14, &st);
    CHECK(bp && st == CAPY_OK && capy_is_clean(bp) == 0);
    CHECK(capy_diag_count(bp) >= 1 && streq(capy_diag_code(bp, 0), "E0001") && capy_diag_severity(bp, 0) == 0);
    CHECK(capy_diag_message(bp, 0).len > 0);
    CHECK(capy_diag_primary(bp, 0, &sp) == CAPY_OK && sp.start_line == 1);
    CapyId broot = capy_root(bp);
    CHECK(capy_block_error_count(bp, broot) >= 1);
    CapyId en = capy_block_error(bp, broot, 0);
    CHECK(en != 0 && capy_err_token_count(bp, en) >= 1 && capy_err_token_text(bp, en, 0).len > 0);
    CHECK(capy_err_diag_index(bp, en) < capy_diag_count(bp));
    CHECK(capy_err_diag_index(bp, broot) == UINT32_MAX);                 /* a block id is not an error node */
    CHECK(capy_err_span(bp, broot, &sp) == CAPY_E_KIND);
    capy_parse_free(bp);

    /* --- JSON -------------------------------------------------------------------------------------- */
    CapyParse *jp = capy_parse(lib, U8("greet world\n"), 12, &st);
    uint8_t *buf = NULL; size_t len = 0;
    CHECK(capy_parse_to_json(jp, 0x80, &buf, &len) == CAPY_E_ARG && buf == NULL && len == 0);   /* unknown flag */
    CHECK(capy_parse_to_json(jp, 0, NULL, &len) == CAPY_E_NULL);
    CHECK(capy_parse_to_json(NULL, 0, &buf, &len) == CAPY_E_NULL);
    CHECK(capy_parse_to_json(jp, 0, &buf, &len) == CAPY_OK && len > 20 && buf[0] == '{');
    CHECK(memmem(buf, len, "\"schema_version\":1", 18) != NULL && memmem(buf, len, "\"expr\"", 6) == NULL);
    capy_buf_free(buf, len);
    CHECK(capy_parse_to_json(jp, CAPY_JSON_EXPR | CAPY_JSON_PRETTY, &buf, &len) == CAPY_OK);
    CHECK(memmem(buf, len, "\"expr\"", 6) != NULL);
    capy_buf_free(buf, len);
    capy_parse_free(jp);

    capy_library_free(lib);
    return finish("misuse");
}
