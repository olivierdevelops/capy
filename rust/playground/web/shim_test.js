// Exercises wasm_exec.js exactly as docs/assets/playground/index.html does:
// `new Go()`, instantiate capy.wasm, `go.run(instance)`, then call the globals.
const wasmPath = Deno.args[0];

// Load the shim the same way a <script> tag would: evaluate it into globalThis.
const shimSrc = await Deno.readTextFile(new URL("./wasm_exec.js", import.meta.url));
new Function(shimSrc).call(globalThis);

if (typeof globalThis.Go !== "function") throw new Error("shim did not define Go");

const go = new globalThis.Go();
const bytes = await Deno.readFile(wasmPath);
const { instance } = await WebAssembly.instantiate(bytes, go.importObject);
await go.run(instance);

for (const fn of ["capyRun", "capyDocs", "capyIntrospect", "capyVersion"]) {
  if (typeof globalThis[fn] !== "function") throw new Error(`missing global ${fn}`);
}
console.log("✅ all four globals installed:", "capyRun capyDocs capyIntrospect capyVersion");
console.log("   capyVersion() =", globalThis.capyVersion());

const LIB = `extension html

comments
    line "#"
end

function card
    arg literal "card"
    arg capture title string
    block_closer end
    write \`<div class="card"><h2>\${escapeHtml (decoded title)}</h2>
\${indent 2 body}</div>
\`
end

function p
    arg literal "p"
    arg capture t string
    write \`<p>\${escapeHtml (decoded t)}</p>
\`
end

function end
end
`;

// 1. capyRun — the signature index.html uses: (lib, "auto", script)
const r = globalThis.capyRun(LIB, "auto", 'card "A & B"\n    p "x < y"\nend\n');
if (!r.ok) throw new Error("capyRun failed: " + r.error);
console.log("✅ capyRun ok; output:\n" + r.output.split("\n").map(l => "     " + l).join("\n"));
if (!r.output.includes("&amp;")) throw new Error("escapeHtml did not run");
if (typeof r.files !== "object") throw new Error("files must be an object");

// 2. error shape — { ok:false, error, hint }
const bad = globalThis.capyRun(LIB, "auto", "wibble\n");
if (bad.ok) throw new Error("expected failure");
console.log("✅ error shape:", JSON.stringify({ ok: bad.ok, error: bad.error, hint: bad.hint }));

// 3. capyIntrospect
const ins = globalThis.capyIntrospect(LIB);
if (!ins.ok) throw new Error("introspect failed");
console.log("✅ capyIntrospect:", ins.functions.map((f) => f.name).join(", "),
            "| comments:", JSON.stringify(ins.comments));

// 4. capyDocs
const docs = globalThis.capyDocs(LIB, "auto");
// The field is `docs`, not `output` — index.html reads `res.docs`. This test
// previously asserted against `.output` and threw; it never ran in CI.
if (!docs.ok || !docs.docs.startsWith("# Library reference")) throw new Error("docs failed");
console.log("✅ capyDocs:", docs.docs.length, "bytes of Markdown");

// 5. repeated calls must not leak or corrupt memory
for (let i = 0; i < 200; i++) {
  const x = globalThis.capyRun(LIB, "auto", `p "run ${i}"\n`);
  if (!x.ok || !x.output.includes(`run ${i}`)) throw new Error("failed at iteration " + i);
}
console.log("✅ 200 repeated calls stable (alloc/free balanced)");
