// args: cases.json wasm_exec.js capy.wasm
const execSrc = await Deno.readTextFile(Deno.args[1]);
new Function(execSrc).call(globalThis);
const go = new globalThis.Go();
const bytes = await Deno.readFile(Deno.args[2]);
const { instance } = await WebAssembly.instantiate(bytes, go.importObject);
// Go's run() blocks on its scheduler and NEVER resolves (capy-wasm's main waits
// on a channel forever), so the promise must not be awaited. Both builds install
// their globals synchronously enough for the poll below.
const pending = go.run(instance);
if (pending && typeof pending.catch === "function") pending.catch(() => {});
for (let i = 0; i < 400 && typeof globalThis.capyRun !== "function"; i++) {
  await new Promise((r) => setTimeout(r, 5));
}
if (typeof globalThis.capyRun !== "function") throw new Error("capyRun never installed");
const cases = JSON.parse(await Deno.readTextFile(Deno.args[0]));
const out = [];
for (const c of cases) {
  const r = globalThis.capyRun(c.lib, "auto", c.script);
  const ins = globalThis.capyIntrospect(c.lib);
  const d = globalThis.capyDocs(c.lib, "auto");
  out.push({
    id: c.id,
    run: { ok: r.ok, output: r.output ?? null, extension: r.extension ?? null,
           files: r.files ? Object.keys(r.files).sort() : null,
           error: r.error ?? null, hint: r.hint ?? null,
           line: r.line ?? null, col: r.col ?? null, pretty: r.pretty ?? null },
    introspect: { ok: ins.ok, comments: ins.comments ?? null,
                  fns: ins.ok ? ins.functions.map((f) => ({ n: f.name, b: f.block,
                       p: f.priority, a: f.args.map((a) => [a.kind, a.value, a.name, a.type, a.description]) })) : null,
                  error: ins.error ?? null },
    docs: { ok: d.ok, docs: d.docs ?? null, error: d.error ?? null },
    version: globalThis.capyVersion(),
  });
}
console.log(JSON.stringify(out, null, 1));
