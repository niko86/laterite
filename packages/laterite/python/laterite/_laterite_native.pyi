# AUTO-GENERATED from rust-packages/ags5-core/data/ags5_dictionary.json
# DO NOT EDIT BY HAND. Regenerate via:
#   uv run python tools/generate_pyi.py
#
# Type-stub file for the compiled `laterite._laterite_native`
# extension. IDEs and type-checkers consult this to type-check
# code that imports the 92 standard AGS5 typed-graph classes
# plus the `read_db` / `write_db` functions.
#
# Custom / passthrough groups built at runtime via
# `laterite.dynamic.get_or_register` are NOT typed in this stub —
# they show as `Any` to type checkers (acceptable; their schema
# isn't known until a file is read).

from __future__ import annotations

import datetime as _dt
from os import PathLike
from typing import Any

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
    cbrg_nmc: float | None
    cbrg_200: int | None
    cbrg_stab: float | None
    cbrg_styp: str | None
    cbrg_rem: str | None
    cbrg_meth: str | None
    cbrg_lab: str | None
    cbrg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    cbrg_size: float | None
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
        cbrg_nmc: float | None = ...,
        cbrg_200: int | None = ...,
        cbrg_stab: float | None = ...,
        cbrg_styp: str | None = ...,
        cbrg_rem: str | None = ...,
        cbrg_meth: str | None = ...,
        cbrg_lab: str | None = ...,
        cbrg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        cbrg_size: float | None = ...,
        cbrts: list[CBRT] | None = ...,
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
    cbrt_top: float | None
    cbrt_base: float | None
    cbrt_mct: float | None
    cbrt_mcbt: float | None
    cbrt_imc: float | None
    cbrt_bden: float | None
    cbrt_dden: float | None
    cbrt_surc: int | None
    cbrt_skdt: str | None
    cbrt_swel: float | None
    cbrt_rem: str | None
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
        cbrt_tesn: str | None = ...,
        cbrt_top: float | None = ...,
        cbrt_base: float | None = ...,
        cbrt_mct: float | None = ...,
        cbrt_mcbt: float | None = ...,
        cbrt_imc: float | None = ...,
        cbrt_bden: float | None = ...,
        cbrt_dden: float | None = ...,
        cbrt_surc: int | None = ...,
        cbrt_skdt: str | None = ...,
        cbrt_swel: float | None = ...,
        cbrt_rem: str | None = ...,
        file_fset: str | None = ...,
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
    cmpg_siz1: float | None
    cmpg_siz2: float | None
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
        cmpg_siz1: float | None = ...,
        cmpg_siz2: float | None = ...,
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
    cmpt_mc: float | None
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
        cmpt_mc: float | None = ...,
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
    cong_mci: float | None
    cong_mcf: float | None
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
        cong_mci: float | None = ...,
        cong_mcf: float | None = ...,
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
        conss: list[CONS] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class CONL:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    conl_mnum: int | None
    conl_ttim: float | None
    conl_ttdt: _dt.datetime | None
    conl_stim: float | None
    conl_stgn: int | None
    conl_stgd: str | None
    conl_szt: float | None
    conl_hght: float | None
    conl_ezet: float | None
    conl_vr: float | None
    conl_pwp: float | None
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
        conl_mnum: int | None = ...,
        conl_ttim: float | None = ...,
        conl_ttdt: _dt.datetime | None = ...,
        conl_stim: float | None = ...,
        conl_stgn: int | None = ...,
        conl_stgd: str | None = ...,
        conl_szt: float | None = ...,
        conl_hght: float | None = ...,
        conl_ezet: float | None = ...,
        conl_vr: float | None = ...,
        conl_pwp: float | None = ...,
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
    core_diam: str | None
    core_rem: str | None
    core_durn: str | None
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
        core_diam: str | None = ...,
        core_rem: str | None = ...,
        core_durn: str | None = ...,
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
    disc_dip: int | None
    disc_dir: int | None
    disc_rgh: str | None
    disc_plan: str | None
    disc_wave: float | None
    disc_amp: float | None
    disc_jrc: int | None
    disc_app: str | None
    disc_apt: int | None
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
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        disc_top: float | None = ...,
        disc_base: float | None = ...,
        frac_set: str | None = ...,
        disc_numb: str | None = ...,
        disc_type: str | None = ...,
        disc_dip: int | None = ...,
        disc_dir: int | None = ...,
        disc_rgh: str | None = ...,
        disc_plan: str | None = ...,
        disc_wave: float | None = ...,
        disc_amp: float | None = ...,
        disc_jrc: int | None = ...,
        disc_app: str | None = ...,
        disc_apt: int | None = ...,
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
    dprg_mass: int | None
    dprg_drop: int | None
    dprg_cone: int | None
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
    dprbs: list[DPRB]
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        dprg_tesn: str | None = ...,
        dprg_date: _dt.datetime | None = ...,
        dprg_type: str | None = ...,
        dprg_meth: str | None = ...,
        dprg_mass: int | None = ...,
        dprg_drop: int | None = ...,
        dprg_cone: int | None = ...,
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
    samp_type: str | None
    samp_id: str | None
    samp_ref: str | None
    ectn_id: str | None
    ectn_rem: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        samp_top: float | None = ...,
        samp_type: str | None = ...,
        samp_id: str | None = ...,
        samp_ref: str | None = ...,
        ectn_id: str | None = ...,
        ectn_rem: str | None = ...,
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
    eres_rval: float | None
    eres_runi: str | None
    eres_rtxt: str | None
    eres_rtcd: str | None
    eres_rres: bool | None
    eres_detf: bool | None
    eres_org: bool | None
    eres_iqlf: str | None
    eres_lqlf: str | None
    eres_rdlm: float | None
    eres_mdlm: float | None
    eres_qlm: float | None
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
        eres_rval: float | None = ...,
        eres_runi: str | None = ...,
        eres_rtxt: str | None = ...,
        eres_rtcd: str | None = ...,
        eres_rres: bool | None = ...,
        eres_detf: bool | None = ...,
        eres_org: bool | None = ...,
        eres_iqlf: str | None = ...,
        eres_lqlf: str | None = ...,
        eres_rdlm: float | None = ...,
        eres_mdlm: float | None = ...,
        eres_qlm: float | None = ...,
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
    frac_imax: int | None
    frac_iave: int | None
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
        frac_imax: int | None = ...,
        frac_iave: int | None = ...,
        frac_imin: str | None = ...,
        frac_fi: str | None = ...,
        frac_rem: str | None = ...,
        file_fset: str | None = ...,
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
    gchm_dlm: float | None
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
        gchm_dlm: float | None = ...,
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
    geol_rem: str | None
    geol_bgs: str | None
    geol_form: str | None
    file_fset: str | None
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
        geol_rem: str | None = ...,
        geol_bgs: str | None = ...,
        geol_form: str | None = ...,
        file_fset: str | None = ...,
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
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class IPID:
    loca_id: str | None
    ipid_dpth: float | None
    ipid_tesn: str | None
    ipid_date: _dt.datetime | None
    ipid_temp: float | None
    ipid_res: float | None
    ipid_rem: str | None
    ipid_env: str | None
    ipid_meth: str | None
    ipid_cont: str | None
    ipid_cred: str | None
    test_stat: str | None
    geol_stat: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        ipid_dpth: float | None = ...,
        ipid_tesn: str | None = ...,
        ipid_date: _dt.datetime | None = ...,
        ipid_temp: float | None = ...,
        ipid_res: float | None = ...,
        ipid_rem: str | None = ...,
        ipid_env: str | None = ...,
        ipid_meth: str | None = ...,
        ipid_cont: str | None = ...,
        ipid_cred: str | None = ...,
        test_stat: str | None = ...,
        geol_stat: str | None = ...,
        file_fset: str | None = ...,
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
    iprg_prwl: str | None
    iprg_swal: str | None
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
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        iprg_top: float | None = ...,
        iprg_tesn: str | None = ...,
        iprg_base: float | None = ...,
        iprg_stg: int | None = ...,
        iprg_type: str | None = ...,
        iprg_prwl: str | None = ...,
        iprg_swal: str | None = ...,
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
    iprt_dpth: str | None
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
        iprt_dpth: str | None = ...,
        iprt_rem: str | None = ...,
        file_fset: str | None = ...,
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
        isats: list[ISAT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class ISAT:
    loca_id: str | None
    isag_tesn: str | None
    isat_time: str | None
    isat_dpth: str | None
    isat_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        isag_tesn: str | None = ...,
        isat_time: str | None = ...,
        isat_dpth: str | None = ...,
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
    lbsg_type: str | None
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
        lbsg_type: str | None = ...,
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
    lbst_tcnt: int | None
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
        lbst_tcnt: int | None = ...,
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
    lden_mc: float | None
    lden_bden: float | None
    lden_dden: float | None
    lden_rem: str | None
    lden_meth: str | None
    lden_lab: str | None
    lden_cred: str | None
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
        lden_type: str | None = ...,
        lden_cond: str | None = ...,
        lden_smty: str | None = ...,
        lden_mc: float | None = ...,
        lden_bden: float | None = ...,
        lden_dden: float | None = ...,
        lden_rem: str | None = ...,
        lden_meth: str | None = ...,
        lden_lab: str | None = ...,
        lden_cred: str | None = ...,
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
    llpl_ll: int | None
    llpl_pl: int | None
    llpl_pi: int | None
    llpl_meth: str | None
    llpl_rem: str | None
    spec_prep: str | None
    llpl_425: float | None
    llpl_prep: str | None
    llpl_stab: float | None
    llpl_styp: str | None
    llpl_lab: str | None
    llpl_cred: str | None
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
        llpl_ll: int | None = ...,
        llpl_pl: int | None = ...,
        llpl_pi: int | None = ...,
        llpl_meth: str | None = ...,
        llpl_rem: str | None = ...,
        spec_prep: str | None = ...,
        llpl_425: float | None = ...,
        llpl_prep: str | None = ...,
        llpl_stab: float | None = ...,
        llpl_styp: str | None = ...,
        llpl_lab: str | None = ...,
        llpl_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
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
    lnmc_mc: float | None
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
        lnmc_mc: float | None = ...,
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
    loca_lat: float | None
    loca_lon: float | None
    loca_fdep: float | None
    loca_rem: str | None
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
    loca_chkg: str | None
    loca_appg: str | None
    bkfls: list[BKFL]
    cdias: list[CDIA]
    chiss: list[CHIS]
    cores: list[CORE]
    dcpgs: list[DCPG]
    detls: list[DETL]
    discs: list[DISC]
    dobss: list[DOBS]
    dprgs: list[DPRG]
    drems: list[DREM]
    flshs: list[FLSH]
    fracs: list[FRAC]
    geols: list[GEOL]
    hdias: list[HDIA]
    hdphs: list[HDPH]
    horns: list[HORN]
    ipens: list[IPEN]
    ipids: list[IPID]
    iprgs: list[IPRG]
    iprts: list[IPRT]
    isags: list[ISAG]
    ispts: list[ISPT]
    ivans: list[IVAN]
    mongs: list[MONG]
    pipes: list[PIPE]
    pltgs: list[PLTG]
    pmtgs: list[PMTG]
    ptims: list[PTIM]
    samps: list[SAMP]
    scpgs: list[SCPG]
    weths: list[WETH]
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
        loca_lat: float | None = ...,
        loca_lon: float | None = ...,
        loca_fdep: float | None = ...,
        loca_rem: str | None = ...,
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
        loca_chkg: str | None = ...,
        loca_appg: str | None = ...,
        bkfls: list[BKFL] | None = ...,
        cdias: list[CDIA] | None = ...,
        chiss: list[CHIS] | None = ...,
        cores: list[CORE] | None = ...,
        dcpgs: list[DCPG] | None = ...,
        detls: list[DETL] | None = ...,
        discs: list[DISC] | None = ...,
        dobss: list[DOBS] | None = ...,
        dprgs: list[DPRG] | None = ...,
        drems: list[DREM] | None = ...,
        flshs: list[FLSH] | None = ...,
        fracs: list[FRAC] | None = ...,
        geols: list[GEOL] | None = ...,
        hdias: list[HDIA] | None = ...,
        hdphs: list[HDPH] | None = ...,
        horns: list[HORN] | None = ...,
        ipens: list[IPEN] | None = ...,
        ipids: list[IPID] | None = ...,
        iprgs: list[IPRG] | None = ...,
        iprts: list[IPRT] | None = ...,
        isags: list[ISAG] | None = ...,
        ispts: list[ISPT] | None = ...,
        ivans: list[IVAN] | None = ...,
        mongs: list[MONG] | None = ...,
        pipes: list[PIPE] | None = ...,
        pltgs: list[PLTG] | None = ...,
        pmtgs: list[PMTG] | None = ...,
        ptims: list[PTIM] | None = ...,
        samps: list[SAMP] | None = ...,
        scpgs: list[SCPG] | None = ...,
        weths: list[WETH] | None = ...,
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
    lres_mc: float | None
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
        lres_mc: float | None = ...,
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
    lvan_vnpk: int | None
    lvan_vnrm: int | None
    lvan_mc: float | None
    lvan_size: float | None
    lvan_vlen: float | None
    lvan_rem: str | None
    lvan_meth: str | None
    lvan_lab: str | None
    lvan_cred: str | None
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
        lvan_vnpk: int | None = ...,
        lvan_vnrm: int | None = ...,
        lvan_mc: float | None = ...,
        lvan_size: float | None = ...,
        lvan_vlen: float | None = ...,
        lvan_rem: str | None = ...,
        lvan_meth: str | None = ...,
        lvan_lab: str | None = ...,
        lvan_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
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
    mcvg_nmc: float | None
    mcvg_stab: float | None
    mcvg_styp: str | None
    mcvg_rem: str | None
    mcvg_meth: str | None
    mcvg_lab: str | None
    mcvg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    mcvg_size: float | None
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
        mcvg_nmc: float | None = ...,
        mcvg_stab: float | None = ...,
        mcvg_styp: str | None = ...,
        mcvg_rem: str | None = ...,
        mcvg_meth: str | None = ...,
        mcvg_lab: str | None = ...,
        mcvg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        mcvg_size: float | None = ...,
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
    mcvt_mc: float | None
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
        mcvt_mc: float | None = ...,
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
    mond_lim: float | None
    mond_ulim: float | None
    mond_name: str | None
    mond_cred: str | None
    mond_cont: str | None
    mond_rem: str | None
    file_fset: str | None
    mond_stat: str | None
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
        mond_lim: float | None = ...,
        mond_ulim: float | None = ...,
        mond_name: str | None = ...,
        mond_cred: str | None = ...,
        mond_cont: str | None = ...,
        mond_rem: str | None = ...,
        file_fset: str | None = ...,
        mond_stat: str | None = ...,
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

class PMTD:
    loca_id: str | None
    pmtg_dpth: float | None
    pmtg_tesn: str | None
    pmtd_seq: int | None
    pmtd_arm1: float | None
    pmtd_arm2: float | None
    pmtd_arm3: float | None
    pmtd_tpc: float | None
    pmtd_ppa: float | None
    pmtd_ppb: float | None
    pmtd_vol: float | None
    pmtd_rem: str | None
    file_fset: str | None
    pmtd_arm4: float | None
    pmtd_arm5: float | None
    pmtd_arm6: float | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        pmtg_dpth: float | None = ...,
        pmtg_tesn: str | None = ...,
        pmtd_seq: int | None = ...,
        pmtd_arm1: float | None = ...,
        pmtd_arm2: float | None = ...,
        pmtd_arm3: float | None = ...,
        pmtd_tpc: float | None = ...,
        pmtd_ppa: float | None = ...,
        pmtd_ppb: float | None = ...,
        pmtd_vol: float | None = ...,
        pmtd_rem: str | None = ...,
        file_fset: str | None = ...,
        pmtd_arm4: float | None = ...,
        pmtd_arm5: float | None = ...,
        pmtd_arm6: float | None = ...,
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
    pmtds: list[PMTD]
    pmtls: list[PMTL]
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
        pmtds: list[PMTD] | None = ...,
        pmtls: list[PMTL] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class PMTL:
    loca_id: str | None
    pmtg_dpth: float | None
    pmtg_tesn: str | None
    pmtd_seq: int | None
    pmtl_lno: int | None
    pmtl_gaa: int | None
    pmtl_sinc: float | None
    pmtl_pinc: int | None
    pmtl_stra: float | None
    pmtl_prsa: int | None
    pmtl_nlsa: float | None
    pmtl_nlsb: float | None
    pmtl_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        pmtg_dpth: float | None = ...,
        pmtg_tesn: str | None = ...,
        pmtd_seq: int | None = ...,
        pmtl_lno: int | None = ...,
        pmtl_gaa: int | None = ...,
        pmtl_sinc: float | None = ...,
        pmtl_pinc: int | None = ...,
        pmtl_stra: float | None = ...,
        pmtl_prsa: int | None = ...,
        pmtl_nlsa: float | None = ...,
        pmtl_nlsb: float | None = ...,
        pmtl_rem: str | None = ...,
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
    proj_offc: str | None
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
        proj_offc: str | None = ...,
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
    ptst_mc: float | None
    ptst_bden: float | None
    ptst_dden: float | None
    ptst_idia: float | None
    ptst_dmet: str | None
    ptst_void: float | None
    ptst_k: float | None
    ptst_tstr: int | None
    ptst_hygr: float | None
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
        ptst_mc: float | None = ...,
        ptst_bden: float | None = ...,
        ptst_dden: float | None = ...,
        ptst_idia: float | None = ...,
        ptst_dmet: str | None = ...,
        ptst_void: float | None = ...,
        ptst_k: float | None = ...,
        ptst_tstr: int | None = ...,
        ptst_hygr: float | None = ...,
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
    rden_mc: float | None
    rden_smc: float | None
    rden_bden: float | None
    rden_dden: float | None
    rden_poro: float | None
    rden_pden: float | None
    rden_temp: int | None
    rden_rem: str | None
    rden_meth: str | None
    rden_lab: str | None
    rden_cred: str | None
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
        rden_mc: float | None = ...,
        rden_smc: float | None = ...,
        rden_bden: float | None = ...,
        rden_dden: float | None = ...,
        rden_poro: float | None = ...,
        rden_pden: float | None = ...,
        rden_temp: int | None = ...,
        rden_rem: str | None = ...,
        rden_meth: str | None = ...,
        rden_lab: str | None = ...,
        rden_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
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
    reld_siz1: float | None
    reld_siz2: float | None
    reld_siz3: float | None
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
        reld_siz1: float | None = ...,
        reld_siz2: float | None = ...,
        reld_siz3: float | None = ...,
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
    rucs_e: float | None
    rucs_mu: float | None
    rucs_estr: str | None
    rucs_etyp: str | None
    rucs_mach: str | None
    rucs_rem: str | None
    rucs_meth: str | None
    rucs_lab: str | None
    rucs_cred: str | None
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
        rucs_sdia: float | None = ...,
        rucs_len: float | None = ...,
        rucs_mc: float | None = ...,
        rucs_cond: str | None = ...,
        rucs_durn: str | None = ...,
        rucs_stra: float | None = ...,
        rucs_ucs: float | None = ...,
        rucs_mode: str | None = ...,
        rucs_e: float | None = ...,
        rucs_mu: float | None = ...,
        rucs_estr: str | None = ...,
        rucs_etyp: str | None = ...,
        rucs_mach: str | None = ...,
        rucs_rem: str | None = ...,
        rucs_meth: str | None = ...,
        rucs_lab: str | None = ...,
        rucs_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
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
    rwco_mc: float | None
    rwco_temp: int | None
    rwco_rem: str | None
    rwco_meth: str | None
    rwco_lab: str | None
    rwco_cred: str | None
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
        rwco_mc: float | None = ...,
        rwco_temp: int | None = ...,
        rwco_rem: str | None = ...,
        rwco_meth: str | None = ...,
        rwco_lab: str | None = ...,
        rwco_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
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
    samp_rem: str | None
    samp_ublo: int | None
    samp_cont: str | None
    samp_prep: str | None
    samp_sdia: int | None
    samp_wdep: str | None
    samp_recv: int | None
    samp_tech: str | None
    samp_matx: str | None
    samp_typc: str | None
    samp_who: str | None
    samp_why: str | None
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
    samp_link: str | None
    geol_stat: str | None
    file_fset: str | None
    samp_recl: float | None
    asdis: list[ASDI]
    cbrgs: list[CBRG]
    chocs: list[CHOC]
    cmpgs: list[CMPG]
    congs: list[CONG]
    conls: list[CONL]
    ectns: list[ECTN]
    eress: list[ERES]
    gchms: list[GCHM]
    grags: list[GRAG]
    ldens: list[LDEN]
    llins: list[LLIN]
    llpls: list[LLPL]
    lnmcs: list[LNMC]
    lpdns: list[LPDN]
    lress: list[LRES]
    lvans: list[LVAN]
    mcvgs: list[MCVG]
    ptsts: list[PTST]
    rdens: list[RDEN]
    relds: list[RELD]
    rplts: list[RPLT]
    rucss: list[RUCS]
    rwcos: list[RWCO]
    shbgs: list[SHBG]
    tregs: list[TREG]
    trems: list[TREM]
    trigs: list[TRIG]
    wadds: list[WADD]
    winss: list[WINS]
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
        samp_rem: str | None = ...,
        samp_ublo: int | None = ...,
        samp_cont: str | None = ...,
        samp_prep: str | None = ...,
        samp_sdia: int | None = ...,
        samp_wdep: str | None = ...,
        samp_recv: int | None = ...,
        samp_tech: str | None = ...,
        samp_matx: str | None = ...,
        samp_typc: str | None = ...,
        samp_who: str | None = ...,
        samp_why: str | None = ...,
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
        samp_link: str | None = ...,
        geol_stat: str | None = ...,
        file_fset: str | None = ...,
        samp_recl: float | None = ...,
        asdis: list[ASDI] | None = ...,
        cbrgs: list[CBRG] | None = ...,
        chocs: list[CHOC] | None = ...,
        cmpgs: list[CMPG] | None = ...,
        congs: list[CONG] | None = ...,
        conls: list[CONL] | None = ...,
        ectns: list[ECTN] | None = ...,
        eress: list[ERES] | None = ...,
        gchms: list[GCHM] | None = ...,
        grags: list[GRAG] | None = ...,
        ldens: list[LDEN] | None = ...,
        llins: list[LLIN] | None = ...,
        llpls: list[LLPL] | None = ...,
        lnmcs: list[LNMC] | None = ...,
        lpdns: list[LPDN] | None = ...,
        lress: list[LRES] | None = ...,
        lvans: list[LVAN] | None = ...,
        mcvgs: list[MCVG] | None = ...,
        ptsts: list[PTST] | None = ...,
        rdens: list[RDEN] | None = ...,
        relds: list[RELD] | None = ...,
        rplts: list[RPLT] | None = ...,
        rucss: list[RUCS] | None = ...,
        rwcos: list[RWCO] | None = ...,
        shbgs: list[SHBG] | None = ...,
        tregs: list[TREG] | None = ...,
        trems: list[TREM] | None = ...,
        trigs: list[TRIG] | None = ...,
        wadds: list[WADD] | None = ...,
        winss: list[WINS] | None = ...,
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
    scdg_t: int | None
    scdg_cv: float | None
    scdg_cvmt: str | None
    scdg_ch: float | None
    scdg_chmt: str | None
    scdg_rem: str | None
    test_stat: str | None
    file_fset: str | None
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
        scdg_t: int | None = ...,
        scdg_cv: float | None = ...,
        scdg_cvmt: str | None = ...,
        scdg_ch: float | None = ...,
        scdg_chmt: str | None = ...,
        scdg_rem: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        scdts: list[SCDT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class SCDT:
    loca_id: str | None
    scpg_tesn: str | None
    scdg_dpth: float | None
    scdt_secs: int | None
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
        scdt_secs: int | None = ...,
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
    scdgs: list[SCDG]
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
        scdgs: list[SCDG] | None = ...,
        scpts: list[SCPT] | None = ...,
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
    shbt_mci: float | None
    shbt_mcf: float | None
    shbt_dia1: float | None
    shbt_dia2: float | None
    shbt_hgt: float | None
    shbt_crit: str | None
    shbt_rem: str | None
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
        shbt_mci: float | None = ...,
        shbt_mcf: float | None = ...,
        shbt_dia1: float | None = ...,
        shbt_dia2: float | None = ...,
        shbt_hgt: float | None = ...,
        shbt_crit: str | None = ...,
        shbt_rem: str | None = ...,
        file_fset: str | None = ...,
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
    spec_base: float | None
    treg_type: str | None
    treg_cond: str | None
    treg_coh: int | None
    treg_phi: float | None
    treg_fcr: str | None
    treg_meth: str | None
    treg_lab: str | None
    treg_cred: str | None
    test_stat: str | None
    file_fset: str | None
    treg_rem: str | None
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
        spec_base: float | None = ...,
        treg_type: str | None = ...,
        treg_cond: str | None = ...,
        treg_coh: int | None = ...,
        treg_phi: float | None = ...,
        treg_fcr: str | None = ...,
        treg_meth: str | None = ...,
        treg_lab: str | None = ...,
        treg_cred: str | None = ...,
        test_stat: str | None = ...,
        file_fset: str | None = ...,
        treg_rem: str | None = ...,
        treg_dev: str | None = ...,
        trets: list[TRET] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class TREL:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    tret_tesn: str | None
    trel_mnum: int | None
    trel_ttim: float | None
    trel_ttdt: _dt.datetime | None
    trel_stim: float | None
    trel_stgn: int | None
    trel_stgd: str | None
    trel_cell: float | None
    trel_back: float | None
    trel_pwp: float | None
    trel_pwpm: float | None
    trel_szt: float | None
    trel_sze: float | None
    trel_srt: float | None
    trel_sre: float | None
    trel_ezet: float | None
    trel_ezes: float | None
    trel_epet: float | None
    trel_epes: float | None
    trel_ez1t: float | None
    trel_ez1s: float | None
    trel_ez2t: float | None
    trel_ez2s: float | None
    trel_er1t: float | None
    trel_er1s: float | None
    trel_cycn: int | None
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
        trel_mnum: int | None = ...,
        trel_ttim: float | None = ...,
        trel_ttdt: _dt.datetime | None = ...,
        trel_stim: float | None = ...,
        trel_stgn: int | None = ...,
        trel_stgd: str | None = ...,
        trel_cell: float | None = ...,
        trel_back: float | None = ...,
        trel_pwp: float | None = ...,
        trel_pwpm: float | None = ...,
        trel_szt: float | None = ...,
        trel_sze: float | None = ...,
        trel_srt: float | None = ...,
        trel_sre: float | None = ...,
        trel_ezet: float | None = ...,
        trel_ezes: float | None = ...,
        trel_epet: float | None = ...,
        trel_epes: float | None = ...,
        trel_ez1t: float | None = ...,
        trel_ez1s: float | None = ...,
        trel_ez2t: float | None = ...,
        trel_ez2s: float | None = ...,
        trel_er1t: float | None = ...,
        trel_er1s: float | None = ...,
        trel_cycn: int | None = ...,
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
    tret_rem: str | None
    tret_sdia: float | None
    tret_len: float | None
    tret_imc: float | None
    tret_fmc: float | None
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
    file_fset: str | None
    trels: list[TREL]
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
        tret_rem: str | None = ...,
        tret_sdia: float | None = ...,
        tret_len: float | None = ...,
        tret_imc: float | None = ...,
        tret_fmc: float | None = ...,
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
        file_fset: str | None = ...,
        trels: list[TREL] | None = ...,
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
        trits: list[TRIT] | None = ...,
    ) -> None: ...
    def walk(self, code: str) -> list[Any]: ...
    def __repr__(self) -> str: ...

class TRIL:
    loca_id: str | None
    samp_top: float | None
    samp_ref: str | None
    samp_type: str | None
    samp_id: str | None
    spec_ref: str | None
    spec_dpth: float | None
    trit_tesn: str | None
    tril_mnum: int | None
    tril_ttim: float | None
    tril_ttdt: _dt.datetime | None
    tril_stim: float | None
    tril_stgn: int | None
    tril_stgd: str | None
    tril_cell: float | None
    tril_sdev: float | None
    tril_ezes: float | None
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
        tril_mnum: int | None = ...,
        tril_ttim: float | None = ...,
        tril_ttdt: _dt.datetime | None = ...,
        tril_stim: float | None = ...,
        tril_stgn: int | None = ...,
        tril_stgd: str | None = ...,
        tril_cell: float | None = ...,
        tril_sdev: float | None = ...,
        tril_ezes: float | None = ...,
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
    trit_imc: float | None
    trit_fmc: float | None
    trit_cell: int | None
    trit_devf: int | None
    trit_bden: float | None
    trit_dden: float | None
    trit_strn: float | None
    trit_cu: int | None
    trit_mode: str | None
    trit_rem: str | None
    file_fset: str | None
    trils: list[TRIL]
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
        trit_imc: float | None = ...,
        trit_fmc: float | None = ...,
        trit_cell: int | None = ...,
        trit_devf: int | None = ...,
        trit_bden: float | None = ...,
        trit_dden: float | None = ...,
        trit_strn: float | None = ...,
        trit_cu: int | None = ...,
        trit_mode: str | None = ...,
        trit_rem: str | None = ...,
        file_fset: str | None = ...,
        trils: list[TRIL] | None = ...,
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
    wstd_post: str | None
    wstd_rem: str | None
    file_fset: str | None
    def __init__(
        self,
        *,
        loca_id: str | None = ...,
        wstg_dpth: float | None = ...,
        wstd_nmin: int | None = ...,
        wstd_post: str | None = ...,
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

def ags5db_read_db(path: str | PathLike[str]) -> PROJ: ...


def ags5db_write_db(
    proj: Any,
    path: str | PathLike[str],
) -> None: ...


def ags5db_attach_blobs(
    path: str | PathLike[str],
    blobs: list[dict[str, Any]],
) -> int: ...
