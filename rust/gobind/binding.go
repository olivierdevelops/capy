// Package rustbind runs the Rust Capy engine as an embedded WebAssembly module
// on wazero — a pure-Go runtime — so Go callers get the engine with NO cgo, no
// C toolchain, and no loss of cross-compilation.
//
// This mirrors the architecture go-pdfium uses for PDFium, but the boundary here
// is far simpler: Capy's API is a pure function (libSrc, scriptSrc) → output, so
// there are no object handles to marshal and no reentrancy constraints. Each
// call gets a fresh module instance, which also makes runs trivially isolated.
//
// The exported API deliberately mirrors the native Go package's shape, so
// swapping the backend is a one-line import change:
//
//	lib, err := rustbind.NewLibrary(src)   // instead of capy.NewLibrary(src)
//	out, err := lib.Run(script)
package rustbind

import (
	"context"
	_ "embed"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"sync"

	"github.com/tetratelabs/wazero"
)

// The Rust engine, compiled to wasm32-unknown-unknown and embedded into this
// package's binary. No external file to ship, nothing to install.
//
//go:embed capy_core.wasm
var engineWasm []byte

// Runtime holds a compiled module, reusable across many calls. Compiling is the
// expensive step; instantiating per call is cheap and keeps runs isolated.
type Runtime struct {
	rt       wazero.Runtime
	compiled wazero.CompiledModule
	mu       sync.Mutex
}

// NewRuntime compiles the embedded engine. Reuse one Runtime for the life of the
// process.
func NewRuntime(ctx context.Context) (*Runtime, error) {
	rt := wazero.NewRuntime(ctx)
	compiled, err := rt.CompileModule(ctx, engineWasm)
	if err != nil {
		rt.Close(ctx)
		return nil, fmt.Errorf("compile capy engine: %w", err)
	}
	return &Runtime{rt: rt, compiled: compiled}, nil
}

// Close releases the runtime's resources.
func (r *Runtime) Close(ctx context.Context) error { return r.rt.Close(ctx) }

// result is the JSON shape every engine entry point returns.
type result struct {
	OK     bool              `json:"ok"`
	Output string            `json:"output"`
	Files  map[string]string `json:"files"`
	Error  string            `json:"error"`
	Hint   string            `json:"hint"`
	Line   int               `json:"line"`
	Col    int               `json:"col"`
}

// Error is the Go-side error carrying the engine's structured position + hint,
// so callers get the same diagnostics the native engine produces.
type Error struct {
	Msg  string
	Hint string
	Line int
	Col  int
}

func (e *Error) Error() string { return e.Msg }

// call is the locking wrapper around callRaw that decodes the common result
// shape. Callers wanting a different shape (Introspect) take the lock
// themselves and use callRaw directly.
func (r *Runtime) call(ctx context.Context, fn string, inputs ...string) (*result, error) {
	r.mu.Lock()
	raw, err := r.callRaw(ctx, fn, inputs...)
	r.mu.Unlock()
	if err != nil {
		return nil, err
	}
	var res result
	if err := json.Unmarshal(raw, &res); err != nil {
		return nil, fmt.Errorf("decode %s result: %w", fn, err)
	}
	return &res, nil
}

// callRaw instantiates a fresh module, copies the inputs into its linear memory,
// invokes `fn`, and returns the length-prefixed result buffer's payload.
//
// The caller must hold r.mu.
func (r *Runtime) callRaw(ctx context.Context, fn string, inputs ...string) ([]byte, error) {
	mod, err := r.rt.InstantiateModule(ctx, r.compiled, wazero.NewModuleConfig().WithName(""))
	if err != nil {
		return nil, fmt.Errorf("instantiate: %w", err)
	}
	defer mod.Close(ctx)

	alloc := mod.ExportedFunction("capy_alloc")
	free := mod.ExportedFunction("capy_free")
	target := mod.ExportedFunction(fn)
	if alloc == nil || free == nil || target == nil {
		return nil, fmt.Errorf("engine is missing exports (want capy_alloc/capy_free/%s)", fn)
	}
	mem := mod.Memory()

	// Copy each input into wasm memory; remember the allocations to free.
	type alloced struct {
		ptr uint32
		len uint32
	}
	var owned []alloced
	args := make([]uint64, 0, len(inputs)*2)
	for _, in := range inputs {
		b := []byte(in)
		n := uint32(len(b))
		var ptr uint32
		if n > 0 {
			res, err := alloc.Call(ctx, uint64(n))
			if err != nil {
				return nil, fmt.Errorf("alloc: %w", err)
			}
			ptr = uint32(res[0])
			if !mem.Write(ptr, b) {
				return nil, fmt.Errorf("write %d bytes at %d: out of range", n, ptr)
			}
			owned = append(owned, alloced{ptr, n})
		}
		args = append(args, uint64(ptr), uint64(n))
	}
	defer func() {
		for _, a := range owned {
			_, _ = free.Call(ctx, uint64(a.ptr), uint64(a.len))
		}
	}()

	out, err := target.Call(ctx, args...)
	if err != nil {
		return nil, fmt.Errorf("%s: %w", fn, err)
	}
	resPtr := uint32(out[0])
	if resPtr == 0 {
		return nil, fmt.Errorf("%s returned a null result", fn)
	}
	// [0..4) little-endian payload length, then the payload.
	hdr, ok := mem.Read(resPtr, 4)
	if !ok {
		return nil, fmt.Errorf("read result header at %d: out of range", resPtr)
	}
	n := binary.LittleEndian.Uint32(hdr)
	payload, ok := mem.Read(resPtr+4, n)
	if !ok {
		return nil, fmt.Errorf("read %d result bytes at %d: out of range", n, resPtr+4)
	}
	// Copy before freeing — mem.Read may alias wasm memory.
	buf := make([]byte, len(payload))
	copy(buf, payload)
	_, _ = free.Call(ctx, uint64(resPtr), uint64(4+n))

	return buf, nil
}

