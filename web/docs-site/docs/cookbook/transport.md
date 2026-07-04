# Pack / encrypt for transport

**Available in:** Python · Node · Browser

Compress an AGS4 file for storage or transfer, then restore it byte-for-byte —
no schema, no re-emit, just zstd (with optional age encryption on top).

=== "Python"

    ```python
    --8<-- "python/ex12_transport.py"
    ```

    ```text
    original:   7007 bytes
    compressed: 1743 bytes
    round-trip byte-identical: True
    ```

    `transport.pack` zstd-compresses the file as-is and writes a `.zst`
    alongside it; `transport.unpack` reverses it. AGS4 is line-oriented,
    quote-heavy text, so it squeezes hard — roughly **4x** here (7007 → 1743
    bytes), and more on real deliveries with repeated headings. The round-trip
    is **byte-identical**: `pack` moves bytes, it doesn't parse or normalise
    them, so a packed-then-unpacked file matches the original down to its line
    endings and trailing whitespace. That makes it safe to pack a file you
    intend to validate later — nothing about it changes.

    Both calls take a `dest=` to control where the output lands; omit it and
    the output sits next to the source with the `.zst` suffix added (`pack`)
    or removed (`unpack`).

    **Add encryption** — `lock` / `unlock` are the same round-trip with a
    passphrase-encrypted [age](https://age-encryption.org) envelope on top of
    the zstd pack; without the passphrase the payload is opaque:

    ```python
    from laterite.transport import lock, unlock

    locked = lock("delivery.ags", password="correct horse battery staple")
    unlock(locked, password="correct horse battery staple", dest="delivery.ags")
    ```

=== "Node"

    ```js
    --8<-- "node/ex12_transport.mjs"
    ```

    ```text
    original:   7007 bytes
    compressed: 1743 bytes
    round-trip byte-identical: true
    ```

    The `transport` namespace mirrors Python call-for-call —
    `transport.pack(src, dest)` / `transport.unpack(src, dest)` — and the
    outputs are the same standard zstd frames, so a file packed in Node
    unpacks in Python (or with plain `zstd -d`) and vice versa.

    **Add encryption** — same envelope, passphrase-based:

    ```js
    transport.lock("delivery.ags", "delivery.ags.zst.age", "correct horse battery staple");
    transport.unlock("delivery.ags.zst.age", "delivery.ags", "correct horse battery staple");
    ```

=== "Browser"

    The [web app](../surfaces/browser.md)'s **Tools** pane has the same
    lock/unlock round-trip running client-side: drop a file, enter a
    passphrase, download the sealed `.zst.age` (or unseal one you received).
    The output is byte-compatible with the Python/Node `lock` — a file sealed
    in the browser opens with `transport.unlock` anywhere, and vice versa —
    and nothing ever leaves your machine.

The sealed format is standard **zstd inside a standard age envelope** — not a
laterite-only container — so recipients without laterite can still recover the
file with stock `age` + `zstd` tooling given the passphrase.

See also: [Cheatsheet](../reference/cheatsheet.md) · [Certify a file](./certify.md)
