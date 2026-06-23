# AUTO-GENERATED from rust-packages/laterite-ags4-core/data/ags_dictionary.json
# DO NOT EDIT BY HAND. Regenerate via:
#   uv run python tools/generate_pyi.py
#
# Type-stub file for the compiled `laterite._laterite_native`
# extension. IDEs and type-checkers consult this to type-check
# code that imports the standard AGS4 typed-graph classes
# (`from laterite import PROJ, LOCA, ...`). The module's internal
# functions (run_check / fix_file / list_rules / parse_* / the
# excel + transport helpers / Sidecar) are reached through the
# typed Python wrappers in `laterite/__init__.py`, which carry the
# annotations, so they are not stubbed here.
#
# Custom / passthrough groups built at runtime via
# `laterite.dynamic.get_or_register` are NOT typed in this stub —
# they show as `Any` to type checkers (acceptable; their schema
# isn't known until a file is read).

from __future__ import annotations

import datetime as _dt
from typing import Any

class AAVT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    aavt_aav: float | None
    aavt_rem: str | None
    aavt_meth: str | None
    aavt_lab: str | None
    aavt_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    aavt_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        aavt_aav: float | None = ...,
        aavt_rem: str | None = ...,
        aavt_meth: str | None = ...,
        aavt_lab: str | None = ...,
        aavt_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        aavt_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ABBR:
    abbr_hdng: str | None
    abbr_code: str | None
    abbr_desc: str | None
    abbr_list: str | None
    abbr_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        abbr_hdng: str | None = ...,
        abbr_code: str | None = ...,
        abbr_desc: str | None = ...,
        abbr_list: str | None = ...,
        abbr_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ACVT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    acvt_acv: int | None
    acvt_frac: str | None
    acvt_rem: str | None
    acvt_meth: str | None
    acvt_lab: str | None
    acvt_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    acvt_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        acvt_acv: int | None = ...,
        acvt_frac: str | None = ...,
        acvt_rem: str | None = ...,
        acvt_meth: str | None = ...,
        acvt_lab: str | None = ...,
        acvt_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        acvt_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class AELO:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    aelo_ei: int | None
    aelo_rem: str | None
    aelo_meth: str | None
    aelo_lab: str | None
    aelo_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    aelo_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        aelo_ei: int | None = ...,
        aelo_rem: str | None = ...,
        aelo_meth: str | None = ...,
        aelo_lab: str | None = ...,
        aelo_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        aelo_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class AFLK:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    aflk_fi: int | None
    aflk_mass: float | None
    aflk_rem: str | None
    aflk_meth: str | None
    aflk_lab: str | None
    aflk_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    aflk_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        aflk_fi: int | None = ...,
        aflk_mass: float | None = ...,
        aflk_rem: str | None = ...,
        aflk_meth: str | None = ...,
        aflk_lab: str | None = ...,
        aflk_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        aflk_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class AIVT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    aivt_aiv1: float | None
    aivt_aiv2: float | None
    aivt_aiv: float | None
    aivt_frac: str | None
    aivt_pden: float | None
    aivt_rem: str | None
    aivt_meth: str | None
    aivt_lab: str | None
    aivt_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    aivt_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        aivt_aiv1: float | None = ...,
        aivt_aiv2: float | None = ...,
        aivt_aiv: float | None = ...,
        aivt_frac: str | None = ...,
        aivt_pden: float | None = ...,
        aivt_rem: str | None = ...,
        aivt_meth: str | None = ...,
        aivt_lab: str | None = ...,
        aivt_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        aivt_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ALOS:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    alos_losa: int | None
    alos_lopw: int | None
    alos_lowr: int | None
    alos_frac: str | None
    alos_char: str | None
    alos_rem: str | None
    alos_meth: str | None
    alos_lab: str | None
    alos_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    alos_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        alos_losa: int | None = ...,
        alos_lopw: int | None = ...,
        alos_lowr: int | None = ...,
        alos_frac: str | None = ...,
        alos_char: str | None = ...,
        alos_rem: str | None = ...,
        alos_meth: str | None = ...,
        alos_lab: str | None = ...,
        alos_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        alos_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class APSV:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    apsv_aav: int | None
    apsv_rem: str | None
    apsv_meth: str | None
    apsv_lab: str | None
    apsv_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    apsv_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        apsv_aav: int | None = ...,
        apsv_rem: str | None = ...,
        apsv_meth: str | None = ...,
        apsv_lab: str | None = ...,
        apsv_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        apsv_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ARTW:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    artw_frac: str | None
    artw_type: str | None
    artw_md1: float | None
    artw_md2: float | None
    artw_mde: int | None
    artw_mds: int | None
    artw_date: _dt.datetime | None
    artw_rem: str | None
    artw_meth: str | None
    artw_lab: str | None
    artw_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    artw_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        artw_frac: str | None = ...,
        artw_type: str | None = ...,
        artw_md1: float | None = ...,
        artw_md2: float | None = ...,
        artw_mde: int | None = ...,
        artw_mds: int | None = ...,
        artw_date: _dt.datetime | None = ...,
        artw_rem: str | None = ...,
        artw_meth: str | None = ...,
        artw_lab: str | None = ...,
        artw_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        artw_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ASDI:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    asdi_sdi1: float | None
    asdi_sdi2: float | None
    asdi_soln: str | None
    asdi_indr: str | None
    asdi_padr: str | None
    asdi_rem: str | None
    asdi_meth: str | None
    asdi_lab: str | None
    asdi_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    asdi_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        asdi_sdi1: float | None = ...,
        asdi_sdi2: float | None = ...,
        asdi_soln: str | None = ...,
        asdi_indr: str | None = ...,
        asdi_padr: str | None = ...,
        asdi_rem: str | None = ...,
        asdi_meth: str | None = ...,
        asdi_lab: str | None = ...,
        asdi_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        asdi_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ASNS:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    asns_soun: int | None
    asns_frac: str | None
    asns_rem: str | None
    asns_meth: str | None
    asns_lab: str | None
    asns_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    asns_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        asns_soun: int | None = ...,
        asns_frac: str | None = ...,
        asns_rem: str | None = ...,
        asns_meth: str | None = ...,
        asns_lab: str | None = ...,
        asns_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        asns_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class AWAD:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    awad_wtab: float | None
    awad_rem: str | None
    awad_meth: str | None
    awad_lab: str | None
    awad_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    awad_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        awad_wtab: float | None = ...,
        awad_rem: str | None = ...,
        awad_meth: str | None = ...,
        awad_lab: str | None = ...,
        awad_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        awad_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class BKFL:
    loca_id: str | None
    bkfl_top: float | None
    bkfl_base: float | None
    bkfl_desc: str | None
    bkfl_leg: str | None
    bkfl_date: _dt.datetime | None
    bkfl_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        bkfl_top: float | None = ...,
        bkfl_base: float | None = ...,
        bkfl_desc: str | None = ...,
        bkfl_leg: str | None = ...,
        bkfl_date: _dt.datetime | None = ...,
        bkfl_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CBRG:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    cbrg_cond: str | None
    cbrg_nmc: str | None
    cbrg_200: int | None
    cbrg_stab: float | None
    cbrg_styp: str | None
    cbrg_rem: str | None
    cbrg_meth: str | None
    cbrg_lab: str | None
    cbrg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    cbrg_dev: str | None
    cbrts: list[CBRT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        cbrg_cond: str | None = ...,
        cbrg_nmc: str | None = ...,
        cbrg_200: int | None = ...,
        cbrg_stab: float | None = ...,
        cbrg_styp: str | None = ...,
        cbrg_rem: str | None = ...,
        cbrg_meth: str | None = ...,
        cbrg_lab: str | None = ...,
        cbrg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        cbrg_dev: str | None = ...,
        cbrts: list[CBRT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CBRP:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    cbrt_tesn: str | None
    cbrp_end: str | None
    cbrp_pen: float | None
    cbrp_load: float | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        cbrt_tesn: str | None = ...,
        cbrp_end: str | None = ...,
        cbrp_pen: float | None = ...,
        cbrp_load: float | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CBRT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    cbrt_tesn: str | None
    cbrt_top: str | None
    cbrt_base: str | None
    cbrt_mct: str | None
    cbrt_mcbt: str | None
    cbrt_imc: str | None
    cbrt_bden: float | None
    cbrt_dden: float | None
    cbrt_surc: int | None
    cbrt_skdt: str | None
    cbrt_swel: float | None
    cbrt_rem: str | None
    file_fset: str | None
    cbrps: list[CBRP]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        cbrt_tesn: str | None = ...,
        cbrt_top: str | None = ...,
        cbrt_base: str | None = ...,
        cbrt_mct: str | None = ...,
        cbrt_mcbt: str | None = ...,
        cbrt_imc: str | None = ...,
        cbrt_bden: float | None = ...,
        cbrt_dden: float | None = ...,
        cbrt_surc: int | None = ...,
        cbrt_skdt: str | None = ...,
        cbrt_swel: float | None = ...,
        cbrt_rem: str | None = ...,
        file_fset: str | None = ...,
        cbrps: list[CBRP] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CDIA:
    loca_id: str | None
    cdia_dpth: float | None
    cdia_diam: int | None
    cdia_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        cdia_dpth: float | None = ...,
        cdia_diam: int | None = ...,
        cdia_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CHIS:
    loca_id: str | None
    chis_from: float | None
    chis_to: float | None
    chis_time: str | None
    chis_star: _dt.datetime | None
    chis_tool: str | None
    chis_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        chis_from: float | None = ...,
        chis_to: float | None = ...,
        chis_time: str | None = ...,
        chis_star: _dt.datetime | None = ...,
        chis_tool: str | None = ...,
        chis_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CHOC:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    choc_ref: str | None
    choc_from: str | None
    choc_to: str | None
    choc_ddis: _dt.datetime | None
    choc_btch: str | None
    choc_rem: str | None
    choc_cont: int | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        choc_ref: str | None = ...,
        choc_from: str | None = ...,
        choc_to: str | None = ...,
        choc_ddis: _dt.datetime | None = ...,
        choc_btch: str | None = ...,
        choc_rem: str | None = ...,
        choc_cont: int | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CMPG:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    cmpg_tesn: str | None
    spec_prep: str | None
    spec_desc: str | None
    cmpg_type: str | None
    cmpg_mold: str | None
    cmpg_375: int | None
    cmpg_200: int | None
    cmpg_pden: str | None
    cmpg_maxd: float | None
    cmpg_mcop: float | None
    cmpg_stab: float | None
    cmpg_styp: str | None
    cmpg_rem: str | None
    cmpg_meth: str | None
    cmpg_lab: str | None
    cmpg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    cmpg_dev: str | None
    cmpg_zone: str | None
    cmpts: list[CMPT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        cmpg_tesn: str | None = ...,
        spec_prep: str | None = ...,
        spec_desc: str | None = ...,
        cmpg_type: str | None = ...,
        cmpg_mold: str | None = ...,
        cmpg_375: int | None = ...,
        cmpg_200: int | None = ...,
        cmpg_pden: str | None = ...,
        cmpg_maxd: float | None = ...,
        cmpg_mcop: float | None = ...,
        cmpg_stab: float | None = ...,
        cmpg_styp: str | None = ...,
        cmpg_rem: str | None = ...,
        cmpg_meth: str | None = ...,
        cmpg_lab: str | None = ...,
        cmpg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        cmpg_dev: str | None = ...,
        cmpg_zone: str | None = ...,
        cmpts: list[CMPT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CMPT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    cmpg_tesn: str | None
    cmpt_tesn: str | None
    cmpt_mc: str | None
    cmpt_dden: float | None
    cmpt_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        cmpg_tesn: str | None = ...,
        cmpt_tesn: str | None = ...,
        cmpt_mc: str | None = ...,
        cmpt_dden: float | None = ...,
        cmpt_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CONG:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    cong_type: str | None
    cong_cond: str | None
    cong_sdia: float | None
    cong_higt: float | None
    cong_mci: str | None
    cong_mcf: str | None
    cong_bden: float | None
    cong_dden: float | None
    cong_pden: str | None
    cong_satr: int | None
    cong_sprs: float | None
    cong_sath: float | None
    cong_ivr: float | None
    cong_rem: str | None
    cong_meth: str | None
    cong_lab: str | None
    cong_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    cong_dev: str | None
    cong_mcis: str | None
    cong_corr: bool | None
    conss: list[CONS]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        cong_type: str | None = ...,
        cong_cond: str | None = ...,
        cong_sdia: float | None = ...,
        cong_higt: float | None = ...,
        cong_mci: str | None = ...,
        cong_mcf: str | None = ...,
        cong_bden: float | None = ...,
        cong_dden: float | None = ...,
        cong_pden: str | None = ...,
        cong_satr: int | None = ...,
        cong_sprs: float | None = ...,
        cong_sath: float | None = ...,
        cong_ivr: float | None = ...,
        cong_rem: str | None = ...,
        cong_meth: str | None = ...,
        cong_lab: str | None = ...,
        cong_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        cong_dev: str | None = ...,
        cong_mcis: str | None = ...,
        cong_corr: bool | None = ...,
        conss: list[CONS] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CONS:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    cons_incn: str | None
    cons_ivr: float | None
    cons_incf: int | None
    cons_ince: float | None
    cons_inmv: float | None
    cons_insc: float | None
    cons_cvrt: float | None
    cons_cvlg: float | None
    cons_temp: float | None
    cons_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        cons_incn: str | None = ...,
        cons_ivr: float | None = ...,
        cons_incf: int | None = ...,
        cons_ince: float | None = ...,
        cons_inmv: float | None = ...,
        cons_insc: float | None = ...,
        cons_cvrt: float | None = ...,
        cons_cvlg: float | None = ...,
        cons_temp: float | None = ...,
        cons_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CORE:
    loca_id: str | None
    core_top: float | None
    core_base: float | None
    core_prec: int | None
    core_srec: int | None
    core_rqd: int | None
    core_diam: int | None
    core_durn: str | None
    core_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        core_top: float | None = ...,
        core_base: float | None = ...,
        core_prec: int | None = ...,
        core_srec: int | None = ...,
        core_rqd: int | None = ...,
        core_diam: int | None = ...,
        core_durn: str | None = ...,
        core_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CPDG:
    loca_id: str | None
    cptg_tesn: str | None
    cpdg_dpth: float | None
    cpdg_ir: float | None
    cpdg_rcmp: bool | None
    cpdg_ui: float | None
    cpdg_uip: str | None
    cpdg_m: float | None
    cpdg_ueq: float | None
    cpdg_uep: str | None
    cpdg_ddis: int | None
    cpdg_t: float | None
    cpdg_ch: float | None
    cpdg_chmt: str | None
    cpdg_cv: float | None
    cpdg_cvmt: str | None
    cpdg_rem: str | None
    cpdg_date: _dt.datetime | None
    cpdg_oper: str | None
    cpdg_anby: str | None
    test_stat: str | None
    file_fset: str | None
    cpdts: list[CPDT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        cptg_tesn: str | None = ...,
        cpdg_dpth: float | None = ...,
        cpdg_ir: float | None = ...,
        cpdg_rcmp: bool | None = ...,
        cpdg_ui: float | None = ...,
        cpdg_uip: str | None = ...,
        cpdg_m: float | None = ...,
        cpdg_ueq: float | None = ...,
        cpdg_uep: str | None = ...,
        cpdg_ddis: int | None = ...,
        cpdg_t: float | None = ...,
        cpdg_ch: float | None = ...,
        cpdg_chmt: str | None = ...,
        cpdg_cv: float | None = ...,
        cpdg_cvmt: str | None = ...,
        cpdg_rem: str | None = ...,
        cpdg_date: _dt.datetime | None = ...,
        cpdg_oper: str | None = ...,
        cpdg_anby: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        cpdts: list[CPDT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CPDT:
    loca_id: str | None
    cptg_tesn: str | None
    cpdg_dpth: float | None
    cpdt_time: float | None
    cpdt_qc: float | None
    cpdt_tf: float | None
    cpdt_fs: float | None
    cpdt_u1: float | None
    cpdt_u2: float | None
    cpdt_u3: float | None
    cpdt_tmpi: float | None
    cpdt_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        cptg_tesn: str | None = ...,
        cpdg_dpth: float | None = ...,
        cpdt_time: float | None = ...,
        cpdt_qc: float | None = ...,
        cpdt_tf: float | None = ...,
        cpdt_fs: float | None = ...,
        cpdt_u1: float | None = ...,
        cpdt_u2: float | None = ...,
        cpdt_u3: float | None = ...,
        cpdt_tmpi: float | None = ...,
        cpdt_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CPTG:
    loca_id: str | None
    cptg_tesn: str | None
    cptg_type: str | None
    cptg_date: _dt.datetime | None
    cptg_ped: float | None
    cptg_rate: int | None
    cptg_ornt: int | None
    cptg_rloc: str | None
    cptg_wat: float | None
    cptg_wata: str | None
    cptg_term: str | None
    cptg_ref: str | None
    cptg_man: str | None
    cptg_fill: str | None
    cptg_csa: float | None
    cptg_csan: int | None
    cptg_car: float | None
    cptg_sla: float | None
    cptg_slan: int | None
    cptg_sha: int | None
    cptg_slar: float | None
    cptg_cfos: int | None
    cptg_cfoa: int | None
    cptg_tbl: float | None
    cptg_tbd: float | None
    cptg_cpc: float | None
    cptg_fpc: float | None
    cptg_upc: float | None
    cptg_cpcl: str | None
    cptg_crdt: _dt.datetime | None
    cptg_cddt: _dt.datetime | None
    cptg_lca: str | None
    cptg_filt: str | None
    cptg_fric: bool | None
    cptg_frid: int | None
    cptg_fris: int | None
    cptg_sat: str | None
    cptg_eqpt: str | None
    cptg_apcl: str | None
    cptg_dazv: str | None
    cptg_corr: str | None
    cptg_rem: str | None
    cptg_oper: str | None
    cptg_anby: str | None
    cptg_env: str | None
    cptg_meth: str | None
    cptg_dev: str | None
    cptg_cont: str | None
    cptg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    cpdgs: list[CPDG]
    cptts: list[CPTT]
    cptys: list[CPTY]
    cptzs: list[CPTZ]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        cptg_tesn: str | None = ...,
        cptg_type: str | None = ...,
        cptg_date: _dt.datetime | None = ...,
        cptg_ped: float | None = ...,
        cptg_rate: int | None = ...,
        cptg_ornt: int | None = ...,
        cptg_rloc: str | None = ...,
        cptg_wat: float | None = ...,
        cptg_wata: str | None = ...,
        cptg_term: str | None = ...,
        cptg_ref: str | None = ...,
        cptg_man: str | None = ...,
        cptg_fill: str | None = ...,
        cptg_csa: float | None = ...,
        cptg_csan: int | None = ...,
        cptg_car: float | None = ...,
        cptg_sla: float | None = ...,
        cptg_slan: int | None = ...,
        cptg_sha: int | None = ...,
        cptg_slar: float | None = ...,
        cptg_cfos: int | None = ...,
        cptg_cfoa: int | None = ...,
        cptg_tbl: float | None = ...,
        cptg_tbd: float | None = ...,
        cptg_cpc: float | None = ...,
        cptg_fpc: float | None = ...,
        cptg_upc: float | None = ...,
        cptg_cpcl: str | None = ...,
        cptg_crdt: _dt.datetime | None = ...,
        cptg_cddt: _dt.datetime | None = ...,
        cptg_lca: str | None = ...,
        cptg_filt: str | None = ...,
        cptg_fric: bool | None = ...,
        cptg_frid: int | None = ...,
        cptg_fris: int | None = ...,
        cptg_sat: str | None = ...,
        cptg_eqpt: str | None = ...,
        cptg_apcl: str | None = ...,
        cptg_dazv: str | None = ...,
        cptg_corr: str | None = ...,
        cptg_rem: str | None = ...,
        cptg_oper: str | None = ...,
        cptg_anby: str | None = ...,
        cptg_env: str | None = ...,
        cptg_meth: str | None = ...,
        cptg_dev: str | None = ...,
        cptg_cont: str | None = ...,
        cptg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        cpdgs: list[CPDG] | None = ...,
        cptts: list[CPTT] | None = ...,
        cptys: list[CPTY] | None = ...,
        cptzs: list[CPTZ] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CPTM:
    loca_id: str | None
    cptm_dpth: float | None
    cptm_base: float | None
    cptm_sbt1: str | None
    cptm_su1: str | None
    cptm_su2: str | None
    cptm_dr1: str | None
    cptm_dr2: str | None
    cptm_phi1: str | None
    cptm_ic1: str | None
    cptm_n601: str | None
    cptm_e1: str | None
    cptm_mv1: str | None
    cptm_g01: str | None
    cptm_vs1: str | None
    cptm_duw1: str | None
    cptm_suw1: str | None
    cptm_m1: str | None
    cptm_cc1: str | None
    cptm_p01: str | None
    cptm_st1: str | None
    cptm_k01: str | None
    cptm_ir1: str | None
    cptm_k1: str | None
    cptm_fc1: str | None
    cptm_csr1: str | None
    cptm_crr1: str | None
    cptm_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        cptm_dpth: float | None = ...,
        cptm_base: float | None = ...,
        cptm_sbt1: str | None = ...,
        cptm_su1: str | None = ...,
        cptm_su2: str | None = ...,
        cptm_dr1: str | None = ...,
        cptm_dr2: str | None = ...,
        cptm_phi1: str | None = ...,
        cptm_ic1: str | None = ...,
        cptm_n601: str | None = ...,
        cptm_e1: str | None = ...,
        cptm_mv1: str | None = ...,
        cptm_g01: str | None = ...,
        cptm_vs1: str | None = ...,
        cptm_duw1: str | None = ...,
        cptm_suw1: str | None = ...,
        cptm_m1: str | None = ...,
        cptm_cc1: str | None = ...,
        cptm_p01: str | None = ...,
        cptm_st1: str | None = ...,
        cptm_k01: str | None = ...,
        cptm_ir1: str | None = ...,
        cptm_k1: str | None = ...,
        cptm_fc1: str | None = ...,
        cptm_csr1: str | None = ...,
        cptm_crr1: str | None = ...,
        cptm_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CPTP:
    loca_id: str | None
    cptp_dpth: float | None
    cptp_base: float | None
    cptp_sbt1: str | None
    cptp_su1: float | None
    cptp_su2: float | None
    cptp_dr1: float | None
    cptp_dr2: float | None
    cptp_phi1: float | None
    cptp_ic1: float | None
    cptp_n601: int | None
    cptp_e1: float | None
    cptp_mv1: float | None
    cptp_g01: float | None
    cptp_vs1: float | None
    cptp_duw1: float | None
    cptp_suw1: float | None
    cptp_m1: float | None
    cptp_cc1: float | None
    cptp_p01: float | None
    cptp_st1: float | None
    cptp_k01: float | None
    cptp_ir1: float | None
    cptp_k1: float | None
    cptp_fc1: float | None
    cptp_csr1: float | None
    cptp_crr1: float | None
    cptp_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        cptp_dpth: float | None = ...,
        cptp_base: float | None = ...,
        cptp_sbt1: str | None = ...,
        cptp_su1: float | None = ...,
        cptp_su2: float | None = ...,
        cptp_dr1: float | None = ...,
        cptp_dr2: float | None = ...,
        cptp_phi1: float | None = ...,
        cptp_ic1: float | None = ...,
        cptp_n601: int | None = ...,
        cptp_e1: float | None = ...,
        cptp_mv1: float | None = ...,
        cptp_g01: float | None = ...,
        cptp_vs1: float | None = ...,
        cptp_duw1: float | None = ...,
        cptp_suw1: float | None = ...,
        cptp_m1: float | None = ...,
        cptp_cc1: float | None = ...,
        cptp_p01: float | None = ...,
        cptp_st1: float | None = ...,
        cptp_k01: float | None = ...,
        cptp_ir1: float | None = ...,
        cptp_k1: float | None = ...,
        cptp_fc1: float | None = ...,
        cptp_csr1: float | None = ...,
        cptp_crr1: float | None = ...,
        cptp_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CPTT:
    loca_id: str | None
    cptg_tesn: str | None
    cptt_redn: int | None
    cptt_dpth: float | None
    cptt_plen: str | None
    cptt_qc: float | None
    cptt_fs: float | None
    cptt_u1: float | None
    cptt_u2: float | None
    cptt_u3: float | None
    cptt_incx: float | None
    cptt_incy: float | None
    cptt_time: _dt.datetime | None
    cptt_dur: float | None
    cptt_tf: float | None
    cptt_rf: float | None
    cptt_bden: float | None
    cptt_cpo: float | None
    cptt_ispp: float | None
    cptt_cpod: float | None
    cptt_qt: float | None
    cptt_ft: float | None
    cptt_qnet: float | None
    cptt_qe: float | None
    cptt_rft: float | None
    cptt_expp: float | None
    cptt_bq: float | None
    cptt_nqt: float | None
    cptt_nfr: float | None
    cptt_magx: int | None
    cptt_magy: int | None
    cptt_magz: int | None
    cptt_magt: float | None
    cptt_magg: float | None
    cptt_con: int | None
    cptt_temp: float | None
    cptt_tpqc: float | None
    cptt_tpfs: float | None
    cptt_tpu: float | None
    cptt_ph: float | None
    cptt_redx: float | None
    cptt_smp: float | None
    cptt_ngam: float | None
    cptt_ffd1: int | None
    cptt_ffd2: int | None
    cptt_pid: int | None
    cptt_fid: int | None
    cptt_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        cptg_tesn: str | None = ...,
        cptt_redn: int | None = ...,
        cptt_dpth: float | None = ...,
        cptt_plen: str | None = ...,
        cptt_qc: float | None = ...,
        cptt_fs: float | None = ...,
        cptt_u1: float | None = ...,
        cptt_u2: float | None = ...,
        cptt_u3: float | None = ...,
        cptt_incx: float | None = ...,
        cptt_incy: float | None = ...,
        cptt_time: _dt.datetime | None = ...,
        cptt_dur: float | None = ...,
        cptt_tf: float | None = ...,
        cptt_rf: float | None = ...,
        cptt_bden: float | None = ...,
        cptt_cpo: float | None = ...,
        cptt_ispp: float | None = ...,
        cptt_cpod: float | None = ...,
        cptt_qt: float | None = ...,
        cptt_ft: float | None = ...,
        cptt_qnet: float | None = ...,
        cptt_qe: float | None = ...,
        cptt_rft: float | None = ...,
        cptt_expp: float | None = ...,
        cptt_bq: float | None = ...,
        cptt_nqt: float | None = ...,
        cptt_nfr: float | None = ...,
        cptt_magx: int | None = ...,
        cptt_magy: int | None = ...,
        cptt_magz: int | None = ...,
        cptt_magt: float | None = ...,
        cptt_magg: float | None = ...,
        cptt_con: int | None = ...,
        cptt_temp: float | None = ...,
        cptt_tpqc: float | None = ...,
        cptt_tpfs: float | None = ...,
        cptt_tpu: float | None = ...,
        cptt_ph: float | None = ...,
        cptt_redx: float | None = ...,
        cptt_smp: float | None = ...,
        cptt_ngam: float | None = ...,
        cptt_ffd1: int | None = ...,
        cptt_ffd2: int | None = ...,
        cptt_pid: int | None = ...,
        cptt_fid: int | None = ...,
        cptt_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CPTY:
    loca_id: str | None
    cptg_tesn: str | None
    cpty_tesn: str | None
    cpty_dpth: float | None
    cpty_dint: float | None
    cpty_numc: int | None
    cpty_redi: int | None
    cpty_redf: int | None
    cpty_timi: float | None
    cpty_timf: float | None
    cpty_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        cptg_tesn: str | None = ...,
        cpty_tesn: str | None = ...,
        cpty_dpth: float | None = ...,
        cpty_dint: float | None = ...,
        cpty_numc: int | None = ...,
        cpty_redi: int | None = ...,
        cpty_redf: int | None = ...,
        cpty_timi: float | None = ...,
        cpty_timf: float | None = ...,
        cpty_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CPTZ:
    loca_id: str | None
    cptg_tesn: str | None
    cptz_parm: str | None
    cptz_zbd: str | None
    cptz_zb: str | None
    cptz_za: str | None
    cptz_zad: str | None
    cptz_zac: str | None
    cptz_zd: float | None
    cptz_zdd: float | None
    cptz_zdc: float | None
    cptz_cd: float | None
    cptz_zs: float | None
    cptz_zss: str | None
    cptz_zvuc: str | None
    cptz_egut: str | None
    cptz_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        cptg_tesn: str | None = ...,
        cptz_parm: str | None = ...,
        cptz_zbd: str | None = ...,
        cptz_zb: str | None = ...,
        cptz_za: str | None = ...,
        cptz_zad: str | None = ...,
        cptz_zac: str | None = ...,
        cptz_zd: float | None = ...,
        cptz_zdd: float | None = ...,
        cptz_zdc: float | None = ...,
        cptz_cd: float | None = ...,
        cptz_zs: float | None = ...,
        cptz_zss: str | None = ...,
        cptz_zvuc: str | None = ...,
        cptz_egut: str | None = ...,
        cptz_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CTRC:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    ctrc_tesn: str | None
    ctrc_cell: float | None
    ctrc_bpwp: float | None
    ctrc_mpwp: float | None
    ctrc_mpb: float | None
    ctrc_bb: float | None
    ctrc_type: str | None
    ctrc_bacf: float | None
    ctrc_elap: str | None
    ctrc_chgt: float | None
    ctrc_diae: float | None
    ctrc_mce: str | None
    ctrc_bde: float | None
    ctrc_dde: float | None
    ctrc_rde: float | None
    ctrc_ince: float | None
    ctrc_ase: float | None
    ctrc_rse: float | None
    ctrc_sse: float | None
    ctrc_deve: float | None
    ctrc_mnse: float | None
    ctrc_rtoe: float | None
    ctrc_ease: float | None
    ctrc_vlse: float | None
    ctrc_rdse: float | None
    ctrc_b: float | None
    ctrc_bets: str | None
    ctrc_beax: str | None
    ctrc_beds: float | None
    ctrc_mat: float | None
    ctrc_matm: str | None
    ctrc_swv: int | None
    ctrc_smgm: float | None
    ctrc_rem: str | None
    file_fset: str | None
    ctrps: list[CTRP]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        ctrc_tesn: str | None = ...,
        ctrc_cell: float | None = ...,
        ctrc_bpwp: float | None = ...,
        ctrc_mpwp: float | None = ...,
        ctrc_mpb: float | None = ...,
        ctrc_bb: float | None = ...,
        ctrc_type: str | None = ...,
        ctrc_bacf: float | None = ...,
        ctrc_elap: str | None = ...,
        ctrc_chgt: float | None = ...,
        ctrc_diae: float | None = ...,
        ctrc_mce: str | None = ...,
        ctrc_bde: float | None = ...,
        ctrc_dde: float | None = ...,
        ctrc_rde: float | None = ...,
        ctrc_ince: float | None = ...,
        ctrc_ase: float | None = ...,
        ctrc_rse: float | None = ...,
        ctrc_sse: float | None = ...,
        ctrc_deve: float | None = ...,
        ctrc_mnse: float | None = ...,
        ctrc_rtoe: float | None = ...,
        ctrc_ease: float | None = ...,
        ctrc_vlse: float | None = ...,
        ctrc_rdse: float | None = ...,
        ctrc_b: float | None = ...,
        ctrc_bets: str | None = ...,
        ctrc_beax: str | None = ...,
        ctrc_beds: float | None = ...,
        ctrc_mat: float | None = ...,
        ctrc_matm: str | None = ...,
        ctrc_swv: int | None = ...,
        ctrc_smgm: float | None = ...,
        ctrc_rem: str | None = ...,
        file_fset: str | None = ...,
        ctrps: list[CTRP] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CTRD:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    ctrc_tesn: str | None
    ctrp_cyc: int | None
    ctrd_time: _dt.datetime | None
    ctrd_cond: str | None
    ctrd_sdia: float | None
    ctrd_high: float | None
    ctrd_cell: float | None
    ctrd_bpwp: float | None
    ctrd_mpwp: float | None
    ctrd_eas: float | None
    ctrd_las1: float | None
    ctrd_las2: float | None
    ctrd_vol: float | None
    ctrd_rad: float | None
    ctrd_shsn: float | None
    ctrd_shst: float | None
    ctrd_dev: float | None
    ctrd_psd: float | None
    ctrd_mees: float | None
    ctrd_sece: float | None
    ctrd_tane: float | None
    ctrd_freq: float | None
    ctrd_csts: float | None
    ctrd_acvs: float | None
    ctrd_davs: float | None
    ctrd_cesr: float | None
    ctrd_empr: float | None
    ctrd_ebpr: float | None
    ctrd_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        ctrc_tesn: str | None = ...,
        ctrp_cyc: int | None = ...,
        ctrd_time: _dt.datetime | None = ...,
        ctrd_cond: str | None = ...,
        ctrd_sdia: float | None = ...,
        ctrd_high: float | None = ...,
        ctrd_cell: float | None = ...,
        ctrd_bpwp: float | None = ...,
        ctrd_mpwp: float | None = ...,
        ctrd_eas: float | None = ...,
        ctrd_las1: float | None = ...,
        ctrd_las2: float | None = ...,
        ctrd_vol: float | None = ...,
        ctrd_rad: float | None = ...,
        ctrd_shsn: float | None = ...,
        ctrd_shst: float | None = ...,
        ctrd_dev: float | None = ...,
        ctrd_psd: float | None = ...,
        ctrd_mees: float | None = ...,
        ctrd_sece: float | None = ...,
        ctrd_tane: float | None = ...,
        ctrd_freq: float | None = ...,
        ctrd_csts: float | None = ...,
        ctrd_acvs: float | None = ...,
        ctrd_davs: float | None = ...,
        ctrd_cesr: float | None = ...,
        ctrd_empr: float | None = ...,
        ctrd_ebpr: float | None = ...,
        ctrd_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CTRG:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    spec_base: float | None
    ctrg_type: str | None
    ctrg_mci: str | None
    ctrg_mcf: str | None
    ctrg_h2o: str | None
    ctrg_sbp: float | None
    ctrg_satr: int | None
    ctrg_ird: float | None
    ctrg_sdia: float | None
    ctrg_higt: float | None
    ctrg_tmss: float | None
    ctrg_pden: str | None
    ctrg_madd: float | None
    ctrg_midd: float | None
    ctrg_dden: float | None
    ctrg_bden: float | None
    ctrg_ivr: float | None
    ctrg_sat: str | None
    ctrg_durn: float | None
    ctrg_rem: str | None
    ctrg_meth: str | None
    ctrg_dev: str | None
    ctrg_lab: str | None
    ctrg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    ctrcs: list[CTRC]
    ctrss: list[CTRS]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        spec_base: float | None = ...,
        ctrg_type: str | None = ...,
        ctrg_mci: str | None = ...,
        ctrg_mcf: str | None = ...,
        ctrg_h2o: str | None = ...,
        ctrg_sbp: float | None = ...,
        ctrg_satr: int | None = ...,
        ctrg_ird: float | None = ...,
        ctrg_sdia: float | None = ...,
        ctrg_higt: float | None = ...,
        ctrg_tmss: float | None = ...,
        ctrg_pden: str | None = ...,
        ctrg_madd: float | None = ...,
        ctrg_midd: float | None = ...,
        ctrg_dden: float | None = ...,
        ctrg_bden: float | None = ...,
        ctrg_ivr: float | None = ...,
        ctrg_sat: str | None = ...,
        ctrg_durn: float | None = ...,
        ctrg_rem: str | None = ...,
        ctrg_meth: str | None = ...,
        ctrg_dev: str | None = ...,
        ctrg_lab: str | None = ...,
        ctrg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        ctrcs: list[CTRC] | None = ...,
        ctrss: list[CTRS] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CTRP:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    ctrc_tesn: str | None
    ctrp_cyc: int | None
    ctrp_cycf: int | None
    ctrp_pwpm: float | None
    ctrp_mnpp: float | None
    ctrp_mxss: float | None
    ctrp_mnss: float | None
    ctrp_avss: float | None
    ctrp_css: float | None
    ctrp_acvs: float | None
    ctrp_asf: float | None
    ctrp_fpwp: float | None
    ctrp_qmax: float | None
    ctrp_qmin: float | None
    ctrp_mnes: float | None
    ctrp_eamx: float | None
    ctrp_eamn: float | None
    ctrp_fvr: float | None
    ctrp_qemx: float | None
    ctrp_qemn: float | None
    ctrp_esec: float | None
    ctrp_damp: float | None
    ctrp_mode: str | None
    ctrp_dipl: float | None
    ctrp_obp: str | None
    ctrp_rem: str | None
    file_fset: str | None
    ctrds: list[CTRD]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        ctrc_tesn: str | None = ...,
        ctrp_cyc: int | None = ...,
        ctrp_cycf: int | None = ...,
        ctrp_pwpm: float | None = ...,
        ctrp_mnpp: float | None = ...,
        ctrp_mxss: float | None = ...,
        ctrp_mnss: float | None = ...,
        ctrp_avss: float | None = ...,
        ctrp_css: float | None = ...,
        ctrp_acvs: float | None = ...,
        ctrp_asf: float | None = ...,
        ctrp_fpwp: float | None = ...,
        ctrp_qmax: float | None = ...,
        ctrp_qmin: float | None = ...,
        ctrp_mnes: float | None = ...,
        ctrp_eamx: float | None = ...,
        ctrp_eamn: float | None = ...,
        ctrp_fvr: float | None = ...,
        ctrp_qemx: float | None = ...,
        ctrp_qemn: float | None = ...,
        ctrp_esec: float | None = ...,
        ctrp_damp: float | None = ...,
        ctrp_mode: str | None = ...,
        ctrp_dipl: float | None = ...,
        ctrp_obp: str | None = ...,
        ctrp_rem: str | None = ...,
        file_fset: str | None = ...,
        ctrds: list[CTRD] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CTRS:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    ctrs_tesn: str | None
    ctrs_cell: float | None
    ctrs_bpwp: float | None
    ctrs_mpwp: float | None
    ctrs_mpb: float | None
    ctrs_bb: float | None
    ctrs_sat: str | None
    ctrs_fsat: float | None
    ctrs_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        ctrs_tesn: str | None = ...,
        ctrs_cell: float | None = ...,
        ctrs_bpwp: float | None = ...,
        ctrs_mpwp: float | None = ...,
        ctrs_mpb: float | None = ...,
        ctrs_bb: float | None = ...,
        ctrs_sat: str | None = ...,
        ctrs_fsat: float | None = ...,
        ctrs_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DCPG:
    loca_id: str | None
    dcpg_date: _dt.datetime | None
    dcpg_tesn: str | None
    dcpg_dpth: float | None
    dcpg_zero: int | None
    dcpg_lrem: str | None
    dcpg_rem: str | None
    dcpg_env: str | None
    dcpg_meth: str | None
    dcpg_cont: str | None
    dcpg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    dcpg_oper: str | None
    dcpts: list[DCPT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        dcpg_date: _dt.datetime | None = ...,
        dcpg_tesn: str | None = ...,
        dcpg_dpth: float | None = ...,
        dcpg_zero: int | None = ...,
        dcpg_lrem: str | None = ...,
        dcpg_rem: str | None = ...,
        dcpg_env: str | None = ...,
        dcpg_meth: str | None = ...,
        dcpg_cont: str | None = ...,
        dcpg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        dcpg_oper: str | None = ...,
        dcpts: list[DCPT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DCPT:
    loca_id: str | None
    dcpg_date: _dt.datetime | None
    dcpg_tesn: str | None
    dcpg_dpth: float | None
    dcpt_cblo: int | None
    dcpt_pen: int | None
    dcpt_del: str | None
    dcpt_rem: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        dcpg_date: _dt.datetime | None = ...,
        dcpg_tesn: str | None = ...,
        dcpg_dpth: float | None = ...,
        dcpt_cblo: int | None = ...,
        dcpt_pen: int | None = ...,
        dcpt_del: str | None = ...,
        dcpt_rem: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DETL:
    loca_id: str | None
    detl_top: float | None
    detl_base: float | None
    detl_desc: str | None
    detl_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        detl_top: float | None = ...,
        detl_base: float | None = ...,
        detl_desc: str | None = ...,
        detl_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DICT:
    dict_type: str | None
    dict_grp: str | None
    dict_hdng: str | None
    dict_stat: str | None
    dict_dtyp: str | None
    dict_desc: str | None
    dict_unit: str | None
    dict_exmp: str | None
    dict_pgrp: str | None
    dict_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        dict_type: str | None = ...,
        dict_grp: str | None = ...,
        dict_hdng: str | None = ...,
        dict_stat: str | None = ...,
        dict_dtyp: str | None = ...,
        dict_desc: str | None = ...,
        dict_unit: str | None = ...,
        dict_exmp: str | None = ...,
        dict_pgrp: str | None = ...,
        dict_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DISC:
    loca_id: str | None
    disc_top: float | None
    disc_base: float | None
    frac_set: str | None
    disc_numb: str | None
    disc_type: str | None
    disc_dip: str | None
    disc_dir: str | None
    disc_rgh: str | None
    disc_plan: str | None
    disc_wave: float | None
    disc_amp: float | None
    disc_jrc: int | None
    disc_app: str | None
    disc_apt: str | None
    disc_apob: str | None
    disc_infm: str | None
    disc_term: str | None
    disc_pers: float | None
    disc_str: int | None
    disc_weth: str | None
    disc_seep: str | None
    disc_flow: int | None
    disc_rem: str | None
    file_fset: str | None
    disc_mid: float | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        disc_top: float | None = ...,
        disc_base: float | None = ...,
        frac_set: str | None = ...,
        disc_numb: str | None = ...,
        disc_type: str | None = ...,
        disc_dip: str | None = ...,
        disc_dir: str | None = ...,
        disc_rgh: str | None = ...,
        disc_plan: str | None = ...,
        disc_wave: float | None = ...,
        disc_amp: float | None = ...,
        disc_jrc: int | None = ...,
        disc_app: str | None = ...,
        disc_apt: str | None = ...,
        disc_apob: str | None = ...,
        disc_infm: str | None = ...,
        disc_term: str | None = ...,
        disc_pers: float | None = ...,
        disc_str: int | None = ...,
        disc_weth: str | None = ...,
        disc_seep: str | None = ...,
        disc_flow: int | None = ...,
        disc_rem: str | None = ...,
        file_fset: str | None = ...,
        disc_mid: float | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DLOG:
    loca_id: str | None
    dlog_top: float | None
    dlog_base: float | None
    dlog_desc: str | None
    dlog_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        dlog_top: float | None = ...,
        dlog_base: float | None = ...,
        dlog_desc: str | None = ...,
        dlog_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DMDG:
    loca_id: str | None
    dmtg_tesn: str | None
    dmdg_dpth: float | None
    dmdg_tflx: float | None
    dmdg_ch: float | None
    dmdg_chmt: str | None
    dmdg_mh: float | None
    dmdg_mhmt: str | None
    dmdg_kh: float | None
    dmdg_khmt: str | None
    dmdg_date: _dt.datetime | None
    test_stat: str | None
    dmdg_rem: str | None
    file_fset: str | None
    dmdts: list[DMDT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        dmtg_tesn: str | None = ...,
        dmdg_dpth: float | None = ...,
        dmdg_tflx: float | None = ...,
        dmdg_ch: float | None = ...,
        dmdg_chmt: str | None = ...,
        dmdg_mh: float | None = ...,
        dmdg_mhmt: str | None = ...,
        dmdg_kh: float | None = ...,
        dmdg_khmt: str | None = ...,
        dmdg_date: _dt.datetime | None = ...,
        test_stat: str | None = ...,
        dmdg_rem: str | None = ...,
        file_fset: str | None = ...,
        dmdts: list[DMDT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DMDT:
    loca_id: str | None
    dmtg_tesn: str | None
    dmdg_dpth: float | None
    dmdt_time: float | None
    dmdt_a: float | None
    dmdt_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        dmtg_tesn: str | None = ...,
        dmdg_dpth: float | None = ...,
        dmdt_time: float | None = ...,
        dmdt_a: float | None = ...,
        dmdt_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DMTG:
    loca_id: str | None
    dmtg_tesn: str | None
    dmtg_date: _dt.datetime | None
    dmtg_ornt: int | None
    dmtg_ped: float | None
    dmtg_wat: float | None
    dmtg_wata: str | None
    dmtg_type: str | None
    dmtg_refb: str | None
    dmtg_refa: str | None
    dmtg_man: str | None
    dmtg_rig: str | None
    dmtg_eqpt: str | None
    dmtg_cot: str | None
    dmtg_tdr: str | None
    dmtg_dims: str | None
    dmtg_prsg: str | None
    dmtg_fric: str | None
    dmtg_dith: float | None
    dmtg_bcva: float | None
    dmtg_bcvb: float | None
    dmtg_faed: float | None
    dmtg_fas0: float | None
    dmtg_term: str | None
    dmtg_corr: str | None
    dmtg_rem: str | None
    dmtg_oper: str | None
    dmtg_anby: str | None
    dmtg_env: str | None
    dmtg_meth: str | None
    dmtg_dev: str | None
    dmtg_cont: str | None
    dmtg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    dmdgs: list[DMDG]
    dmtts: list[DMTT]
    dmtzs: list[DMTZ]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        dmtg_tesn: str | None = ...,
        dmtg_date: _dt.datetime | None = ...,
        dmtg_ornt: int | None = ...,
        dmtg_ped: float | None = ...,
        dmtg_wat: float | None = ...,
        dmtg_wata: str | None = ...,
        dmtg_type: str | None = ...,
        dmtg_refb: str | None = ...,
        dmtg_refa: str | None = ...,
        dmtg_man: str | None = ...,
        dmtg_rig: str | None = ...,
        dmtg_eqpt: str | None = ...,
        dmtg_cot: str | None = ...,
        dmtg_tdr: str | None = ...,
        dmtg_dims: str | None = ...,
        dmtg_prsg: str | None = ...,
        dmtg_fric: str | None = ...,
        dmtg_dith: float | None = ...,
        dmtg_bcva: float | None = ...,
        dmtg_bcvb: float | None = ...,
        dmtg_faed: float | None = ...,
        dmtg_fas0: float | None = ...,
        dmtg_term: str | None = ...,
        dmtg_corr: str | None = ...,
        dmtg_rem: str | None = ...,
        dmtg_oper: str | None = ...,
        dmtg_anby: str | None = ...,
        dmtg_env: str | None = ...,
        dmtg_meth: str | None = ...,
        dmtg_dev: str | None = ...,
        dmtg_cont: str | None = ...,
        dmtg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        dmdgs: list[DMDG] | None = ...,
        dmtts: list[DMTT] | None = ...,
        dmtzs: list[DMTZ] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DMTP:
    loca_id: str | None
    dmtg_tesn: str | None
    dmtt_dpth: float | None
    dmtp_buw: float | None
    dmtp_tvs: int | None
    dmtp_evs: int | None
    dmtp_u0: float | None
    dmtp_id: float | None
    dmtp_kd: float | None
    dmtp_ed: float | None
    dmtp_ud: float | None
    dmtp_vs: int | None
    dmtp_vdm: float | None
    dmtp_su: int | None
    dmtp_phi: float | None
    dmtp_k0: float | None
    dmtp_ths: int | None
    dmtp_ehs: int | None
    dmtp_ocr: float | None
    dmtp_mps: float | None
    dmtp_dsd: str | None
    dmtp_buwm: str | None
    dmtp_tvsm: str | None
    dmtp_evsm: str | None
    dmtp_u0m: str | None
    dmtp_idm: str | None
    dmtp_kdm: str | None
    dmtp_edm: str | None
    dmtp_udm: str | None
    dmtp_vsm: str | None
    dmtp_vdmm: str | None
    dmtp_sum: str | None
    dmtp_phim: str | None
    dmtp_k0m: str | None
    dmtp_thsm: str | None
    dmtp_ehsm: str | None
    dmtp_ocrm: str | None
    dmtp_mpsm: str | None
    dmtp_dsdm: str | None
    dmtp_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        dmtg_tesn: str | None = ...,
        dmtt_dpth: float | None = ...,
        dmtp_buw: float | None = ...,
        dmtp_tvs: int | None = ...,
        dmtp_evs: int | None = ...,
        dmtp_u0: float | None = ...,
        dmtp_id: float | None = ...,
        dmtp_kd: float | None = ...,
        dmtp_ed: float | None = ...,
        dmtp_ud: float | None = ...,
        dmtp_vs: int | None = ...,
        dmtp_vdm: float | None = ...,
        dmtp_su: int | None = ...,
        dmtp_phi: float | None = ...,
        dmtp_k0: float | None = ...,
        dmtp_ths: int | None = ...,
        dmtp_ehs: int | None = ...,
        dmtp_ocr: float | None = ...,
        dmtp_mps: float | None = ...,
        dmtp_dsd: str | None = ...,
        dmtp_buwm: str | None = ...,
        dmtp_tvsm: str | None = ...,
        dmtp_evsm: str | None = ...,
        dmtp_u0m: str | None = ...,
        dmtp_idm: str | None = ...,
        dmtp_kdm: str | None = ...,
        dmtp_edm: str | None = ...,
        dmtp_udm: str | None = ...,
        dmtp_vsm: str | None = ...,
        dmtp_vdmm: str | None = ...,
        dmtp_sum: str | None = ...,
        dmtp_phim: str | None = ...,
        dmtp_k0m: str | None = ...,
        dmtp_thsm: str | None = ...,
        dmtp_ehsm: str | None = ...,
        dmtp_ocrm: str | None = ...,
        dmtp_mpsm: str | None = ...,
        dmtp_dsdm: str | None = ...,
        dmtp_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DMTT:
    loca_id: str | None
    dmtg_tesn: str | None
    dmtt_dpth: float | None
    dmtt_mth: int | None
    dmtt_bcva: float | None
    dmtt_bcvb: float | None
    dmtt_tmst: _dt.datetime | None
    dmtt_a: float | None
    dmtt_tma: float | None
    dmtt_b: float | None
    dmtt_tmb: float | None
    dmtt_c: float | None
    dmtt_tmc: float | None
    dmtt_p0: int | None
    dmtt_p1: int | None
    dmtt_p2: int | None
    dmtt_incx: float | None
    dmtt_incy: float | None
    dmtt_rate: int | None
    dmtt_rem: str | None
    file_fset: str | None
    dmtps: list[DMTP]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        dmtg_tesn: str | None = ...,
        dmtt_dpth: float | None = ...,
        dmtt_mth: int | None = ...,
        dmtt_bcva: float | None = ...,
        dmtt_bcvb: float | None = ...,
        dmtt_tmst: _dt.datetime | None = ...,
        dmtt_a: float | None = ...,
        dmtt_tma: float | None = ...,
        dmtt_b: float | None = ...,
        dmtt_tmb: float | None = ...,
        dmtt_c: float | None = ...,
        dmtt_tmc: float | None = ...,
        dmtt_p0: int | None = ...,
        dmtt_p1: int | None = ...,
        dmtt_p2: int | None = ...,
        dmtt_incx: float | None = ...,
        dmtt_incy: float | None = ...,
        dmtt_rate: int | None = ...,
        dmtt_rem: str | None = ...,
        file_fset: str | None = ...,
        dmtps: list[DMTP] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DMTZ:
    loca_id: str | None
    dmtg_tesn: str | None
    dmtz_date: _dt.datetime | None
    dmtz_type: str | None
    dmtz_bcva: float | None
    dmtz_bcvb: float | None
    dmtz_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        dmtg_tesn: str | None = ...,
        dmtz_date: _dt.datetime | None = ...,
        dmtz_type: str | None = ...,
        dmtz_bcva: float | None = ...,
        dmtz_bcvb: float | None = ...,
        dmtz_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DOBS:
    loca_id: str | None
    dobs_top: float | None
    dobs_base: float | None
    dobs_set: str | None
    dobs_durn: str | None
    dobs_stim: _dt.datetime | None
    dobs_etim: _dt.datetime | None
    dobs_dhrt: float | None
    dobs_dhrs: int | None
    dobs_penr: float | None
    dobs_hamm: bool | None
    dobs_thrp: float | None
    dobs_resp: float | None
    dobs_torp: float | None
    dobs_torq: float | None
    dobs_thst: float | None
    dobs_rest: float | None
    dobs_hamp: float | None
    dobs_spen: float | None
    dobs_fmpo: float | None
    dobs_fmcr: float | None
    dobs_fmrr: float | None
    dobs_rem: str | None
    file_fset: str | None
    dobs_meth: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        dobs_top: float | None = ...,
        dobs_base: float | None = ...,
        dobs_set: str | None = ...,
        dobs_durn: str | None = ...,
        dobs_stim: _dt.datetime | None = ...,
        dobs_etim: _dt.datetime | None = ...,
        dobs_dhrt: float | None = ...,
        dobs_dhrs: int | None = ...,
        dobs_penr: float | None = ...,
        dobs_hamm: bool | None = ...,
        dobs_thrp: float | None = ...,
        dobs_resp: float | None = ...,
        dobs_torp: float | None = ...,
        dobs_torq: float | None = ...,
        dobs_thst: float | None = ...,
        dobs_rest: float | None = ...,
        dobs_hamp: float | None = ...,
        dobs_spen: float | None = ...,
        dobs_fmpo: float | None = ...,
        dobs_fmcr: float | None = ...,
        dobs_fmrr: float | None = ...,
        dobs_rem: str | None = ...,
        file_fset: str | None = ...,
        dobs_meth: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DPRB:
    loca_id: str | None
    dprg_tesn: str | None
    dprb_dpth: float | None
    dprb_blow: int | None
    dprb_cblw: int | None
    dprb_torq: int | None
    dprb_del: str | None
    dprb_inc: int | None
    dprb_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        dprg_tesn: str | None = ...,
        dprb_dpth: float | None = ...,
        dprb_blow: int | None = ...,
        dprb_cblw: int | None = ...,
        dprb_torq: int | None = ...,
        dprb_del: str | None = ...,
        dprb_inc: int | None = ...,
        dprb_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DPRG:
    loca_id: str | None
    dprg_tesn: str | None
    dprg_date: _dt.datetime | None
    dprg_type: str | None
    dprg_meth: str | None
    dprg_mass: float | None
    dprg_drop: int | None
    dprg_cone: float | None
    dprg_rod: int | None
    dprg_tanv: str | None
    dprg_damp: str | None
    dprg_tip: float | None
    dprg_rem: str | None
    dprg_ang: int | None
    dprg_rmss: float | None
    dprg_parf: str | None
    dprg_pdiu: str | None
    dprg_bcf: str | None
    dprg_gw: float | None
    dprg_reet: str | None
    dprg_env: str | None
    dprg_cont: str | None
    dprg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    dprg_oper: str | None
    dprbs: list[DPRB]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        dprg_tesn: str | None = ...,
        dprg_date: _dt.datetime | None = ...,
        dprg_type: str | None = ...,
        dprg_meth: str | None = ...,
        dprg_mass: float | None = ...,
        dprg_drop: int | None = ...,
        dprg_cone: float | None = ...,
        dprg_rod: int | None = ...,
        dprg_tanv: str | None = ...,
        dprg_damp: str | None = ...,
        dprg_tip: float | None = ...,
        dprg_rem: str | None = ...,
        dprg_ang: int | None = ...,
        dprg_rmss: float | None = ...,
        dprg_parf: str | None = ...,
        dprg_pdiu: str | None = ...,
        dprg_bcf: str | None = ...,
        dprg_gw: float | None = ...,
        dprg_reet: str | None = ...,
        dprg_env: str | None = ...,
        dprg_cont: str | None = ...,
        dprg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        dprg_oper: str | None = ...,
        dprbs: list[DPRB] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class DREM:
    loca_id: str | None
    drem_top: float | None
    drem_base: float | None
    drem_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        drem_top: float | None = ...,
        drem_base: float | None = ...,
        drem_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ECTN:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    ectn_id: str | None
    ectn_type: str | None
    ectn_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        ectn_id: str | None = ...,
        ectn_type: str | None = ...,
        ectn_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ELRG:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    elrg_code: str | None
    elrg_meth: str | None
    elrg_matx: str | None
    elrg_rtyp: str | None
    elrg_tade: str | None
    elrg_ticn: str | None
    elrg_runi: str | None
    spec_desc: str | None
    spec_prep: str | None
    spec_base: float | None
    elrg_lsid: str | None
    elrg_rtcd: str | None
    elrg_iqlf: str | None
    elrg_lqlf: str | None
    elrg_rval: str | None
    elrg_rtxt: str | None
    elrg_name: str | None
    elrg_tnam: str | None
    elrg_dcat: str | None
    elrg_tesn: str | None
    elrg_fdev: bool | None
    elrg_dev: str | None
    elrg_rres: bool | None
    elrg_detf: bool | None
    elrg_org: bool | None
    elrg_rdlm: str | None
    elrg_mdlm: str | None
    elrg_qlm: str | None
    elrg_duni: str | None
    elrg_casc: str | None
    elrg_ticp: int | None
    elrg_tict: int | None
    elrg_rdat: _dt.datetime | None
    elrg_sgrp: str | None
    elrg_dtim: _dt.datetime | None
    elrg_test: str | None
    elrg_tord: str | None
    elrg_locn: str | None
    elrg_bas: str | None
    elrg_dil: int | None
    elrg_lmth: str | None
    elrg_ldtm: _dt.datetime | None
    elrg_iref: str | None
    elrg_ityp: str | None
    elrg_size: int | None
    elrg_perp: float | None
    elrg_rem: str | None
    elrg_lab: str | None
    elrg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        elrg_code: str | None = ...,
        elrg_meth: str | None = ...,
        elrg_matx: str | None = ...,
        elrg_rtyp: str | None = ...,
        elrg_tade: str | None = ...,
        elrg_ticn: str | None = ...,
        elrg_runi: str | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        spec_base: float | None = ...,
        elrg_lsid: str | None = ...,
        elrg_rtcd: str | None = ...,
        elrg_iqlf: str | None = ...,
        elrg_lqlf: str | None = ...,
        elrg_rval: str | None = ...,
        elrg_rtxt: str | None = ...,
        elrg_name: str | None = ...,
        elrg_tnam: str | None = ...,
        elrg_dcat: str | None = ...,
        elrg_tesn: str | None = ...,
        elrg_fdev: bool | None = ...,
        elrg_dev: str | None = ...,
        elrg_rres: bool | None = ...,
        elrg_detf: bool | None = ...,
        elrg_org: bool | None = ...,
        elrg_rdlm: str | None = ...,
        elrg_mdlm: str | None = ...,
        elrg_qlm: str | None = ...,
        elrg_duni: str | None = ...,
        elrg_casc: str | None = ...,
        elrg_ticp: int | None = ...,
        elrg_tict: int | None = ...,
        elrg_rdat: _dt.datetime | None = ...,
        elrg_sgrp: str | None = ...,
        elrg_dtim: _dt.datetime | None = ...,
        elrg_test: str | None = ...,
        elrg_tord: str | None = ...,
        elrg_locn: str | None = ...,
        elrg_bas: str | None = ...,
        elrg_dil: int | None = ...,
        elrg_lmth: str | None = ...,
        elrg_ldtm: _dt.datetime | None = ...,
        elrg_iref: str | None = ...,
        elrg_ityp: str | None = ...,
        elrg_size: int | None = ...,
        elrg_perp: float | None = ...,
        elrg_rem: str | None = ...,
        elrg_lab: str | None = ...,
        elrg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ERES:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    eres_code: str | None
    eres_meth: str | None
    eres_matx: str | None
    eres_rtyp: str | None
    eres_tesn: str | None
    eres_name: str | None
    eres_tnam: str | None
    eres_rval: str | None
    eres_runi: str | None
    eres_rtxt: str | None
    eres_rtcd: str | None
    eres_rres: bool | None
    eres_detf: bool | None
    eres_org: bool | None
    eres_iqlf: str | None
    eres_lqlf: str | None
    eres_rdlm: str | None
    eres_mdlm: str | None
    eres_qlm: str | None
    eres_duni: str | None
    eres_ticp: int | None
    eres_tict: int | None
    eres_rdat: _dt.datetime | None
    eres_sgrp: str | None
    spec_prep: str | None
    spec_desc: str | None
    eres_dtim: _dt.datetime | None
    eres_test: str | None
    eres_tord: str | None
    eres_locn: str | None
    eres_bas: str | None
    eres_dil: int | None
    eres_lmth: str | None
    eres_ldtm: _dt.datetime | None
    eres_iref: str | None
    eres_size: int | None
    eres_perp: float | None
    eres_rem: str | None
    eres_lab: str | None
    eres_cred: str | None
    test_stat: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        eres_code: str | None = ...,
        eres_meth: str | None = ...,
        eres_matx: str | None = ...,
        eres_rtyp: str | None = ...,
        eres_tesn: str | None = ...,
        eres_name: str | None = ...,
        eres_tnam: str | None = ...,
        eres_rval: str | None = ...,
        eres_runi: str | None = ...,
        eres_rtxt: str | None = ...,
        eres_rtcd: str | None = ...,
        eres_rres: bool | None = ...,
        eres_detf: bool | None = ...,
        eres_org: bool | None = ...,
        eres_iqlf: str | None = ...,
        eres_lqlf: str | None = ...,
        eres_rdlm: str | None = ...,
        eres_mdlm: str | None = ...,
        eres_qlm: str | None = ...,
        eres_duni: str | None = ...,
        eres_ticp: int | None = ...,
        eres_tict: int | None = ...,
        eres_rdat: _dt.datetime | None = ...,
        eres_sgrp: str | None = ...,
        spec_prep: str | None = ...,
        spec_desc: str | None = ...,
        eres_dtim: _dt.datetime | None = ...,
        eres_test: str | None = ...,
        eres_tord: str | None = ...,
        eres_locn: str | None = ...,
        eres_bas: str | None = ...,
        eres_dil: int | None = ...,
        eres_lmth: str | None = ...,
        eres_ldtm: _dt.datetime | None = ...,
        eres_iref: str | None = ...,
        eres_size: int | None = ...,
        eres_perp: float | None = ...,
        eres_rem: str | None = ...,
        eres_lab: str | None = ...,
        eres_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ESCG:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    escg_type: str | None
    escg_cell: str | None
    escg_cond: str | None
    escg_sdia: float | None
    escg_higt: float | None
    escg_mci: str | None
    escg_mcf: str | None
    escg_bden: float | None
    escg_bdef: float | None
    escg_dden: float | None
    escg_pden: str | None
    escg_ivr: float | None
    escg_satr: int | None
    escg_load: str | None
    escg_drag: str | None
    escg_ppm: str | None
    escg_sprs: float | None
    escg_satm: str | None
    escg_sinc: int | None
    escg_sdif: int | None
    escg_celf: int | None
    escg_bacf: int | None
    escg_bval: float | None
    escg_svol: float | None
    escg_rem: str | None
    escg_meth: str | None
    escg_lab: str | None
    escg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    escg_dev: str | None
    escg_isvr: float | None
    escg_isvs: int | None
    escg_isst: float | None
    escg_pcp: int | None
    escg_ysr: float | None
    escg_cc: float | None
    escg_cs: float | None
    escts: list[ESCT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        escg_type: str | None = ...,
        escg_cell: str | None = ...,
        escg_cond: str | None = ...,
        escg_sdia: float | None = ...,
        escg_higt: float | None = ...,
        escg_mci: str | None = ...,
        escg_mcf: str | None = ...,
        escg_bden: float | None = ...,
        escg_bdef: float | None = ...,
        escg_dden: float | None = ...,
        escg_pden: str | None = ...,
        escg_ivr: float | None = ...,
        escg_satr: int | None = ...,
        escg_load: str | None = ...,
        escg_drag: str | None = ...,
        escg_ppm: str | None = ...,
        escg_sprs: float | None = ...,
        escg_satm: str | None = ...,
        escg_sinc: int | None = ...,
        escg_sdif: int | None = ...,
        escg_celf: int | None = ...,
        escg_bacf: int | None = ...,
        escg_bval: float | None = ...,
        escg_svol: float | None = ...,
        escg_rem: str | None = ...,
        escg_meth: str | None = ...,
        escg_lab: str | None = ...,
        escg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        escg_dev: str | None = ...,
        escg_isvr: float | None = ...,
        escg_isvs: int | None = ...,
        escg_isst: float | None = ...,
        escg_pcp: int | None = ...,
        escg_ysr: float | None = ...,
        escg_cc: float | None = ...,
        escg_cs: float | None = ...,
        escts: list[ESCT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ESCT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    esct_incn: str | None
    esct_rem: str | None
    esct_incc: int | None
    esct_incb: int | None
    esct_pwp0: int | None
    esct_pwpf: int | None
    esct_incf: int | None
    esct_vr0: float | None
    esct_vre: float | None
    esct_diss: int | None
    esct_dset: float | None
    esct_dvol: float | None
    esct_inmv: float | None
    esct_incv: float | None
    esct_insc: float | None
    esct_cvme: str | None
    esct_temp: float | None
    file_fset: str | None
    esct_ink: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        esct_incn: str | None = ...,
        esct_rem: str | None = ...,
        esct_incc: int | None = ...,
        esct_incb: int | None = ...,
        esct_pwp0: int | None = ...,
        esct_pwpf: int | None = ...,
        esct_incf: int | None = ...,
        esct_vr0: float | None = ...,
        esct_vre: float | None = ...,
        esct_diss: int | None = ...,
        esct_dset: float | None = ...,
        esct_dvol: float | None = ...,
        esct_inmv: float | None = ...,
        esct_incv: float | None = ...,
        esct_insc: float | None = ...,
        esct_cvme: str | None = ...,
        esct_temp: float | None = ...,
        file_fset: str | None = ...,
        esct_ink: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class FGHG:
    loca_id: str | None
    fghg_top: float | None
    fghg_base: float | None
    fghg_tesn: str | None
    fghg_tdia: int | None
    fghg_sdia: int | None
    fghg_odia: int | None
    fghg_hbas: float | None
    fghg_cas: float | None
    fghg_sfac: float | None
    fghg_sfrf: str | None
    fghg_date: _dt.datetime | None
    fghg_type: str | None
    fghg_cnfg: str | None
    fghg_meth: str | None
    fghg_prwl: float | None
    fghg_awl: float | None
    fghg_head: float | None
    fghg_flow: float | None
    fghg_iprm: float | None
    fghg_ilug: str | None
    fghg_ftyp: str | None
    fghg_rem: str | None
    fghg_env: str | None
    fghg_cont: str | None
    fghg_oper: str | None
    fghg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    fghis: list[FGHI]
    fghss: list[FGHS]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        fghg_top: float | None = ...,
        fghg_base: float | None = ...,
        fghg_tesn: str | None = ...,
        fghg_tdia: int | None = ...,
        fghg_sdia: int | None = ...,
        fghg_odia: int | None = ...,
        fghg_hbas: float | None = ...,
        fghg_cas: float | None = ...,
        fghg_sfac: float | None = ...,
        fghg_sfrf: str | None = ...,
        fghg_date: _dt.datetime | None = ...,
        fghg_type: str | None = ...,
        fghg_cnfg: str | None = ...,
        fghg_meth: str | None = ...,
        fghg_prwl: float | None = ...,
        fghg_awl: float | None = ...,
        fghg_head: float | None = ...,
        fghg_flow: float | None = ...,
        fghg_iprm: float | None = ...,
        fghg_ilug: str | None = ...,
        fghg_ftyp: str | None = ...,
        fghg_rem: str | None = ...,
        fghg_env: str | None = ...,
        fghg_cont: str | None = ...,
        fghg_oper: str | None = ...,
        fghg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        fghis: list[FGHI] | None = ...,
        fghss: list[FGHS] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class FGHI:
    loca_id: str | None
    fghg_top: float | None
    fghg_base: float | None
    fghg_tesn: str | None
    fghi_inst: str | None
    fghi_type: str | None
    fghi_detl: str | None
    fghi_loct: str | None
    fghi_rem: str | None
    file_fset: str | None
    fghts: list[FGHT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        fghg_top: float | None = ...,
        fghg_base: float | None = ...,
        fghg_tesn: str | None = ...,
        fghi_inst: str | None = ...,
        fghi_type: str | None = ...,
        fghi_detl: str | None = ...,
        fghi_loct: str | None = ...,
        fghi_rem: str | None = ...,
        file_fset: str | None = ...,
        fghts: list[FGHT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class FGHS:
    loca_id: str | None
    fghg_top: float | None
    fghg_base: float | None
    fghg_tesn: str | None
    fghs_stg: int | None
    fghs_sttm: _dt.datetime | None
    fghs_entm: _dt.datetime | None
    fghs_head: float | None
    fghs_flow: float | None
    fghs_iprm: float | None
    fghs_ilug: str | None
    fghs_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        fghg_top: float | None = ...,
        fghg_base: float | None = ...,
        fghg_tesn: str | None = ...,
        fghs_stg: int | None = ...,
        fghs_sttm: _dt.datetime | None = ...,
        fghs_entm: _dt.datetime | None = ...,
        fghs_head: float | None = ...,
        fghs_flow: float | None = ...,
        fghs_iprm: float | None = ...,
        fghs_ilug: str | None = ...,
        fghs_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class FGHT:
    loca_id: str | None
    fghg_top: float | None
    fghg_base: float | None
    fghg_tesn: str | None
    fghi_inst: str | None
    fght_time: _dt.datetime | None
    fght_type: str | None
    fghs_stg: int | None
    fght_durn: str | None
    fght_rdng: str | None
    fght_unit: str | None
    fght_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        fghg_top: float | None = ...,
        fghg_base: float | None = ...,
        fghg_tesn: str | None = ...,
        fghi_inst: str | None = ...,
        fght_time: _dt.datetime | None = ...,
        fght_type: str | None = ...,
        fghs_stg: int | None = ...,
        fght_durn: str | None = ...,
        fght_rdng: str | None = ...,
        fght_unit: str | None = ...,
        fght_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class FILE:
    file_fset: str | None
    file_name: str | None
    file_desc: str | None
    file_type: str | None
    file_prog: str | None
    file_doct: str | None
    file_date: _dt.datetime | None
    file_rem: str | None
    def __init__(
        self,
        *,
        file_fset: str | None = ...,
        file_name: str | None = ...,
        file_desc: str | None = ...,
        file_type: str | None = ...,
        file_prog: str | None = ...,
        file_doct: str | None = ...,
        file_date: _dt.datetime | None = ...,
        file_rem: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class FLSH:
    loca_id: str | None
    flsh_top: float | None
    flsh_base: float | None
    flsh_type: str | None
    flsh_retn: int | None
    flsh_retx: int | None
    flsh_col: str | None
    flsh_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        flsh_top: float | None = ...,
        flsh_base: float | None = ...,
        flsh_type: str | None = ...,
        flsh_retn: int | None = ...,
        flsh_retx: int | None = ...,
        flsh_col: str | None = ...,
        flsh_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class FRAC:
    loca_id: str | None
    frac_from: float | None
    frac_to: float | None
    frac_set: str | None
    frac_imax: str | None
    frac_iave: str | None
    frac_imin: str | None
    frac_fi: str | None
    frac_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        frac_from: float | None = ...,
        frac_to: float | None = ...,
        frac_set: str | None = ...,
        frac_imax: str | None = ...,
        frac_iave: str | None = ...,
        frac_imin: str | None = ...,
        frac_fi: str | None = ...,
        frac_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class FRST:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    frst_cond: str | None
    frst_dden: float | None
    frst_mc: str | None
    frst_hve1: float | None
    frst_hve2: float | None
    frst_hve3: float | None
    frst_hve: float | None
    frst_stab: float | None
    frst_styp: str | None
    frst_rem: str | None
    frst_meth: str | None
    frst_lab: str | None
    frst_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    frst_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        frst_cond: str | None = ...,
        frst_dden: float | None = ...,
        frst_mc: str | None = ...,
        frst_hve1: float | None = ...,
        frst_hve2: float | None = ...,
        frst_hve3: float | None = ...,
        frst_hve: float | None = ...,
        frst_stab: float | None = ...,
        frst_styp: str | None = ...,
        frst_rem: str | None = ...,
        frst_meth: str | None = ...,
        frst_lab: str | None = ...,
        frst_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        frst_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class GCHM:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    gchm_code: str | None
    gchm_meth: str | None
    gchm_ttyp: str | None
    gchm_resl: str | None
    gchm_unit: str | None
    gchm_name: str | None
    spec_desc: str | None
    spec_prep: str | None
    gchm_rem: str | None
    gchm_lab: str | None
    gchm_cred: str | None
    test_stat: str | None
    file_fset: str | None
    gchm_rtxt: str | None
    gchm_dlm: str | None
    spec_base: float | None
    gchm_dev: str | None
    gchm_sgrp: str | None
    gchm_lsid: str | None
    gchm_rdat: _dt.datetime | None
    gchm_dtim: _dt.datetime | None
    gchm_test: str | None
    gchm_iref: str | None
    gchm_ityp: str | None
    gchm_size: int | None
    gchm_perp: float | None
    gchm_rdev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        gchm_code: str | None = ...,
        gchm_meth: str | None = ...,
        gchm_ttyp: str | None = ...,
        gchm_resl: str | None = ...,
        gchm_unit: str | None = ...,
        gchm_name: str | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        gchm_rem: str | None = ...,
        gchm_lab: str | None = ...,
        gchm_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        gchm_rtxt: str | None = ...,
        gchm_dlm: str | None = ...,
        spec_base: float | None = ...,
        gchm_dev: str | None = ...,
        gchm_sgrp: str | None = ...,
        gchm_lsid: str | None = ...,
        gchm_rdat: _dt.datetime | None = ...,
        gchm_dtim: _dt.datetime | None = ...,
        gchm_test: str | None = ...,
        gchm_iref: str | None = ...,
        gchm_ityp: str | None = ...,
        gchm_size: int | None = ...,
        gchm_perp: float | None = ...,
        gchm_rdev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class GEOL:
    loca_id: str | None
    geol_top: float | None
    geol_base: float | None
    geol_desc: str | None
    geol_leg: str | None
    geol_geol: str | None
    geol_geo2: str | None
    geol_stat: str | None
    geol_bgs: str | None
    geol_form: str | None
    geol_rem: str | None
    file_fset: str | None
    geol_bndf: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        geol_top: float | None = ...,
        geol_base: float | None = ...,
        geol_desc: str | None = ...,
        geol_leg: str | None = ...,
        geol_geol: str | None = ...,
        geol_geo2: str | None = ...,
        geol_stat: str | None = ...,
        geol_bgs: str | None = ...,
        geol_form: str | None = ...,
        geol_rem: str | None = ...,
        file_fset: str | None = ...,
        geol_bndf: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class GRAG:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    grag_uc: float | None
    grag_vcre: float | None
    grag_grav: float | None
    grag_sand: float | None
    grag_silt: float | None
    grag_clay: float | None
    grag_fine: float | None
    grag_rem: str | None
    grag_meth: str | None
    grag_lab: str | None
    grag_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    grag_dev: str | None
    grag_pden: str | None
    grag_pret: str | None
    grag_suff: bool | None
    grag_excl: str | None
    grag_cc: float | None
    grats: list[GRAT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        grag_uc: float | None = ...,
        grag_vcre: float | None = ...,
        grag_grav: float | None = ...,
        grag_sand: float | None = ...,
        grag_silt: float | None = ...,
        grag_clay: float | None = ...,
        grag_fine: float | None = ...,
        grag_rem: str | None = ...,
        grag_meth: str | None = ...,
        grag_lab: str | None = ...,
        grag_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        grag_dev: str | None = ...,
        grag_pden: str | None = ...,
        grag_pret: str | None = ...,
        grag_suff: bool | None = ...,
        grag_excl: str | None = ...,
        grag_cc: float | None = ...,
        grats: list[GRAT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class GRAT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    grat_size: float | None
    grat_perp: int | None
    grat_type: str | None
    grat_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        grat_size: float | None = ...,
        grat_perp: int | None = ...,
        grat_type: str | None = ...,
        grat_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class HDIA:
    loca_id: str | None
    hdia_dpth: float | None
    hdia_diam: int | None
    hdia_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        hdia_dpth: float | None = ...,
        hdia_diam: int | None = ...,
        hdia_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class HDPH:
    loca_id: str | None
    hdph_top: float | None
    hdph_base: float | None
    hdph_type: str | None
    hdph_star: _dt.datetime | None
    hdph_endd: _dt.datetime | None
    hdph_crew: str | None
    hdph_exc: str | None
    hdph_shor: str | None
    hdph_stab: str | None
    hdph_diml: float | None
    hdph_dimw: float | None
    hdph_dbit: str | None
    hdph_bcon: str | None
    hdph_btyp: str | None
    hdph_blen: float | None
    hdph_log: str | None
    hdph_logd: _dt.datetime | None
    hdph_rem: str | None
    hdph_env: str | None
    hdph_meth: str | None
    hdph_cont: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        hdph_top: float | None = ...,
        hdph_base: float | None = ...,
        hdph_type: str | None = ...,
        hdph_star: _dt.datetime | None = ...,
        hdph_endd: _dt.datetime | None = ...,
        hdph_crew: str | None = ...,
        hdph_exc: str | None = ...,
        hdph_shor: str | None = ...,
        hdph_stab: str | None = ...,
        hdph_diml: float | None = ...,
        hdph_dimw: float | None = ...,
        hdph_dbit: str | None = ...,
        hdph_bcon: str | None = ...,
        hdph_btyp: str | None = ...,
        hdph_blen: float | None = ...,
        hdph_log: str | None = ...,
        hdph_logd: _dt.datetime | None = ...,
        hdph_rem: str | None = ...,
        hdph_env: str | None = ...,
        hdph_meth: str | None = ...,
        hdph_cont: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class HORN:
    loca_id: str | None
    horn_top: float | None
    horn_base: float | None
    horn_ornt: int | None
    horn_incl: int | None
    horn_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        horn_top: float | None = ...,
        horn_base: float | None = ...,
        horn_ornt: int | None = ...,
        horn_incl: int | None = ...,
        horn_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ICBR:
    loca_id: str | None
    icbr_dpth: float | None
    icbr_tesn: str | None
    icbr_icbr: float | None
    icbr_mc: str | None
    icbr_date: _dt.datetime | None
    icbr_kent: str | None
    icbr_seat: int | None
    icbr_surc: int | None
    icbr_type: str | None
    icbr_rem: str | None
    icbr_env: str | None
    icbr_meth: str | None
    icbr_cont: str | None
    icbr_cred: str | None
    test_stat: str | None
    geol_stat: str | None
    file_fset: str | None
    icbr_oper: str | None
    icbr_base: float | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        icbr_dpth: float | None = ...,
        icbr_tesn: str | None = ...,
        icbr_icbr: float | None = ...,
        icbr_mc: str | None = ...,
        icbr_date: _dt.datetime | None = ...,
        icbr_kent: str | None = ...,
        icbr_seat: int | None = ...,
        icbr_surc: int | None = ...,
        icbr_type: str | None = ...,
        icbr_rem: str | None = ...,
        icbr_env: str | None = ...,
        icbr_meth: str | None = ...,
        icbr_cont: str | None = ...,
        icbr_cred: str | None = ...,
        test_stat: str | None = ...,
        geol_stat: str | None = ...,
        file_fset: str | None = ...,
        icbr_oper: str | None = ...,
        icbr_base: float | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class IDEN:
    loca_id: str | None
    iden_dpth: float | None
    iden_tesn: str | None
    iden_date: _dt.datetime | None
    iden_type: str | None
    iden_iden: float | None
    iden_mc: str | None
    iden_stab: float | None
    iden_styp: str | None
    iden_rem: str | None
    iden_env: str | None
    iden_meth: str | None
    iden_cont: str | None
    iden_cred: str | None
    test_stat: str | None
    geol_stat: str | None
    file_fset: str | None
    iden_oper: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        iden_dpth: float | None = ...,
        iden_tesn: str | None = ...,
        iden_date: _dt.datetime | None = ...,
        iden_type: str | None = ...,
        iden_iden: float | None = ...,
        iden_mc: str | None = ...,
        iden_stab: float | None = ...,
        iden_styp: str | None = ...,
        iden_rem: str | None = ...,
        iden_env: str | None = ...,
        iden_meth: str | None = ...,
        iden_cont: str | None = ...,
        iden_cred: str | None = ...,
        test_stat: str | None = ...,
        geol_stat: str | None = ...,
        file_fset: str | None = ...,
        iden_oper: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class IFID:
    loca_id: str | None
    ifid_dpth: float | None
    ifid_tesn: str | None
    ifid_date: _dt.datetime | None
    ifid_res: str | None
    ifid_rem: str | None
    ifid_env: str | None
    ifid_meth: str | None
    ifid_cont: str | None
    ifid_cred: str | None
    test_stat: str | None
    geol_stat: str | None
    file_fset: str | None
    ifid_oper: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        ifid_dpth: float | None = ...,
        ifid_tesn: str | None = ...,
        ifid_date: _dt.datetime | None = ...,
        ifid_res: str | None = ...,
        ifid_rem: str | None = ...,
        ifid_env: str | None = ...,
        ifid_meth: str | None = ...,
        ifid_cont: str | None = ...,
        ifid_cred: str | None = ...,
        test_stat: str | None = ...,
        geol_stat: str | None = ...,
        file_fset: str | None = ...,
        ifid_oper: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class IPEN:
    loca_id: str | None
    ipen_dpth: float | None
    ipen_tesn: str | None
    ipen_ipen: str | None
    ipen_date: _dt.datetime | None
    ipen_rem: str | None
    ipen_env: str | None
    ipen_meth: str | None
    ipen_cont: str | None
    ipen_cred: str | None
    test_stat: str | None
    geol_stat: str | None
    file_fset: str | None
    ipen_oper: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        ipen_dpth: float | None = ...,
        ipen_tesn: str | None = ...,
        ipen_ipen: str | None = ...,
        ipen_date: _dt.datetime | None = ...,
        ipen_rem: str | None = ...,
        ipen_env: str | None = ...,
        ipen_meth: str | None = ...,
        ipen_cont: str | None = ...,
        ipen_cred: str | None = ...,
        test_stat: str | None = ...,
        geol_stat: str | None = ...,
        file_fset: str | None = ...,
        ipen_oper: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class IPID:
    loca_id: str | None
    ipid_dpth: float | None
    ipid_tesn: str | None
    ipid_date: _dt.datetime | None
    ipid_temp: float | None
    ipid_res: str | None
    ipid_rem: str | None
    ipid_env: str | None
    ipid_meth: str | None
    ipid_cont: str | None
    ipid_cred: str | None
    test_stat: str | None
    geol_stat: str | None
    file_fset: str | None
    ipid_oper: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        ipid_dpth: float | None = ...,
        ipid_tesn: str | None = ...,
        ipid_date: _dt.datetime | None = ...,
        ipid_temp: float | None = ...,
        ipid_res: str | None = ...,
        ipid_rem: str | None = ...,
        ipid_env: str | None = ...,
        ipid_meth: str | None = ...,
        ipid_cont: str | None = ...,
        ipid_cred: str | None = ...,
        test_stat: str | None = ...,
        geol_stat: str | None = ...,
        file_fset: str | None = ...,
        ipid_oper: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class IPRG:
    loca_id: str | None
    iprg_top: float | None
    iprg_tesn: str | None
    iprg_base: float | None
    iprg_stg: int | None
    iprg_type: str | None
    iprg_prwl: float | None
    iprg_swal: float | None
    iprg_tdia: float | None
    iprg_sdia: float | None
    iprg_iprm: float | None
    iprg_flow: float | None
    iprg_awl: float | None
    iprg_head: float | None
    iprg_date: _dt.datetime | None
    iprg_rem: str | None
    iprg_env: str | None
    iprg_meth: str | None
    iprg_cont: str | None
    iprg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    iprts: list[IPRT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        iprg_top: float | None = ...,
        iprg_tesn: str | None = ...,
        iprg_base: float | None = ...,
        iprg_stg: int | None = ...,
        iprg_type: str | None = ...,
        iprg_prwl: float | None = ...,
        iprg_swal: float | None = ...,
        iprg_tdia: float | None = ...,
        iprg_sdia: float | None = ...,
        iprg_iprm: float | None = ...,
        iprg_flow: float | None = ...,
        iprg_awl: float | None = ...,
        iprg_head: float | None = ...,
        iprg_date: _dt.datetime | None = ...,
        iprg_rem: str | None = ...,
        iprg_env: str | None = ...,
        iprg_meth: str | None = ...,
        iprg_cont: str | None = ...,
        iprg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        iprts: list[IPRT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class IPRT:
    loca_id: str | None
    iprg_top: float | None
    iprg_tesn: str | None
    iprg_base: float | None
    iprg_stg: int | None
    iprt_time: str | None
    iprt_dpth: float | None
    iprt_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        iprg_top: float | None = ...,
        iprg_tesn: str | None = ...,
        iprg_base: float | None = ...,
        iprg_stg: int | None = ...,
        iprt_time: str | None = ...,
        iprt_dpth: float | None = ...,
        iprt_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class IRDX:
    loca_id: str | None
    irdx_dpth: float | None
    irdx_tesn: str | None
    irdx_date: _dt.datetime | None
    irdx_ph: float | None
    irdx_mpot: int | None
    irdx_irdx: int | None
    irdx_rem: str | None
    irdx_env: str | None
    irdx_meth: str | None
    irdx_cont: str | None
    irdx_cred: str | None
    test_stat: str | None
    geol_stat: str | None
    file_fset: str | None
    irdx_oper: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        irdx_dpth: float | None = ...,
        irdx_tesn: str | None = ...,
        irdx_date: _dt.datetime | None = ...,
        irdx_ph: float | None = ...,
        irdx_mpot: int | None = ...,
        irdx_irdx: int | None = ...,
        irdx_rem: str | None = ...,
        irdx_env: str | None = ...,
        irdx_meth: str | None = ...,
        irdx_cont: str | None = ...,
        irdx_cred: str | None = ...,
        test_stat: str | None = ...,
        geol_stat: str | None = ...,
        file_fset: str | None = ...,
        irdx_oper: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class IRES:
    loca_id: str | None
    ires_dpth: float | None
    ires_tesn: str | None
    ires_base: float | None
    ires_type: str | None
    ires_date: _dt.datetime | None
    ires_ires: float | None
    ires_res1: float | None
    ires_res2: float | None
    ires_rem: str | None
    ires_env: str | None
    ires_meth: str | None
    ires_cont: str | None
    ires_cred: str | None
    test_stat: str | None
    geol_stat: str | None
    file_fset: str | None
    ires_oper: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        ires_dpth: float | None = ...,
        ires_tesn: str | None = ...,
        ires_base: float | None = ...,
        ires_type: str | None = ...,
        ires_date: _dt.datetime | None = ...,
        ires_ires: float | None = ...,
        ires_res1: float | None = ...,
        ires_res2: float | None = ...,
        ires_rem: str | None = ...,
        ires_env: str | None = ...,
        ires_meth: str | None = ...,
        ires_cont: str | None = ...,
        ires_cred: str | None = ...,
        test_stat: str | None = ...,
        geol_stat: str | None = ...,
        file_fset: str | None = ...,
        ires_oper: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ISAG:
    loca_id: str | None
    isag_tesn: str | None
    isag_date: _dt.datetime | None
    isag_durn: str | None
    isag_pwid: float | None
    isag_plen: float | None
    isag_pdia: float | None
    isag_dpts: float | None
    isag_dpte: float | None
    isag_cons: str | None
    isag_si: float | None
    isag_poro: int | None
    isag_rem: str | None
    isag_env: str | None
    isag_meth: str | None
    isag_cont: str | None
    isag_cred: str | None
    test_stat: str | None
    file_fset: str | None
    isag_oper: str | None
    isats: list[ISAT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        isag_tesn: str | None = ...,
        isag_date: _dt.datetime | None = ...,
        isag_durn: str | None = ...,
        isag_pwid: float | None = ...,
        isag_plen: float | None = ...,
        isag_pdia: float | None = ...,
        isag_dpts: float | None = ...,
        isag_dpte: float | None = ...,
        isag_cons: str | None = ...,
        isag_si: float | None = ...,
        isag_poro: int | None = ...,
        isag_rem: str | None = ...,
        isag_env: str | None = ...,
        isag_meth: str | None = ...,
        isag_cont: str | None = ...,
        isag_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        isag_oper: str | None = ...,
        isats: list[ISAT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ISAT:
    loca_id: str | None
    isag_tesn: str | None
    isat_time: str | None
    isat_dpth: float | None
    isat_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        isag_tesn: str | None = ...,
        isat_time: str | None = ...,
        isat_dpth: float | None = ...,
        isat_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ISPT:
    loca_id: str | None
    ispt_top: float | None
    ispt_seat: int | None
    ispt_main: int | None
    ispt_npen: int | None
    ispt_nval: int | None
    ispt_rep: str | None
    ispt_cas: float | None
    ispt_wat: str | None
    ispt_type: str | None
    ispt_ham: str | None
    ispt_erat: int | None
    ispt_swp: int | None
    ispt_inc1: int | None
    ispt_inc2: int | None
    ispt_inc3: int | None
    ispt_inc4: int | None
    ispt_inc5: int | None
    ispt_inc6: int | None
    ispt_pen1: int | None
    ispt_pen2: int | None
    ispt_pen3: int | None
    ispt_pen4: int | None
    ispt_pen5: int | None
    ispt_pen6: int | None
    ispt_rock: bool | None
    ispt_rem: str | None
    ispt_env: str | None
    ispt_meth: str | None
    ispt_cred: str | None
    test_stat: str | None
    file_fset: str | None
    ispt_n60: int | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        ispt_top: float | None = ...,
        ispt_seat: int | None = ...,
        ispt_main: int | None = ...,
        ispt_npen: int | None = ...,
        ispt_nval: int | None = ...,
        ispt_rep: str | None = ...,
        ispt_cas: float | None = ...,
        ispt_wat: str | None = ...,
        ispt_type: str | None = ...,
        ispt_ham: str | None = ...,
        ispt_erat: int | None = ...,
        ispt_swp: int | None = ...,
        ispt_inc1: int | None = ...,
        ispt_inc2: int | None = ...,
        ispt_inc3: int | None = ...,
        ispt_inc4: int | None = ...,
        ispt_inc5: int | None = ...,
        ispt_inc6: int | None = ...,
        ispt_pen1: int | None = ...,
        ispt_pen2: int | None = ...,
        ispt_pen3: int | None = ...,
        ispt_pen4: int | None = ...,
        ispt_pen5: int | None = ...,
        ispt_pen6: int | None = ...,
        ispt_rock: bool | None = ...,
        ispt_rem: str | None = ...,
        ispt_env: str | None = ...,
        ispt_meth: str | None = ...,
        ispt_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        ispt_n60: int | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ISTA:
    loca_id: str | None
    istg_tesn: str | None
    ista_top: float | None
    ista_base: float | None
    ista_anyn: str | None
    ista_dpth: float | None
    ista_rect: float | None
    ista_recb: float | None
    ista_rcom: str | None
    ista_mivl: str | None
    ista_wvty: str | None
    ista_upsr: float | None
    ista_ftu: str | None
    ista_fmin: int | None
    ista_fmax: int | None
    ista_watt: float | None
    ista_watb: float | None
    ista_watm: str | None
    ista_itm: str | None
    ista_wvl: float | None
    ista_wvlm: str | None
    ista_stac: bool | None
    ista_ival: bool | None
    ista_rem: str | None
    ista_anby: str | None
    ista_cont: str | None
    ista_date: _dt.datetime | None
    test_stat: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        istg_tesn: str | None = ...,
        ista_top: float | None = ...,
        ista_base: float | None = ...,
        ista_anyn: str | None = ...,
        ista_dpth: float | None = ...,
        ista_rect: float | None = ...,
        ista_recb: float | None = ...,
        ista_rcom: str | None = ...,
        ista_mivl: str | None = ...,
        ista_wvty: str | None = ...,
        ista_upsr: float | None = ...,
        ista_ftu: str | None = ...,
        ista_fmin: int | None = ...,
        ista_fmax: int | None = ...,
        ista_watt: float | None = ...,
        ista_watb: float | None = ...,
        ista_watm: str | None = ...,
        ista_itm: str | None = ...,
        ista_wvl: float | None = ...,
        ista_wvlm: str | None = ...,
        ista_stac: bool | None = ...,
        ista_ival: bool | None = ...,
        ista_rem: str | None = ...,
        ista_anby: str | None = ...,
        ista_cont: str | None = ...,
        ista_date: _dt.datetime | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ISTG:
    loca_id: str | None
    istg_tesn: str | None
    istg_type: str | None
    istg_link: float | None
    istg_star: _dt.datetime | None
    istg_end: _dt.datetime | None
    istg_ref: str | None
    istg_recc: str | None
    istg_recd: str | None
    istg_sour: str | None
    istg_rord: str | None
    istg_shof: float | None
    istg_ornt: int | None
    istg_svof: float | None
    istg_otop: float | None
    istg_obot: float | None
    istg_bhcp: str | None
    istg_mto: str | None
    istg_oper: str | None
    istg_anby: str | None
    istg_rem: str | None
    istg_env: str | None
    istg_meth: str | None
    istg_cont: str | None
    istg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    istas: list[ISTA]
    istss: list[ISTS]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        istg_tesn: str | None = ...,
        istg_type: str | None = ...,
        istg_link: float | None = ...,
        istg_star: _dt.datetime | None = ...,
        istg_end: _dt.datetime | None = ...,
        istg_ref: str | None = ...,
        istg_recc: str | None = ...,
        istg_recd: str | None = ...,
        istg_sour: str | None = ...,
        istg_rord: str | None = ...,
        istg_shof: float | None = ...,
        istg_ornt: int | None = ...,
        istg_svof: float | None = ...,
        istg_otop: float | None = ...,
        istg_obot: float | None = ...,
        istg_bhcp: str | None = ...,
        istg_mto: str | None = ...,
        istg_oper: str | None = ...,
        istg_anby: str | None = ...,
        istg_rem: str | None = ...,
        istg_env: str | None = ...,
        istg_meth: str | None = ...,
        istg_cont: str | None = ...,
        istg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        istas: list[ISTA] | None = ...,
        istss: list[ISTS] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ISTR:
    loca_id: str | None
    istg_tesn: str | None
    ists_sgln: str | None
    istr_dpth: float | None
    istr_ref: str | None
    istr_ssd: float | None
    istr_qual: str | None
    istr_quam: str | None
    istr_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        istg_tesn: str | None = ...,
        ists_sgln: str | None = ...,
        istr_dpth: float | None = ...,
        istr_ref: str | None = ...,
        istr_ssd: float | None = ...,
        istr_qual: str | None = ...,
        istr_quam: str | None = ...,
        istr_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ISTS:
    loca_id: str | None
    istg_tesn: str | None
    ists_sgln: str | None
    ists_type: str | None
    ists_dtim: _dt.datetime | None
    ists_rate: float | None
    ists_ptrt: float | None
    ists_ttly: float | None
    ists_rem: str | None
    file_fset: str | None
    istrs: list[ISTR]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        istg_tesn: str | None = ...,
        ists_sgln: str | None = ...,
        ists_type: str | None = ...,
        ists_dtim: _dt.datetime | None = ...,
        ists_rate: float | None = ...,
        ists_ptrt: float | None = ...,
        ists_ttly: float | None = ...,
        ists_rem: str | None = ...,
        file_fset: str | None = ...,
        istrs: list[ISTR] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ITCH:
    loca_id: str | None
    itch_dpth: float | None
    itch_tesn: str | None
    itch_date: _dt.datetime | None
    itch_tcon: float | None
    itch_tres: float | None
    itch_temp: int | None
    itch_rem: str | None
    itch_env: str | None
    itch_meth: str | None
    itch_oper: str | None
    itch_cont: str | None
    itch_cred: str | None
    test_stat: str | None
    geol_stat: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        itch_dpth: float | None = ...,
        itch_tesn: str | None = ...,
        itch_date: _dt.datetime | None = ...,
        itch_tcon: float | None = ...,
        itch_tres: float | None = ...,
        itch_temp: int | None = ...,
        itch_rem: str | None = ...,
        itch_env: str | None = ...,
        itch_meth: str | None = ...,
        itch_oper: str | None = ...,
        itch_cont: str | None = ...,
        itch_cred: str | None = ...,
        test_stat: str | None = ...,
        geol_stat: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class IVAN:
    loca_id: str | None
    ivan_dpth: float | None
    ivan_tesn: str | None
    ivan_type: str | None
    ivan_ivan: str | None
    ivan_ivar: str | None
    ivan_date: _dt.datetime | None
    ivan_rem: str | None
    ivan_env: str | None
    ivan_meth: str | None
    ivan_cont: str | None
    ivan_cred: str | None
    test_stat: str | None
    geol_stat: str | None
    file_fset: str | None
    ivan_oper: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        ivan_dpth: float | None = ...,
        ivan_tesn: str | None = ...,
        ivan_type: str | None = ...,
        ivan_ivan: str | None = ...,
        ivan_ivar: str | None = ...,
        ivan_date: _dt.datetime | None = ...,
        ivan_rem: str | None = ...,
        ivan_env: str | None = ...,
        ivan_meth: str | None = ...,
        ivan_cont: str | None = ...,
        ivan_cred: str | None = ...,
        test_stat: str | None = ...,
        geol_stat: str | None = ...,
        file_fset: str | None = ...,
        ivan_oper: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LBSG:
    lbsg_ref: str | None
    lbsg_date: _dt.datetime | None
    lbsg_from: str | None
    lbsg_to: str | None
    lbsg_due: _dt.datetime | None
    lbsg_rem: str | None
    lbsg_stat: str | None
    file_fset: str | None
    lbsts: list[LBST]
    def __init__(
        self,
        *,
        lbsg_ref: str | None = ...,
        lbsg_date: _dt.datetime | None = ...,
        lbsg_from: str | None = ...,
        lbsg_to: str | None = ...,
        lbsg_due: _dt.datetime | None = ...,
        lbsg_rem: str | None = ...,
        lbsg_stat: str | None = ...,
        file_fset: str | None = ...,
        lbsts: list[LBST] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LBST:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    lbsg_ref: str | None
    lbst_test: str | None
    choc_ref: str | None
    lbst_ttyp: str | None
    lbst_meth: str | None
    lbst_prep: str | None
    lbst_depn: str | None
    lbst_stat: str | None
    lbst_rem: str | None
    lbst_due: _dt.datetime | None
    lbst_detl: str | None
    lbst_done: _dt.datetime | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        lbsg_ref: str | None = ...,
        lbst_test: str | None = ...,
        choc_ref: str | None = ...,
        lbst_ttyp: str | None = ...,
        lbst_meth: str | None = ...,
        lbst_prep: str | None = ...,
        lbst_depn: str | None = ...,
        lbst_stat: str | None = ...,
        lbst_rem: str | None = ...,
        lbst_due: _dt.datetime | None = ...,
        lbst_detl: str | None = ...,
        lbst_done: _dt.datetime | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LDEN:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    lden_type: str | None
    lden_cond: str | None
    lden_smty: str | None
    lden_mc: str | None
    lden_bden: float | None
    lden_dden: float | None
    lden_rem: str | None
    lden_meth: str | None
    lden_lab: str | None
    lden_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    lden_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        lden_type: str | None = ...,
        lden_cond: str | None = ...,
        lden_smty: str | None = ...,
        lden_mc: str | None = ...,
        lden_bden: float | None = ...,
        lden_dden: float | None = ...,
        lden_rem: str | None = ...,
        lden_meth: str | None = ...,
        lden_lab: str | None = ...,
        lden_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        lden_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LDYN:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    ldyn_pwav: int | None
    ldyn_swav: int | None
    ldyn_emod: int | None
    ldyn_sg: int | None
    ldyn_rem: str | None
    ldyn_meth: str | None
    ldyn_lab: str | None
    ldyn_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    ldyn_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        ldyn_pwav: int | None = ...,
        ldyn_swav: int | None = ...,
        ldyn_emod: int | None = ...,
        ldyn_sg: int | None = ...,
        ldyn_rem: str | None = ...,
        ldyn_meth: str | None = ...,
        ldyn_lab: str | None = ...,
        ldyn_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        ldyn_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LFCN:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    spec_base: float | None
    lfcn_dev: str | None
    lfcn_cmas: int | None
    lfcn_cang: int | None
    lfcn_pena: float | None
    lfcn_pen1: float | None
    lfcn_pen2: float | None
    lfcn_pen3: float | None
    lfcn_pen4: float | None
    lfcn_conf: bool | None
    lfcn_fcpk: float | None
    lfcn_fcrm: float | None
    lfcn_wc: str | None
    lfcn_wcst: str | None
    lfcn_rem: str | None
    lfcn_meth: str | None
    lfcn_lab: str | None
    lfcn_cred: str | None
    test_stat: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        spec_base: float | None = ...,
        lfcn_dev: str | None = ...,
        lfcn_cmas: int | None = ...,
        lfcn_cang: int | None = ...,
        lfcn_pena: float | None = ...,
        lfcn_pen1: float | None = ...,
        lfcn_pen2: float | None = ...,
        lfcn_pen3: float | None = ...,
        lfcn_pen4: float | None = ...,
        lfcn_conf: bool | None = ...,
        lfcn_fcpk: float | None = ...,
        lfcn_fcrm: float | None = ...,
        lfcn_wc: str | None = ...,
        lfcn_wcst: str | None = ...,
        lfcn_rem: str | None = ...,
        lfcn_meth: str | None = ...,
        lfcn_lab: str | None = ...,
        lfcn_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LLIN:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    llin_ls: int | None
    llin_425: int | None
    llin_prep: str | None
    llin_rem: str | None
    llin_meth: str | None
    llin_lab: str | None
    llin_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    llin_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        llin_ls: int | None = ...,
        llin_425: int | None = ...,
        llin_prep: str | None = ...,
        llin_rem: str | None = ...,
        llin_meth: str | None = ...,
        llin_lab: str | None = ...,
        llin_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        llin_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LLPL:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    llpl_ll: int | None
    llpl_pl: str | None
    llpl_pi: int | None
    llpl_425: int | None
    llpl_prep: str | None
    llpl_stab: float | None
    llpl_styp: str | None
    llpl_rem: str | None
    llpl_meth: str | None
    llpl_lab: str | None
    llpl_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    llpl_dev: str | None
    llpl_type: str | None
    llpl_poin: str | None
    llpl_cone: str | None
    llpl_1pre: float | None
    llpl_1pcf: float | None
    llpl_size: str | None
    llpl_pass: float | None
    llpl_wc: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        llpl_ll: int | None = ...,
        llpl_pl: str | None = ...,
        llpl_pi: int | None = ...,
        llpl_425: int | None = ...,
        llpl_prep: str | None = ...,
        llpl_stab: float | None = ...,
        llpl_styp: str | None = ...,
        llpl_rem: str | None = ...,
        llpl_meth: str | None = ...,
        llpl_lab: str | None = ...,
        llpl_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        llpl_dev: str | None = ...,
        llpl_type: str | None = ...,
        llpl_poin: str | None = ...,
        llpl_cone: str | None = ...,
        llpl_1pre: float | None = ...,
        llpl_1pcf: float | None = ...,
        llpl_size: str | None = ...,
        llpl_pass: float | None = ...,
        llpl_wc: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LNMC:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    lnmc_mc: str | None
    lnmc_temp: int | None
    lnmc_stab: float | None
    lnmc_styp: str | None
    lnmc_isnt: bool | None
    lnmc_comm: str | None
    lnmc_rem: str | None
    lnmc_meth: str | None
    lnmc_lab: str | None
    lnmc_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    lnmc_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        lnmc_mc: str | None = ...,
        lnmc_temp: int | None = ...,
        lnmc_stab: float | None = ...,
        lnmc_styp: str | None = ...,
        lnmc_isnt: bool | None = ...,
        lnmc_comm: str | None = ...,
        lnmc_rem: str | None = ...,
        lnmc_meth: str | None = ...,
        lnmc_lab: str | None = ...,
        lnmc_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        lnmc_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LOCA:
    loca_id: str | None
    loca_type: str | None
    loca_stat: str | None
    loca_nate: float | None
    loca_natn: float | None
    loca_gref: str | None
    loca_gl: float | None
    loca_rem: str | None
    loca_fdep: float | None
    loca_star: _dt.datetime | None
    loca_purp: str | None
    loca_term: str | None
    loca_endd: _dt.datetime | None
    loca_lett: str | None
    loca_locx: float | None
    loca_locy: float | None
    loca_locz: float | None
    loca_lref: str | None
    loca_datm: str | None
    loca_etrv: float | None
    loca_ntrv: float | None
    loca_ltrv: float | None
    loca_xtrl: float | None
    loca_ytrl: float | None
    loca_ztrl: float | None
    loca_lat: str | None
    loca_lon: str | None
    loca_elat: str | None
    loca_elon: str | None
    loca_llz: str | None
    loca_locm: str | None
    loca_loca: str | None
    loca_clst: str | None
    loca_alid: str | None
    loca_offs: float | None
    loca_cnge: str | None
    loca_tran: str | None
    file_fset: str | None
    loca_natd: str | None
    loca_orid: str | None
    loca_orjo: str | None
    loca_orco: str | None
    loca_gldt: _dt.datetime | None
    loca_vssl: str | None
    loca_nsri: int | None
    loca_lsri: int | None
    loca_llsi: int | None
    bkfls: list[BKFL]
    cdias: list[CDIA]
    chiss: list[CHIS]
    cores: list[CORE]
    cptgs: list[CPTG]
    cptms: list[CPTM]
    cptps: list[CPTP]
    dcpgs: list[DCPG]
    detls: list[DETL]
    discs: list[DISC]
    dlogs: list[DLOG]
    dmtgs: list[DMTG]
    dobss: list[DOBS]
    dprgs: list[DPRG]
    drems: list[DREM]
    fghgs: list[FGHG]
    flshs: list[FLSH]
    fracs: list[FRAC]
    geols: list[GEOL]
    hdias: list[HDIA]
    hdphs: list[HDPH]
    horns: list[HORN]
    icbrs: list[ICBR]
    idens: list[IDEN]
    ifids: list[IFID]
    ipens: list[IPEN]
    ipids: list[IPID]
    iprgs: list[IPRG]
    irdxs: list[IRDX]
    iress: list[IRES]
    isags: list[ISAG]
    ispts: list[ISPT]
    istgs: list[ISTG]
    itchs: list[ITCH]
    ivans: list[IVAN]
    mongs: list[MONG]
    pipes: list[PIPE]
    pltgs: list[PLTG]
    pmmgs: list[PMMG]
    pmtgs: list[PMTG]
    ptims: list[PTIM]
    pumgs: list[PUMG]
    samps: list[SAMP]
    scpgs: list[SCPG]
    trems: list[TREM]
    wadds: list[WADD]
    weths: list[WETH]
    wgpgs: list[WGPG]
    winss: list[WINS]
    wstgs: list[WSTG]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        loca_type: str | None = ...,
        loca_stat: str | None = ...,
        loca_nate: float | None = ...,
        loca_natn: float | None = ...,
        loca_gref: str | None = ...,
        loca_gl: float | None = ...,
        loca_rem: str | None = ...,
        loca_fdep: float | None = ...,
        loca_star: _dt.datetime | None = ...,
        loca_purp: str | None = ...,
        loca_term: str | None = ...,
        loca_endd: _dt.datetime | None = ...,
        loca_lett: str | None = ...,
        loca_locx: float | None = ...,
        loca_locy: float | None = ...,
        loca_locz: float | None = ...,
        loca_lref: str | None = ...,
        loca_datm: str | None = ...,
        loca_etrv: float | None = ...,
        loca_ntrv: float | None = ...,
        loca_ltrv: float | None = ...,
        loca_xtrl: float | None = ...,
        loca_ytrl: float | None = ...,
        loca_ztrl: float | None = ...,
        loca_lat: str | None = ...,
        loca_lon: str | None = ...,
        loca_elat: str | None = ...,
        loca_elon: str | None = ...,
        loca_llz: str | None = ...,
        loca_locm: str | None = ...,
        loca_loca: str | None = ...,
        loca_clst: str | None = ...,
        loca_alid: str | None = ...,
        loca_offs: float | None = ...,
        loca_cnge: str | None = ...,
        loca_tran: str | None = ...,
        file_fset: str | None = ...,
        loca_natd: str | None = ...,
        loca_orid: str | None = ...,
        loca_orjo: str | None = ...,
        loca_orco: str | None = ...,
        loca_gldt: _dt.datetime | None = ...,
        loca_vssl: str | None = ...,
        loca_nsri: int | None = ...,
        loca_lsri: int | None = ...,
        loca_llsi: int | None = ...,
        bkfls: list[BKFL] | None = ...,
        cdias: list[CDIA] | None = ...,
        chiss: list[CHIS] | None = ...,
        cores: list[CORE] | None = ...,
        cptgs: list[CPTG] | None = ...,
        cptms: list[CPTM] | None = ...,
        cptps: list[CPTP] | None = ...,
        dcpgs: list[DCPG] | None = ...,
        detls: list[DETL] | None = ...,
        discs: list[DISC] | None = ...,
        dlogs: list[DLOG] | None = ...,
        dmtgs: list[DMTG] | None = ...,
        dobss: list[DOBS] | None = ...,
        dprgs: list[DPRG] | None = ...,
        drems: list[DREM] | None = ...,
        fghgs: list[FGHG] | None = ...,
        flshs: list[FLSH] | None = ...,
        fracs: list[FRAC] | None = ...,
        geols: list[GEOL] | None = ...,
        hdias: list[HDIA] | None = ...,
        hdphs: list[HDPH] | None = ...,
        horns: list[HORN] | None = ...,
        icbrs: list[ICBR] | None = ...,
        idens: list[IDEN] | None = ...,
        ifids: list[IFID] | None = ...,
        ipens: list[IPEN] | None = ...,
        ipids: list[IPID] | None = ...,
        iprgs: list[IPRG] | None = ...,
        irdxs: list[IRDX] | None = ...,
        iress: list[IRES] | None = ...,
        isags: list[ISAG] | None = ...,
        ispts: list[ISPT] | None = ...,
        istgs: list[ISTG] | None = ...,
        itchs: list[ITCH] | None = ...,
        ivans: list[IVAN] | None = ...,
        mongs: list[MONG] | None = ...,
        pipes: list[PIPE] | None = ...,
        pltgs: list[PLTG] | None = ...,
        pmmgs: list[PMMG] | None = ...,
        pmtgs: list[PMTG] | None = ...,
        ptims: list[PTIM] | None = ...,
        pumgs: list[PUMG] | None = ...,
        samps: list[SAMP] | None = ...,
        scpgs: list[SCPG] | None = ...,
        trems: list[TREM] | None = ...,
        wadds: list[WADD] | None = ...,
        weths: list[WETH] | None = ...,
        wgpgs: list[WGPG] | None = ...,
        winss: list[WINS] | None = ...,
        wstgs: list[WSTG] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LPDN:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    lpdn_pden: str | None
    lpdn_type: str | None
    lpdn_rem: str | None
    lpdn_meth: str | None
    lpdn_lab: str | None
    lpdn_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    lpdn_dev: str | None
    lpdn_pvol: int | None
    lpdn_gas: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        lpdn_pden: str | None = ...,
        lpdn_type: str | None = ...,
        lpdn_rem: str | None = ...,
        lpdn_meth: str | None = ...,
        lpdn_lab: str | None = ...,
        lpdn_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        lpdn_dev: str | None = ...,
        lpdn_pvol: int | None = ...,
        lpdn_gas: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LPEN:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    lpen_ppen: int | None
    lpen_mc: str | None
    lpen_rem: str | None
    lpen_meth: str | None
    lpen_lab: str | None
    lpen_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    lpen_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        lpen_ppen: int | None = ...,
        lpen_mc: str | None = ...,
        lpen_rem: str | None = ...,
        lpen_meth: str | None = ...,
        lpen_lab: str | None = ...,
        lpen_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        lpen_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LRES:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    lres_bden: float | None
    lres_dden: float | None
    lres_mc: str | None
    lres_cond: str | None
    lres_lres: int | None
    lres_cdia: int | None
    lres_ccsa: int | None
    lres_clen: int | None
    lres_temp: int | None
    lres_elec: str | None
    lres_pent: str | None
    lres_cshp: str | None
    lres_wat: int | None
    lres_wres: float | None
    lres_part: str | None
    lres_rem: str | None
    lres_meth: str | None
    lres_lab: str | None
    lres_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    lres_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        lres_bden: float | None = ...,
        lres_dden: float | None = ...,
        lres_mc: str | None = ...,
        lres_cond: str | None = ...,
        lres_lres: int | None = ...,
        lres_cdia: int | None = ...,
        lres_ccsa: int | None = ...,
        lres_clen: int | None = ...,
        lres_temp: int | None = ...,
        lres_elec: str | None = ...,
        lres_pent: str | None = ...,
        lres_cshp: str | None = ...,
        lres_wat: int | None = ...,
        lres_wres: float | None = ...,
        lres_part: str | None = ...,
        lres_rem: str | None = ...,
        lres_meth: str | None = ...,
        lres_lab: str | None = ...,
        lres_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        lres_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LSLT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    lslt_slim: float | None
    lslt_shra: int | None
    lslt_iden: float | None
    lslt_mci: str | None
    lslt_425: int | None
    lslt_rem: str | None
    lslt_meth: str | None
    lslt_lab: str | None
    lslt_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    lslt_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        lslt_slim: float | None = ...,
        lslt_shra: int | None = ...,
        lslt_iden: float | None = ...,
        lslt_mci: str | None = ...,
        lslt_425: int | None = ...,
        lslt_rem: str | None = ...,
        lslt_meth: str | None = ...,
        lslt_lab: str | None = ...,
        lslt_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        lslt_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LSTG:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    lstg_icl: float | None
    lstg_ph: float | None
    lstg_lime: str | None
    lstg_suit: float | None
    lstg_425: float | None
    lstg_rem: str | None
    lstg_meth: str | None
    lstg_lab: str | None
    lstg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    lstg_dev: str | None
    lstts: list[LSTT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        lstg_icl: float | None = ...,
        lstg_ph: float | None = ...,
        lstg_lime: str | None = ...,
        lstg_suit: float | None = ...,
        lstg_425: float | None = ...,
        lstg_rem: str | None = ...,
        lstg_meth: str | None = ...,
        lstg_lab: str | None = ...,
        lstg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        lstg_dev: str | None = ...,
        lstts: list[LSTT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LSTT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    lstt_tesn: str | None
    lstt_lcon: float | None
    lstt_ph: float | None
    lstt_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        lstt_tesn: str | None = ...,
        lstt_lcon: float | None = ...,
        lstt_ph: float | None = ...,
        lstt_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LSWL:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    lswl_swpr: int | None
    lswl_swsi: float | None
    lswl_mci: float | None
    lswl_sdia: float | None
    lswl_thck: float | None
    lswl_bden: int | None
    lswl_dden: int | None
    lswl_rem: str | None
    lswl_meth: str | None
    lswl_lab: str | None
    lswl_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    lswl_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        lswl_swpr: int | None = ...,
        lswl_swsi: float | None = ...,
        lswl_mci: float | None = ...,
        lswl_sdia: float | None = ...,
        lswl_thck: float | None = ...,
        lswl_bden: int | None = ...,
        lswl_dden: int | None = ...,
        lswl_rem: str | None = ...,
        lswl_meth: str | None = ...,
        lswl_lab: str | None = ...,
        lswl_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        lswl_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LTCH:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    spec_base: float | None
    ltch_cond: str | None
    ltch_bden: float | None
    ltch_dden: float | None
    ltch_mc: str | None
    ltch_tcon: float | None
    ltch_tres: float | None
    ltch_temp: int | None
    ltch_pdia: int | None
    ltch_pspa: int | None
    ltch_ppen: int | None
    ltch_prbe: str | None
    ltch_part: str | None
    ltch_dev: str | None
    ltch_rem: str | None
    ltch_meth: str | None
    ltch_lab: str | None
    ltch_cred: str | None
    test_stat: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        spec_base: float | None = ...,
        ltch_cond: str | None = ...,
        ltch_bden: float | None = ...,
        ltch_dden: float | None = ...,
        ltch_mc: str | None = ...,
        ltch_tcon: float | None = ...,
        ltch_tres: float | None = ...,
        ltch_temp: int | None = ...,
        ltch_pdia: int | None = ...,
        ltch_pspa: int | None = ...,
        ltch_ppen: int | None = ...,
        ltch_prbe: str | None = ...,
        ltch_part: str | None = ...,
        ltch_dev: str | None = ...,
        ltch_rem: str | None = ...,
        ltch_meth: str | None = ...,
        ltch_lab: str | None = ...,
        ltch_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LUCT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    spec_base: float | None
    luct_dev: str | None
    luct_type: str | None
    luct_dia: float | None
    luct_slen: float | None
    luct_iwc: str | None
    luct_bden: float | None
    luct_dden: float | None
    luct_rate: float | None
    luct_ucs: int | None
    luct_stra: float | None
    luct_mode: str | None
    luct_rem: str | None
    luct_meth: str | None
    luct_lab: str | None
    luct_cred: str | None
    test_stat: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        spec_base: float | None = ...,
        luct_dev: str | None = ...,
        luct_type: str | None = ...,
        luct_dia: float | None = ...,
        luct_slen: float | None = ...,
        luct_iwc: str | None = ...,
        luct_bden: float | None = ...,
        luct_dden: float | None = ...,
        luct_rate: float | None = ...,
        luct_ucs: int | None = ...,
        luct_stra: float | None = ...,
        luct_mode: str | None = ...,
        luct_rem: str | None = ...,
        luct_meth: str | None = ...,
        luct_lab: str | None = ...,
        luct_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class LVAN:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    lvan_vnpk: str | None
    lvan_vnrm: str | None
    lvan_mc: str | None
    lvan_size: float | None
    lvan_vlen: float | None
    lvan_rem: str | None
    lvan_meth: str | None
    lvan_lab: str | None
    lvan_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    lvan_dev: str | None
    lvan_type: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        lvan_vnpk: str | None = ...,
        lvan_vnrm: str | None = ...,
        lvan_mc: str | None = ...,
        lvan_size: float | None = ...,
        lvan_vlen: float | None = ...,
        lvan_rem: str | None = ...,
        lvan_meth: str | None = ...,
        lvan_lab: str | None = ...,
        lvan_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        lvan_dev: str | None = ...,
        lvan_type: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class MCVG:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    mcvg_200: int | None
    mcvg_nmc: str | None
    mcvg_stab: float | None
    mcvg_styp: str | None
    mcvg_rem: str | None
    mcvg_meth: str | None
    mcvg_lab: str | None
    mcvg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    mcvg_dev: str | None
    mcvts: list[MCVT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        mcvg_200: int | None = ...,
        mcvg_nmc: str | None = ...,
        mcvg_stab: float | None = ...,
        mcvg_styp: str | None = ...,
        mcvg_rem: str | None = ...,
        mcvg_meth: str | None = ...,
        mcvg_lab: str | None = ...,
        mcvg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        mcvg_dev: str | None = ...,
        mcvts: list[MCVT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class MCVT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    mcvt_tesn: str | None
    mcvt_mc: str | None
    mcvt_curv: str | None
    mcvt_relk: float | None
    mcvt_bden: float | None
    mcvt_diff: float | None
    mcvt_rapd: str | None
    mcvt_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        mcvt_tesn: str | None = ...,
        mcvt_mc: str | None = ...,
        mcvt_curv: str | None = ...,
        mcvt_relk: float | None = ...,
        mcvt_bden: float | None = ...,
        mcvt_diff: float | None = ...,
        mcvt_rapd: str | None = ...,
        mcvt_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class MOND:
    loca_id: str | None
    mong_id: str | None
    mong_dis: float | None
    mond_dtim: _dt.datetime | None
    mond_type: str | None
    mond_ref: str | None
    mond_inst: str | None
    mond_rdng: str | None
    mond_unit: str | None
    mond_meth: str | None
    mond_lim: str | None
    mond_ulim: str | None
    mond_name: str | None
    mond_cred: str | None
    mond_cont: str | None
    mond_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        mong_id: str | None = ...,
        mong_dis: float | None = ...,
        mond_dtim: _dt.datetime | None = ...,
        mond_type: str | None = ...,
        mond_ref: str | None = ...,
        mond_inst: str | None = ...,
        mond_rdng: str | None = ...,
        mond_unit: str | None = ...,
        mond_meth: str | None = ...,
        mond_lim: str | None = ...,
        mond_ulim: str | None = ...,
        mond_name: str | None = ...,
        mond_cred: str | None = ...,
        mond_cont: str | None = ...,
        mond_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class MONG:
    loca_id: str | None
    mong_id: str | None
    mong_dis: float | None
    pipe_ref: str | None
    mong_date: _dt.datetime | None
    mong_type: str | None
    mong_detl: str | None
    mong_trz: float | None
    mong_brz: float | None
    mong_brga: int | None
    mong_brgb: int | None
    mong_brgc: int | None
    mong_inca: int | None
    mong_incb: int | None
    mong_incc: int | None
    mong_rsca: str | None
    mong_rscb: str | None
    mong_rscc: str | None
    mong_rem: str | None
    mong_cont: str | None
    file_fset: str | None
    monds: list[MOND]
    monss: list[MONS]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        mong_id: str | None = ...,
        mong_dis: float | None = ...,
        pipe_ref: str | None = ...,
        mong_date: _dt.datetime | None = ...,
        mong_type: str | None = ...,
        mong_detl: str | None = ...,
        mong_trz: float | None = ...,
        mong_brz: float | None = ...,
        mong_brga: int | None = ...,
        mong_brgb: int | None = ...,
        mong_brgc: int | None = ...,
        mong_inca: int | None = ...,
        mong_incb: int | None = ...,
        mong_incc: int | None = ...,
        mong_rsca: str | None = ...,
        mong_rscb: str | None = ...,
        mong_rscc: str | None = ...,
        mong_rem: str | None = ...,
        mong_cont: str | None = ...,
        file_fset: str | None = ...,
        monds: list[MOND] | None = ...,
        monss: list[MONS] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class MONS:
    loca_id: str | None
    mong_id: str | None
    mong_dis: float | None
    mons_star: _dt.datetime | None
    mons_endd: _dt.datetime | None
    mons_by: str | None
    mons_type: str | None
    mons_stat: str | None
    mons_rplo: str | None
    mons_rpid: str | None
    mons_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        mong_id: str | None = ...,
        mong_dis: float | None = ...,
        mons_star: _dt.datetime | None = ...,
        mons_endd: _dt.datetime | None = ...,
        mons_by: str | None = ...,
        mons_type: str | None = ...,
        mons_stat: str | None = ...,
        mons_rplo: str | None = ...,
        mons_rpid: str | None = ...,
        mons_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PIPE:
    loca_id: str | None
    pipe_ref: str | None
    pipe_top: float | None
    pipe_base: float | None
    pipe_diam: int | None
    pipe_type: str | None
    pipe_cons: str | None
    pipe_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        pipe_ref: str | None = ...,
        pipe_top: float | None = ...,
        pipe_base: float | None = ...,
        pipe_diam: int | None = ...,
        pipe_type: str | None = ...,
        pipe_cons: str | None = ...,
        pipe_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PLTG:
    loca_id: str | None
    pltg_dpth: float | None
    pltg_tesn: str | None
    pltg_cyc: str | None
    pltg_pdia: int | None
    pltg_seat: float | None
    pltg_fa0: float | None
    pltg_fa1: float | None
    pltg_fa2: float | None
    pltg_smod: float | None
    pltg_ev2: float | None
    pltg_mosr: float | None
    pltg_emod: float | None
    pltg_date: _dt.datetime | None
    pltg_stab: float | None
    pltg_styp: str | None
    pltg_rem: str | None
    pltg_env: str | None
    pltg_meth: str | None
    pltg_cont: str | None
    pltg_cred: str | None
    test_stat: str | None
    geol_stat: str | None
    file_fset: str | None
    pltg_oper: str | None
    pltts: list[PLTT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        pltg_dpth: float | None = ...,
        pltg_tesn: str | None = ...,
        pltg_cyc: str | None = ...,
        pltg_pdia: int | None = ...,
        pltg_seat: float | None = ...,
        pltg_fa0: float | None = ...,
        pltg_fa1: float | None = ...,
        pltg_fa2: float | None = ...,
        pltg_smod: float | None = ...,
        pltg_ev2: float | None = ...,
        pltg_mosr: float | None = ...,
        pltg_emod: float | None = ...,
        pltg_date: _dt.datetime | None = ...,
        pltg_stab: float | None = ...,
        pltg_styp: str | None = ...,
        pltg_rem: str | None = ...,
        pltg_env: str | None = ...,
        pltg_meth: str | None = ...,
        pltg_cont: str | None = ...,
        pltg_cred: str | None = ...,
        test_stat: str | None = ...,
        geol_stat: str | None = ...,
        file_fset: str | None = ...,
        pltg_oper: str | None = ...,
        pltts: list[PLTT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PLTT:
    loca_id: str | None
    pltg_dpth: float | None
    pltg_tesn: str | None
    pltg_cyc: str | None
    pltt_stg: str | None
    pltt_time: float | None
    pltt_load: float | None
    pltt_set1: float | None
    pltt_set2: float | None
    pltt_set3: float | None
    pltt_set4: float | None
    pltt_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        pltg_dpth: float | None = ...,
        pltg_tesn: str | None = ...,
        pltg_cyc: str | None = ...,
        pltt_stg: str | None = ...,
        pltt_time: float | None = ...,
        pltt_load: float | None = ...,
        pltt_set1: float | None = ...,
        pltt_set2: float | None = ...,
        pltt_set3: float | None = ...,
        pltt_set4: float | None = ...,
        pltt_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PMMC:
    loca_id: str | None
    pmmg_dpth: float | None
    pmmg_tesn: str | None
    pmmc_cyno: str | None
    pmmc_p1cy: float | None
    pmmc_p2cy: float | None
    pmmc_emcy: float | None
    pmmc_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        pmmg_dpth: float | None = ...,
        pmmg_tesn: str | None = ...,
        pmmc_cyno: str | None = ...,
        pmmc_p1cy: float | None = ...,
        pmmc_p2cy: float | None = ...,
        pmmc_emcy: float | None = ...,
        pmmc_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PMMD:
    loca_id: str | None
    pmmg_dpth: float | None
    pmmg_tesn: str | None
    pmmd_seq: int | None
    pmmd_p01s: float | None
    pmmd_p15s: float | None
    pmmd_p30s: float | None
    pmmd_p60s: float | None
    pmmd_v01s: float | None
    pmmd_v15s: float | None
    pmmd_v30s: float | None
    pmmd_v60s: float | None
    pmmd_cp: float | None
    pmmd_cvol: float | None
    pmmd_slop: int | None
    pmmd_crep: float | None
    pmmd_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        pmmg_dpth: float | None = ...,
        pmmg_tesn: str | None = ...,
        pmmd_seq: int | None = ...,
        pmmd_p01s: float | None = ...,
        pmmd_p15s: float | None = ...,
        pmmd_p30s: float | None = ...,
        pmmd_p60s: float | None = ...,
        pmmd_v01s: float | None = ...,
        pmmd_v15s: float | None = ...,
        pmmd_v30s: float | None = ...,
        pmmd_v60s: float | None = ...,
        pmmd_cp: float | None = ...,
        pmmd_cvol: float | None = ...,
        pmmd_slop: int | None = ...,
        pmmd_crep: float | None = ...,
        pmmd_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PMMG:
    loca_id: str | None
    pmmg_dpth: float | None
    pmmg_tesn: str | None
    pmmg_date: _dt.datetime | None
    pmmg_dcu: float | None
    pmmg_prwl: float | None
    pmmg_ref: str | None
    pmmg_type: str | None
    pmmg_diam: int | None
    pmmg_prc: int | None
    pmmg_tc: str | None
    pmmg_p1: float | None
    pmmg_p2: float | None
    pmmg_em: float | None
    pmmg_mpl: float | None
    pmmg_mplm: str | None
    pmmg_pf: float | None
    pmmg_meth: str | None
    pmmg_crem: str | None
    pmmg_rem: str | None
    pmmg_crdt: _dt.datetime | None
    pmmg_oper: str | None
    pmmg_anby: str | None
    pmmg_cont: str | None
    pmmg_cred: str | None
    test_stat: str | None
    pmmg_env: str | None
    file_fset: str | None
    pmmcs: list[PMMC]
    pmmds: list[PMMD]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        pmmg_dpth: float | None = ...,
        pmmg_tesn: str | None = ...,
        pmmg_date: _dt.datetime | None = ...,
        pmmg_dcu: float | None = ...,
        pmmg_prwl: float | None = ...,
        pmmg_ref: str | None = ...,
        pmmg_type: str | None = ...,
        pmmg_diam: int | None = ...,
        pmmg_prc: int | None = ...,
        pmmg_tc: str | None = ...,
        pmmg_p1: float | None = ...,
        pmmg_p2: float | None = ...,
        pmmg_em: float | None = ...,
        pmmg_mpl: float | None = ...,
        pmmg_mplm: str | None = ...,
        pmmg_pf: float | None = ...,
        pmmg_meth: str | None = ...,
        pmmg_crem: str | None = ...,
        pmmg_rem: str | None = ...,
        pmmg_crdt: _dt.datetime | None = ...,
        pmmg_oper: str | None = ...,
        pmmg_anby: str | None = ...,
        pmmg_cont: str | None = ...,
        pmmg_cred: str | None = ...,
        test_stat: str | None = ...,
        pmmg_env: str | None = ...,
        file_fset: str | None = ...,
        pmmcs: list[PMMC] | None = ...,
        pmmds: list[PMMD] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PMTD:
    loca_id: str | None
    pmtg_dpth: float | None
    pmtg_tesn: str | None
    pmtd_seq: int | None
    pmtd_tpc: float | None
    pmtd_ppa: float | None
    pmtd_ppb: float | None
    pmtd_vol: float | None
    pmtd_rem: str | None
    file_fset: str | None
    pmtd_ax1: float | None
    pmtd_ax2: float | None
    pmtd_ax3: float | None
    pmtd_sa1: float | None
    pmtd_sa2: float | None
    pmtd_sa3: float | None
    pmtd_sa4: float | None
    pmtd_sa5: float | None
    pmtd_sa6: float | None
    pmtd_same: float | None
    pmtd_time: int | None
    pmtd_arm1: float | None
    pmtd_arm2: float | None
    pmtd_arm3: float | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        pmtg_dpth: float | None = ...,
        pmtg_tesn: str | None = ...,
        pmtd_seq: int | None = ...,
        pmtd_tpc: float | None = ...,
        pmtd_ppa: float | None = ...,
        pmtd_ppb: float | None = ...,
        pmtd_vol: float | None = ...,
        pmtd_rem: str | None = ...,
        file_fset: str | None = ...,
        pmtd_ax1: float | None = ...,
        pmtd_ax2: float | None = ...,
        pmtd_ax3: float | None = ...,
        pmtd_sa1: float | None = ...,
        pmtd_sa2: float | None = ...,
        pmtd_sa3: float | None = ...,
        pmtd_sa4: float | None = ...,
        pmtd_sa5: float | None = ...,
        pmtd_sa6: float | None = ...,
        pmtd_same: float | None = ...,
        pmtd_time: int | None = ...,
        pmtd_arm1: float | None = ...,
        pmtd_arm2: float | None = ...,
        pmtd_arm3: float | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PMTG:
    loca_id: str | None
    pmtg_dpth: float | None
    pmtg_tesn: str | None
    pmtg_date: _dt.datetime | None
    pmtg_wat: float | None
    pmtg_cont: str | None
    pmtg_crew: str | None
    pmtg_ref: str | None
    pmtg_type: str | None
    pmtg_diam: float | None
    pmtg_ho: int | None
    pmtg_gi: int | None
    pmtg_cu: int | None
    pmtg_pl: int | None
    pmtg_af: float | None
    pmtg_ad: int | None
    pmtg_afcv: float | None
    pmtg_meth: str | None
    pmtg_cred: str | None
    test_stat: str | None
    pmtg_env: str | None
    pmtg_rem: str | None
    file_fset: str | None
    pmtg_nuar: int | None
    pmtg_ornt: int | None
    pmtg_axis: str | None
    pmtg_prwl: float | None
    pmtg_tc: str | None
    pmtg_stad: _dt.datetime | None
    pmtg_endd: _dt.datetime | None
    pmtg_topp: float | None
    pmtg_botp: float | None
    pmtg_sbht: str | None
    pmtg_sbcs: float | None
    pmtg_sbct: str | None
    pmtg_sbcd: float | None
    pmtg_sbcp: int | None
    pmtg_flft: str | None
    pmtg_flfp: int | None
    pmtg_trst: int | None
    pmtg_pprd: bool | None
    pmtg_cmt: str | None
    pmtg_crem: str | None
    pmtg_crdt: _dt.datetime | None
    pmtg_anby: str | None
    pmtds: list[PMTD]
    pmtls: list[PMTL]
    pmtps: list[PMTP]
    pmtzs: list[PMTZ]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        pmtg_dpth: float | None = ...,
        pmtg_tesn: str | None = ...,
        pmtg_date: _dt.datetime | None = ...,
        pmtg_wat: float | None = ...,
        pmtg_cont: str | None = ...,
        pmtg_crew: str | None = ...,
        pmtg_ref: str | None = ...,
        pmtg_type: str | None = ...,
        pmtg_diam: float | None = ...,
        pmtg_ho: int | None = ...,
        pmtg_gi: int | None = ...,
        pmtg_cu: int | None = ...,
        pmtg_pl: int | None = ...,
        pmtg_af: float | None = ...,
        pmtg_ad: int | None = ...,
        pmtg_afcv: float | None = ...,
        pmtg_meth: str | None = ...,
        pmtg_cred: str | None = ...,
        test_stat: str | None = ...,
        pmtg_env: str | None = ...,
        pmtg_rem: str | None = ...,
        file_fset: str | None = ...,
        pmtg_nuar: int | None = ...,
        pmtg_ornt: int | None = ...,
        pmtg_axis: str | None = ...,
        pmtg_prwl: float | None = ...,
        pmtg_tc: str | None = ...,
        pmtg_stad: _dt.datetime | None = ...,
        pmtg_endd: _dt.datetime | None = ...,
        pmtg_topp: float | None = ...,
        pmtg_botp: float | None = ...,
        pmtg_sbht: str | None = ...,
        pmtg_sbcs: float | None = ...,
        pmtg_sbct: str | None = ...,
        pmtg_sbcd: float | None = ...,
        pmtg_sbcp: int | None = ...,
        pmtg_flft: str | None = ...,
        pmtg_flfp: int | None = ...,
        pmtg_trst: int | None = ...,
        pmtg_pprd: bool | None = ...,
        pmtg_cmt: str | None = ...,
        pmtg_crem: str | None = ...,
        pmtg_crdt: _dt.datetime | None = ...,
        pmtg_anby: str | None = ...,
        pmtds: list[PMTD] | None = ...,
        pmtls: list[PMTL] | None = ...,
        pmtps: list[PMTP] | None = ...,
        pmtzs: list[PMTZ] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PMTL:
    loca_id: str | None
    pmtg_dpth: float | None
    pmtg_tesn: str | None
    pmtl_lno: int | None
    pmtl_gaa: float | None
    pmtl_sinc: float | None
    pmtl_pinc: int | None
    pmtl_stra: float | None
    pmtl_prsa: int | None
    pmtl_nlsa: float | None
    pmtl_nlsb: float | None
    pmtl_rem: str | None
    file_fset: str | None
    pmtl_axis: str | None
    pmtl_hp: int | None
    pmtl_ht: int | None
    pmtl_cr: float | None
    pmtd_seq: int | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        pmtg_dpth: float | None = ...,
        pmtg_tesn: str | None = ...,
        pmtl_lno: int | None = ...,
        pmtl_gaa: float | None = ...,
        pmtl_sinc: float | None = ...,
        pmtl_pinc: int | None = ...,
        pmtl_stra: float | None = ...,
        pmtl_prsa: int | None = ...,
        pmtl_nlsa: float | None = ...,
        pmtl_nlsb: float | None = ...,
        pmtl_rem: str | None = ...,
        file_fset: str | None = ...,
        pmtl_axis: str | None = ...,
        pmtl_hp: int | None = ...,
        pmtl_ht: int | None = ...,
        pmtl_cr: float | None = ...,
        pmtd_seq: int | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PMTP:
    loca_id: str | None
    pmtg_dpth: float | None
    pmtg_tesn: str | None
    pmtp_u0: int | None
    pmtp_sto: float | None
    pmtp_ho: int | None
    pmtp_hom: str | None
    pmtp_gi: float | None
    pmtp_su: float | None
    pmtp_sum: str | None
    pmtp_af: float | None
    pmtp_ad: float | None
    pmtp_afdm: str | None
    pmtp_afcv: float | None
    pmtp_dc: int | None
    pmtp_dcm: str | None
    pmtp_pl: int | None
    pmtp_pf: int | None
    pmtp_pfm: str | None
    pmtp_ym: float | None
    pmtp_ymm: str | None
    pmtp_mu: float | None
    pmtp_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        pmtg_dpth: float | None = ...,
        pmtg_tesn: str | None = ...,
        pmtp_u0: int | None = ...,
        pmtp_sto: float | None = ...,
        pmtp_ho: int | None = ...,
        pmtp_hom: str | None = ...,
        pmtp_gi: float | None = ...,
        pmtp_su: float | None = ...,
        pmtp_sum: str | None = ...,
        pmtp_af: float | None = ...,
        pmtp_ad: float | None = ...,
        pmtp_afdm: str | None = ...,
        pmtp_afcv: float | None = ...,
        pmtp_dc: int | None = ...,
        pmtp_dcm: str | None = ...,
        pmtp_pl: int | None = ...,
        pmtp_pf: int | None = ...,
        pmtp_pfm: str | None = ...,
        pmtp_ym: float | None = ...,
        pmtp_ymm: str | None = ...,
        pmtp_mu: float | None = ...,
        pmtp_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PMTZ:
    loca_id: str | None
    pmtg_dpth: float | None
    pmtg_tesn: str | None
    pmtz_parm: str | None
    pmtz_mrs: str | None
    pmtz_zc: str | None
    pmtz_zb: str | None
    pmtz_zh: str | None
    pmtz_za: str | None
    pmtz_zd: str | None
    pmtz_egut: str | None
    pmtz_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        pmtg_dpth: float | None = ...,
        pmtg_tesn: str | None = ...,
        pmtz_parm: str | None = ...,
        pmtz_mrs: str | None = ...,
        pmtz_zc: str | None = ...,
        pmtz_zb: str | None = ...,
        pmtz_zh: str | None = ...,
        pmtz_za: str | None = ...,
        pmtz_zd: str | None = ...,
        pmtz_egut: str | None = ...,
        pmtz_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PREM:
    prem_dtim: _dt.datetime | None
    prem_comp: str | None
    prem_rem: str | None
    prem_durn: str | None
    prem_etim: _dt.datetime | None
    file_fset: str | None
    def __init__(
        self,
        *,
        prem_dtim: _dt.datetime | None = ...,
        prem_comp: str | None = ...,
        prem_rem: str | None = ...,
        prem_durn: str | None = ...,
        prem_etim: _dt.datetime | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PROJ:
    proj_id: str | None
    proj_name: str | None
    proj_loc: str | None
    proj_clnt: str | None
    proj_cont: str | None
    proj_eng: str | None
    proj_memo: str | None
    file_fset: str | None
    locas: list[LOCA]
    def __init__(
        self,
        *,
        proj_id: str | None = ...,
        proj_name: str | None = ...,
        proj_loc: str | None = ...,
        proj_clnt: str | None = ...,
        proj_cont: str | None = ...,
        proj_eng: str | None = ...,
        proj_memo: str | None = ...,
        file_fset: str | None = ...,
        locas: list[LOCA] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PTIM:
    loca_id: str | None
    ptim_dtim: _dt.datetime | None
    ptim_dpth: float | None
    ptim_cas: float | None
    ptim_wat: str | None
    ptim_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        ptim_dtim: _dt.datetime | None = ...,
        ptim_dpth: float | None = ...,
        ptim_cas: float | None = ...,
        ptim_wat: str | None = ...,
        ptim_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PTST:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    ptst_tesn: str | None
    spec_desc: str | None
    spec_prep: str | None
    ptst_cond: str | None
    ptst_szun: int | None
    ptst_uns: int | None
    ptst_diam: float | None
    ptst_len: float | None
    ptst_mc: str | None
    ptst_bden: float | None
    ptst_dden: float | None
    ptst_idia: float | None
    ptst_dmet: str | None
    ptst_void: float | None
    ptst_k: float | None
    ptst_tstr: int | None
    ptst_hygr: int | None
    ptst_isat: float | None
    ptst_sat: str | None
    ptst_cons: str | None
    ptst_pden: str | None
    ptst_type: str | None
    ptst_cell: str | None
    ptst_rem: str | None
    ptst_meth: str | None
    ptst_lab: str | None
    ptst_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    ptst_dev: str | None
    ptst_wcis: str | None
    ptst_wcf: str | None
    ptst_fsat: float | None
    ptst_temp: float | None
    ptst_sour: str | None
    ptst_back: int | None
    ptst_bval: float | None
    ptst_loss: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        ptst_tesn: str | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        ptst_cond: str | None = ...,
        ptst_szun: int | None = ...,
        ptst_uns: int | None = ...,
        ptst_diam: float | None = ...,
        ptst_len: float | None = ...,
        ptst_mc: str | None = ...,
        ptst_bden: float | None = ...,
        ptst_dden: float | None = ...,
        ptst_idia: float | None = ...,
        ptst_dmet: str | None = ...,
        ptst_void: float | None = ...,
        ptst_k: float | None = ...,
        ptst_tstr: int | None = ...,
        ptst_hygr: int | None = ...,
        ptst_isat: float | None = ...,
        ptst_sat: str | None = ...,
        ptst_cons: str | None = ...,
        ptst_pden: str | None = ...,
        ptst_type: str | None = ...,
        ptst_cell: str | None = ...,
        ptst_rem: str | None = ...,
        ptst_meth: str | None = ...,
        ptst_lab: str | None = ...,
        ptst_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        ptst_dev: str | None = ...,
        ptst_wcis: str | None = ...,
        ptst_wcf: str | None = ...,
        ptst_fsat: float | None = ...,
        ptst_temp: float | None = ...,
        ptst_sour: str | None = ...,
        ptst_back: int | None = ...,
        ptst_bval: float | None = ...,
        ptst_loss: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PUMG:
    loca_id: str | None
    pumg_test: str | None
    pumg_cont: str | None
    pumg_meth: str | None
    pumg_cred: str | None
    test_stat: str | None
    pumg_env: str | None
    pumg_rem: str | None
    file_fset: str | None
    pumg_oper: str | None
    pumts: list[PUMT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        pumg_test: str | None = ...,
        pumg_cont: str | None = ...,
        pumg_meth: str | None = ...,
        pumg_cred: str | None = ...,
        test_stat: str | None = ...,
        pumg_env: str | None = ...,
        pumg_rem: str | None = ...,
        file_fset: str | None = ...,
        pumg_oper: str | None = ...,
        pumts: list[PUMT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PUMT:
    loca_id: str | None
    pumg_test: str | None
    pumt_dtim: _dt.datetime | None
    pumt_dpth: float | None
    pumt_quat: float | None
    pumt_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        pumg_test: str | None = ...,
        pumt_dtim: _dt.datetime | None = ...,
        pumt_dpth: float | None = ...,
        pumt_quat: float | None = ...,
        pumt_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RCAG:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    spec_base: float | None
    rcag_dev: str | None
    rcag_date: _dt.datetime | None
    rcag_cond: str | None
    rcag_gsiz: float | None
    rcag_anis: str | None
    rcag_mach: str | None
    rcag_mmtd: str | None
    rcag_caim: float | None
    rcag_cais: float | None
    rcag_abcl: str | None
    rcag_rem: str | None
    rcag_meth: str | None
    rcag_lab: str | None
    rcag_cred: str | None
    test_stat: str | None
    file_fset: str | None
    rcats: list[RCAT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        spec_base: float | None = ...,
        rcag_dev: str | None = ...,
        rcag_date: _dt.datetime | None = ...,
        rcag_cond: str | None = ...,
        rcag_gsiz: float | None = ...,
        rcag_anis: str | None = ...,
        rcag_mach: str | None = ...,
        rcag_mmtd: str | None = ...,
        rcag_caim: float | None = ...,
        rcag_cais: float | None = ...,
        rcag_abcl: str | None = ...,
        rcag_rem: str | None = ...,
        rcag_meth: str | None = ...,
        rcag_lab: str | None = ...,
        rcag_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        rcats: list[RCAT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RCAT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    rcat_tesn: str | None
    rcat_cut: str | None
    rcat_sdir: str | None
    rcat_styh: int | None
    rcat_styc: str | None
    rcat_cai: float | None
    rcat_cais: float | None
    rcat_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        rcat_tesn: str | None = ...,
        rcat_cut: str | None = ...,
        rcat_sdir: str | None = ...,
        rcat_styh: int | None = ...,
        rcat_styc: str | None = ...,
        rcat_cai: float | None = ...,
        rcat_cais: float | None = ...,
        rcat_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RCCV:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    rccv_tesn: str | None
    spec_desc: str | None
    spec_prep: str | None
    rccv_mc: str | None
    rccv_ccv: float | None
    rccv_100: int | None
    rccv_rem: str | None
    rccv_meth: str | None
    rccv_lab: str | None
    rccv_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    rccv_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        rccv_tesn: str | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        rccv_mc: str | None = ...,
        rccv_ccv: float | None = ...,
        rccv_100: int | None = ...,
        rccv_rem: str | None = ...,
        rccv_meth: str | None = ...,
        rccv_lab: str | None = ...,
        rccv_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        rccv_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RDEN:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    rden_mc: str | None
    rden_smc: str | None
    rden_bden: int | None
    rden_dden: int | None
    rden_poro: float | None
    rden_pden: int | None
    rden_temp: int | None
    rden_rem: str | None
    rden_meth: str | None
    rden_lab: str | None
    rden_cred: str | None
    test_stat: str | None
    file_fset: str | None
    rden_iden: float | None
    spec_base: float | None
    rden_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        rden_mc: str | None = ...,
        rden_smc: str | None = ...,
        rden_bden: int | None = ...,
        rden_dden: int | None = ...,
        rden_poro: float | None = ...,
        rden_pden: int | None = ...,
        rden_temp: int | None = ...,
        rden_rem: str | None = ...,
        rden_meth: str | None = ...,
        rden_lab: str | None = ...,
        rden_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        rden_iden: float | None = ...,
        spec_base: float | None = ...,
        rden_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RELD:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    reld_dmax: float | None
    reld_375: int | None
    reld_063: int | None
    reld_020: int | None
    reld_dmin: float | None
    reld_rem: str | None
    reld_meth: str | None
    reld_lab: str | None
    reld_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    reld_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        reld_dmax: float | None = ...,
        reld_375: int | None = ...,
        reld_063: int | None = ...,
        reld_020: int | None = ...,
        reld_dmin: float | None = ...,
        reld_rem: str | None = ...,
        reld_meth: str | None = ...,
        reld_lab: str | None = ...,
        reld_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        reld_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RESC:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    resc_tesn: str | None
    resc_sdia: float | None
    resc_high: float | None
    resc_ctyp: str | None
    resc_elap: str | None
    resc_chgt: float | None
    resc_cdia: float | None
    resc_cmc: str | None
    resc_cddn: float | None
    resc_crd: float | None
    resc_ince: float | None
    resc_easc: float | None
    resc_ersc: float | None
    resc_devs: float | None
    resc_shrs: float | None
    resc_mnes: float | None
    resc_axsn: float | None
    resc_vlsn: float | None
    resc_rdsn: float | None
    resc_bese: str | None
    resc_beax: str | None
    resc_dbte: float | None
    resc_mat: float | None
    resc_matm: str | None
    resc_swv: int | None
    resc_smgm: float | None
    resc_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        resc_tesn: str | None = ...,
        resc_sdia: float | None = ...,
        resc_high: float | None = ...,
        resc_ctyp: str | None = ...,
        resc_elap: str | None = ...,
        resc_chgt: float | None = ...,
        resc_cdia: float | None = ...,
        resc_cmc: str | None = ...,
        resc_cddn: float | None = ...,
        resc_crd: float | None = ...,
        resc_ince: float | None = ...,
        resc_easc: float | None = ...,
        resc_ersc: float | None = ...,
        resc_devs: float | None = ...,
        resc_shrs: float | None = ...,
        resc_mnes: float | None = ...,
        resc_axsn: float | None = ...,
        resc_vlsn: float | None = ...,
        resc_rdsn: float | None = ...,
        resc_bese: str | None = ...,
        resc_beax: str | None = ...,
        resc_dbte: float | None = ...,
        resc_mat: float | None = ...,
        resc_matm: str | None = ...,
        resc_swv: int | None = ...,
        resc_smgm: float | None = ...,
        resc_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RESD:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    resd_tesn: str | None
    resd_mnum: str | None
    resd_cnds: str | None
    resd_sdia: float | None
    resd_high: float | None
    resd_cell: float | None
    resd_bp: float | None
    resd_axl: float | None
    resd_bpwp: float | None
    resd_mpwp: float | None
    resd_ppr: float | None
    resd_pwpm: float | None
    resd_eas: float | None
    resd_vol: float | None
    resd_dev: float | None
    resd_mees: float | None
    resd_mips: float | None
    resd_maps: float | None
    resd_avss: float | None
    resd_sm: float | None
    resd_dmp: float | None
    resd_rem: str | None
    file_fset: str | None
    resps: list[RESP]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        resd_tesn: str | None = ...,
        resd_mnum: str | None = ...,
        resd_cnds: str | None = ...,
        resd_sdia: float | None = ...,
        resd_high: float | None = ...,
        resd_cell: float | None = ...,
        resd_bp: float | None = ...,
        resd_axl: float | None = ...,
        resd_bpwp: float | None = ...,
        resd_mpwp: float | None = ...,
        resd_ppr: float | None = ...,
        resd_pwpm: float | None = ...,
        resd_eas: float | None = ...,
        resd_vol: float | None = ...,
        resd_dev: float | None = ...,
        resd_mees: float | None = ...,
        resd_mips: float | None = ...,
        resd_maps: float | None = ...,
        resd_avss: float | None = ...,
        resd_sm: float | None = ...,
        resd_dmp: float | None = ...,
        resd_rem: str | None = ...,
        file_fset: str | None = ...,
        resps: list[RESP] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RESG:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    spec_base: float | None
    resg_cond: str | None
    resg_cons: str | None
    resg_drag: str | None
    resg_ornt: str | None
    resg_sdia: float | None
    resg_higt: float | None
    resg_mci: str | None
    resg_mcf: str | None
    resg_bden: float | None
    resg_dden: float | None
    resg_midd: float | None
    resg_madd: float | None
    resg_irdi: float | None
    resg_ivr: float | None
    resg_isat: int | None
    resg_pden: str | None
    resg_damp: str | None
    resg_dev: str | None
    resg_rem: str | None
    resg_meth: str | None
    resg_lab: str | None
    resg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    rescs: list[RESC]
    resds: list[RESD]
    resss: list[RESS]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        spec_base: float | None = ...,
        resg_cond: str | None = ...,
        resg_cons: str | None = ...,
        resg_drag: str | None = ...,
        resg_ornt: str | None = ...,
        resg_sdia: float | None = ...,
        resg_higt: float | None = ...,
        resg_mci: str | None = ...,
        resg_mcf: str | None = ...,
        resg_bden: float | None = ...,
        resg_dden: float | None = ...,
        resg_midd: float | None = ...,
        resg_madd: float | None = ...,
        resg_irdi: float | None = ...,
        resg_ivr: float | None = ...,
        resg_isat: int | None = ...,
        resg_pden: str | None = ...,
        resg_damp: str | None = ...,
        resg_dev: str | None = ...,
        resg_rem: str | None = ...,
        resg_meth: str | None = ...,
        resg_lab: str | None = ...,
        resg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        rescs: list[RESC] | None = ...,
        resds: list[RESD] | None = ...,
        resss: list[RESS] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RESP:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    resd_tesn: str | None
    resd_mnum: str | None
    resp_ctyp: str | None
    resp_cstg: int | None
    resp_cell: float | None
    resp_back: float | None
    resp_ersc: float | None
    resp_easc: float | None
    resp_dev: float | None
    resp_vols: float | None
    resp_strn: float | None
    resp_smod: float | None
    resp_sstr: float | None
    resp_damp: float | None
    resp_smra: float | None
    resp_sr: float | None
    resp_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        resd_tesn: str | None = ...,
        resd_mnum: str | None = ...,
        resp_ctyp: str | None = ...,
        resp_cstg: int | None = ...,
        resp_cell: float | None = ...,
        resp_back: float | None = ...,
        resp_ersc: float | None = ...,
        resp_easc: float | None = ...,
        resp_dev: float | None = ...,
        resp_vols: float | None = ...,
        resp_strn: float | None = ...,
        resp_smod: float | None = ...,
        resp_sstr: float | None = ...,
        resp_damp: float | None = ...,
        resp_smra: float | None = ...,
        resp_sr: float | None = ...,
        resp_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RESS:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    ress_tesn: str | None
    ress_inc: float | None
    ress_diff: float | None
    ress_cell: float | None
    ress_bpwp: float | None
    ress_strn: float | None
    ress_mcf: str | None
    ress_bden: float | None
    ress_dden: float | None
    ress_fvr: float | None
    ress_fsat: int | None
    ress_b: float | None
    ress_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        ress_tesn: str | None = ...,
        ress_inc: float | None = ...,
        ress_diff: float | None = ...,
        ress_cell: float | None = ...,
        ress_bpwp: float | None = ...,
        ress_strn: float | None = ...,
        ress_mcf: str | None = ...,
        ress_bden: float | None = ...,
        ress_dden: float | None = ...,
        ress_fvr: float | None = ...,
        ress_fsat: int | None = ...,
        ress_b: float | None = ...,
        ress_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RPLT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    rplt_pls: float | None
    rplt_plsi: float | None
    rplt_pltf: str | None
    rplt_mc: float | None
    rplt_rem: str | None
    rplt_meth: str | None
    rplt_lab: str | None
    rplt_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    rplt_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        rplt_pls: float | None = ...,
        rplt_plsi: float | None = ...,
        rplt_pltf: str | None = ...,
        rplt_mc: float | None = ...,
        rplt_rem: str | None = ...,
        rplt_meth: str | None = ...,
        rplt_lab: str | None = ...,
        rplt_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        rplt_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RSCH:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    rsch_schv: int | None
    rsch_axis: str | None
    rsch_clam: str | None
    rsch_rem: str | None
    rsch_meth: str | None
    rsch_lab: str | None
    rsch_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    rsch_dev: str | None
    rsch_styp: str | None
    rsch_excv: str | None
    rsch_diam: float | None
    rsch_len: float | None
    rsch_wc: float | None
    rsch_wctx: str | None
    rsch_htyp: str | None
    rsch_orn: str | None
    rsch_mean: int | None
    rsch_med: int | None
    rsch_mode: int | None
    rsch_rang: int | None
    rsch_num: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        rsch_schv: int | None = ...,
        rsch_axis: str | None = ...,
        rsch_clam: str | None = ...,
        rsch_rem: str | None = ...,
        rsch_meth: str | None = ...,
        rsch_lab: str | None = ...,
        rsch_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        rsch_dev: str | None = ...,
        rsch_styp: str | None = ...,
        rsch_excv: str | None = ...,
        rsch_diam: float | None = ...,
        rsch_len: float | None = ...,
        rsch_wc: float | None = ...,
        rsch_wctx: str | None = ...,
        rsch_htyp: str | None = ...,
        rsch_orn: str | None = ...,
        rsch_mean: int | None = ...,
        rsch_med: int | None = ...,
        rsch_mode: int | None = ...,
        rsch_rang: int | None = ...,
        rsch_num: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RSHR:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    rshr_shor: float | None
    rshr_axis: str | None
    rshr_num: int | None
    rshr_rem: str | None
    rshr_meth: str | None
    rshr_lab: str | None
    rshr_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    rshr_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        rshr_shor: float | None = ...,
        rshr_axis: str | None = ...,
        rshr_num: int | None = ...,
        rshr_rem: str | None = ...,
        rshr_meth: str | None = ...,
        rshr_lab: str | None = ...,
        rshr_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        rshr_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RTEN:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    rten_sdia: float | None
    rten_len: float | None
    rten_mc: float | None
    rten_cond: str | None
    rten_durn: str | None
    rten_stra: int | None
    rten_tens: float | None
    rten_mode: str | None
    rten_mach: str | None
    rten_rem: str | None
    rten_meth: str | None
    rten_lab: str | None
    rten_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    rten_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        rten_sdia: float | None = ...,
        rten_len: float | None = ...,
        rten_mc: float | None = ...,
        rten_cond: str | None = ...,
        rten_durn: str | None = ...,
        rten_stra: int | None = ...,
        rten_tens: float | None = ...,
        rten_mode: str | None = ...,
        rten_mach: str | None = ...,
        rten_rem: str | None = ...,
        rten_meth: str | None = ...,
        rten_lab: str | None = ...,
        rten_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        rten_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RUCS:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    rucs_sdia: float | None
    rucs_len: float | None
    rucs_mc: float | None
    rucs_cond: str | None
    rucs_durn: str | None
    rucs_stra: float | None
    rucs_ucs: float | None
    rucs_mode: str | None
    rucs_mach: str | None
    rucs_rem: str | None
    rucs_meth: str | None
    rucs_lab: str | None
    rucs_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    rucs_dev: str | None
    rucs_esec: float | None
    rucs_etan: float | None
    rucs_eavg: float | None
    rucs_ssec: str | None
    rucs_stan: str | None
    rucs_savg: str | None
    rucs_mus: float | None
    rucs_mut: float | None
    rucs_muav: float | None
    rucs_e: float | None
    rucs_mu: float | None
    rucs_estr: str | None
    rucs_etyp: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        rucs_sdia: float | None = ...,
        rucs_len: float | None = ...,
        rucs_mc: float | None = ...,
        rucs_cond: str | None = ...,
        rucs_durn: str | None = ...,
        rucs_stra: float | None = ...,
        rucs_ucs: float | None = ...,
        rucs_mode: str | None = ...,
        rucs_mach: str | None = ...,
        rucs_rem: str | None = ...,
        rucs_meth: str | None = ...,
        rucs_lab: str | None = ...,
        rucs_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        rucs_dev: str | None = ...,
        rucs_esec: float | None = ...,
        rucs_etan: float | None = ...,
        rucs_eavg: float | None = ...,
        rucs_ssec: str | None = ...,
        rucs_stan: str | None = ...,
        rucs_savg: str | None = ...,
        rucs_mus: float | None = ...,
        rucs_mut: float | None = ...,
        rucs_muav: float | None = ...,
        rucs_e: float | None = ...,
        rucs_mu: float | None = ...,
        rucs_estr: str | None = ...,
        rucs_etyp: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class RWCO:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    rwco_mc: str | None
    rwco_temp: int | None
    rwco_rem: str | None
    rwco_meth: str | None
    rwco_lab: str | None
    rwco_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    rwco_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        rwco_mc: str | None = ...,
        rwco_temp: int | None = ...,
        rwco_rem: str | None = ...,
        rwco_meth: str | None = ...,
        rwco_lab: str | None = ...,
        rwco_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        rwco_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class SAMP:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    samp_base: float | None
    samp_dtim: _dt.datetime | None
    samp_ublo: int | None
    samp_cont: str | None
    samp_prep: str | None
    samp_sdia: int | None
    samp_wdep: float | None
    samp_recv: int | None
    samp_tech: str | None
    samp_matx: str | None
    samp_typc: str | None
    samp_who: str | None
    samp_why: str | None
    samp_rem: str | None
    samp_desc: str | None
    samp_desd: _dt.datetime | None
    samp_log: str | None
    samp_cond: str | None
    samp_clss: str | None
    samp_bar: float | None
    samp_temp: int | None
    samp_pres: float | None
    samp_flow: float | None
    samp_etim: _dt.datetime | None
    samp_durn: str | None
    samp_capt: str | None
    samp_link: float | None
    geol_stat: str | None
    file_fset: str | None
    samp_recl: int | None
    aavts: list[AAVT]
    acvts: list[ACVT]
    aelos: list[AELO]
    aflks: list[AFLK]
    aivts: list[AIVT]
    aloss: list[ALOS]
    apsvs: list[APSV]
    artws: list[ARTW]
    asdis: list[ASDI]
    asnss: list[ASNS]
    awads: list[AWAD]
    cbrgs: list[CBRG]
    chocs: list[CHOC]
    cmpgs: list[CMPG]
    congs: list[CONG]
    ctrgs: list[CTRG]
    ectns: list[ECTN]
    elrgs: list[ELRG]
    eress: list[ERES]
    escgs: list[ESCG]
    frsts: list[FRST]
    gchms: list[GCHM]
    grags: list[GRAG]
    ldens: list[LDEN]
    ldyns: list[LDYN]
    lfcns: list[LFCN]
    llins: list[LLIN]
    llpls: list[LLPL]
    lnmcs: list[LNMC]
    lpdns: list[LPDN]
    lpens: list[LPEN]
    lress: list[LRES]
    lslts: list[LSLT]
    lstgs: list[LSTG]
    lswls: list[LSWL]
    ltchs: list[LTCH]
    lucts: list[LUCT]
    lvans: list[LVAN]
    mcvgs: list[MCVG]
    ptsts: list[PTST]
    rcags: list[RCAG]
    rccvs: list[RCCV]
    rdens: list[RDEN]
    relds: list[RELD]
    resgs: list[RESG]
    rplts: list[RPLT]
    rschs: list[RSCH]
    rshrs: list[RSHR]
    rtens: list[RTEN]
    rucss: list[RUCS]
    rwcos: list[RWCO]
    shbgs: list[SHBG]
    sucts: list[SUCT]
    tnpcs: list[TNPC]
    tregs: list[TREG]
    trigs: list[TRIG]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        samp_base: float | None = ...,
        samp_dtim: _dt.datetime | None = ...,
        samp_ublo: int | None = ...,
        samp_cont: str | None = ...,
        samp_prep: str | None = ...,
        samp_sdia: int | None = ...,
        samp_wdep: float | None = ...,
        samp_recv: int | None = ...,
        samp_tech: str | None = ...,
        samp_matx: str | None = ...,
        samp_typc: str | None = ...,
        samp_who: str | None = ...,
        samp_why: str | None = ...,
        samp_rem: str | None = ...,
        samp_desc: str | None = ...,
        samp_desd: _dt.datetime | None = ...,
        samp_log: str | None = ...,
        samp_cond: str | None = ...,
        samp_clss: str | None = ...,
        samp_bar: float | None = ...,
        samp_temp: int | None = ...,
        samp_pres: float | None = ...,
        samp_flow: float | None = ...,
        samp_etim: _dt.datetime | None = ...,
        samp_durn: str | None = ...,
        samp_capt: str | None = ...,
        samp_link: float | None = ...,
        geol_stat: str | None = ...,
        file_fset: str | None = ...,
        samp_recl: int | None = ...,
        aavts: list[AAVT] | None = ...,
        acvts: list[ACVT] | None = ...,
        aelos: list[AELO] | None = ...,
        aflks: list[AFLK] | None = ...,
        aivts: list[AIVT] | None = ...,
        aloss: list[ALOS] | None = ...,
        apsvs: list[APSV] | None = ...,
        artws: list[ARTW] | None = ...,
        asdis: list[ASDI] | None = ...,
        asnss: list[ASNS] | None = ...,
        awads: list[AWAD] | None = ...,
        cbrgs: list[CBRG] | None = ...,
        chocs: list[CHOC] | None = ...,
        cmpgs: list[CMPG] | None = ...,
        congs: list[CONG] | None = ...,
        ctrgs: list[CTRG] | None = ...,
        ectns: list[ECTN] | None = ...,
        elrgs: list[ELRG] | None = ...,
        eress: list[ERES] | None = ...,
        escgs: list[ESCG] | None = ...,
        frsts: list[FRST] | None = ...,
        gchms: list[GCHM] | None = ...,
        grags: list[GRAG] | None = ...,
        ldens: list[LDEN] | None = ...,
        ldyns: list[LDYN] | None = ...,
        lfcns: list[LFCN] | None = ...,
        llins: list[LLIN] | None = ...,
        llpls: list[LLPL] | None = ...,
        lnmcs: list[LNMC] | None = ...,
        lpdns: list[LPDN] | None = ...,
        lpens: list[LPEN] | None = ...,
        lress: list[LRES] | None = ...,
        lslts: list[LSLT] | None = ...,
        lstgs: list[LSTG] | None = ...,
        lswls: list[LSWL] | None = ...,
        ltchs: list[LTCH] | None = ...,
        lucts: list[LUCT] | None = ...,
        lvans: list[LVAN] | None = ...,
        mcvgs: list[MCVG] | None = ...,
        ptsts: list[PTST] | None = ...,
        rcags: list[RCAG] | None = ...,
        rccvs: list[RCCV] | None = ...,
        rdens: list[RDEN] | None = ...,
        relds: list[RELD] | None = ...,
        resgs: list[RESG] | None = ...,
        rplts: list[RPLT] | None = ...,
        rschs: list[RSCH] | None = ...,
        rshrs: list[RSHR] | None = ...,
        rtens: list[RTEN] | None = ...,
        rucss: list[RUCS] | None = ...,
        rwcos: list[RWCO] | None = ...,
        shbgs: list[SHBG] | None = ...,
        sucts: list[SUCT] | None = ...,
        tnpcs: list[TNPC] | None = ...,
        tregs: list[TREG] | None = ...,
        trigs: list[TRIG] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class SCDG:
    loca_id: str | None
    scpg_tesn: str | None
    scdg_dpth: float | None
    scdg_pwpi: float | None
    scdg_pwpe: float | None
    scdg_ddis: int | None
    scdg_t: float | None
    scdg_cv: float | None
    scdg_cvmt: str | None
    scdg_ch: float | None
    scdg_chmt: str | None
    scdg_rem: str | None
    test_stat: str | None
    file_fset: str | None
    scdg_oper: str | None
    scdts: list[SCDT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        scpg_tesn: str | None = ...,
        scdg_dpth: float | None = ...,
        scdg_pwpi: float | None = ...,
        scdg_pwpe: float | None = ...,
        scdg_ddis: int | None = ...,
        scdg_t: float | None = ...,
        scdg_cv: float | None = ...,
        scdg_cvmt: str | None = ...,
        scdg_ch: float | None = ...,
        scdg_chmt: str | None = ...,
        scdg_rem: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        scdg_oper: str | None = ...,
        scdts: list[SCDT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class SCDT:
    loca_id: str | None
    scpg_tesn: str | None
    scdg_dpth: float | None
    scdt_secs: float | None
    scdt_res: float | None
    scdt_pwp1: float | None
    scdt_pwp2: float | None
    scdt_pwp3: float | None
    scdt_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        scpg_tesn: str | None = ...,
        scdg_dpth: float | None = ...,
        scdt_secs: float | None = ...,
        scdt_res: float | None = ...,
        scdt_pwp1: float | None = ...,
        scdt_pwp2: float | None = ...,
        scdt_pwp3: float | None = ...,
        scdt_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class SCPG:
    loca_id: str | None
    scpg_tesn: str | None
    scpg_type: str | None
    scpg_ref: str | None
    scpg_csa: int | None
    scpg_rate: int | None
    scpg_filt: str | None
    scpg_fric: bool | None
    scpg_wat: float | None
    scpg_wata: str | None
    scpg_rem: str | None
    scpg_env: str | None
    scpg_cont: str | None
    scpg_meth: str | None
    scpg_cred: str | None
    scpg_car: float | None
    scpg_slar: float | None
    file_fset: str | None
    scpg_oper: str | None
    scdgs: list[SCDG]
    scpps: list[SCPP]
    scpts: list[SCPT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        scpg_tesn: str | None = ...,
        scpg_type: str | None = ...,
        scpg_ref: str | None = ...,
        scpg_csa: int | None = ...,
        scpg_rate: int | None = ...,
        scpg_filt: str | None = ...,
        scpg_fric: bool | None = ...,
        scpg_wat: float | None = ...,
        scpg_wata: str | None = ...,
        scpg_rem: str | None = ...,
        scpg_env: str | None = ...,
        scpg_cont: str | None = ...,
        scpg_meth: str | None = ...,
        scpg_cred: str | None = ...,
        scpg_car: float | None = ...,
        scpg_slar: float | None = ...,
        file_fset: str | None = ...,
        scpg_oper: str | None = ...,
        scdgs: list[SCDG] | None = ...,
        scpps: list[SCPP] | None = ...,
        scpts: list[SCPT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class SCPP:
    loca_id: str | None
    scpg_tesn: str | None
    scpp_top: float | None
    scpp_base: float | None
    scpp_ref: str | None
    scpp_rem: str | None
    scpp_csbt: str | None
    scpp_csu: float | None
    scpp_crd: float | None
    scpp_cphi: float | None
    scpp_cic: float | None
    scpp_cspt: int | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        scpg_tesn: str | None = ...,
        scpp_top: float | None = ...,
        scpp_base: float | None = ...,
        scpp_ref: str | None = ...,
        scpp_rem: str | None = ...,
        scpp_csbt: str | None = ...,
        scpp_csu: float | None = ...,
        scpp_crd: float | None = ...,
        scpp_cphi: float | None = ...,
        scpp_cic: float | None = ...,
        scpp_cspt: int | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class SCPT:
    loca_id: str | None
    scpg_tesn: str | None
    scpt_dpth: float | None
    scpt_res: float | None
    scpt_fres: float | None
    scpt_pwp1: float | None
    scpt_pwp2: float | None
    scpt_pwp3: float | None
    scpt_con: float | None
    scpt_temp: float | None
    scpt_ph: float | None
    scpt_slp1: float | None
    scpt_slp2: float | None
    scpt_redx: float | None
    scpt_magt: float | None
    scpt_magx: float | None
    scpt_magy: float | None
    scpt_magz: float | None
    scpt_smp: float | None
    scpt_ngam: float | None
    scpt_rem: str | None
    scpt_frr: float | None
    scpt_qt: float | None
    scpt_ft: float | None
    scpt_qe: float | None
    scpt_bden: float | None
    scpt_cpo: float | None
    scpt_cpod: float | None
    scpt_qnet: float | None
    scpt_frrc: float | None
    scpt_expp: float | None
    scpt_bq: float | None
    scpt_ispp: float | None
    scpt_nqt: float | None
    scpt_nfr: float | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        scpg_tesn: str | None = ...,
        scpt_dpth: float | None = ...,
        scpt_res: float | None = ...,
        scpt_fres: float | None = ...,
        scpt_pwp1: float | None = ...,
        scpt_pwp2: float | None = ...,
        scpt_pwp3: float | None = ...,
        scpt_con: float | None = ...,
        scpt_temp: float | None = ...,
        scpt_ph: float | None = ...,
        scpt_slp1: float | None = ...,
        scpt_slp2: float | None = ...,
        scpt_redx: float | None = ...,
        scpt_magt: float | None = ...,
        scpt_magx: float | None = ...,
        scpt_magy: float | None = ...,
        scpt_magz: float | None = ...,
        scpt_smp: float | None = ...,
        scpt_ngam: float | None = ...,
        scpt_rem: str | None = ...,
        scpt_frr: float | None = ...,
        scpt_qt: float | None = ...,
        scpt_ft: float | None = ...,
        scpt_qe: float | None = ...,
        scpt_bden: float | None = ...,
        scpt_cpo: float | None = ...,
        scpt_cpod: float | None = ...,
        scpt_qnet: float | None = ...,
        scpt_frrc: float | None = ...,
        scpt_expp: float | None = ...,
        scpt_bq: float | None = ...,
        scpt_ispp: float | None = ...,
        scpt_nqt: float | None = ...,
        scpt_nfr: float | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class SHBG:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    shbg_type: str | None
    shbg_cond: str | None
    shbg_cons: str | None
    shbg_pcoh: float | None
    shbg_phi: float | None
    shbg_rcoh: float | None
    shbg_rphi: float | None
    shbg_enca: str | None
    shbg_rem: str | None
    shbg_meth: str | None
    shbg_lab: str | None
    shbg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    shbg_dev: str | None
    shbts: list[SHBT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        shbg_type: str | None = ...,
        shbg_cond: str | None = ...,
        shbg_cons: str | None = ...,
        shbg_pcoh: float | None = ...,
        shbg_phi: float | None = ...,
        shbg_rcoh: float | None = ...,
        shbg_rphi: float | None = ...,
        shbg_enca: str | None = ...,
        shbg_rem: str | None = ...,
        shbg_meth: str | None = ...,
        shbg_lab: str | None = ...,
        shbg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        shbg_dev: str | None = ...,
        shbts: list[SHBT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class SHBT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    shbt_tesn: str | None
    shbt_bden: float | None
    shbt_dden: float | None
    shbt_norm: int | None
    shbt_disp: float | None
    shbt_disr: float | None
    shbt_revs: int | None
    shbt_peak: float | None
    shbt_res: float | None
    shbt_pdis: float | None
    shbt_rdis: float | None
    shbt_pdin: float | None
    shbt_rdin: float | None
    shbt_pden: str | None
    shbt_ivr: float | None
    shbt_mci: str | None
    shbt_mcf: str | None
    shbt_dia1: float | None
    shbt_dia2: float | None
    shbt_hgt: float | None
    shbt_crit: str | None
    shbt_rem: str | None
    file_fset: str | None
    shbt_pvst: int | None
    shbt_rvst: int | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        shbt_tesn: str | None = ...,
        shbt_bden: float | None = ...,
        shbt_dden: float | None = ...,
        shbt_norm: int | None = ...,
        shbt_disp: float | None = ...,
        shbt_disr: float | None = ...,
        shbt_revs: int | None = ...,
        shbt_peak: float | None = ...,
        shbt_res: float | None = ...,
        shbt_pdis: float | None = ...,
        shbt_rdis: float | None = ...,
        shbt_pdin: float | None = ...,
        shbt_rdin: float | None = ...,
        shbt_pden: str | None = ...,
        shbt_ivr: float | None = ...,
        shbt_mci: str | None = ...,
        shbt_mcf: str | None = ...,
        shbt_dia1: float | None = ...,
        shbt_dia2: float | None = ...,
        shbt_hgt: float | None = ...,
        shbt_crit: str | None = ...,
        shbt_rem: str | None = ...,
        file_fset: str | None = ...,
        shbt_pvst: int | None = ...,
        shbt_rvst: int | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class STND:
    stnd_ref: str | None
    stnd_ttle: str | None
    stnd_scpe: str | None
    stnd_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        stnd_ref: str | None = ...,
        stnd_ttle: str | None = ...,
        stnd_scpe: str | None = ...,
        stnd_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class SUCT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    suct_diam: float | None
    suct_len: float | None
    suct_cond: str | None
    suct_bden: float | None
    suct_dden: float | None
    suct_mc: float | None
    suct_val: int | None
    suct_rem: str | None
    suct_meth: str | None
    suct_lab: str | None
    suct_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    suct_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        suct_diam: float | None = ...,
        suct_len: float | None = ...,
        suct_cond: str | None = ...,
        suct_bden: float | None = ...,
        suct_dden: float | None = ...,
        suct_mc: float | None = ...,
        suct_val: int | None = ...,
        suct_rem: str | None = ...,
        suct_meth: str | None = ...,
        suct_lab: str | None = ...,
        suct_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        suct_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class TNPC:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    tnpc_tesn: str | None
    tnpc_dry: str | None
    tnpc_wet: str | None
    tnpc_rem: str | None
    tnpc_meth: str | None
    tnpc_lab: str | None
    tnpc_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    tnpc_dev: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        tnpc_tesn: str | None = ...,
        tnpc_dry: str | None = ...,
        tnpc_wet: str | None = ...,
        tnpc_rem: str | None = ...,
        tnpc_meth: str | None = ...,
        tnpc_lab: str | None = ...,
        tnpc_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        tnpc_dev: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class TRAN:
    tran_isno: str | None
    tran_date: _dt.datetime | None
    tran_prod: str | None
    tran_stat: str | None
    tran_desc: str | None
    tran_ags: str | None
    tran_recv: str | None
    tran_dlim: str | None
    tran_rcon: str | None
    tran_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        tran_isno: str | None = ...,
        tran_date: _dt.datetime | None = ...,
        tran_prod: str | None = ...,
        tran_stat: str | None = ...,
        tran_desc: str | None = ...,
        tran_ags: str | None = ...,
        tran_recv: str | None = ...,
        tran_dlim: str | None = ...,
        tran_rcon: str | None = ...,
        tran_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class TREG:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    treg_type: str | None
    treg_cond: str | None
    treg_coh: int | None
    treg_phi: float | None
    treg_fcr: str | None
    treg_rem: str | None
    treg_meth: str | None
    treg_lab: str | None
    treg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    treg_dev: str | None
    trets: list[TRET]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        treg_type: str | None = ...,
        treg_cond: str | None = ...,
        treg_coh: int | None = ...,
        treg_phi: float | None = ...,
        treg_fcr: str | None = ...,
        treg_rem: str | None = ...,
        treg_meth: str | None = ...,
        treg_lab: str | None = ...,
        treg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        treg_dev: str | None = ...,
        trets: list[TRET] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class TREM:
    loca_id: str | None
    trem_dtim: _dt.datetime | None
    trem_comp: str | None
    trem_rem: str | None
    trem_durn: str | None
    trem_etim: _dt.datetime | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        trem_dtim: _dt.datetime | None = ...,
        trem_comp: str | None = ...,
        trem_rem: str | None = ...,
        trem_durn: str | None = ...,
        trem_etim: _dt.datetime | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class TRET:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    tret_tesn: str | None
    tret_sdia: float | None
    tret_len: float | None
    tret_imc: str | None
    tret_fmc: str | None
    tret_bden: float | None
    tret_dden: float | None
    tret_sat: str | None
    tret_cons: str | None
    tret_conp: int | None
    tret_cell: int | None
    tret_pwpi: int | None
    tret_strr: float | None
    tret_strn: float | None
    tret_devf: int | None
    tret_pwpf: int | None
    tret_stv: float | None
    tret_mode: str | None
    tret_rem: str | None
    file_fset: str | None
    tret_back: int | None
    tret_vert: float | None
    tret_volm: float | None
    tret_rate: float | None
    tret_bval: float | None
    tret_drn: str | None
    tret_memb: int | None
    tret_filc: int | None
    tret_ivr: float | None
    tret_satr: int | None
    tret_cvp: int | None
    tret_crp: int | None
    tret_mean: int | None
    tret_cu: int | None
    tret_ep50: float | None
    tret_e50: float | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        tret_tesn: str | None = ...,
        tret_sdia: float | None = ...,
        tret_len: float | None = ...,
        tret_imc: str | None = ...,
        tret_fmc: str | None = ...,
        tret_bden: float | None = ...,
        tret_dden: float | None = ...,
        tret_sat: str | None = ...,
        tret_cons: str | None = ...,
        tret_conp: int | None = ...,
        tret_cell: int | None = ...,
        tret_pwpi: int | None = ...,
        tret_strr: float | None = ...,
        tret_strn: float | None = ...,
        tret_devf: int | None = ...,
        tret_pwpf: int | None = ...,
        tret_stv: float | None = ...,
        tret_mode: str | None = ...,
        tret_rem: str | None = ...,
        file_fset: str | None = ...,
        tret_back: int | None = ...,
        tret_vert: float | None = ...,
        tret_volm: float | None = ...,
        tret_rate: float | None = ...,
        tret_bval: float | None = ...,
        tret_drn: str | None = ...,
        tret_memb: int | None = ...,
        tret_filc: int | None = ...,
        tret_ivr: float | None = ...,
        tret_satr: int | None = ...,
        tret_cvp: int | None = ...,
        tret_crp: int | None = ...,
        tret_mean: int | None = ...,
        tret_cu: int | None = ...,
        tret_ep50: float | None = ...,
        tret_e50: float | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class TRIG:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    spec_desc: str | None
    spec_prep: str | None
    trig_type: str | None
    trig_cond: str | None
    trig_rem: str | None
    trig_meth: str | None
    trig_lab: str | None
    trig_cred: str | None
    test_stat: str | None
    file_fset: str | None
    spec_base: float | None
    trig_dev: str | None
    trits: list[TRIT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        spec_desc: str | None = ...,
        spec_prep: str | None = ...,
        trig_type: str | None = ...,
        trig_cond: str | None = ...,
        trig_rem: str | None = ...,
        trig_meth: str | None = ...,
        trig_lab: str | None = ...,
        trig_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        spec_base: float | None = ...,
        trig_dev: str | None = ...,
        trits: list[TRIT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class TRIT:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    trit_tesn: str | None
    trit_sdia: float | None
    trit_slen: float | None
    trit_imc: str | None
    trit_fmc: str | None
    trit_cell: int | None
    trit_devf: int | None
    trit_bden: float | None
    trit_dden: float | None
    trit_strn: float | None
    trit_cu: int | None
    trit_mode: str | None
    trit_rem: str | None
    file_fset: str | None
    trit_fzwc: str | None
    trit_rate: float | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_ref: str | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        spec_ref: str | None = ...,
        spec_dpth: float | None = ...,
        trit_tesn: str | None = ...,
        trit_sdia: float | None = ...,
        trit_slen: float | None = ...,
        trit_imc: str | None = ...,
        trit_fmc: str | None = ...,
        trit_cell: int | None = ...,
        trit_devf: int | None = ...,
        trit_bden: float | None = ...,
        trit_dden: float | None = ...,
        trit_strn: float | None = ...,
        trit_cu: int | None = ...,
        trit_mode: str | None = ...,
        trit_rem: str | None = ...,
        file_fset: str | None = ...,
        trit_fzwc: str | None = ...,
        trit_rate: float | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class TYPE:
    type_type: str | None
    type_desc: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        type_type: str | None = ...,
        type_desc: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class UNIT:
    unit_unit: str | None
    unit_desc: str | None
    unit_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        unit_unit: str | None = ...,
        unit_desc: str | None = ...,
        unit_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class WADD:
    loca_id: str | None
    wadd_top: float | None
    wadd_base: float | None
    wadd_volm: int | None
    wadd_meth: str | None
    wadd_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        wadd_top: float | None = ...,
        wadd_base: float | None = ...,
        wadd_volm: int | None = ...,
        wadd_meth: str | None = ...,
        wadd_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class WETH:
    loca_id: str | None
    weth_top: float | None
    weth_base: float | None
    weth_sch: str | None
    weth_sys: str | None
    weth_weth: str | None
    weth_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        weth_top: float | None = ...,
        weth_base: float | None = ...,
        weth_sch: str | None = ...,
        weth_sys: str | None = ...,
        weth_weth: str | None = ...,
        weth_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class WGPG:
    loca_id: str | None
    wgpg_id: str | None
    wgpg_tool: str | None
    wgpg_date: _dt.datetime | None
    wgpg_strt: float | None
    wgpg_stop: float | None
    wgpg_bhd: float | None
    wgpg_wat: str | None
    wgpg_detl: str | None
    wgpg_cdia: str | None
    wgpg_rem: str | None
    wgpg_env: str | None
    wgpg_meth: str | None
    wgpg_cont: str | None
    wgpg_cred: str | None
    wgpg_stat: str | None
    file_fset: str | None
    wgpg_oper: str | None
    wgpg_lim: str | None
    wgpg_ulim: str | None
    wgpts: list[WGPT]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        wgpg_id: str | None = ...,
        wgpg_tool: str | None = ...,
        wgpg_date: _dt.datetime | None = ...,
        wgpg_strt: float | None = ...,
        wgpg_stop: float | None = ...,
        wgpg_bhd: float | None = ...,
        wgpg_wat: str | None = ...,
        wgpg_detl: str | None = ...,
        wgpg_cdia: str | None = ...,
        wgpg_rem: str | None = ...,
        wgpg_env: str | None = ...,
        wgpg_meth: str | None = ...,
        wgpg_cont: str | None = ...,
        wgpg_cred: str | None = ...,
        wgpg_stat: str | None = ...,
        file_fset: str | None = ...,
        wgpg_oper: str | None = ...,
        wgpg_lim: str | None = ...,
        wgpg_ulim: str | None = ...,
        wgpts: list[WGPT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class WGPT:
    loca_id: str | None
    wgpg_id: str | None
    wgpg_tool: str | None
    wgpt_para: str | None
    wgpt_unit: str | None
    wgpt_dpth: float | None
    wgpt_rdng: str | None
    wgpt_cas: str | None
    wgpt_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        wgpg_id: str | None = ...,
        wgpg_tool: str | None = ...,
        wgpt_para: str | None = ...,
        wgpt_unit: str | None = ...,
        wgpt_dpth: float | None = ...,
        wgpt_rdng: str | None = ...,
        wgpt_cas: str | None = ...,
        wgpt_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class WINS:
    loca_id: str | None
    wins_tesn: str | None
    wins_top: float | None
    wins_base: float | None
    wins_diam: int | None
    wins_durn: str | None
    wins_rec: int | None
    wins_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        wins_tesn: str | None = ...,
        wins_top: float | None = ...,
        wins_base: float | None = ...,
        wins_diam: int | None = ...,
        wins_durn: str | None = ...,
        wins_rec: int | None = ...,
        wins_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class WSTD:
    loca_id: str | None
    wstg_dpth: float | None
    wstd_nmin: int | None
    wstd_post: float | None
    wstd_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        wstg_dpth: float | None = ...,
        wstd_nmin: int | None = ...,
        wstd_post: float | None = ...,
        wstd_rem: str | None = ...,
        file_fset: str | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class WSTG:
    loca_id: str | None
    wstg_dpth: float | None
    wstg_dtim: _dt.datetime | None
    wstg_seal: float | None
    wstg_cas: float | None
    wstg_rem: str | None
    file_fset: str | None
    wstds: list[WSTD]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        wstg_dpth: float | None = ...,
        wstg_dtim: _dt.datetime | None = ...,
        wstg_seal: float | None = ...,
        wstg_cas: float | None = ...,
        wstg_rem: str | None = ...,
        file_fset: str | None = ...,
        wstds: list[WSTD] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

