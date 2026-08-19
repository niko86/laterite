---
type: concept
title: "Arrow C FFI under a foreign global allocator: who frees what"
status: drafted
tags: [concept, architecture, pyo3, arrow, ffi, allocator]
volatile: [versions]
volatile_asof: 2026-08-13
ags_editions: []
repo_refs:
  latpy: "repo:rust-packages/laterite-py/src/lib.rs"
  latpy_manifest: "repo:rust-packages/laterite-py/Cargo.toml"
  emit: "repo:rust-packages/laterite-py/src/emit_typed.rs"
  guard: "repo:packages/laterite/python/laterite/__init__.py"
  compat_guard: "repo:packages/laterite/python/laterite/compat/_impl.py"
related: [pyo3-boundary, ags4-output, crate-map, laterite-py, laterite, abi3-perf]
sources: []
---
# Arrow C FFI under a foreign global allocator: who frees what

## Definition

[[pyo3-boundary]] establishes *where* Rust stops and Python starts and *what*
crosses (Arrow C Data Interface capsules, both directions). This page answers
the question that page does not: **when `laterite-py` installs mimalloc as
`#[global_allocator]`, whose allocator frees the Arrow buffers?**

**The answer, up front — and it is the opposite of the intuitive one:**

1. **The Arrow C Data Interface is allocator-agnostic by construction.** A
   buffer is never freed by "the allocator"; it is freed by the **producer's own
   release callback**, a function pointer carried inside the struct. Whoever
   allocated the memory supplies the code that frees it, so the free always runs
   producer-side. **Verified in arrow-rs source**, not inferred (§2).
2. **The Arrow FFI is not implicated at all — and this was measured, not
   inferred.** The minimal reproduction is two imports and a pyarrow table, with
   **no laterite call in it whatsoever** (§7). arrow-rs is exonerated by
   construction *and* by experiment. Two corroborating existence proofs: polars
   ships jemalloc/mimalloc in its Python extension and consumes pyarrow daily;
   datafusion-python ships mimalloc unconditionally (§6).
