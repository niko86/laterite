// AUTO-GENERATED from ags5_dictionary.json by tools/generate-typed-graph.mjs.
// DO NOT EDIT — re-run the generator after a dictionary change.

/* eslint-disable */
// A typed builder graph: `new PROJ({ PROJ_ID: 'P1', locas: [new LOCA({…})] })`,
// then `emitAgs4(proj)` walks it into per-group rows. Each class carries a
// static `code` and extends AgsGroup; child arrays are
// `<childCode>`.toLowerCase() + 's'.
import { AgsGroup } from "./ags-group";

export class ABBR extends AgsGroup {
  static readonly code = "ABBR";
  ABBR_HDNG: string | null = null;
  ABBR_CODE: string | null = null;
  ABBR_DESC: string | null = null;
  ABBR_LIST: string | null = null;
  ABBR_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<ABBR> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ASDI extends AgsGroup {
  static readonly code = "ASDI";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  ASDI_SDI1: number | null = null;
  ASDI_SDI2: number | null = null;
  ASDI_SOLN: string | null = null;
  ASDI_INDR: string | null = null;
  ASDI_PADR: string | null = null;
  ASDI_REM: string | null = null;
  ASDI_METH: string | null = null;
  ASDI_LAB: string | null = null;
  ASDI_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<ASDI> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class BKFL extends AgsGroup {
  static readonly code = "BKFL";
  LOCA_ID: string | null = null;
  BKFL_TOP: number | null = null;
  BKFL_BASE: number | null = null;
  BKFL_DESC: string | null = null;
  BKFL_LEG: string | null = null;
  BKFL_DATE: Date | null = null;
  BKFL_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<BKFL> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CBRG extends AgsGroup {
  static readonly code = "CBRG";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  CBRG_COND: string | null = null;
  CBRG_NMC: number | null = null;
  CBRG_200: number | null = null;
  CBRG_STAB: number | null = null;
  CBRG_STYP: string | null = null;
  CBRG_REM: string | null = null;
  CBRG_METH: string | null = null;
  CBRG_LAB: string | null = null;
  CBRG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  CBRG_SIZE: number | null = null;
  cbrts: CBRT[] = [];
  constructor(init: Partial<CBRG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CBRT extends AgsGroup {
  static readonly code = "CBRT";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  CBRT_TESN: string | null = null;
  CBRT_TOP: number | null = null;
  CBRT_BASE: number | null = null;
  CBRT_MCT: number | null = null;
  CBRT_MCBT: number | null = null;
  CBRT_IMC: number | null = null;
  CBRT_BDEN: number | null = null;
  CBRT_DDEN: number | null = null;
  CBRT_SURC: number | null = null;
  CBRT_SKDT: string | null = null;
  CBRT_SWEL: number | null = null;
  CBRT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CBRT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CDIA extends AgsGroup {
  static readonly code = "CDIA";
  LOCA_ID: string | null = null;
  CDIA_DPTH: number | null = null;
  CDIA_DIAM: number | null = null;
  CDIA_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CDIA> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CHIS extends AgsGroup {
  static readonly code = "CHIS";
  LOCA_ID: string | null = null;
  CHIS_FROM: number | null = null;
  CHIS_TO: number | null = null;
  CHIS_TIME: string | null = null;
  CHIS_STAR: Date | null = null;
  CHIS_TOOL: string | null = null;
  CHIS_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CHIS> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CHOC extends AgsGroup {
  static readonly code = "CHOC";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  CHOC_REF: string | null = null;
  CHOC_FROM: string | null = null;
  CHOC_TO: string | null = null;
  CHOC_DDIS: Date | null = null;
  CHOC_BTCH: string | null = null;
  CHOC_REM: string | null = null;
  CHOC_CONT: number | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CHOC> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CMPG extends AgsGroup {
  static readonly code = "CMPG";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  CMPG_TESN: string | null = null;
  SPEC_PREP: string | null = null;
  SPEC_DESC: string | null = null;
  CMPG_TYPE: string | null = null;
  CMPG_MOLD: string | null = null;
  CMPG_375: number | null = null;
  CMPG_200: number | null = null;
  CMPG_PDEN: string | null = null;
  CMPG_MAXD: number | null = null;
  CMPG_MCOP: number | null = null;
  CMPG_STAB: number | null = null;
  CMPG_STYP: string | null = null;
  CMPG_REM: string | null = null;
  CMPG_METH: string | null = null;
  CMPG_LAB: string | null = null;
  CMPG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  CMPG_SIZ1: number | null = null;
  CMPG_SIZ2: number | null = null;
  cmpts: CMPT[] = [];
  constructor(init: Partial<CMPG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CMPT extends AgsGroup {
  static readonly code = "CMPT";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  CMPG_TESN: string | null = null;
  CMPT_TESN: string | null = null;
  CMPT_MC: number | null = null;
  CMPT_DDEN: number | null = null;
  CMPT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CMPT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CONG extends AgsGroup {
  static readonly code = "CONG";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  CONG_TYPE: string | null = null;
  CONG_COND: string | null = null;
  CONG_SDIA: number | null = null;
  CONG_HIGT: number | null = null;
  CONG_MCI: number | null = null;
  CONG_MCF: number | null = null;
  CONG_BDEN: number | null = null;
  CONG_DDEN: number | null = null;
  CONG_PDEN: string | null = null;
  CONG_SATR: number | null = null;
  CONG_SPRS: number | null = null;
  CONG_SATH: number | null = null;
  CONG_IVR: number | null = null;
  CONG_REM: string | null = null;
  CONG_METH: string | null = null;
  CONG_LAB: string | null = null;
  CONG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  conss: CONS[] = [];
  constructor(init: Partial<CONG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CONL extends AgsGroup {
  static readonly code = "CONL";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  CONL_MNUM: number | null = null;
  CONL_TTIM: number | null = null;
  CONL_TTDT: Date | null = null;
  CONL_STIM: number | null = null;
  CONL_STGN: number | null = null;
  CONL_STGD: string | null = null;
  CONL_SZT: number | null = null;
  CONL_HGHT: number | null = null;
  CONL_EZET: number | null = null;
  CONL_VR: number | null = null;
  CONL_PWP: number | null = null;
  constructor(init: Partial<CONL> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CONS extends AgsGroup {
  static readonly code = "CONS";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  CONS_INCN: string | null = null;
  CONS_IVR: number | null = null;
  CONS_INCF: number | null = null;
  CONS_INCE: number | null = null;
  CONS_INMV: number | null = null;
  CONS_INSC: number | null = null;
  CONS_CVRT: number | null = null;
  CONS_CVLG: number | null = null;
  CONS_TEMP: number | null = null;
  CONS_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CONS> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CORE extends AgsGroup {
  static readonly code = "CORE";
  LOCA_ID: string | null = null;
  CORE_TOP: number | null = null;
  CORE_BASE: number | null = null;
  CORE_PREC: number | null = null;
  CORE_SREC: number | null = null;
  CORE_RQD: number | null = null;
  CORE_DIAM: string | null = null;
  CORE_REM: string | null = null;
  CORE_DURN: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CORE> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DCPG extends AgsGroup {
  static readonly code = "DCPG";
  LOCA_ID: string | null = null;
  DCPG_DATE: Date | null = null;
  DCPG_TESN: string | null = null;
  DCPG_DPTH: number | null = null;
  DCPG_ZERO: number | null = null;
  DCPG_LREM: string | null = null;
  DCPG_REM: string | null = null;
  DCPG_ENV: string | null = null;
  DCPG_METH: string | null = null;
  DCPG_CONT: string | null = null;
  DCPG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  dcpts: DCPT[] = [];
  constructor(init: Partial<DCPG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DCPT extends AgsGroup {
  static readonly code = "DCPT";
  LOCA_ID: string | null = null;
  DCPG_DATE: Date | null = null;
  DCPG_TESN: string | null = null;
  DCPG_DPTH: number | null = null;
  DCPT_CBLO: number | null = null;
  DCPT_PEN: number | null = null;
  DCPT_DEL: string | null = null;
  DCPT_REM: string | null = null;
  constructor(init: Partial<DCPT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DETL extends AgsGroup {
  static readonly code = "DETL";
  LOCA_ID: string | null = null;
  DETL_TOP: number | null = null;
  DETL_BASE: number | null = null;
  DETL_DESC: string | null = null;
  DETL_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<DETL> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DICT extends AgsGroup {
  static readonly code = "DICT";
  DICT_TYPE: string | null = null;
  DICT_GRP: string | null = null;
  DICT_HDNG: string | null = null;
  DICT_STAT: string | null = null;
  DICT_DTYP: string | null = null;
  DICT_DESC: string | null = null;
  DICT_UNIT: string | null = null;
  DICT_EXMP: string | null = null;
  DICT_PGRP: string | null = null;
  DICT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<DICT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DISC extends AgsGroup {
  static readonly code = "DISC";
  LOCA_ID: string | null = null;
  DISC_TOP: number | null = null;
  DISC_BASE: number | null = null;
  FRAC_SET: string | null = null;
  DISC_NUMB: string | null = null;
  DISC_TYPE: string | null = null;
  DISC_DIP: number | null = null;
  DISC_DIR: number | null = null;
  DISC_RGH: string | null = null;
  DISC_PLAN: string | null = null;
  DISC_WAVE: number | null = null;
  DISC_AMP: number | null = null;
  DISC_JRC: number | null = null;
  DISC_APP: string | null = null;
  DISC_APT: number | null = null;
  DISC_APOB: string | null = null;
  DISC_INFM: string | null = null;
  DISC_TERM: string | null = null;
  DISC_PERS: number | null = null;
  DISC_STR: number | null = null;
  DISC_WETH: string | null = null;
  DISC_SEEP: string | null = null;
  DISC_FLOW: number | null = null;
  DISC_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<DISC> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DOBS extends AgsGroup {
  static readonly code = "DOBS";
  LOCA_ID: string | null = null;
  DOBS_TOP: number | null = null;
  DOBS_BASE: number | null = null;
  DOBS_SET: string | null = null;
  DOBS_DURN: string | null = null;
  DOBS_STIM: Date | null = null;
  DOBS_ETIM: Date | null = null;
  DOBS_DHRT: number | null = null;
  DOBS_DHRS: number | null = null;
  DOBS_PENR: number | null = null;
  DOBS_HAMM: boolean | null = null;
  DOBS_THRP: number | null = null;
  DOBS_RESP: number | null = null;
  DOBS_TORP: number | null = null;
  DOBS_TORQ: number | null = null;
  DOBS_THST: number | null = null;
  DOBS_REST: number | null = null;
  DOBS_HAMP: number | null = null;
  DOBS_SPEN: number | null = null;
  DOBS_FMPO: number | null = null;
  DOBS_FMCR: number | null = null;
  DOBS_FMRR: number | null = null;
  DOBS_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<DOBS> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DPRB extends AgsGroup {
  static readonly code = "DPRB";
  LOCA_ID: string | null = null;
  DPRG_TESN: string | null = null;
  DPRB_DPTH: number | null = null;
  DPRB_BLOW: number | null = null;
  DPRB_CBLW: number | null = null;
  DPRB_TORQ: number | null = null;
  DPRB_DEL: string | null = null;
  DPRB_INC: number | null = null;
  DPRB_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<DPRB> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DPRG extends AgsGroup {
  static readonly code = "DPRG";
  LOCA_ID: string | null = null;
  DPRG_TESN: string | null = null;
  DPRG_DATE: Date | null = null;
  DPRG_TYPE: string | null = null;
  DPRG_METH: string | null = null;
  DPRG_MASS: number | null = null;
  DPRG_DROP: number | null = null;
  DPRG_CONE: number | null = null;
  DPRG_ROD: number | null = null;
  DPRG_TANV: string | null = null;
  DPRG_DAMP: string | null = null;
  DPRG_TIP: number | null = null;
  DPRG_REM: string | null = null;
  DPRG_ANG: number | null = null;
  DPRG_RMSS: number | null = null;
  DPRG_PARF: string | null = null;
  DPRG_PDIU: string | null = null;
  DPRG_BCF: string | null = null;
  DPRG_GW: number | null = null;
  DPRG_REET: string | null = null;
  DPRG_ENV: string | null = null;
  DPRG_CONT: string | null = null;
  DPRG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  dprbs: DPRB[] = [];
  constructor(init: Partial<DPRG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DREM extends AgsGroup {
  static readonly code = "DREM";
  LOCA_ID: string | null = null;
  DREM_TOP: number | null = null;
  DREM_BASE: number | null = null;
  DREM_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<DREM> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ECTN extends AgsGroup {
  static readonly code = "ECTN";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SAMP_REF: string | null = null;
  ECTN_ID: string | null = null;
  ECTN_REM: string | null = null;
  constructor(init: Partial<ECTN> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ERES extends AgsGroup {
  static readonly code = "ERES";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  ERES_CODE: string | null = null;
  ERES_METH: string | null = null;
  ERES_MATX: string | null = null;
  ERES_RTYP: string | null = null;
  ERES_TESN: string | null = null;
  ERES_NAME: string | null = null;
  ERES_TNAM: string | null = null;
  ERES_RVAL: number | null = null;
  ERES_RUNI: string | null = null;
  ERES_RTXT: string | null = null;
  ERES_RTCD: string | null = null;
  ERES_RRES: boolean | null = null;
  ERES_DETF: boolean | null = null;
  ERES_ORG: boolean | null = null;
  ERES_IQLF: string | null = null;
  ERES_LQLF: string | null = null;
  ERES_RDLM: number | null = null;
  ERES_MDLM: number | null = null;
  ERES_QLM: number | null = null;
  ERES_DUNI: string | null = null;
  ERES_TICP: number | null = null;
  ERES_TICT: number | null = null;
  ERES_RDAT: Date | null = null;
  ERES_SGRP: string | null = null;
  SPEC_PREP: string | null = null;
  SPEC_DESC: string | null = null;
  ERES_DTIM: Date | null = null;
  ERES_TEST: string | null = null;
  ERES_TORD: string | null = null;
  ERES_LOCN: string | null = null;
  ERES_BAS: string | null = null;
  ERES_DIL: number | null = null;
  ERES_LMTH: string | null = null;
  ERES_LDTM: Date | null = null;
  ERES_IREF: string | null = null;
  ERES_SIZE: number | null = null;
  ERES_PERP: number | null = null;
  ERES_REM: string | null = null;
  ERES_LAB: string | null = null;
  ERES_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<ERES> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class FLSH extends AgsGroup {
  static readonly code = "FLSH";
  LOCA_ID: string | null = null;
  FLSH_TOP: number | null = null;
  FLSH_BASE: number | null = null;
  FLSH_TYPE: string | null = null;
  FLSH_RETN: number | null = null;
  FLSH_RETX: number | null = null;
  FLSH_COL: string | null = null;
  FLSH_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<FLSH> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class FRAC extends AgsGroup {
  static readonly code = "FRAC";
  LOCA_ID: string | null = null;
  FRAC_FROM: number | null = null;
  FRAC_TO: number | null = null;
  FRAC_SET: string | null = null;
  FRAC_IMAX: number | null = null;
  FRAC_IAVE: number | null = null;
  FRAC_IMIN: string | null = null;
  FRAC_FI: string | null = null;
  FRAC_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<FRAC> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class GCHM extends AgsGroup {
  static readonly code = "GCHM";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  GCHM_CODE: string | null = null;
  GCHM_METH: string | null = null;
  GCHM_TTYP: string | null = null;
  GCHM_RESL: string | null = null;
  GCHM_UNIT: string | null = null;
  GCHM_NAME: string | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  GCHM_REM: string | null = null;
  GCHM_LAB: string | null = null;
  GCHM_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  GCHM_DLM: number | null = null;
  constructor(init: Partial<GCHM> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class GEOL extends AgsGroup {
  static readonly code = "GEOL";
  LOCA_ID: string | null = null;
  GEOL_TOP: number | null = null;
  GEOL_BASE: number | null = null;
  GEOL_DESC: string | null = null;
  GEOL_LEG: string | null = null;
  GEOL_GEOL: string | null = null;
  GEOL_GEO2: string | null = null;
  GEOL_STAT: string | null = null;
  GEOL_REM: string | null = null;
  GEOL_BGS: string | null = null;
  GEOL_FORM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<GEOL> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class GRAG extends AgsGroup {
  static readonly code = "GRAG";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  GRAG_UC: number | null = null;
  GRAG_VCRE: number | null = null;
  GRAG_GRAV: number | null = null;
  GRAG_SAND: number | null = null;
  GRAG_SILT: number | null = null;
  GRAG_CLAY: number | null = null;
  GRAG_FINE: number | null = null;
  GRAG_REM: string | null = null;
  GRAG_METH: string | null = null;
  GRAG_LAB: string | null = null;
  GRAG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  grats: GRAT[] = [];
  constructor(init: Partial<GRAG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class GRAT extends AgsGroup {
  static readonly code = "GRAT";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  GRAT_SIZE: number | null = null;
  GRAT_PERP: number | null = null;
  GRAT_TYPE: string | null = null;
  GRAT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<GRAT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class HDIA extends AgsGroup {
  static readonly code = "HDIA";
  LOCA_ID: string | null = null;
  HDIA_DPTH: number | null = null;
  HDIA_DIAM: number | null = null;
  HDIA_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<HDIA> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class HDPH extends AgsGroup {
  static readonly code = "HDPH";
  LOCA_ID: string | null = null;
  HDPH_TOP: number | null = null;
  HDPH_BASE: number | null = null;
  HDPH_TYPE: string | null = null;
  HDPH_STAR: Date | null = null;
  HDPH_ENDD: Date | null = null;
  HDPH_CREW: string | null = null;
  HDPH_EXC: string | null = null;
  HDPH_SHOR: string | null = null;
  HDPH_STAB: string | null = null;
  HDPH_DIML: number | null = null;
  HDPH_DIMW: number | null = null;
  HDPH_DBIT: string | null = null;
  HDPH_BCON: string | null = null;
  HDPH_BTYP: string | null = null;
  HDPH_BLEN: number | null = null;
  HDPH_LOG: string | null = null;
  HDPH_LOGD: Date | null = null;
  HDPH_REM: string | null = null;
  HDPH_ENV: string | null = null;
  HDPH_METH: string | null = null;
  HDPH_CONT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<HDPH> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class HORN extends AgsGroup {
  static readonly code = "HORN";
  LOCA_ID: string | null = null;
  HORN_TOP: number | null = null;
  HORN_BASE: number | null = null;
  HORN_ORNT: number | null = null;
  HORN_INCL: number | null = null;
  HORN_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<HORN> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class IPEN extends AgsGroup {
  static readonly code = "IPEN";
  LOCA_ID: string | null = null;
  IPEN_DPTH: number | null = null;
  IPEN_TESN: string | null = null;
  IPEN_IPEN: string | null = null;
  IPEN_DATE: Date | null = null;
  IPEN_REM: string | null = null;
  IPEN_ENV: string | null = null;
  IPEN_METH: string | null = null;
  IPEN_CONT: string | null = null;
  IPEN_CRED: string | null = null;
  TEST_STAT: string | null = null;
  GEOL_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<IPEN> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class IPID extends AgsGroup {
  static readonly code = "IPID";
  LOCA_ID: string | null = null;
  IPID_DPTH: number | null = null;
  IPID_TESN: string | null = null;
  IPID_DATE: Date | null = null;
  IPID_TEMP: number | null = null;
  IPID_RES: number | null = null;
  IPID_REM: string | null = null;
  IPID_ENV: string | null = null;
  IPID_METH: string | null = null;
  IPID_CONT: string | null = null;
  IPID_CRED: string | null = null;
  TEST_STAT: string | null = null;
  GEOL_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<IPID> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class IPRG extends AgsGroup {
  static readonly code = "IPRG";
  LOCA_ID: string | null = null;
  IPRG_TOP: number | null = null;
  IPRG_TESN: string | null = null;
  IPRG_BASE: number | null = null;
  IPRG_STG: number | null = null;
  IPRG_TYPE: string | null = null;
  IPRG_PRWL: string | null = null;
  IPRG_SWAL: string | null = null;
  IPRG_TDIA: number | null = null;
  IPRG_SDIA: number | null = null;
  IPRG_IPRM: number | null = null;
  IPRG_FLOW: number | null = null;
  IPRG_AWL: number | null = null;
  IPRG_HEAD: number | null = null;
  IPRG_DATE: Date | null = null;
  IPRG_REM: string | null = null;
  IPRG_ENV: string | null = null;
  IPRG_METH: string | null = null;
  IPRG_CONT: string | null = null;
  IPRG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<IPRG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class IPRT extends AgsGroup {
  static readonly code = "IPRT";
  LOCA_ID: string | null = null;
  IPRG_TOP: number | null = null;
  IPRG_TESN: string | null = null;
  IPRG_BASE: number | null = null;
  IPRG_STG: number | null = null;
  IPRT_TIME: string | null = null;
  IPRT_DPTH: string | null = null;
  IPRT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<IPRT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ISAG extends AgsGroup {
  static readonly code = "ISAG";
  LOCA_ID: string | null = null;
  ISAG_TESN: string | null = null;
  ISAG_DATE: Date | null = null;
  ISAG_DURN: string | null = null;
  ISAG_PWID: number | null = null;
  ISAG_PLEN: number | null = null;
  ISAG_PDIA: number | null = null;
  ISAG_DPTS: number | null = null;
  ISAG_DPTE: number | null = null;
  ISAG_CONS: string | null = null;
  ISAG_SI: number | null = null;
  ISAG_PORO: number | null = null;
  ISAG_REM: string | null = null;
  ISAG_ENV: string | null = null;
  ISAG_METH: string | null = null;
  ISAG_CONT: string | null = null;
  ISAG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  isats: ISAT[] = [];
  constructor(init: Partial<ISAG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ISAT extends AgsGroup {
  static readonly code = "ISAT";
  LOCA_ID: string | null = null;
  ISAG_TESN: string | null = null;
  ISAT_TIME: string | null = null;
  ISAT_DPTH: string | null = null;
  ISAT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<ISAT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ISPT extends AgsGroup {
  static readonly code = "ISPT";
  LOCA_ID: string | null = null;
  ISPT_TOP: number | null = null;
  ISPT_SEAT: number | null = null;
  ISPT_MAIN: number | null = null;
  ISPT_NPEN: number | null = null;
  ISPT_NVAL: number | null = null;
  ISPT_REP: string | null = null;
  ISPT_CAS: number | null = null;
  ISPT_WAT: string | null = null;
  ISPT_TYPE: string | null = null;
  ISPT_HAM: string | null = null;
  ISPT_ERAT: number | null = null;
  ISPT_SWP: number | null = null;
  ISPT_INC1: number | null = null;
  ISPT_INC2: number | null = null;
  ISPT_INC3: number | null = null;
  ISPT_INC4: number | null = null;
  ISPT_INC5: number | null = null;
  ISPT_INC6: number | null = null;
  ISPT_PEN1: number | null = null;
  ISPT_PEN2: number | null = null;
  ISPT_PEN3: number | null = null;
  ISPT_PEN4: number | null = null;
  ISPT_PEN5: number | null = null;
  ISPT_PEN6: number | null = null;
  ISPT_ROCK: boolean | null = null;
  ISPT_REM: string | null = null;
  ISPT_ENV: string | null = null;
  ISPT_METH: string | null = null;
  ISPT_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<ISPT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class IVAN extends AgsGroup {
  static readonly code = "IVAN";
  LOCA_ID: string | null = null;
  IVAN_DPTH: number | null = null;
  IVAN_TESN: string | null = null;
  IVAN_TYPE: string | null = null;
  IVAN_IVAN: string | null = null;
  IVAN_IVAR: string | null = null;
  IVAN_DATE: Date | null = null;
  IVAN_REM: string | null = null;
  IVAN_ENV: string | null = null;
  IVAN_METH: string | null = null;
  IVAN_CONT: string | null = null;
  IVAN_CRED: string | null = null;
  TEST_STAT: string | null = null;
  GEOL_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<IVAN> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LBSG extends AgsGroup {
  static readonly code = "LBSG";
  LBSG_REF: string | null = null;
  LBSG_DATE: Date | null = null;
  LBSG_FROM: string | null = null;
  LBSG_TO: string | null = null;
  LBSG_DUE: Date | null = null;
  LBSG_REM: string | null = null;
  LBSG_STAT: string | null = null;
  FILE_FSET: string | null = null;
  LBSG_TYPE: string | null = null;
  lbsts: LBST[] = [];
  constructor(init: Partial<LBSG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LBST extends AgsGroup {
  static readonly code = "LBST";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  LBSG_REF: string | null = null;
  LBST_TEST: string | null = null;
  CHOC_REF: string | null = null;
  LBST_TTYP: string | null = null;
  LBST_METH: string | null = null;
  LBST_PREP: string | null = null;
  LBST_DEPN: string | null = null;
  LBST_STAT: string | null = null;
  LBST_REM: string | null = null;
  LBST_DUE: Date | null = null;
  LBST_DETL: string | null = null;
  LBST_DONE: Date | null = null;
  FILE_FSET: string | null = null;
  LBST_TCNT: number | null = null;
  constructor(init: Partial<LBST> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LDEN extends AgsGroup {
  static readonly code = "LDEN";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  LDEN_TYPE: string | null = null;
  LDEN_COND: string | null = null;
  LDEN_SMTY: string | null = null;
  LDEN_MC: number | null = null;
  LDEN_BDEN: number | null = null;
  LDEN_DDEN: number | null = null;
  LDEN_REM: string | null = null;
  LDEN_METH: string | null = null;
  LDEN_LAB: string | null = null;
  LDEN_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<LDEN> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LLIN extends AgsGroup {
  static readonly code = "LLIN";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  LLIN_LS: number | null = null;
  LLIN_425: number | null = null;
  LLIN_PREP: string | null = null;
  LLIN_REM: string | null = null;
  LLIN_METH: string | null = null;
  LLIN_LAB: string | null = null;
  LLIN_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<LLIN> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LLPL extends AgsGroup {
  static readonly code = "LLPL";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  LLPL_LL: number | null = null;
  LLPL_PL: number | null = null;
  LLPL_PI: number | null = null;
  LLPL_METH: string | null = null;
  LLPL_REM: string | null = null;
  SPEC_PREP: string | null = null;
  LLPL_425: number | null = null;
  LLPL_PREP: string | null = null;
  LLPL_STAB: number | null = null;
  LLPL_STYP: string | null = null;
  LLPL_LAB: string | null = null;
  LLPL_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<LLPL> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LNMC extends AgsGroup {
  static readonly code = "LNMC";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  LNMC_MC: number | null = null;
  LNMC_TEMP: number | null = null;
  LNMC_STAB: number | null = null;
  LNMC_STYP: string | null = null;
  LNMC_ISNT: boolean | null = null;
  LNMC_COMM: string | null = null;
  LNMC_REM: string | null = null;
  LNMC_METH: string | null = null;
  LNMC_LAB: string | null = null;
  LNMC_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<LNMC> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LOCA extends AgsGroup {
  static readonly code = "LOCA";
  LOCA_ID: string | null = null;
  LOCA_TYPE: string | null = null;
  LOCA_STAT: string | null = null;
  LOCA_NATE: number | null = null;
  LOCA_NATN: number | null = null;
  LOCA_GREF: string | null = null;
  LOCA_GL: number | null = null;
  LOCA_LAT: number | null = null;
  LOCA_LON: number | null = null;
  LOCA_FDEP: number | null = null;
  LOCA_REM: string | null = null;
  LOCA_STAR: Date | null = null;
  LOCA_PURP: string | null = null;
  LOCA_TERM: string | null = null;
  LOCA_ENDD: Date | null = null;
  LOCA_LETT: string | null = null;
  LOCA_LOCX: number | null = null;
  LOCA_LOCY: number | null = null;
  LOCA_LOCZ: number | null = null;
  LOCA_LREF: string | null = null;
  LOCA_DATM: string | null = null;
  LOCA_ETRV: number | null = null;
  LOCA_NTRV: number | null = null;
  LOCA_LTRV: number | null = null;
  LOCA_XTRL: number | null = null;
  LOCA_YTRL: number | null = null;
  LOCA_ZTRL: number | null = null;
  LOCA_ELAT: string | null = null;
  LOCA_ELON: string | null = null;
  LOCA_LLZ: string | null = null;
  LOCA_LOCM: string | null = null;
  LOCA_LOCA: string | null = null;
  LOCA_CLST: string | null = null;
  LOCA_ALID: string | null = null;
  LOCA_OFFS: number | null = null;
  LOCA_CNGE: string | null = null;
  LOCA_TRAN: string | null = null;
  FILE_FSET: string | null = null;
  LOCA_CHKG: string | null = null;
  LOCA_APPG: string | null = null;
  bkfls: BKFL[] = [];
  cdias: CDIA[] = [];
  chiss: CHIS[] = [];
  cores: CORE[] = [];
  dcpgs: DCPG[] = [];
  detls: DETL[] = [];
  discs: DISC[] = [];
  dobss: DOBS[] = [];
  dprgs: DPRG[] = [];
  drems: DREM[] = [];
  flshs: FLSH[] = [];
  fracs: FRAC[] = [];
  geols: GEOL[] = [];
  hdias: HDIA[] = [];
  hdphs: HDPH[] = [];
  horns: HORN[] = [];
  ipens: IPEN[] = [];
  ipids: IPID[] = [];
  iprgs: IPRG[] = [];
  iprts: IPRT[] = [];
  isags: ISAG[] = [];
  ispts: ISPT[] = [];
  ivans: IVAN[] = [];
  mongs: MONG[] = [];
  pipes: PIPE[] = [];
  pltgs: PLTG[] = [];
  pmtgs: PMTG[] = [];
  ptims: PTIM[] = [];
  samps: SAMP[] = [];
  scpgs: SCPG[] = [];
  weths: WETH[] = [];
  wstgs: WSTG[] = [];
  constructor(init: Partial<LOCA> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LPDN extends AgsGroup {
  static readonly code = "LPDN";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  LPDN_PDEN: string | null = null;
  LPDN_TYPE: string | null = null;
  LPDN_REM: string | null = null;
  LPDN_METH: string | null = null;
  LPDN_LAB: string | null = null;
  LPDN_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<LPDN> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LRES extends AgsGroup {
  static readonly code = "LRES";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  LRES_BDEN: number | null = null;
  LRES_DDEN: number | null = null;
  LRES_MC: number | null = null;
  LRES_COND: string | null = null;
  LRES_LRES: number | null = null;
  LRES_CDIA: number | null = null;
  LRES_CCSA: number | null = null;
  LRES_CLEN: number | null = null;
  LRES_TEMP: number | null = null;
  LRES_ELEC: string | null = null;
  LRES_PENT: string | null = null;
  LRES_CSHP: string | null = null;
  LRES_WAT: number | null = null;
  LRES_WRES: number | null = null;
  LRES_PART: string | null = null;
  LRES_REM: string | null = null;
  LRES_METH: string | null = null;
  LRES_LAB: string | null = null;
  LRES_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<LRES> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LVAN extends AgsGroup {
  static readonly code = "LVAN";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  LVAN_VNPK: number | null = null;
  LVAN_VNRM: number | null = null;
  LVAN_MC: number | null = null;
  LVAN_SIZE: number | null = null;
  LVAN_VLEN: number | null = null;
  LVAN_REM: string | null = null;
  LVAN_METH: string | null = null;
  LVAN_LAB: string | null = null;
  LVAN_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<LVAN> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class MCVG extends AgsGroup {
  static readonly code = "MCVG";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  MCVG_200: number | null = null;
  MCVG_NMC: number | null = null;
  MCVG_STAB: number | null = null;
  MCVG_STYP: string | null = null;
  MCVG_REM: string | null = null;
  MCVG_METH: string | null = null;
  MCVG_LAB: string | null = null;
  MCVG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  MCVG_SIZE: number | null = null;
  mcvts: MCVT[] = [];
  constructor(init: Partial<MCVG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class MCVT extends AgsGroup {
  static readonly code = "MCVT";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  MCVT_TESN: string | null = null;
  MCVT_MC: number | null = null;
  MCVT_CURV: string | null = null;
  MCVT_RELK: number | null = null;
  MCVT_BDEN: number | null = null;
  MCVT_DIFF: number | null = null;
  MCVT_RAPD: string | null = null;
  MCVT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<MCVT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class MOND extends AgsGroup {
  static readonly code = "MOND";
  LOCA_ID: string | null = null;
  MONG_ID: string | null = null;
  MONG_DIS: number | null = null;
  MOND_DTIM: Date | null = null;
  MOND_TYPE: string | null = null;
  MOND_REF: string | null = null;
  MOND_INST: string | null = null;
  MOND_RDNG: string | null = null;
  MOND_UNIT: string | null = null;
  MOND_METH: string | null = null;
  MOND_LIM: number | null = null;
  MOND_ULIM: number | null = null;
  MOND_NAME: string | null = null;
  MOND_CRED: string | null = null;
  MOND_CONT: string | null = null;
  MOND_REM: string | null = null;
  FILE_FSET: string | null = null;
  MOND_STAT: string | null = null;
  constructor(init: Partial<MOND> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class MONG extends AgsGroup {
  static readonly code = "MONG";
  LOCA_ID: string | null = null;
  MONG_ID: string | null = null;
  MONG_DIS: number | null = null;
  PIPE_REF: string | null = null;
  MONG_DATE: Date | null = null;
  MONG_TYPE: string | null = null;
  MONG_DETL: string | null = null;
  MONG_TRZ: number | null = null;
  MONG_BRZ: number | null = null;
  MONG_BRGA: number | null = null;
  MONG_BRGB: number | null = null;
  MONG_BRGC: number | null = null;
  MONG_INCA: number | null = null;
  MONG_INCB: number | null = null;
  MONG_INCC: number | null = null;
  MONG_RSCA: string | null = null;
  MONG_RSCB: string | null = null;
  MONG_RSCC: string | null = null;
  MONG_REM: string | null = null;
  MONG_CONT: string | null = null;
  FILE_FSET: string | null = null;
  monds: MOND[] = [];
  constructor(init: Partial<MONG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PIPE extends AgsGroup {
  static readonly code = "PIPE";
  LOCA_ID: string | null = null;
  PIPE_REF: string | null = null;
  PIPE_TOP: number | null = null;
  PIPE_BASE: number | null = null;
  PIPE_DIAM: number | null = null;
  PIPE_TYPE: string | null = null;
  PIPE_CONS: string | null = null;
  PIPE_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<PIPE> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PLTG extends AgsGroup {
  static readonly code = "PLTG";
  LOCA_ID: string | null = null;
  PLTG_DPTH: number | null = null;
  PLTG_TESN: string | null = null;
  PLTG_CYC: string | null = null;
  PLTG_PDIA: number | null = null;
  PLTG_SEAT: number | null = null;
  PLTG_FA0: number | null = null;
  PLTG_FA1: number | null = null;
  PLTG_FA2: number | null = null;
  PLTG_SMOD: number | null = null;
  PLTG_EV2: number | null = null;
  PLTG_MOSR: number | null = null;
  PLTG_EMOD: number | null = null;
  PLTG_DATE: Date | null = null;
  PLTG_STAB: number | null = null;
  PLTG_STYP: string | null = null;
  PLTG_REM: string | null = null;
  PLTG_ENV: string | null = null;
  PLTG_METH: string | null = null;
  PLTG_CONT: string | null = null;
  PLTG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  GEOL_STAT: string | null = null;
  FILE_FSET: string | null = null;
  pltts: PLTT[] = [];
  constructor(init: Partial<PLTG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PLTT extends AgsGroup {
  static readonly code = "PLTT";
  LOCA_ID: string | null = null;
  PLTG_DPTH: number | null = null;
  PLTG_TESN: string | null = null;
  PLTG_CYC: string | null = null;
  PLTT_STG: string | null = null;
  PLTT_TIME: number | null = null;
  PLTT_LOAD: number | null = null;
  PLTT_SET1: number | null = null;
  PLTT_SET2: number | null = null;
  PLTT_SET3: number | null = null;
  PLTT_SET4: number | null = null;
  PLTT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<PLTT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PMTD extends AgsGroup {
  static readonly code = "PMTD";
  LOCA_ID: string | null = null;
  PMTG_DPTH: number | null = null;
  PMTG_TESN: string | null = null;
  PMTD_SEQ: number | null = null;
  PMTD_ARM1: number | null = null;
  PMTD_ARM2: number | null = null;
  PMTD_ARM3: number | null = null;
  PMTD_TPC: number | null = null;
  PMTD_PPA: number | null = null;
  PMTD_PPB: number | null = null;
  PMTD_VOL: number | null = null;
  PMTD_REM: string | null = null;
  FILE_FSET: string | null = null;
  PMTD_ARM4: number | null = null;
  PMTD_ARM5: number | null = null;
  PMTD_ARM6: number | null = null;
  constructor(init: Partial<PMTD> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PMTG extends AgsGroup {
  static readonly code = "PMTG";
  LOCA_ID: string | null = null;
  PMTG_DPTH: number | null = null;
  PMTG_TESN: string | null = null;
  PMTG_DATE: Date | null = null;
  PMTG_WAT: number | null = null;
  PMTG_CONT: string | null = null;
  PMTG_CREW: string | null = null;
  PMTG_REF: string | null = null;
  PMTG_TYPE: string | null = null;
  PMTG_DIAM: number | null = null;
  PMTG_HO: number | null = null;
  PMTG_GI: number | null = null;
  PMTG_CU: number | null = null;
  PMTG_PL: number | null = null;
  PMTG_AF: number | null = null;
  PMTG_AD: number | null = null;
  PMTG_AFCV: number | null = null;
  PMTG_METH: string | null = null;
  PMTG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  PMTG_ENV: string | null = null;
  PMTG_REM: string | null = null;
  FILE_FSET: string | null = null;
  pmtds: PMTD[] = [];
  pmtls: PMTL[] = [];
  constructor(init: Partial<PMTG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PMTL extends AgsGroup {
  static readonly code = "PMTL";
  LOCA_ID: string | null = null;
  PMTG_DPTH: number | null = null;
  PMTG_TESN: string | null = null;
  PMTD_SEQ: number | null = null;
  PMTL_LNO: number | null = null;
  PMTL_GAA: number | null = null;
  PMTL_SINC: number | null = null;
  PMTL_PINC: number | null = null;
  PMTL_STRA: number | null = null;
  PMTL_PRSA: number | null = null;
  PMTL_NLSA: number | null = null;
  PMTL_NLSB: number | null = null;
  PMTL_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<PMTL> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PROJ extends AgsGroup {
  static readonly code = "PROJ";
  PROJ_ID: string | null = null;
  PROJ_NAME: string | null = null;
  PROJ_LOC: string | null = null;
  PROJ_CLNT: string | null = null;
  PROJ_CONT: string | null = null;
  PROJ_ENG: string | null = null;
  PROJ_MEMO: string | null = null;
  FILE_FSET: string | null = null;
  PROJ_OFFC: string | null = null;
  locas: LOCA[] = [];
  constructor(init: Partial<PROJ> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PTIM extends AgsGroup {
  static readonly code = "PTIM";
  LOCA_ID: string | null = null;
  PTIM_DTIM: Date | null = null;
  PTIM_DPTH: number | null = null;
  PTIM_CAS: number | null = null;
  PTIM_WAT: string | null = null;
  PTIM_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<PTIM> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PTST extends AgsGroup {
  static readonly code = "PTST";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  PTST_TESN: string | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  PTST_COND: string | null = null;
  PTST_SZUN: number | null = null;
  PTST_UNS: number | null = null;
  PTST_DIAM: number | null = null;
  PTST_LEN: number | null = null;
  PTST_MC: number | null = null;
  PTST_BDEN: number | null = null;
  PTST_DDEN: number | null = null;
  PTST_IDIA: number | null = null;
  PTST_DMET: string | null = null;
  PTST_VOID: number | null = null;
  PTST_K: number | null = null;
  PTST_TSTR: number | null = null;
  PTST_HYGR: number | null = null;
  PTST_ISAT: number | null = null;
  PTST_SAT: string | null = null;
  PTST_CONS: string | null = null;
  PTST_PDEN: string | null = null;
  PTST_TYPE: string | null = null;
  PTST_CELL: string | null = null;
  PTST_REM: string | null = null;
  PTST_METH: string | null = null;
  PTST_LAB: string | null = null;
  PTST_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<PTST> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RDEN extends AgsGroup {
  static readonly code = "RDEN";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  RDEN_MC: number | null = null;
  RDEN_SMC: number | null = null;
  RDEN_BDEN: number | null = null;
  RDEN_DDEN: number | null = null;
  RDEN_PORO: number | null = null;
  RDEN_PDEN: number | null = null;
  RDEN_TEMP: number | null = null;
  RDEN_REM: string | null = null;
  RDEN_METH: string | null = null;
  RDEN_LAB: string | null = null;
  RDEN_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<RDEN> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RELD extends AgsGroup {
  static readonly code = "RELD";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  RELD_DMAX: number | null = null;
  RELD_375: number | null = null;
  RELD_063: number | null = null;
  RELD_020: number | null = null;
  RELD_DMIN: number | null = null;
  RELD_REM: string | null = null;
  RELD_METH: string | null = null;
  RELD_LAB: string | null = null;
  RELD_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  RELD_SIZ1: number | null = null;
  RELD_SIZ2: number | null = null;
  RELD_SIZ3: number | null = null;
  constructor(init: Partial<RELD> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RPLT extends AgsGroup {
  static readonly code = "RPLT";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  RPLT_PLS: number | null = null;
  RPLT_PLSI: number | null = null;
  RPLT_PLTF: string | null = null;
  RPLT_MC: number | null = null;
  RPLT_REM: string | null = null;
  RPLT_METH: string | null = null;
  RPLT_LAB: string | null = null;
  RPLT_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<RPLT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RUCS extends AgsGroup {
  static readonly code = "RUCS";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  RUCS_SDIA: number | null = null;
  RUCS_LEN: number | null = null;
  RUCS_MC: number | null = null;
  RUCS_COND: string | null = null;
  RUCS_DURN: string | null = null;
  RUCS_STRA: number | null = null;
  RUCS_UCS: number | null = null;
  RUCS_MODE: string | null = null;
  RUCS_E: number | null = null;
  RUCS_MU: number | null = null;
  RUCS_ESTR: string | null = null;
  RUCS_ETYP: string | null = null;
  RUCS_MACH: string | null = null;
  RUCS_REM: string | null = null;
  RUCS_METH: string | null = null;
  RUCS_LAB: string | null = null;
  RUCS_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<RUCS> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RWCO extends AgsGroup {
  static readonly code = "RWCO";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  RWCO_MC: number | null = null;
  RWCO_TEMP: number | null = null;
  RWCO_REM: string | null = null;
  RWCO_METH: string | null = null;
  RWCO_LAB: string | null = null;
  RWCO_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<RWCO> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class SAMP extends AgsGroup {
  static readonly code = "SAMP";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SAMP_BASE: number | null = null;
  SAMP_DTIM: Date | null = null;
  SAMP_REM: string | null = null;
  SAMP_UBLO: number | null = null;
  SAMP_CONT: string | null = null;
  SAMP_PREP: string | null = null;
  SAMP_SDIA: number | null = null;
  SAMP_WDEP: string | null = null;
  SAMP_RECV: number | null = null;
  SAMP_TECH: string | null = null;
  SAMP_MATX: string | null = null;
  SAMP_TYPC: string | null = null;
  SAMP_WHO: string | null = null;
  SAMP_WHY: string | null = null;
  SAMP_DESC: string | null = null;
  SAMP_DESD: Date | null = null;
  SAMP_LOG: string | null = null;
  SAMP_COND: string | null = null;
  SAMP_CLSS: string | null = null;
  SAMP_BAR: number | null = null;
  SAMP_TEMP: number | null = null;
  SAMP_PRES: number | null = null;
  SAMP_FLOW: number | null = null;
  SAMP_ETIM: Date | null = null;
  SAMP_DURN: string | null = null;
  SAMP_CAPT: string | null = null;
  SAMP_LINK: string | null = null;
  GEOL_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SAMP_RECL: number | null = null;
  asdis: ASDI[] = [];
  cbrgs: CBRG[] = [];
  chocs: CHOC[] = [];
  cmpgs: CMPG[] = [];
  congs: CONG[] = [];
  conls: CONL[] = [];
  ectns: ECTN[] = [];
  eress: ERES[] = [];
  gchms: GCHM[] = [];
  grags: GRAG[] = [];
  ldens: LDEN[] = [];
  llins: LLIN[] = [];
  llpls: LLPL[] = [];
  lnmcs: LNMC[] = [];
  lpdns: LPDN[] = [];
  lress: LRES[] = [];
  lvans: LVAN[] = [];
  mcvgs: MCVG[] = [];
  ptsts: PTST[] = [];
  rdens: RDEN[] = [];
  relds: RELD[] = [];
  rplts: RPLT[] = [];
  rucss: RUCS[] = [];
  rwcos: RWCO[] = [];
  shbgs: SHBG[] = [];
  tregs: TREG[] = [];
  trems: TREM[] = [];
  trigs: TRIG[] = [];
  wadds: WADD[] = [];
  winss: WINS[] = [];
  constructor(init: Partial<SAMP> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class SCDG extends AgsGroup {
  static readonly code = "SCDG";
  LOCA_ID: string | null = null;
  SCPG_TESN: string | null = null;
  SCDG_DPTH: number | null = null;
  SCDG_PWPI: number | null = null;
  SCDG_PWPE: number | null = null;
  SCDG_DDIS: number | null = null;
  SCDG_T: number | null = null;
  SCDG_CV: number | null = null;
  SCDG_CVMT: string | null = null;
  SCDG_CH: number | null = null;
  SCDG_CHMT: string | null = null;
  SCDG_REM: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  scdts: SCDT[] = [];
  constructor(init: Partial<SCDG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class SCDT extends AgsGroup {
  static readonly code = "SCDT";
  LOCA_ID: string | null = null;
  SCPG_TESN: string | null = null;
  SCDG_DPTH: number | null = null;
  SCDT_SECS: number | null = null;
  SCDT_RES: number | null = null;
  SCDT_PWP1: number | null = null;
  SCDT_PWP2: number | null = null;
  SCDT_PWP3: number | null = null;
  SCDT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<SCDT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class SCPG extends AgsGroup {
  static readonly code = "SCPG";
  LOCA_ID: string | null = null;
  SCPG_TESN: string | null = null;
  SCPG_TYPE: string | null = null;
  SCPG_REF: string | null = null;
  SCPG_CSA: number | null = null;
  SCPG_RATE: number | null = null;
  SCPG_FILT: string | null = null;
  SCPG_FRIC: boolean | null = null;
  SCPG_WAT: number | null = null;
  SCPG_WATA: string | null = null;
  SCPG_REM: string | null = null;
  SCPG_ENV: string | null = null;
  SCPG_CONT: string | null = null;
  SCPG_METH: string | null = null;
  SCPG_CRED: string | null = null;
  SCPG_CAR: number | null = null;
  SCPG_SLAR: number | null = null;
  FILE_FSET: string | null = null;
  scdgs: SCDG[] = [];
  scpts: SCPT[] = [];
  constructor(init: Partial<SCPG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class SCPT extends AgsGroup {
  static readonly code = "SCPT";
  LOCA_ID: string | null = null;
  SCPG_TESN: string | null = null;
  SCPT_DPTH: number | null = null;
  SCPT_RES: number | null = null;
  SCPT_FRES: number | null = null;
  SCPT_PWP1: number | null = null;
  SCPT_PWP2: number | null = null;
  SCPT_PWP3: number | null = null;
  SCPT_CON: number | null = null;
  SCPT_TEMP: number | null = null;
  SCPT_PH: number | null = null;
  SCPT_SLP1: number | null = null;
  SCPT_SLP2: number | null = null;
  SCPT_REDX: number | null = null;
  SCPT_MAGT: number | null = null;
  SCPT_MAGX: number | null = null;
  SCPT_MAGY: number | null = null;
  SCPT_MAGZ: number | null = null;
  SCPT_SMP: number | null = null;
  SCPT_NGAM: number | null = null;
  SCPT_REM: string | null = null;
  SCPT_FRR: number | null = null;
  SCPT_QT: number | null = null;
  SCPT_FT: number | null = null;
  SCPT_QE: number | null = null;
  SCPT_BDEN: number | null = null;
  SCPT_CPO: number | null = null;
  SCPT_CPOD: number | null = null;
  SCPT_QNET: number | null = null;
  SCPT_FRRC: number | null = null;
  SCPT_EXPP: number | null = null;
  SCPT_BQ: number | null = null;
  SCPT_ISPP: number | null = null;
  SCPT_NQT: number | null = null;
  SCPT_NFR: number | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<SCPT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class SHBG extends AgsGroup {
  static readonly code = "SHBG";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  SHBG_TYPE: string | null = null;
  SHBG_COND: string | null = null;
  SHBG_CONS: string | null = null;
  SHBG_PCOH: number | null = null;
  SHBG_PHI: number | null = null;
  SHBG_RCOH: number | null = null;
  SHBG_RPHI: number | null = null;
  SHBG_ENCA: string | null = null;
  SHBG_REM: string | null = null;
  SHBG_METH: string | null = null;
  SHBG_LAB: string | null = null;
  SHBG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  shbts: SHBT[] = [];
  constructor(init: Partial<SHBG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class SHBT extends AgsGroup {
  static readonly code = "SHBT";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SHBT_TESN: string | null = null;
  SHBT_BDEN: number | null = null;
  SHBT_DDEN: number | null = null;
  SHBT_NORM: number | null = null;
  SHBT_DISP: number | null = null;
  SHBT_DISR: number | null = null;
  SHBT_REVS: number | null = null;
  SHBT_PEAK: number | null = null;
  SHBT_RES: number | null = null;
  SHBT_PDIS: number | null = null;
  SHBT_RDIS: number | null = null;
  SHBT_PDIN: number | null = null;
  SHBT_RDIN: number | null = null;
  SHBT_PDEN: string | null = null;
  SHBT_IVR: number | null = null;
  SHBT_MCI: number | null = null;
  SHBT_MCF: number | null = null;
  SHBT_DIA1: number | null = null;
  SHBT_DIA2: number | null = null;
  SHBT_HGT: number | null = null;
  SHBT_CRIT: string | null = null;
  SHBT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<SHBT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class TRAN extends AgsGroup {
  static readonly code = "TRAN";
  TRAN_ISNO: string | null = null;
  TRAN_DATE: Date | null = null;
  TRAN_PROD: string | null = null;
  TRAN_STAT: string | null = null;
  TRAN_DESC: string | null = null;
  TRAN_AGS: string | null = null;
  TRAN_RECV: string | null = null;
  TRAN_DLIM: string | null = null;
  TRAN_RCON: string | null = null;
  TRAN_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<TRAN> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class TREG extends AgsGroup {
  static readonly code = "TREG";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  SPEC_BASE: number | null = null;
  TREG_TYPE: string | null = null;
  TREG_COND: string | null = null;
  TREG_COH: number | null = null;
  TREG_PHI: number | null = null;
  TREG_FCR: string | null = null;
  TREG_METH: string | null = null;
  TREG_LAB: string | null = null;
  TREG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  TREG_REM: string | null = null;
  TREG_DEV: string | null = null;
  trets: TRET[] = [];
  constructor(init: Partial<TREG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class TREL extends AgsGroup {
  static readonly code = "TREL";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  TRET_TESN: string | null = null;
  TREL_MNUM: number | null = null;
  TREL_TTIM: number | null = null;
  TREL_TTDT: Date | null = null;
  TREL_STIM: number | null = null;
  TREL_STGN: number | null = null;
  TREL_STGD: string | null = null;
  TREL_CELL: number | null = null;
  TREL_BACK: number | null = null;
  TREL_PWP: number | null = null;
  TREL_PWPM: number | null = null;
  TREL_SZT: number | null = null;
  TREL_SZE: number | null = null;
  TREL_SRT: number | null = null;
  TREL_SRE: number | null = null;
  TREL_EZET: number | null = null;
  TREL_EZES: number | null = null;
  TREL_EPET: number | null = null;
  TREL_EPES: number | null = null;
  TREL_EZ1T: number | null = null;
  TREL_EZ1S: number | null = null;
  TREL_EZ2T: number | null = null;
  TREL_EZ2S: number | null = null;
  TREL_ER1T: number | null = null;
  TREL_ER1S: number | null = null;
  TREL_CYCN: number | null = null;
  constructor(init: Partial<TREL> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class TREM extends AgsGroup {
  static readonly code = "TREM";
  LOCA_ID: string | null = null;
  TREM_DTIM: Date | null = null;
  TREM_COMP: string | null = null;
  TREM_REM: string | null = null;
  TREM_DURN: string | null = null;
  TREM_ETIM: Date | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<TREM> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class TRET extends AgsGroup {
  static readonly code = "TRET";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  TRET_TESN: string | null = null;
  TRET_REM: string | null = null;
  TRET_SDIA: number | null = null;
  TRET_LEN: number | null = null;
  TRET_IMC: number | null = null;
  TRET_FMC: number | null = null;
  TRET_BDEN: number | null = null;
  TRET_DDEN: number | null = null;
  TRET_SAT: string | null = null;
  TRET_CONS: string | null = null;
  TRET_CONP: number | null = null;
  TRET_CELL: number | null = null;
  TRET_PWPI: number | null = null;
  TRET_STRR: number | null = null;
  TRET_STRN: number | null = null;
  TRET_DEVF: number | null = null;
  TRET_PWPF: number | null = null;
  TRET_STV: number | null = null;
  TRET_MODE: string | null = null;
  FILE_FSET: string | null = null;
  trels: TREL[] = [];
  constructor(init: Partial<TRET> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class TRIG extends AgsGroup {
  static readonly code = "TRIG";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  TRIG_TYPE: string | null = null;
  TRIG_COND: string | null = null;
  TRIG_REM: string | null = null;
  TRIG_METH: string | null = null;
  TRIG_LAB: string | null = null;
  TRIG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  trits: TRIT[] = [];
  constructor(init: Partial<TRIG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class TRIL extends AgsGroup {
  static readonly code = "TRIL";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  TRIT_TESN: string | null = null;
  TRIL_MNUM: number | null = null;
  TRIL_TTIM: number | null = null;
  TRIL_TTDT: Date | null = null;
  TRIL_STIM: number | null = null;
  TRIL_STGN: number | null = null;
  TRIL_STGD: string | null = null;
  TRIL_CELL: number | null = null;
  TRIL_SDEV: number | null = null;
  TRIL_EZES: number | null = null;
  constructor(init: Partial<TRIL> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class TRIT extends AgsGroup {
  static readonly code = "TRIT";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  TRIT_TESN: string | null = null;
  TRIT_SDIA: number | null = null;
  TRIT_SLEN: number | null = null;
  TRIT_IMC: number | null = null;
  TRIT_FMC: number | null = null;
  TRIT_CELL: number | null = null;
  TRIT_DEVF: number | null = null;
  TRIT_BDEN: number | null = null;
  TRIT_DDEN: number | null = null;
  TRIT_STRN: number | null = null;
  TRIT_CU: number | null = null;
  TRIT_MODE: string | null = null;
  TRIT_REM: string | null = null;
  FILE_FSET: string | null = null;
  trils: TRIL[] = [];
  constructor(init: Partial<TRIT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class TYPE extends AgsGroup {
  static readonly code = "TYPE";
  TYPE_TYPE: string | null = null;
  TYPE_DESC: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<TYPE> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class UNIT extends AgsGroup {
  static readonly code = "UNIT";
  UNIT_UNIT: string | null = null;
  UNIT_DESC: string | null = null;
  UNIT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<UNIT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class WADD extends AgsGroup {
  static readonly code = "WADD";
  LOCA_ID: string | null = null;
  WADD_TOP: number | null = null;
  WADD_BASE: number | null = null;
  WADD_VOLM: number | null = null;
  WADD_METH: string | null = null;
  WADD_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<WADD> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class WETH extends AgsGroup {
  static readonly code = "WETH";
  LOCA_ID: string | null = null;
  WETH_TOP: number | null = null;
  WETH_BASE: number | null = null;
  WETH_SCH: string | null = null;
  WETH_SYS: string | null = null;
  WETH_WETH: string | null = null;
  WETH_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<WETH> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class WINS extends AgsGroup {
  static readonly code = "WINS";
  LOCA_ID: string | null = null;
  WINS_TESN: string | null = null;
  WINS_TOP: number | null = null;
  WINS_BASE: number | null = null;
  WINS_DIAM: number | null = null;
  WINS_DURN: string | null = null;
  WINS_REC: number | null = null;
  WINS_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<WINS> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class WSTD extends AgsGroup {
  static readonly code = "WSTD";
  LOCA_ID: string | null = null;
  WSTG_DPTH: number | null = null;
  WSTD_NMIN: number | null = null;
  WSTD_POST: string | null = null;
  WSTD_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<WSTD> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class WSTG extends AgsGroup {
  static readonly code = "WSTG";
  LOCA_ID: string | null = null;
  WSTG_DPTH: number | null = null;
  WSTG_DTIM: Date | null = null;
  WSTG_SEAL: number | null = null;
  WSTG_CAS: number | null = null;
  WSTG_REM: string | null = null;
  FILE_FSET: string | null = null;
  wstds: WSTD[] = [];
  constructor(init: Partial<WSTG> = {}) {
    super();
    Object.assign(this, init);
  }
}