func (r *result) err() error {
	if r.OK {
		return nil
	}
	return &Error{Msg: r.Error, Hint: r.Hint, Line: r.Line, Col: r.Col}
}

// Library mirrors capy.Library: a library source held ready to run scripts.
//
// The engine boundary is stateless, so this holds the source text and hands it
// to the module on each call. That costs one re-parse per Run; a later revision
// can add a handle-based fast path if profiling shows it matters.
type Library struct {
	rt  *Runtime
	src string
}

// NewLibrary validates a library source and returns a handle for running scripts.
func (r *Runtime) NewLibrary(ctx context.Context, librarySrc string) (*Library, error) {
	// Introspect doubles as a compile check: it fails exactly when the library
	// fails to load.
	res, err := r.call(ctx, "capy_introspect", librarySrc)
	if err != nil {
		return nil, err
	}
	if err := res.err(); err != nil {
		return nil, err
	}
	return &Library{rt: r, src: librarySrc}, nil
}

// Run transpiles a script through the library, mirroring capy.Library.Run.
func (l *Library) Run(ctx context.Context, scriptSrc string) (string, error) {
	out, _, err := l.RunMulti(ctx, scriptSrc)
	return out, err
}

// RunMulti mirrors capy.Library.RunMulti, returning the multi-file map too.
func (l *Library) RunMulti(ctx context.Context, scriptSrc string) (string, map[string]string, error) {
	res, err := l.rt.call(ctx, "capy_run", l.src, scriptSrc)
	if err != nil {
		return "", nil, err
	}
	if err := res.err(); err != nil {
		return "", nil, err
	}
	files := res.Files
	if files == nil {
		files = map[string]string{}
	}
	return res.Output, files, nil
}

// ArgInfo mirrors capy.ArgInfo.
type ArgInfo struct {
	Kind        string `json:"kind"`
	Value       string `json:"value"`
	Name        string `json:"name"`
	Type        string `json:"type"`
	Description string `json:"description"`
	Optional    bool   `json:"optional"`
	Default     string `json:"default"`
}

// FunctionInfo mirrors capy.FunctionInfo.
type FunctionInfo struct {
	Name        string    `json:"name"`
	Description string    `json:"description"`
	Args        []ArgInfo `json:"args"`
	Block       string    `json:"block"`
	Priority    int       `json:"priority"`
}

type introspectResult struct {
	OK        bool           `json:"ok"`
	Functions []FunctionInfo `json:"functions"`
	Comments  []string       `json:"comments"`
	Error     string         `json:"error"`
	Hint      string         `json:"hint"`
}

// Introspect mirrors capy.Library.Introspect.
func (l *Library) Introspect(ctx context.Context) ([]FunctionInfo, []string, error) {
	l.rt.mu.Lock()
	raw, err := l.rt.callRaw(ctx, "capy_introspect", l.src)
	l.rt.mu.Unlock()
	if err != nil {
		return nil, nil, err
	}
	var ir introspectResult
	if err := json.Unmarshal(raw, &ir); err != nil {
		return nil, nil, fmt.Errorf("decode introspect: %w", err)
	}
	if !ir.OK {
		return nil, nil, &Error{Msg: ir.Error, Hint: ir.Hint}
	}
	return ir.Functions, ir.Comments, nil
}

// Docs mirrors capy.RenderLibraryDocs.
func (l *Library) Docs(ctx context.Context) (string, error) {
	res, err := l.rt.call(ctx, "capy_docs", l.src)
	if err != nil {
		return "", err
	}
	if err := res.err(); err != nil {
		return "", err
	}
	return res.Output, nil
}

// Version reports the embedded engine's version.
func (r *Runtime) Version(ctx context.Context) (string, error) {
	res, err := r.call(ctx, "capy_version")
	if err != nil {
		return "", err
	}
	return res.Output, res.err()
}
