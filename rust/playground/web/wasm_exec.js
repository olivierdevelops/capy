// Drop-in replacement for Go's `wasm_exec.js`, for the Rust-engine `capy.wasm`.
//
// The playground does:
//
//     <script src="wasm_exec.js"></script>
//     const go = new Go();
//     const resp = await fetch("capy.wasm");
//     …instantiate with go.importObject…
//     go.run(instance);
//     // then calls window.capyRun / capyDocs / capyIntrospect / capyVersion
//
// so this file defines a `Go` class with the same surface. `run(instance)`
// installs the four globals with byte-identical signatures and return shapes to
// the Go build's `syscall/js` exports, which lets index.html work UNCHANGED.
//
// The Rust module imports nothing, so `importObject` is empty.

(function () {
  "use strict";

  // --- linear-memory ABI (see rust/src/wasm_abi.rs) -------------------------
  //
  //   capy_alloc(len) -> ptr
  //   capy_free(ptr, len)
  //   capy_run(libPtr, libLen, scriptPtr, scriptLen) -> resultPtr
  //   capy_introspect(libPtr, libLen)                -> resultPtr
  //   capy_docs(libPtr, libLen)                      -> resultPtr
  //   capy_version()                                 -> resultPtr
  //
  // Every resultPtr addresses [u32 little-endian length][UTF-8 JSON payload].

  function makeBridge(exports) {
    const enc = new TextEncoder();
    const dec = new TextDecoder("utf-8");

    function mem() {
      // Re-read the buffer each time: it is detached and replaced whenever the
      // module grows its memory.
      return new Uint8Array(exports.memory.buffer);
    }

    // Copies a JS string into wasm memory. Returns [ptr, len] to free later.
    function putStr(s) {
      const bytes = enc.encode(s);
      if (bytes.length === 0) return [0, 0];
      const ptr = exports.capy_alloc(bytes.length);
      mem().set(bytes, ptr);
      return [ptr, bytes.length];
    }

    // Reads and frees a length-prefixed result buffer, returning parsed JSON.
    function takeResult(ptr) {
      if (!ptr) throw new Error("capy: engine returned a null result");
      const m = mem();
      const len =
        m[ptr] | (m[ptr + 1] << 8) | (m[ptr + 2] << 16) | (m[ptr + 3] << 24);
      const payload = dec.decode(m.subarray(ptr + 4, ptr + 4 + len));
      exports.capy_free(ptr, 4 + len);
      return JSON.parse(payload);
    }

    // Runs `fn` with each string argument copied in, freeing everything after.
    function call(fn, strings) {
      const owned = [];
      try {
        const args = [];
        for (const s of strings) {
          const [ptr, len] = putStr(s);
          if (len) owned.push([ptr, len]);
          args.push(ptr, len);
        }
        return takeResult(fn.apply(null, args));
      } finally {
        for (const [ptr, len] of owned) exports.capy_free(ptr, len);
      }
    }

    return {
      // capyRun(libSrc, format, scriptSrc) -> { ok, output, files } | { ok:false, error, hint }
      //
      // `format` is accepted and ignored, exactly as the Go build does once it
      // has sniffed the library — only "capy" is supported.
      capyRun(libSrc, _format, scriptSrc) {
        try {
          return call(exports.capy_run, [String(libSrc), String(scriptSrc)]);
        } catch (e) {
          return { ok: false, error: String(e && e.message ? e.message : e), hint: "" };
        }
      },

      // capyDocs(libSrc, format) -> { ok, docs } | { ok:false, error, hint, … }
      capyDocs(libSrc, _format) {
        try {
          return call(exports.capy_docs, [String(libSrc)]);
        } catch (e) {
          return { ok: false, error: String(e && e.message ? e.message : e), hint: "" };
        }
      },

      // capyIntrospect(libSrc) -> { ok, functions, comments } | { ok:false, … }
      capyIntrospect(libSrc) {
        try {
          return call(exports.capy_introspect, [String(libSrc)]);
        } catch (e) {
          return { ok: false, error: String(e && e.message ? e.message : e), hint: "" };
        }
      },

      // capyVersion() -> string
      capyVersion() {
        try {
          return call(exports.capy_version, []).output;
        } catch (e) {
          return "unknown";
        }
      },
    };
  }

  // --- the `Go` shim the playground constructs ------------------------------

  class Go {
    constructor() {
      // The Rust module imports nothing.
      this.importObject = {};
      this.exited = false;
    }

    // Go's run() starts the scheduler and never returns until exit; ours simply
    // installs the globals and returns, which is what the playground needs (it
    // polls for `window.capyRun` right after).
    run(instance) {
      const g = typeof globalThis !== "undefined" ? globalThis : window;
      const bridge = makeBridge(instance.exports);
      g.capyRun = bridge.capyRun;
      g.capyDocs = bridge.capyDocs;
      g.capyIntrospect = bridge.capyIntrospect;
      g.capyVersion = bridge.capyVersion;
      return Promise.resolve();
    }
  }

  const g = typeof globalThis !== "undefined" ? globalThis : window;
  g.Go = Go;
  // Also export for Node, so the shim can be exercised in tests.
  if (typeof module !== "undefined" && module.exports) {
    module.exports = { Go, makeBridge };
  }
})();
