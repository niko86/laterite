# Support modules

The supporting surfaces: the dictionary registry, the AGS type system, and the
transport (pack / encrypt) helpers. See [Concepts](../concepts/born-typed.md) for the
ideas behind them.

## `laterite.registry`

The AGS group graph projected from the single-source dictionary, covering group
descriptors, the KEY chain, and parent/child links.

::: laterite.registry
options:
show_root_heading: false
members_order: source

## `laterite.ags_types`

The AGS type system: canonical types and value casting.

::: laterite.ags_types
options:
show_root_heading: false
members_order: source

## `laterite.transport`

Pack / unpack and lock / unlock for content-agnostic transport (zstd + optional
age encryption).

::: laterite.transport
options:
show_root_heading: false
members_order: source