3. **The cause is two co-resident *instances of the same allocator*** — ours and
   the one inside pyarrow's default memory pool — with ours built as **v3.3.2**
   (§3b). The upstream co-residency reports are **microsoft/mimalloc#1327** and
   **apache/arrow GH-50428**. **Fixed our side by pinning `features = ["v2"]`**
   (PR #301); **fixed upstream in pyarrow 25.0.1**, so the exposure window is
   pyarrow **24.0.0–25.0.0**.
4. **The `pl.from_pandas` guard stays — but for a different reason than its
   comment claimed.** It never made the pandas path memory-safe, and on the
   pyarrow path it copies nothing at all. It is load-bearing for **dep shape**:
   it keeps a pyarrow-free `[compat]` install working (§5).

> [!note] Resolved — the experiment in §7 has been run
> This page began as a research pass while the cause was still open. The matrix
> in §7 has since been **executed**, and the fix shipped as
> [PR #301](https://github.com/niko86/laterite/pull/301)
> (`fix/mimalloc-coresident-heap-corruption`). Measured results are recorded
> inline throughout, marked **Measured**.

> [!important] The fault has a second face — recognise it by its precondition
> #297 recorded `tests/test_docs_snippets.py` as never terminating when selected
> on its own: CPU-bound, no output, no diagnostic. It was filed as a separate
> ordering bug, explicitly argued not to be #294, because the shape was a spin
> rather than corruption. It was this fault. Reconstructed as a 2x2 — our pin
> (v2 / v3) against pyarrow (inside the window / past it) — with the module's
> own selection as the probe: only v3 with a pre-25.0.1 pyarrow hangs, and
> either fix alone clears it. A corrupted free list can spin as readily as it
> can crash, and which one surfaces depends on allocation order — which is why
> the module was fine with its siblings collected and not on its own.
>
> So do not triage this fault by symptom. The precondition is two co-resident
> allocators; the symptom is whatever the allocation order makes of it.
>
> The pin is now gated off the manifests by
> `repo:tests/test_allocator_pin_faithful.py`, which discovers every crate that
> sets a `#[global_allocator]` and asserts the feature. That gate exists because
> the runtime guard in `repo:packages/laterite/tests/test_public_api_surface.py`
> can only go red INSIDE the fault window — on a fixed pyarrow it passes
> whatever the pin says, and the pin is the half we control.

Versions this page is scoped to (`repo:rust-packages/Cargo.lock`,
`repo:packages/laterite/pyproject.toml`, dev env at 2026-08-13):
`arrow` **59.2.0**, `pyo3` **0.29.2**, `pyo3-arrow` **0.19.0**,
`mimalloc` **0.1.52** / `libmimalloc-sys` **0.1.49** (v3.3.2 on default features;
**all three surfaces that install a global allocator — `laterite-py`,
`laterite-node` and `laterite-cli` — now pin `v2` = 2.3.2**, py in PR #301 and
the other two alongside it),
pyarrow **25.0.0** (**inside the 24.0.0–25.0.0 fault window**; 25.0.1 fixes it
upstream), pandas **2.3.3**, polars **1.43.2**.

## 1. Ownership across the Arrow C Data Interface

The spec is unambiguous: **the producer owns everything, and the consumer's only
lever is the release callback.**

> "The producer allocates and maintains all data pointed to by `ArrowSchema` and
> `ArrowArray`… The consumer must not interfere with this data's lifetime except
> through the release callback."
> — <https://arrow.apache.org/docs/format/CDataInterface.html>

The division of labour on release:

- **Consumer** MUST call the *base* structure's release callback when finished,
  and MUST NOT call any child's (or the dictionary's) callback — the producer's
  callback walks those itself.
- **Producer's callback** MUST walk all children, MUST free any data area it
  directly owns, and MUST then set `release` to NULL to mark the struct released.

"Moving an array" is a bitwise or shallow member-wise copy; the mover **marks the
source released without calling the callback**, so exactly one release happens.
The struct must be *trivially relocatable* — no pointer member (including
`private_data`) may point inside the struct.

The PyCapsule binding layers this onto Python
(<https://arrow.apache.org/docs/format/CDataInterface/PyCapsuleInterface.html>):
the capsule destructor calls the release callback **only if not already null**,
which is precisely why a consumer that moves the struct out must null the source.
As the spec puts it: *"If the capsule has been passed to a consumer, the consumer
should have moved the data and marked the release callback as null, so there
isn't a risk of releasing data the consumer is using."* Capsules are
**single-use**.

**Neither document mentions allocators at all.** That silence is the design: the
contract is expressed entirely in terms of *callbacks*, never in terms of
*allocators*, so an allocator never has to be named or matched.

## 2. Is the release callback guaranteed to run on the producer's allocator?

**Yes — and in arrow-rs the global allocator is not merely avoided, it is
structurally unreachable for foreign buffers.** This is the crux, so here is the
whole chain, verified by reading `arrow-array` at tag `59.1.0`. The workspace has
since moved to **59.2.0**, and the chain is untouched by that bump:
`arrow-array/src/ffi.rs` is byte-identical between the two tags
(`sha256 df4d434b…`, 1968 lines both), so every quotation below still describes
the code we ship.

**Step 1 — a foreign pointer becomes a `Buffer` with the FFI struct as its
owner** (`arrow-array/src/ffi.rs`,
<https://github.com/apache/arrow-rs/blob/59.1.0/arrow-array/src/ffi.rs>):

```rust
unsafe fn create_buffer(
    owner: Arc<FFI_ArrowArray>,
    array: &FFI_ArrowArray,
    index: usize,
    len: usize,
) -> Option<Buffer> {
    if array.num_buffers() == 0 { return None; }
    NonNull::new(array.buffer(index) as _)
        .map(|ptr| unsafe { Buffer::from_custom_allocation(ptr, len, owner) })
}
```

**Step 2 — that constructor tags the buffer `Deallocation::Custom`**
(`arrow-buffer/src/buffer/immutable.rs`):

```rust
pub unsafe fn from_custom_allocation(
    ptr: NonNull<u8>, len: usize, owner: Arc<dyn Allocation>,
) -> Self {
    unsafe { Buffer::build_with_arguments(ptr, len, Deallocation::Custom(owner, len)) }
}
```

`Deallocation` has exactly two variants (`arrow-buffer/src/alloc/mod.rs`):
`Standard(Layout)` — "An allocation using `std::alloc`" — and `Custom(Arc<dyn
Allocation>, usize)` — "An allocation from an external source like the FFI
interface. Deallocation will happen on `Allocation::drop`".

**Step 3 — the drop that decides everything** (`arrow-buffer/src/bytes.rs`):

```rust
impl Drop for Bytes {
    #[inline]
    fn drop(&mut self) {
        match &self.deallocation {
            Deallocation::Standard(layout) => match layout.size() {
                0 => {}
                _ => unsafe { std::alloc::dealloc(self.ptr.as_ptr(), *layout) },
            },
            // The automatic drop implementation will free the memory once the
            // reference count reaches zero
            Deallocation::Custom(_allocation, _size) => (),
        }
    }
}
```

**This is the whole answer.** `std::alloc::dealloc` — the call that routes to
`#[global_allocator]`, i.e. mimalloc — sits on the `Standard` arm only. The
`Custom` arm is *literally an empty body*. A pyarrow-allocated pointer can never
reach mimalloc's `dealloc`, because the only branch that could take it there is
not the branch it is on.

**Step 4 — what actually frees it.** Dropping the last `Buffer` drops the
`Arc<FFI_ArrowArray>`, and (`arrow-data/src/ffi.rs`):

```rust
impl Drop for FFI_ArrowArray {
    fn drop(&mut self) {
        match self.release {
            None => (),
            Some(release) => unsafe { release(self) },
        };
    }
}
```

`release` is **pyarrow's own C function pointer**, so the free executes inside
Arrow C++ against Arrow C++'s memory pool. Producer allocated, producer freed.

**The symmetry matters, and it is the reason the export direction is safe too.**
When arrow-rs *exports*, it installs its own Rust callback:

```rust
unsafe extern "C" fn release_array(array: *mut FFI_ArrowArray) {
    …
    let private = unsafe { Box::from_raw(array.private_data as *mut ArrayPrivateData) };
    …
    array.release = None;
}
```

`Box::from_raw` → drop → `std::alloc::dealloc` → **mimalloc**, the same allocator
that allocated it — even though *Python* triggered the call. The callback is a
function pointer, so "producer-side" is a property of the **code**, not of the
call site.

`FFI_ArrowArray::from_raw` implements the spec's move as
`std::ptr::replace(array, Self::empty())`, and `empty()` sets `release: None` —
so the capsule destructor sees a released struct and does nothing. **Double
release is structurally prevented**, matching the local ticket's finding that it
was ruled out.

> [!note] The invariant is already stated correctly in the code
> `repo:rust-packages/laterite-py/src/lib.rs` says it in the comment above the
> allocator: *"the engine leaves the Rust boundary via Arrow release callbacks,
> so Rust frees what Rust allocated (no cross-allocator handoff to
> polars/pyarrow)."* That comment is **right**, and §2 is its proof.

## 3. The failure modes that remain

Given §2, allocator *mismatch* is off the table. What is left:

**(a) Alignment — a real copy, but a safe one.** Both `from_ffi` and
`from_ffi_and_data_type` end with `data.align_buffers()`, because arrow-rs is
stricter than the spec (which only says buffers *MAY* be aligned to their
primitive type and consumers *MAY* refuse unaligned memory).
`arrow-data/src/data.rs`:

```rust
pub fn align_buffers(&mut self) {
    let layout = layout(&self.data_type);
    for (buffer, spec) in self.buffers.iter_mut().zip(&layout.buffers) {
        if let BufferSpec::FixedWidth { alignment, .. } = spec {
            if buffer.as_ptr().align_offset(*alignment) != 0 {
                *buffer = Buffer::from_slice_ref(buffer.as_ref());
            }
        }
    }
    for data in self.child_data.iter_mut() { data.align_buffers() }
}
```

This is **the only place a `Layout`-based dealloc touches a path that received
foreign pointers** — and it is safe, because it *replaces* the foreign buffer
with a fresh Rust allocation (freed by Rust) while the foreign one drops through
the no-op `Custom` arm. It is a no-op for pyarrow and polars, both of which
allocate 64-byte-aligned. **Alignment is not the hazard here.**

**(b) Two co-resident instances of the *same* allocator.** This is the one that
matters, and it is documented in the wild:

- **pyarrow's own default memory pool is mimalloc.** Arrow C++ docs: *"The
  default memory pool depends on how Arrow C++ was compiled: if enabled at
  compile time, a mimalloc heap; otherwise… a jemalloc heap; otherwise, the C
  library malloc heap"* (<https://arrow.apache.org/docs/cpp/memory.html>).
  Official wheels compile both in, so **mimalloc wins**. Confirmed on this
  machine: `pa.default_memory_pool().backend_name` → `'mimalloc'`
  (pyarrow 25.0.0), with `supported_memory_backends()` → `['mimalloc', 'system']`.
- So a laterite process holds **two statically-linked mimalloc instances**: the
  Rust crate's, and the one inside `libarrow`. Each has its own global and
  thread-local state.
- **apache/datafusion-python#1607** — *"Segfault with PyArrow 24 on macOS when
  using mimalloc v3"*, **open** — is the closest real-world precedent, crashing
  inside `arrow::MimallocAllocator::ReallocateAligned`.
  <https://github.com/apache/datafusion-python/issues/1607> Its mitigation is
  ours: pin the Rust side to mimalloc **v2** via the crate's `v2` feature — *"no
  performance loss, no PyArrow pin."*
- **microsoft/mimalloc#1327** is the **co-residency** report, and the right
  upstream anchor: *"Two independently-linked mimalloc v3 instances (CPython 3.14
  vendored + static in a Python extension) SIGSEGV in `_mi_theap_collect_retired`
  at process exit when Arrow's v2 copy is also loaded (macOS arm64)"*, **closed**.
  <https://github.com/microsoft/mimalloc/issues/1327> Its shape matches ours
  closely — macOS arm64, a PyO3/abi3 cdylib linking mimalloc through
  `libmimalloc-sys` as `#[global_allocator]`, and a discriminating matrix over
  extension-version × pyarrow-version. It also records a **third** instance we had
  only inferred: **CPython 3.14 vendors its own mimalloc v3.3.2**.
- **apache/arrow GH-50428** is the same fault seen from Arrow's side —
  *"pyarrow 24.0.0 regression: co-loading a native extension bundling mimalloc v3
  SIGSEGVs in bundled mimalloc at interpreter teardown"* — **closed**, milestone
  **25.0.1**, fixed by **apache/arrow#50549** *"[C++] Better mimalloc
  configuration on macOS"* (**merged** 2026-07-21).
  <https://github.com/apache/arrow/issues/50428> Arrow independently names
  switching the extension's mimalloc to **v2** as the workaround — arm A, reached
  from the other direction.

> [!caution] Cite these as the same *family*, not the same *symptom*
> #1327 and GH-50428 both crash at **interpreter teardown** (#1327's repro exits
> 139, and `os._exit(0)` avoids it entirely). Laterite's fault is **mid-run**:
> deterministic zeroing of the first 64 bytes during `to_pylist`, with no teardown
> involved. #1327 also has pyarrow on the **v2** line, whereas we measured
> pyarrow's pool as mimalloc. Shared root cause — multiple co-resident mimalloc
> instances on macOS — **different manifestation**.
>
> An **earlier revision of this page cited microsoft/mimalloc#1287** as the
> version-range authority. That is a *threading* bug (*"mimalloc >= 3.3.0 causes
> segmentation faults when used from multiple threads"*,
> <https://github.com/microsoft/mimalloc/issues/1287>). It is a real issue and
> datafusion-python#1607 genuinely does root-cause to it, so it is kept here — but
> using a threading bug as the authority for a non-threading fault was an error,
> and it propagated into source comments and a public upstream issue before being
> caught. **#1327 is the co-residency citation; #1287 is not.**

> [!success] Measured — this is the cause, and the cross-matrix is unambiguous
> Varying **our `.so`'s allocator** against **pyarrow's memory pool**
> independently (issue #294 / PR #301):
>
> | our allocator | pyarrow pool | result |
> |---|---|---|
> | mimalloc (v3) | mimalloc | **corrupt** |
> | mimalloc (v3) | system | clean |
> | system | mimalloc | clean |
> | system | system | clean |
>
> Only the **both-mimalloc** cell fails. Neither allocator is faulty alone, which
> is precisely the two-co-resident-instances signature and not an
> allocator-mismatch signature.
>
> **Direct evidence that our allocator never touches pyarrow's memory:** an
> instrumented `GlobalAlloc` wrapping `MiMalloc`, reporting
> `alloc`/`alloc_zeroed`/`dealloc` of a watched address, was pointed at pyarrow's
> data buffer (address read via `Buffer.address`). It **never fired**, while the
> corruption still occurred. §2's structural argument is therefore confirmed
> empirically: the cross-allocator-free story is dead.

**(c) ELF static-TLS under `dlopen`.** **PyO3/pyo3#678**, *"jemallocator & pyo3
?"*, **closed** (self-resolved 2019): `cannot allocate memory in static TLS
block` on import, fixed by `features = ["disable_initial_exec_tls"]`.
<https://github.com/PyO3/pyo3/issues/678> This is the known PyO3 material about
`#[global_allocator]` in extension modules — and note it is about **TLS
mechanics, not buffer ownership**. polars uses that exact flag today.

**Not stated by any source found:** that allocator *mismatch* across the Arrow C
Data Interface causes heap corruption. **arrow-rs#10439**, filed from this repo,
was the only issue advancing that theory — and it has since been **closed** (2
comments) with the evidence and a correction, because §2 and the §3b matrix show
the FFI is not on the path.
<https://github.com/apache/arrow-rs/issues/10439> Searches of apache/arrow-rs for
`mimalloc`/allocator/segfault surfaced no other issue making that claim.

## 4. pyo3-arrow

`pyo3-arrow` **0.19.0** (tag `pyo3-arrow-v0.19.0` in `kylebarron/arro3`; its
manifest pins `arrow-array = "59"`, `pyo3 = "0.29"` — matching ours) is a
**pure passthrough**. It extracts the capsule and hands the raw struct to
arrow-rs; it never touches buffers itself.

```rust
pub(crate) fn import_stream_pycapsule(capsule: &Bound<PyCapsule>)
    -> PyResult<FFI_ArrowArrayStream> {
    let stream_ptr = capsule
        .pointer_checked(Some(ARROW_ARRAY_STREAM_CAPSULE_NAME))?
        .cast::<FFI_ArrowArrayStream>();
    Ok(unsafe { FFI_ArrowArrayStream::from_raw(stream_ptr.as_ptr()) })
}
```
<https://github.com/kylebarron/arro3/blob/pyo3-arrow-v0.19.0/pyo3-arrow/src/ffi/from_python/utils.rs>

`PyTable::from_arrow_pycapsule` — the type
`repo:rust-packages/laterite-py/src/emit_typed.rs` uses — then drives arrow-rs's
own reader and **eagerly drains it** into `Vec<RecordBatch>`:

```rust
let stream = import_stream_pycapsule(capsule)?;
let stream_reader = ArrowRecordBatchStreamReader::try_new(stream)…;
for batch in stream_reader { batches.push(batch?); }
```
<https://github.com/kylebarron/arro3/blob/pyo3-arrow-v0.19.0/pyo3-arrow/src/table.rs>

**On allocators, the crate is silent.** Its README (which *is* the crate docs via
`#![doc = include_str!("../README.md")]`) asserts zero-copy repeatedly — *"zero-copy FFI
conversions between Python objects and Rust representations using the `arrow`
crate"* — but **never mentions allocators, memory pools, mimalloc, jemalloc, or
`#[global_allocator]`**. Its issue tracker has **zero** hits for mimalloc,
jemalloc, heap corruption, use-after-free, or double free; the one real segfault
issue (#230, **closed**) is a PyO3 GIL/interpreter-teardown bug on the numpy
buffer-protocol side, unrelated.

This corroborates the local ticket: the wrapper is not implicated, because there
is nothing in it to implicate.

## 5. `pl.from_pandas` — is it zero-copy?

**Verdict: on the pyarrow path it copies nothing, so the guard never did what its
comment said. It stays anyway — but for a dep-shape reason, not a safety one.**

**It adopts, it does not copy.** `pandas_series_to_arrow` unconditionally calls
`pa.array(values, from_pandas=nan_to_null)`; for an `ArrowDtype` column pyarrow
short-circuits through `__arrow_array__`, which returns the *existing*
`ChunkedArray` verbatim. The Rust import is foreign adoption, not a memcpy
(`crates/polars-arrow/src/ffi/array.rs`):

```rust
// We have to check alignment, for zero-copy to be valid.
if ptr.is_aligned() {
    let slice = core::slice::from_raw_parts(ptr.add(offset), len);
    let storage = SharedStorage::from_slice_with_owner(slice, owner);
    Ok(Buffer::from_storage(storage))
} else {
    // Byte-wise copy for misaligned buffers.
```

and the owner's `Drop` calls the **producer's** callback — the same pattern as
arrow-rs (§2).

**`rechunk=True` (the default) is not an escape.** It is guarded
(`crates/polars-core/src/frame/mod.rs`):

```rust
pub fn rechunk_mut_par(&mut self) -> &mut Self {
    if self.columns().iter().any(|c| c.n_chunks() > 1) {
```

Single-chunk everywhere ⇒ the body never runs. **No copy.**

Measured on this repo's venv (polars 1.43.1 / pandas 2.3.3 / pyarrow 25.0.0):
buffer addresses are **identical** across `pl.from_pandas`, and
`pa.total_allocated_bytes()` stays elevated after the pandas frame is deleted,
dropping to zero only when the **polars** frame is dropped — polars is holding
pyarrow's allocation alive.

**So why does the guard work?** Because it changes **the producer**, not the
memory. When polars re-exports via `__arrow_c_stream__`, it stamps *polars'* own
`c_release_array` and private data onto the exported struct. The Rust consumer
stops seeing a **pyarrow-produced** stream and starts seeing a
**polars-produced** one; pyarrow's buffers are still there, released transitively
one level down. That is exactly consistent with the ticket's observation that
polars streams are never affected — and it relocates the defect to *the
consumer's handling of pyarrow's release semantics*, not to memory ownership.

**One genuine, but data-dependent, escape hatch.** `pandas_to_pydf` has an
all-or-nothing numpy bypass: if **every** column is "simple numpy-backed" it
converts via `to_numpy()` and pyarrow is never touched (measured: 0 bytes
allocated). `object`-dtype string columns count as simple **only when the Series
has no NaNs and is non-empty**. AGS4 frames are mostly all-string, so many
laterite frames really do come out fully polars-owned — but **add one `None` and
it falls back to the pyarrow path.** Resting a heap-corruption fix on that is
fragile.

> [!warning] The guard's comment was wrong — corrected in PR #301
> `repo:packages/laterite/python/laterite/__init__.py` claimed that normalising
> through `pl.from_pandas` avoids handing over pyarrow-backed memory. It does not
> — the polars frame usually wraps *the very same pyarrow buffers*. Same for
> `repo:packages/laterite/python/laterite/compat/_impl.py`'s `_to_polars`. Both
> comments were corrected in PR #301.

> [!success] Measured — the guard is still load-bearing, for dep shape
> After the v2 pin the unguarded pandas path is **memory-safe**, so the guard is
> no longer what makes it correct. It stays for a different reason: pandas'
> `__arrow_c_stream__` calls `import_optional_dependency("pyarrow")`, so on a
> **pyarrow-free `[compat]` install** the unguarded path raises `ImportError`.
> Measured with pyarrow blocked: **guarded OK, unguarded `ImportError`**. Since
> `[compat]` is specified as pandas-only and pyarrow-free (see
> [[pyo3-boundary]] and the dep-shape split), removing the guard would break the
> advertised install shape. **Keep it; the comment must say *dep shape*, not
> *memory safety*.**

## 6. What other projects do

**polars — the existence proof holds.** Its Python extension installs a
non-system global allocator **by default, unconditionally**
(`crates/polars-python/src/c_api/allocator.rs`):

```rust
#[global_allocator]
static ALLOC: polars_ooc::Allocator = polars_ooc::Allocator;
```

delegating per-target (`crates/polars-ooc/src/global_alloc.rs`): **jemalloc on
Unix** (`tikv-jemallocator`, with `disable_initial_exec_tls` — the pyo3#678 fix),
**mimalloc on Windows/emscripten**, system without `fast_alloc` (which is in
`default`). The runtime crate turns the feature on unconditionally. The choice is
a **measured performance** decision (PR #3108), not a safety carve-out; PR #7194,
which would have dropped mimalloc on Windows, was **closed unmerged**.

A library with polars' download volume, shipping a foreign global allocator and
consuming pyarrow buffers every day, is decisive: **allocator identity alone
cannot be the bug.**

**datafusion-python — mimalloc unconditionally, and it hit this exact wall.**
`crates/core/src/lib.rs` sets `#[global_allocator] static GLOBAL: MiMalloc`, with
`default = ["mimalloc", "abi3"]` and no target gating — in a crate that imports
pyarrow directly. Its manifest carries the tell:

```toml
mimalloc = { workspace = true, optional = true, features = [
  "local_dynamic_tls",
  # Pin to mimalloc v2 until apache/datafusion-python#1607 resolves.
  "v2",
] }
```

**Mitigation patterns actually used** (all at the allocator-crate-configuration
layer, none at the FFI call site): TLS-model flags
(`disable_initial_exec_tls` / `local_dynamic_tls`); **version-pinning the
allocator** (datafusion-python → mimalloc v2); platform-conditional choice
(polars). **No project found mitigates by forcing a copy at the FFI boundary or
by dropping the global allocator** — which is worth noting, because the
`pl.from_pandas` guard is closest in spirit to the former.

## 7. What this means for laterite

### The cause, and the fix is one line

laterite declares `mimalloc = "0.1"` in all three surfaces
(`repo:rust-packages/laterite-py/Cargo.toml`, `laterite-cli`, `laterite-node`)
**with no features**. And `libmimalloc-sys` 0.1.49's `build.rs` selects:

```rust
let version = if env::var("CARGO_FEATURE_V2").is_ok() { "v2" } else { "v3" };
```

The vendored headers give `v2` = **2.3.2** and `v3` = **3.3.2**. Nothing in the
workspace opts into `v2` (grep: no `"v2"` in any `Cargo.toml`). **laterite
therefore builds mimalloc v3.3.2** — co-resident with pyarrow's own bundled
mimalloc, in a `dlopen`'d extension module, on macOS. That is the configuration
microsoft/mimalloc#1327 and apache/arrow GH-50428 both report, and
datafusion-python#1607's situation feature-for-feature.

When first written this was **inference**. It has since been **confirmed by
experiment** (§3b matrix, and the reproduction below), and fixed in PR #301 by
pinning `laterite-py` to `features = ["v2"]`. It explains every observed fact the
allocator-mismatch theory could not — why only mimalloc, why the system allocator
masks it, and why the Arrow contract audits clean (§2).

**Scope is narrower than feared: pyarrow only.** polars and duckdb are clean
under the same allocation churn, so base `pip install laterite` — which is
polars + duckdb and no pyarrow — **was never exposed**. Only the
`[pyarrow]` / `[compat,pyarrow]` / `[all]` extras and dev/CI environments could
hit it.

**And it is fixed upstream too, which bounds the exposure.** Measured against an
**unchanged v3 consumer**: pyarrow **25.0.0** corrupts **44 of 50** array shapes;
pyarrow **25.0.1** corrupts **0 of 50**. A `v2` consumer is clean on both. So the
fault window is pyarrow **24.0.0 – 25.0.0**, closed by
apache/arrow#50549 (milestone 25.0.1). The v2 pin and a pyarrow floor are
independent fixes; #301 takes the pin because it protects users who have not
upgraded pyarrow.

> [!note] `backend_name` does not disclose the major
> `pa.default_memory_pool().backend_name` reports `'mimalloc'` on **both** 25.0.0
> and 25.0.1, so it confirms *which allocator* pyarrow's pool uses and says
> nothing about *which version*. It therefore neither corroborates nor
> contradicts GH-50428's note that `libarrow.2400` carries a **v2-line** copy.
> Don't over-read it in either direction.

**Corruption signature** (useful if this is ever taken upstream): it needs a
string buffer **larger than 64 bytes across 2+ rows**, landing in the **first
pyarrow allocations after our module loads**. A 1-row array never corrupts even
at 4 KB. The measured boundary is exact — 2 rows × 32 chars (64 B) is fine,
2 × 33 (66 B) corrupts.

**Performance, since it decided the fix.** v2 keeps **~18–19% read** and
**~14–15% validate** over the system allocator on the 25 MB fixture, and costs
only **~6% read** versus v3. Dropping mimalloc entirely would have cost read
**122 ms → 150 ms** — which is why the fix is a version pin, not a removal.

### Recommendations

**Done (PR #301):**

1. **`features = ["v2"]` on `laterite-py`'s mimalloc** — the datafusion-python
   fix. Confirmed by the §3b matrix.
2. **Both guard comments corrected** (§5), and the guard **kept** on its real
   (dep-shape) justification.
3. **mimalloc not dropped, and nothing "fixed" at the FFI boundary.** §2 shows
   the boundary is correct by construction; §6 shows the ecosystem's mitigations
   all live in allocator configuration. The `no-mimalloc` arm rides in #301 only
   as the **control** that rules out masking.

**Outstanding:**

4. **`laterite-node` and `laterite-cli` still declare bare `mimalloc = "0.1"`**,
   so both still build **v3.3.2**. Neither loads pyarrow today, so neither is
   known-exposed — but they carry the same allocator version, and the node addon
   is the same `dlopen`'d-extension shape that microsoft/mimalloc#1327 describes.
   **Owner's call**, deliberately not swept into #301.
5. ~~**arrow-rs#10439 is mis-filed.**~~ **Done** — closed with the evidence and a
   correction. It blamed the Arrow FFI for what is a mimalloc bug, and §2 plus the
   §3b matrix show the FFI is not on the path.
6. **Consider `local_dynamic_tls`** (§3c) — cheap, and the ecosystem standard for
   allocators inside `dlopen`'d extension modules. Not required by any observed
   failure here.

### The experiment that settled it

The matrix below was run, changing **one** variable at a time. Predictions were
made from the co-residency hypothesis before the arms were executed:

| Arm | Change | Predicted | **Measured** |
|---|---|---|---|
| baseline | as filed | SIGSEGV / corruption | **corrupt** |
| **A** | Rust side `mimalloc` features `["v2"]` | survives | **survives — shipped in #301** |
| B | `pa.set_memory_pool(pa.system_memory_pool())` at import | survives | **survives** |
| C | `ARROW_DEFAULT_MEMORY_POOL=system` in env | survives | **survives** |
| D | drop `#[global_allocator]` entirely | survives | **survives** (control only — proves nothing on its own) |

**A and B both surviving confirms two-mimalloc-instances and refutes
allocator-mismatch**, since A changes only *our* allocator and B changes only
*pyarrow's* — yet either alone is sufficient.

### The reproduction — and what actually gates it

The minimal reproduction contains **no laterite call at all**. Two imports and a
pyarrow table:

```python
import pyarrow as pa
import laterite                       # merely loading the extension is enough

pa.table({"c": ["A" * 32] * 4}).column(0).chunk(0).to_pylist()
# -> first 64 bytes zeroed. 100% deterministic.
```

That importing the module is sufficient — with the Arrow FFI never exercised —
is what makes this **stronger than "the FFI is probably not at fault"**: the FFI
is not on the path.

> [!warning] The variable is **pyarrow's version**, not iteration count — and not pandas
> An earlier revision of this page claimed the bug "cannot be reproduced by
> iteration count", citing an attempt here of 3 pandas shapes × 400 iterations
> plus a 300-iteration emit→`laterite.read()` canary that **survived everything**.
> That non-reproduction has since been explained, and **it was not about iteration
> count**: the environment had resolved to a pyarrow outside the 24.0.0–25.0.0
> fault window. At the reported versions it reproduces **deterministically, first
> try**.
>
> A second wrong turn is worth recording because it is the easy one to take:
> pandas was initially blamed, after a resolver pulled pandas 3.0.5. **The pandas
> major is not the variable** — it SIGSEGVs on both majors at pyarrow 25.0.0 and
> survives on both at 25.0.1. Pin pyarrow when bisecting this, not pandas.
>
> What does still hold is the **shape** of the fault: the corruption lands in the
> first pyarrow allocations after our module loads (the §7 signature), so it is
> startup-order-dependent. Any regression test must therefore run in a
> **subprocess** and **A/B the faulty artifact** — and must pin a pyarrow inside
> the fault window, or it can never go red. That last point is why the existing
> #122 regression test did not catch this.

## Diagram

```mermaid
flowchart LR
  subgraph py["Python (pyarrow pool = mimalloc #1)"]
    pd["pandas frame"] -->|"__arrow_c_stream__"| cap["PyCapsule"]
  end
  subgraph rs["laterite-py cdylib (global_allocator = mimalloc #2, v3.3.2)"]
    cap --> pyo3a["pyo3-arrow PyTable<br/>(passthrough)"]
    pyo3a --> ffi["arrow-rs from_ffi<br/>Deallocation::Custom"]
    ffi -.->|"drop = NO-OP"| ga["std::alloc::dealloc<br/>(never reached)"]
    ffi -->|"Arc drop"| rel["producer release callback"]
  end
  rel -->|"frees via"| py
  style ga stroke-dasharray: 5 5
```

## Where it shows up

The import (foreign-producer) direction is narrow — `emit_ags4_from_arrow` in
`repo:rust-packages/laterite-py/src/emit_typed.rs`, reached from
`build_ags4`/`emit_ags4`. The export direction is `Reading::table_for` returning
`PyTable` (`repo:rust-packages/laterite-py/src/lib.rs`), which is
allocator-symmetric per §2. The guards are
`repo:packages/laterite/python/laterite/__init__.py` and `_to_polars` in
`repo:packages/laterite/python/laterite/compat/_impl.py`. See [[ags4-output]] for
the emit feature and [[pyo3-boundary]] for the boundary itself.

## Related

[[pyo3-boundary]] · [[ags4-output]] · [[crate-map]] · [[laterite-py]] · [[laterite]] · [[abi3-perf]]
