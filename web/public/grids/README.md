# OSTN15 NTv2 grid — bundled coordinate-transformation data

`OSTN15_NTv2_OSGBtoETRS.gsb` is the official Ordnance Survey **OSTN15** NTv2
grid-shift file. The Coordinate tool registers it with proj4
(`+nadgrids=OSTN15`) to convert British National Grid (EPSG:27700) eastings /
northings to WGS84 / ETRS89 at **sub-metre** accuracy — the rigorous OS
transform, not the ~5 m Helmert approximation. This matters when the result is
*exported for use* (e.g. GeoJSON consumed in a GIS), where a 5 m error becomes
baked-in data.

It is **committed to the repo** (not fetched at build time) deliberately: the
OS download URL has changed before, and a build-time fetch + unzip is a fragile
external dependency for every deploy. The file is static (last revised by OS in
2016) so it never needs refreshing.

## Provenance

| | |
|---|---|
| **File** | `OSTN15_NTv2_OSGBtoETRS.gsb` |
| **Size** | 15,240,384 bytes (≈ 14.5 MiB) |
| **`.gsb` SHA-256** | `bcb9c6b3b2760e2740fd80e2182fd2eec4e79038165bac703a9415ed1813dee2` |
| **Source** | `OSTN15-NTv2.zip` from <https://www.ordnancesurvey.co.uk/geodesy-positioning/coordinate-transformations/resources> |
| **Zip SHA-256** | `e2f53239edc399f79b256b198818aada0d268ebb509a37b3e6024a5ec292dc2e` |
| **OS revision** | OSTN15 NTv2, October 2016 (v1.0 user guide) |

Verified against all 40 of OS's published test vectors
(`OSTN15_TestInput/Output_OSGBtoETRS.txt`): **max residual 7.8 mm** through the
proj4 `+nadgrids` path.

## Licence & attribution

The OSTN15 transformation (in any format) is licensed under the **OSI BSD
Licence** (per the OS *Transformation data format and user guide*, October
2016). Redistribution and bundling are permitted provided the OS copyright
notice travels with the software:

> Contains OS data © Crown copyright and database rights, Ordnance Survey
> Limited 2016. OSTN15 transformation licensed under the OSI BSD Licence.

That notice is also shown in-app whenever OSTN15 is active and embedded in the
GeoJSON export metadata (`web/src/lib/coords.ts` → `OS_ATTRIBUTION`).

## Refreshing (should it ever be needed)

```sh
curl -L -o OSTN15-NTv2.zip \
  https://www.ordnancesurvey.co.uk/documents/resources/OSTN15-NTv2.zip
unzip -j OSTN15-NTv2.zip OSTN15_NTv2_OSGBtoETRS.gsb -d web/public/grids/
```
