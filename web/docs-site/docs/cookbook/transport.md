# Pack / encrypt for transport

Compress an AGS4 file for storage or transfer, then restore it byte-for-byte —
no schema, no re-emit, just zstd.

```python
--8<-- "python/ex12_transport.py"
```

```text
original:   7007 bytes
compressed: 1743 bytes
round-trip byte-identical: True
```

`transport.pack` zstd-compresses the file as-is and writes a `.zst` alongside it;
`transport.unpack` reverses it. AGS4 is line-oriented, quote-heavy text, so it
squeezes hard — roughly **4x** here (7007 → 1743 bytes), and more on real
deliveries with repeated headings. The round-trip is **byte-identical**: `pack`
moves bytes, it doesn't parse or normalise them, so a packed-then-unpacked file
matches the original down to its line endings and trailing whitespace. That makes
it safe to pack a file you intend to validate later — nothing about it changes.

Both calls take a `dest=` to control where the output lands; omit it and the
output sits next to the source with the `.zst` suffix added (`pack`) or removed
(`unpack`).

## Add encryption

`transport.lock` / `transport.unlock` are the same round-trip with
[age](https://age-encryption.org) encryption layered on top of the zstd pack —
reach for them when the file leaves your control and the contents are sensitive:

```python
from laterite.transport import lock, unlock

locked = lock("delivery.ags", dest="delivery.ags.age", recipients=[pubkey])
unlock(locked, dest="delivery.ags", identity=privkey)
```

`lock` compresses then encrypts to the given recipient public key(s); `unlock`
decrypts with the matching identity and restores the original bytes. Without the
identity the payload is opaque, so this is the door to use for handing a file off
over an untrusted channel.

See also: [Cheatsheet](../reference/cheatsheet.md) · [Certify a file](./certify.md)
