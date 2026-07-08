# Pack / encrypt for transport

**Available in:** Python · Node · CLI · Browser

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
    --8<-- "python/ex17_lock.py"
    ```

    ```text
    sealed: site.ags.zst.age
    round-trip byte-identical: True
    ```

    The KDF is scrypt at age's standard `log_N` 18 tier — deliberately
    expensive (~256 MiB), so a stolen envelope resists brute force. Pass
    `log_n=` to tune it, `level=` for the zstd ratio, and omit `dest=` to write
    `<src>.zst.age` alongside the source.

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
    --8<-- "node/ex17_lock.mjs"
    ```

    ```text
    round-trip byte-identical: true
    ```

    `transport.lock(src, dest, password)` seals the same **standard** zstd+age
    envelope Python and the browser read; the positional `password` (with
    optional `level` / `logN` after it) is the only shape difference from
    Python's keyword args. A file sealed here opens with `transport.unlock`
    anywhere, or with stock `age` given the passphrase.

=== "CLI"

    ```bash
    --8<-- "cli/transport.sh:cmd"
    ```

    ```text
    --8<-- "cli/transport.out"
    ```

    `lat pack` / `unpack` are the shell face of the same zstd round-trip (the
    summary — ratio + timing — prints to stderr; here the round-trip is proved by
    reading a group back out of the restored file). `lat lock` / `unlock` add the
    age passphrase envelope. The passphrase is **never** a flag — argv leaks into
    `ps` and shell history; the precedence is `--password-file <path>` →
    `$LAT_TRANSPORT_PASSWORD` → an interactive prompt. `--level` tunes the zstd
    ratio and `--log-n` the scrypt tier (default 18). A file sealed here opens
    with `transport.unlock` in Python/Node — or stock `age` — anywhere.

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
