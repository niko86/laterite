// AUTO-GENERATED from ags_dictionary.json by tools/generate-typed-graph.mjs.
// DO NOT EDIT — re-run the generator after a dictionary change.

/* eslint-disable */
// A typed builder graph: `new PROJ({ PROJ_ID: 'P1', locas: [new LOCA({…})] })`,
// then `buildAgs4(proj)` walks it into per-group rows. Each class carries a
// static `code` and extends AgsGroup; child arrays are
// `<childCode>`.toLowerCase() + 's'.
import { AgsGroup } from "./ags-group";

export class AAVT extends AgsGroup {
  static readonly code = "AAVT";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  AAVT_AAV: number | null = null;
  AAVT_REM: string | null = null;
  AAVT_METH: string | null = null;
  AAVT_LAB: string | null = null;
  AAVT_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  AAVT_DEV: string | null = null;
  constructor(init: Partial<AAVT> = {}) {
    super();
    Object.assign(this, init);
  }
}

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

export class ACVT extends AgsGroup {
  static readonly code = "ACVT";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  ACVT_ACV: number | null = null;
  ACVT_FRAC: string | null = null;
  ACVT_REM: string | null = null;
  ACVT_METH: string | null = null;
  ACVT_LAB: string | null = null;
  ACVT_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  ACVT_DEV: string | null = null;
  constructor(init: Partial<ACVT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class AELO extends AgsGroup {
  static readonly code = "AELO";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  AELO_EI: number | null = null;
  AELO_REM: string | null = null;
  AELO_METH: string | null = null;
  AELO_LAB: string | null = null;
  AELO_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  AELO_DEV: string | null = null;
  constructor(init: Partial<AELO> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class AFLK extends AgsGroup {
  static readonly code = "AFLK";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  AFLK_FI: number | null = null;
  AFLK_MASS: number | null = null;
  AFLK_REM: string | null = null;
  AFLK_METH: string | null = null;
  AFLK_LAB: string | null = null;
  AFLK_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  AFLK_DEV: string | null = null;
  constructor(init: Partial<AFLK> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class AIVT extends AgsGroup {
  static readonly code = "AIVT";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  AIVT_AIV1: number | null = null;
  AIVT_AIV2: number | null = null;
  AIVT_AIV: number | null = null;
  AIVT_FRAC: string | null = null;
  AIVT_PDEN: number | null = null;
  AIVT_REM: string | null = null;
  AIVT_METH: string | null = null;
  AIVT_LAB: string | null = null;
  AIVT_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  AIVT_DEV: string | null = null;
  constructor(init: Partial<AIVT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ALOS extends AgsGroup {
  static readonly code = "ALOS";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  ALOS_LOSA: number | null = null;
  ALOS_LOPW: number | null = null;
  ALOS_LOWR: number | null = null;
  ALOS_FRAC: string | null = null;
  ALOS_CHAR: string | null = null;
  ALOS_REM: string | null = null;
  ALOS_METH: string | null = null;
  ALOS_LAB: string | null = null;
  ALOS_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  ALOS_DEV: string | null = null;
  constructor(init: Partial<ALOS> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class APSV extends AgsGroup {
  static readonly code = "APSV";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  APSV_AAV: number | null = null;
  APSV_REM: string | null = null;
  APSV_METH: string | null = null;
  APSV_LAB: string | null = null;
  APSV_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  APSV_DEV: string | null = null;
  constructor(init: Partial<APSV> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ARTW extends AgsGroup {
  static readonly code = "ARTW";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  ARTW_FRAC: string | null = null;
  ARTW_TYPE: string | null = null;
  ARTW_MD1: number | null = null;
  ARTW_MD2: number | null = null;
  ARTW_MDE: number | null = null;
  ARTW_MDS: number | null = null;
  ARTW_DATE: Date | null = null;
  ARTW_REM: string | null = null;
  ARTW_METH: string | null = null;
  ARTW_LAB: string | null = null;
  ARTW_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  ARTW_DEV: string | null = null;
  constructor(init: Partial<ARTW> = {}) {
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
  SPEC_BASE: number | null = null;
  ASDI_DEV: string | null = null;
  constructor(init: Partial<ASDI> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ASNS extends AgsGroup {
  static readonly code = "ASNS";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  ASNS_SOUN: number | null = null;
  ASNS_FRAC: string | null = null;
  ASNS_REM: string | null = null;
  ASNS_METH: string | null = null;
  ASNS_LAB: string | null = null;
  ASNS_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  ASNS_DEV: string | null = null;
  constructor(init: Partial<ASNS> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class AWAD extends AgsGroup {
  static readonly code = "AWAD";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  AWAD_WTAB: number | null = null;
  AWAD_REM: string | null = null;
  AWAD_METH: string | null = null;
  AWAD_LAB: string | null = null;
  AWAD_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  AWAD_DEV: string | null = null;
  constructor(init: Partial<AWAD> = {}) {
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
  CBRG_NMC: string | null = null;
  CBRG_200: number | null = null;
  CBRG_STAB: number | null = null;
  CBRG_STYP: string | null = null;
  CBRG_REM: string | null = null;
  CBRG_METH: string | null = null;
  CBRG_LAB: string | null = null;
  CBRG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  CBRG_DEV: string | null = null;
  cbrts: CBRT[] = [];
  constructor(init: Partial<CBRG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CBRP extends AgsGroup {
  static readonly code = "CBRP";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  CBRT_TESN: string | null = null;
  CBRP_END: string | null = null;
  CBRP_PEN: number | null = null;
  CBRP_LOAD: number | null = null;
  constructor(init: Partial<CBRP> = {}) {
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
  CBRT_TOP: string | null = null;
  CBRT_BASE: string | null = null;
  CBRT_MCT: string | null = null;
  CBRT_MCBT: string | null = null;
  CBRT_IMC: string | null = null;
  CBRT_BDEN: number | null = null;
  CBRT_DDEN: number | null = null;
  CBRT_SURC: number | null = null;
  CBRT_SKDT: string | null = null;
  CBRT_SWEL: number | null = null;
  CBRT_REM: string | null = null;
  FILE_FSET: string | null = null;
  cbrps: CBRP[] = [];
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
  SPEC_BASE: number | null = null;
  CMPG_DEV: string | null = null;
  CMPG_ZONE: string | null = null;
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
  CMPT_MC: string | null = null;
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
  CONG_MCI: string | null = null;
  CONG_MCF: string | null = null;
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
  SPEC_BASE: number | null = null;
  CONG_DEV: string | null = null;
  CONG_MCIS: string | null = null;
  CONG_CORR: boolean | null = null;
  conss: CONS[] = [];
  constructor(init: Partial<CONG> = {}) {
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
  CORE_DIAM: number | null = null;
  CORE_DURN: string | null = null;
  CORE_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CORE> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CPDG extends AgsGroup {
  static readonly code = "CPDG";
  LOCA_ID: string | null = null;
  CPTG_TESN: string | null = null;
  CPDG_DPTH: number | null = null;
  CPDG_IR: number | null = null;
  CPDG_RCMP: boolean | null = null;
  CPDG_UI: number | null = null;
  CPDG_UIP: string | null = null;
  CPDG_M: number | null = null;
  CPDG_UEQ: number | null = null;
  CPDG_UEP: string | null = null;
  CPDG_DDIS: number | null = null;
  CPDG_T: number | null = null;
  CPDG_CH: number | null = null;
  CPDG_CHMT: string | null = null;
  CPDG_CV: number | null = null;
  CPDG_CVMT: string | null = null;
  CPDG_REM: string | null = null;
  CPDG_DATE: Date | null = null;
  CPDG_OPER: string | null = null;
  CPDG_ANBY: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  cpdts: CPDT[] = [];
  constructor(init: Partial<CPDG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CPDT extends AgsGroup {
  static readonly code = "CPDT";
  LOCA_ID: string | null = null;
  CPTG_TESN: string | null = null;
  CPDG_DPTH: number | null = null;
  CPDT_TIME: number | null = null;
  CPDT_QC: number | null = null;
  CPDT_TF: number | null = null;
  CPDT_FS: number | null = null;
  CPDT_U1: number | null = null;
  CPDT_U2: number | null = null;
  CPDT_U3: number | null = null;
  CPDT_TMPI: number | null = null;
  CPDT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CPDT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CPTG extends AgsGroup {
  static readonly code = "CPTG";
  LOCA_ID: string | null = null;
  CPTG_TESN: string | null = null;
  CPTG_TYPE: string | null = null;
  CPTG_DATE: Date | null = null;
  CPTG_PED: number | null = null;
  CPTG_RATE: number | null = null;
  CPTG_ORNT: number | null = null;
  CPTG_RLOC: string | null = null;
  CPTG_WAT: number | null = null;
  CPTG_WATA: string | null = null;
  CPTG_TERM: string | null = null;
  CPTG_REF: string | null = null;
  CPTG_MAN: string | null = null;
  CPTG_FILL: string | null = null;
  CPTG_CSA: number | null = null;
  CPTG_CSAN: number | null = null;
  CPTG_CAR: number | null = null;
  CPTG_SLA: number | null = null;
  CPTG_SLAN: number | null = null;
  CPTG_SHA: number | null = null;
  CPTG_SLAR: number | null = null;
  CPTG_CFOS: number | null = null;
  CPTG_CFOA: number | null = null;
  CPTG_TBL: number | null = null;
  CPTG_TBD: number | null = null;
  CPTG_CPC: number | null = null;
  CPTG_FPC: number | null = null;
  CPTG_UPC: number | null = null;
  CPTG_CPCL: string | null = null;
  CPTG_CRDT: Date | null = null;
  CPTG_CDDT: Date | null = null;
  CPTG_LCA: string | null = null;
  CPTG_FILT: string | null = null;
  CPTG_FRIC: boolean | null = null;
  CPTG_FRID: number | null = null;
  CPTG_FRIS: number | null = null;
  CPTG_SAT: string | null = null;
  CPTG_EQPT: string | null = null;
  CPTG_APCL: string | null = null;
  CPTG_DAZV: string | null = null;
  CPTG_CORR: string | null = null;
  CPTG_REM: string | null = null;
  CPTG_OPER: string | null = null;
  CPTG_ANBY: string | null = null;
  CPTG_ENV: string | null = null;
  CPTG_METH: string | null = null;
  CPTG_DEV: string | null = null;
  CPTG_CONT: string | null = null;
  CPTG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  cpdgs: CPDG[] = [];
  cptts: CPTT[] = [];
  cptys: CPTY[] = [];
  cptzs: CPTZ[] = [];
  constructor(init: Partial<CPTG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CPTM extends AgsGroup {
  static readonly code = "CPTM";
  LOCA_ID: string | null = null;
  CPTM_DPTH: number | null = null;
  CPTM_BASE: number | null = null;
  CPTM_SBT1: string | null = null;
  CPTM_SU1: string | null = null;
  CPTM_SU2: string | null = null;
  CPTM_DR1: string | null = null;
  CPTM_DR2: string | null = null;
  CPTM_PHI1: string | null = null;
  CPTM_IC1: string | null = null;
  CPTM_N601: string | null = null;
  CPTM_E1: string | null = null;
  CPTM_MV1: string | null = null;
  CPTM_G01: string | null = null;
  CPTM_VS1: string | null = null;
  CPTM_DUW1: string | null = null;
  CPTM_SUW1: string | null = null;
  CPTM_M1: string | null = null;
  CPTM_CC1: string | null = null;
  CPTM_P01: string | null = null;
  CPTM_ST1: string | null = null;
  CPTM_K01: string | null = null;
  CPTM_IR1: string | null = null;
  CPTM_K1: string | null = null;
  CPTM_FC1: string | null = null;
  CPTM_CSR1: string | null = null;
  CPTM_CRR1: string | null = null;
  CPTM_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CPTM> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CPTP extends AgsGroup {
  static readonly code = "CPTP";
  LOCA_ID: string | null = null;
  CPTP_DPTH: number | null = null;
  CPTP_BASE: number | null = null;
  CPTP_SBT1: string | null = null;
  CPTP_SU1: number | null = null;
  CPTP_SU2: number | null = null;
  CPTP_DR1: number | null = null;
  CPTP_DR2: number | null = null;
  CPTP_PHI1: number | null = null;
  CPTP_IC1: number | null = null;
  CPTP_N601: number | null = null;
  CPTP_E1: number | null = null;
  CPTP_MV1: number | null = null;
  CPTP_G01: number | null = null;
  CPTP_VS1: number | null = null;
  CPTP_DUW1: number | null = null;
  CPTP_SUW1: number | null = null;
  CPTP_M1: number | null = null;
  CPTP_CC1: number | null = null;
  CPTP_P01: number | null = null;
  CPTP_ST1: number | null = null;
  CPTP_K01: number | null = null;
  CPTP_IR1: number | null = null;
  CPTP_K1: number | null = null;
  CPTP_FC1: number | null = null;
  CPTP_CSR1: number | null = null;
  CPTP_CRR1: number | null = null;
  CPTP_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CPTP> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CPTT extends AgsGroup {
  static readonly code = "CPTT";
  LOCA_ID: string | null = null;
  CPTG_TESN: string | null = null;
  CPTT_REDN: number | null = null;
  CPTT_DPTH: number | null = null;
  CPTT_PLEN: string | null = null;
  CPTT_QC: number | null = null;
  CPTT_FS: number | null = null;
  CPTT_U1: number | null = null;
  CPTT_U2: number | null = null;
  CPTT_U3: number | null = null;
  CPTT_INCX: number | null = null;
  CPTT_INCY: number | null = null;
  CPTT_TIME: Date | null = null;
  CPTT_DUR: number | null = null;
  CPTT_TF: number | null = null;
  CPTT_RF: number | null = null;
  CPTT_BDEN: number | null = null;
  CPTT_CPO: number | null = null;
  CPTT_ISPP: number | null = null;
  CPTT_CPOD: number | null = null;
  CPTT_QT: number | null = null;
  CPTT_FT: number | null = null;
  CPTT_QNET: number | null = null;
  CPTT_QE: number | null = null;
  CPTT_RFT: number | null = null;
  CPTT_EXPP: number | null = null;
  CPTT_BQ: number | null = null;
  CPTT_NQT: number | null = null;
  CPTT_NFR: number | null = null;
  CPTT_MAGX: number | null = null;
  CPTT_MAGY: number | null = null;
  CPTT_MAGZ: number | null = null;
  CPTT_MAGT: number | null = null;
  CPTT_MAGG: number | null = null;
  CPTT_CON: number | null = null;
  CPTT_TEMP: number | null = null;
  CPTT_TPQC: number | null = null;
  CPTT_TPFS: number | null = null;
  CPTT_TPU: number | null = null;
  CPTT_PH: number | null = null;
  CPTT_REDX: number | null = null;
  CPTT_SMP: number | null = null;
  CPTT_NGAM: number | null = null;
  CPTT_FFD1: number | null = null;
  CPTT_FFD2: number | null = null;
  CPTT_PID: number | null = null;
  CPTT_FID: number | null = null;
  CPTT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CPTT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CPTY extends AgsGroup {
  static readonly code = "CPTY";
  LOCA_ID: string | null = null;
  CPTG_TESN: string | null = null;
  CPTY_TESN: string | null = null;
  CPTY_DPTH: number | null = null;
  CPTY_DINT: number | null = null;
  CPTY_NUMC: number | null = null;
  CPTY_REDI: number | null = null;
  CPTY_REDF: number | null = null;
  CPTY_TIMI: number | null = null;
  CPTY_TIMF: number | null = null;
  CPTY_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CPTY> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CPTZ extends AgsGroup {
  static readonly code = "CPTZ";
  LOCA_ID: string | null = null;
  CPTG_TESN: string | null = null;
  CPTZ_PARM: string | null = null;
  CPTZ_ZBD: string | null = null;
  CPTZ_ZB: string | null = null;
  CPTZ_ZA: string | null = null;
  CPTZ_ZAD: string | null = null;
  CPTZ_ZAC: string | null = null;
  CPTZ_ZD: number | null = null;
  CPTZ_ZDD: number | null = null;
  CPTZ_ZDC: number | null = null;
  CPTZ_CD: number | null = null;
  CPTZ_ZS: number | null = null;
  CPTZ_ZSS: string | null = null;
  CPTZ_ZVUC: string | null = null;
  CPTZ_EGUT: string | null = null;
  CPTZ_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CPTZ> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CTRC extends AgsGroup {
  static readonly code = "CTRC";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  CTRC_TESN: string | null = null;
  CTRC_CELL: number | null = null;
  CTRC_BPWP: number | null = null;
  CTRC_MPWP: number | null = null;
  CTRC_MPB: number | null = null;
  CTRC_BB: number | null = null;
  CTRC_TYPE: string | null = null;
  CTRC_BACF: number | null = null;
  CTRC_ELAP: string | null = null;
  CTRC_CHGT: number | null = null;
  CTRC_DIAE: number | null = null;
  CTRC_MCE: string | null = null;
  CTRC_BDE: number | null = null;
  CTRC_DDE: number | null = null;
  CTRC_RDE: number | null = null;
  CTRC_INCE: number | null = null;
  CTRC_ASE: number | null = null;
  CTRC_RSE: number | null = null;
  CTRC_SSE: number | null = null;
  CTRC_DEVE: number | null = null;
  CTRC_MNSE: number | null = null;
  CTRC_RTOE: number | null = null;
  CTRC_EASE: number | null = null;
  CTRC_VLSE: number | null = null;
  CTRC_RDSE: number | null = null;
  CTRC_B: number | null = null;
  CTRC_BETS: string | null = null;
  CTRC_BEAX: string | null = null;
  CTRC_BEDS: number | null = null;
  CTRC_MAT: number | null = null;
  CTRC_MATM: string | null = null;
  CTRC_SWV: number | null = null;
  CTRC_SMGM: number | null = null;
  CTRC_REM: string | null = null;
  FILE_FSET: string | null = null;
  ctrps: CTRP[] = [];
  constructor(init: Partial<CTRC> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CTRD extends AgsGroup {
  static readonly code = "CTRD";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  CTRC_TESN: string | null = null;
  CTRP_CYC: number | null = null;
  CTRD_TIME: Date | null = null;
  CTRD_COND: string | null = null;
  CTRD_SDIA: number | null = null;
  CTRD_HIGH: number | null = null;
  CTRD_CELL: number | null = null;
  CTRD_BPWP: number | null = null;
  CTRD_MPWP: number | null = null;
  CTRD_EAS: number | null = null;
  CTRD_LAS1: number | null = null;
  CTRD_LAS2: number | null = null;
  CTRD_VOL: number | null = null;
  CTRD_RAD: number | null = null;
  CTRD_SHSN: number | null = null;
  CTRD_SHST: number | null = null;
  CTRD_DEV: number | null = null;
  CTRD_PSD: number | null = null;
  CTRD_MEES: number | null = null;
  CTRD_SECE: number | null = null;
  CTRD_TANE: number | null = null;
  CTRD_FREQ: number | null = null;
  CTRD_CSTS: number | null = null;
  CTRD_ACVS: number | null = null;
  CTRD_DAVS: number | null = null;
  CTRD_CESR: number | null = null;
  CTRD_EMPR: number | null = null;
  CTRD_EBPR: number | null = null;
  CTRD_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CTRD> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CTRG extends AgsGroup {
  static readonly code = "CTRG";
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
  CTRG_TYPE: string | null = null;
  CTRG_MCI: string | null = null;
  CTRG_MCF: string | null = null;
  CTRG_H2O: string | null = null;
  CTRG_SBP: number | null = null;
  CTRG_SATR: number | null = null;
  CTRG_IRD: number | null = null;
  CTRG_SDIA: number | null = null;
  CTRG_HIGT: number | null = null;
  CTRG_TMSS: number | null = null;
  CTRG_PDEN: string | null = null;
  CTRG_MADD: number | null = null;
  CTRG_MIDD: number | null = null;
  CTRG_DDEN: number | null = null;
  CTRG_BDEN: number | null = null;
  CTRG_IVR: number | null = null;
  CTRG_SAT: string | null = null;
  CTRG_DURN: number | null = null;
  CTRG_REM: string | null = null;
  CTRG_METH: string | null = null;
  CTRG_DEV: string | null = null;
  CTRG_LAB: string | null = null;
  CTRG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  ctrcs: CTRC[] = [];
  ctrss: CTRS[] = [];
  constructor(init: Partial<CTRG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CTRP extends AgsGroup {
  static readonly code = "CTRP";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  CTRC_TESN: string | null = null;
  CTRP_CYC: number | null = null;
  CTRP_CYCF: number | null = null;
  CTRP_PWPM: number | null = null;
  CTRP_MNPP: number | null = null;
  CTRP_MXSS: number | null = null;
  CTRP_MNSS: number | null = null;
  CTRP_AVSS: number | null = null;
  CTRP_CSS: number | null = null;
  CTRP_ACVS: number | null = null;
  CTRP_ASF: number | null = null;
  CTRP_FPWP: number | null = null;
  CTRP_QMAX: number | null = null;
  CTRP_QMIN: number | null = null;
  CTRP_MNES: number | null = null;
  CTRP_EAMX: number | null = null;
  CTRP_EAMN: number | null = null;
  CTRP_FVR: number | null = null;
  CTRP_QEMX: number | null = null;
  CTRP_QEMN: number | null = null;
  CTRP_ESEC: number | null = null;
  CTRP_DAMP: number | null = null;
  CTRP_MODE: string | null = null;
  CTRP_DIPL: number | null = null;
  CTRP_OBP: string | null = null;
  CTRP_REM: string | null = null;
  FILE_FSET: string | null = null;
  ctrds: CTRD[] = [];
  constructor(init: Partial<CTRP> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class CTRS extends AgsGroup {
  static readonly code = "CTRS";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  CTRS_TESN: string | null = null;
  CTRS_CELL: number | null = null;
  CTRS_BPWP: number | null = null;
  CTRS_MPWP: number | null = null;
  CTRS_MPB: number | null = null;
  CTRS_BB: number | null = null;
  CTRS_SAT: string | null = null;
  CTRS_FSAT: number | null = null;
  CTRS_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<CTRS> = {}) {
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
  DCPG_OPER: string | null = null;
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
  DISC_DIP: string | null = null;
  DISC_DIR: string | null = null;
  DISC_RGH: string | null = null;
  DISC_PLAN: string | null = null;
  DISC_WAVE: number | null = null;
  DISC_AMP: number | null = null;
  DISC_JRC: number | null = null;
  DISC_APP: string | null = null;
  DISC_APT: string | null = null;
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
  DISC_MID: number | null = null;
  constructor(init: Partial<DISC> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DLOG extends AgsGroup {
  static readonly code = "DLOG";
  LOCA_ID: string | null = null;
  DLOG_TOP: number | null = null;
  DLOG_BASE: number | null = null;
  DLOG_DESC: string | null = null;
  DLOG_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<DLOG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DMDG extends AgsGroup {
  static readonly code = "DMDG";
  LOCA_ID: string | null = null;
  DMTG_TESN: string | null = null;
  DMDG_DPTH: number | null = null;
  DMDG_TFLX: number | null = null;
  DMDG_CH: number | null = null;
  DMDG_CHMT: string | null = null;
  DMDG_MH: number | null = null;
  DMDG_MHMT: string | null = null;
  DMDG_KH: number | null = null;
  DMDG_KHMT: string | null = null;
  DMDG_DATE: Date | null = null;
  TEST_STAT: string | null = null;
  DMDG_REM: string | null = null;
  FILE_FSET: string | null = null;
  dmdts: DMDT[] = [];
  constructor(init: Partial<DMDG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DMDT extends AgsGroup {
  static readonly code = "DMDT";
  LOCA_ID: string | null = null;
  DMTG_TESN: string | null = null;
  DMDG_DPTH: number | null = null;
  DMDT_TIME: number | null = null;
  DMDT_A: number | null = null;
  DMDT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<DMDT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DMTG extends AgsGroup {
  static readonly code = "DMTG";
  LOCA_ID: string | null = null;
  DMTG_TESN: string | null = null;
  DMTG_DATE: Date | null = null;
  DMTG_ORNT: number | null = null;
  DMTG_PED: number | null = null;
  DMTG_WAT: number | null = null;
  DMTG_WATA: string | null = null;
  DMTG_TYPE: string | null = null;
  DMTG_REFB: string | null = null;
  DMTG_REFA: string | null = null;
  DMTG_MAN: string | null = null;
  DMTG_RIG: string | null = null;
  DMTG_EQPT: string | null = null;
  DMTG_COT: string | null = null;
  DMTG_TDR: string | null = null;
  DMTG_DIMS: string | null = null;
  DMTG_PRSG: string | null = null;
  DMTG_FRIC: string | null = null;
  DMTG_DITH: number | null = null;
  DMTG_BCVA: number | null = null;
  DMTG_BCVB: number | null = null;
  DMTG_FAED: number | null = null;
  DMTG_FAS0: number | null = null;
  DMTG_TERM: string | null = null;
  DMTG_CORR: string | null = null;
  DMTG_REM: string | null = null;
  DMTG_OPER: string | null = null;
  DMTG_ANBY: string | null = null;
  DMTG_ENV: string | null = null;
  DMTG_METH: string | null = null;
  DMTG_DEV: string | null = null;
  DMTG_CONT: string | null = null;
  DMTG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  dmdgs: DMDG[] = [];
  dmtts: DMTT[] = [];
  dmtzs: DMTZ[] = [];
  constructor(init: Partial<DMTG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DMTP extends AgsGroup {
  static readonly code = "DMTP";
  LOCA_ID: string | null = null;
  DMTG_TESN: string | null = null;
  DMTT_DPTH: number | null = null;
  DMTP_BUW: number | null = null;
  DMTP_TVS: number | null = null;
  DMTP_EVS: number | null = null;
  DMTP_U0: number | null = null;
  DMTP_ID: number | null = null;
  DMTP_KD: number | null = null;
  DMTP_ED: number | null = null;
  DMTP_UD: number | null = null;
  DMTP_VS: number | null = null;
  DMTP_VDM: number | null = null;
  DMTP_SU: number | null = null;
  DMTP_PHI: number | null = null;
  DMTP_K0: number | null = null;
  DMTP_THS: number | null = null;
  DMTP_EHS: number | null = null;
  DMTP_OCR: number | null = null;
  DMTP_MPS: number | null = null;
  DMTP_DSD: string | null = null;
  DMTP_BUWM: string | null = null;
  DMTP_TVSM: string | null = null;
  DMTP_EVSM: string | null = null;
  DMTP_U0M: string | null = null;
  DMTP_IDM: string | null = null;
  DMTP_KDM: string | null = null;
  DMTP_EDM: string | null = null;
  DMTP_UDM: string | null = null;
  DMTP_VSM: string | null = null;
  DMTP_VDMM: string | null = null;
  DMTP_SUM: string | null = null;
  DMTP_PHIM: string | null = null;
  DMTP_K0M: string | null = null;
  DMTP_THSM: string | null = null;
  DMTP_EHSM: string | null = null;
  DMTP_OCRM: string | null = null;
  DMTP_MPSM: string | null = null;
  DMTP_DSDM: string | null = null;
  DMTP_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<DMTP> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DMTT extends AgsGroup {
  static readonly code = "DMTT";
  LOCA_ID: string | null = null;
  DMTG_TESN: string | null = null;
  DMTT_DPTH: number | null = null;
  DMTT_MTH: number | null = null;
  DMTT_BCVA: number | null = null;
  DMTT_BCVB: number | null = null;
  DMTT_TMST: Date | null = null;
  DMTT_A: number | null = null;
  DMTT_TMA: number | null = null;
  DMTT_B: number | null = null;
  DMTT_TMB: number | null = null;
  DMTT_C: number | null = null;
  DMTT_TMC: number | null = null;
  DMTT_P0: number | null = null;
  DMTT_P1: number | null = null;
  DMTT_P2: number | null = null;
  DMTT_INCX: number | null = null;
  DMTT_INCY: number | null = null;
  DMTT_RATE: number | null = null;
  DMTT_REM: string | null = null;
  FILE_FSET: string | null = null;
  dmtps: DMTP[] = [];
  constructor(init: Partial<DMTT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class DMTZ extends AgsGroup {
  static readonly code = "DMTZ";
  LOCA_ID: string | null = null;
  DMTG_TESN: string | null = null;
  DMTZ_DATE: Date | null = null;
  DMTZ_TYPE: string | null = null;
  DMTZ_BCVA: number | null = null;
  DMTZ_BCVB: number | null = null;
  DMTZ_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<DMTZ> = {}) {
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
  DOBS_METH: string | null = null;
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
  DPRG_OPER: string | null = null;
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
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  ECTN_ID: string | null = null;
  ECTN_TYPE: string | null = null;
  ECTN_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<ECTN> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ELRG extends AgsGroup {
  static readonly code = "ELRG";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  ELRG_CODE: string | null = null;
  ELRG_METH: string | null = null;
  ELRG_MATX: string | null = null;
  ELRG_RTYP: string | null = null;
  ELRG_TADE: string | null = null;
  ELRG_TICN: string | null = null;
  ELRG_RUNI: string | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  SPEC_BASE: number | null = null;
  ELRG_LSID: string | null = null;
  ELRG_RTCD: string | null = null;
  ELRG_IQLF: string | null = null;
  ELRG_LQLF: string | null = null;
  ELRG_RVAL: string | null = null;
  ELRG_RTXT: string | null = null;
  ELRG_NAME: string | null = null;
  ELRG_TNAM: string | null = null;
  ELRG_DCAT: string | null = null;
  ELRG_TESN: string | null = null;
  ELRG_FDEV: boolean | null = null;
  ELRG_DEV: string | null = null;
  ELRG_RRES: boolean | null = null;
  ELRG_DETF: boolean | null = null;
  ELRG_ORG: boolean | null = null;
  ELRG_RDLM: string | null = null;
  ELRG_MDLM: string | null = null;
  ELRG_QLM: string | null = null;
  ELRG_DUNI: string | null = null;
  ELRG_CASC: string | null = null;
  ELRG_TICP: number | null = null;
  ELRG_TICT: number | null = null;
  ELRG_RDAT: Date | null = null;
  ELRG_SGRP: string | null = null;
  ELRG_DTIM: Date | null = null;
  ELRG_TEST: string | null = null;
  ELRG_TORD: string | null = null;
  ELRG_LOCN: string | null = null;
  ELRG_BAS: string | null = null;
  ELRG_DIL: number | null = null;
  ELRG_LMTH: string | null = null;
  ELRG_LDTM: Date | null = null;
  ELRG_IREF: string | null = null;
  ELRG_ITYP: string | null = null;
  ELRG_SIZE: number | null = null;
  ELRG_PERP: number | null = null;
  ELRG_REM: string | null = null;
  ELRG_LAB: string | null = null;
  ELRG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<ELRG> = {}) {
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
  ERES_RVAL: string | null = null;
  ERES_RUNI: string | null = null;
  ERES_RTXT: string | null = null;
  ERES_RTCD: string | null = null;
  ERES_RRES: boolean | null = null;
  ERES_DETF: boolean | null = null;
  ERES_ORG: boolean | null = null;
  ERES_IQLF: string | null = null;
  ERES_LQLF: string | null = null;
  ERES_RDLM: string | null = null;
  ERES_MDLM: string | null = null;
  ERES_QLM: string | null = null;
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

export class ESCG extends AgsGroup {
  static readonly code = "ESCG";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  ESCG_TYPE: string | null = null;
  ESCG_CELL: string | null = null;
  ESCG_COND: string | null = null;
  ESCG_SDIA: number | null = null;
  ESCG_HIGT: number | null = null;
  ESCG_MCI: string | null = null;
  ESCG_MCF: string | null = null;
  ESCG_BDEN: number | null = null;
  ESCG_BDEF: number | null = null;
  ESCG_DDEN: number | null = null;
  ESCG_PDEN: string | null = null;
  ESCG_IVR: number | null = null;
  ESCG_SATR: number | null = null;
  ESCG_LOAD: string | null = null;
  ESCG_DRAG: string | null = null;
  ESCG_PPM: string | null = null;
  ESCG_SPRS: number | null = null;
  ESCG_SATM: string | null = null;
  ESCG_SINC: number | null = null;
  ESCG_SDIF: number | null = null;
  ESCG_CELF: number | null = null;
  ESCG_BACF: number | null = null;
  ESCG_BVAL: number | null = null;
  ESCG_SVOL: number | null = null;
  ESCG_REM: string | null = null;
  ESCG_METH: string | null = null;
  ESCG_LAB: string | null = null;
  ESCG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  ESCG_DEV: string | null = null;
  ESCG_ISVR: number | null = null;
  ESCG_ISVS: number | null = null;
  ESCG_ISST: number | null = null;
  ESCG_PCP: number | null = null;
  ESCG_YSR: number | null = null;
  ESCG_CC: number | null = null;
  ESCG_CS: number | null = null;
  escts: ESCT[] = [];
  constructor(init: Partial<ESCG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ESCT extends AgsGroup {
  static readonly code = "ESCT";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  ESCT_INCN: string | null = null;
  ESCT_REM: string | null = null;
  ESCT_INCC: number | null = null;
  ESCT_INCB: number | null = null;
  ESCT_PWP0: number | null = null;
  ESCT_PWPF: number | null = null;
  ESCT_INCF: number | null = null;
  ESCT_VR0: number | null = null;
  ESCT_VRE: number | null = null;
  ESCT_DISS: number | null = null;
  ESCT_DSET: number | null = null;
  ESCT_DVOL: number | null = null;
  ESCT_INMV: number | null = null;
  ESCT_INCV: number | null = null;
  ESCT_INSC: number | null = null;
  ESCT_CVME: string | null = null;
  ESCT_TEMP: number | null = null;
  FILE_FSET: string | null = null;
  ESCT_INK: string | null = null;
  constructor(init: Partial<ESCT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class FGHG extends AgsGroup {
  static readonly code = "FGHG";
  LOCA_ID: string | null = null;
  FGHG_TOP: number | null = null;
  FGHG_BASE: number | null = null;
  FGHG_TESN: string | null = null;
  FGHG_TDIA: number | null = null;
  FGHG_SDIA: number | null = null;
  FGHG_ODIA: number | null = null;
  FGHG_HBAS: number | null = null;
  FGHG_CAS: number | null = null;
  FGHG_SFAC: number | null = null;
  FGHG_SFRF: string | null = null;
  FGHG_DATE: Date | null = null;
  FGHG_TYPE: string | null = null;
  FGHG_CNFG: string | null = null;
  FGHG_METH: string | null = null;
  FGHG_PRWL: number | null = null;
  FGHG_AWL: number | null = null;
  FGHG_HEAD: number | null = null;
  FGHG_FLOW: number | null = null;
  FGHG_IPRM: number | null = null;
  FGHG_ILUG: string | null = null;
  FGHG_FTYP: string | null = null;
  FGHG_REM: string | null = null;
  FGHG_ENV: string | null = null;
  FGHG_CONT: string | null = null;
  FGHG_OPER: string | null = null;
  FGHG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  fghis: FGHI[] = [];
  fghss: FGHS[] = [];
  constructor(init: Partial<FGHG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class FGHI extends AgsGroup {
  static readonly code = "FGHI";
  LOCA_ID: string | null = null;
  FGHG_TOP: number | null = null;
  FGHG_BASE: number | null = null;
  FGHG_TESN: string | null = null;
  FGHI_INST: string | null = null;
  FGHI_TYPE: string | null = null;
  FGHI_DETL: string | null = null;
  FGHI_LOCT: string | null = null;
  FGHI_REM: string | null = null;
  FILE_FSET: string | null = null;
  fghts: FGHT[] = [];
  constructor(init: Partial<FGHI> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class FGHS extends AgsGroup {
  static readonly code = "FGHS";
  LOCA_ID: string | null = null;
  FGHG_TOP: number | null = null;
  FGHG_BASE: number | null = null;
  FGHG_TESN: string | null = null;
  FGHS_STG: number | null = null;
  FGHS_STTM: Date | null = null;
  FGHS_ENTM: Date | null = null;
  FGHS_HEAD: number | null = null;
  FGHS_FLOW: number | null = null;
  FGHS_IPRM: number | null = null;
  FGHS_ILUG: string | null = null;
  FGHS_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<FGHS> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class FGHT extends AgsGroup {
  static readonly code = "FGHT";
  LOCA_ID: string | null = null;
  FGHG_TOP: number | null = null;
  FGHG_BASE: number | null = null;
  FGHG_TESN: string | null = null;
  FGHI_INST: string | null = null;
  FGHT_TIME: Date | null = null;
  FGHT_TYPE: string | null = null;
  FGHS_STG: number | null = null;
  FGHT_DURN: string | null = null;
  FGHT_RDNG: string | null = null;
  FGHT_UNIT: string | null = null;
  FGHT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<FGHT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class FILE extends AgsGroup {
  static readonly code = "FILE";
  FILE_FSET: string | null = null;
  FILE_NAME: string | null = null;
  FILE_DESC: string | null = null;
  FILE_TYPE: string | null = null;
  FILE_PROG: string | null = null;
  FILE_DOCT: string | null = null;
  FILE_DATE: Date | null = null;
  FILE_REM: string | null = null;
  constructor(init: Partial<FILE> = {}) {
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
  FRAC_IMAX: string | null = null;
  FRAC_IAVE: string | null = null;
  FRAC_IMIN: string | null = null;
  FRAC_FI: string | null = null;
  FRAC_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<FRAC> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class FRST extends AgsGroup {
  static readonly code = "FRST";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  FRST_COND: string | null = null;
  FRST_DDEN: number | null = null;
  FRST_MC: string | null = null;
  FRST_HVE1: number | null = null;
  FRST_HVE2: number | null = null;
  FRST_HVE3: number | null = null;
  FRST_HVE: number | null = null;
  FRST_STAB: number | null = null;
  FRST_STYP: string | null = null;
  FRST_REM: string | null = null;
  FRST_METH: string | null = null;
  FRST_LAB: string | null = null;
  FRST_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  FRST_DEV: string | null = null;
  constructor(init: Partial<FRST> = {}) {
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
  GCHM_RTXT: string | null = null;
  GCHM_DLM: string | null = null;
  SPEC_BASE: number | null = null;
  GCHM_DEV: string | null = null;
  GCHM_SGRP: string | null = null;
  GCHM_LSID: string | null = null;
  GCHM_RDAT: Date | null = null;
  GCHM_DTIM: Date | null = null;
  GCHM_TEST: string | null = null;
  GCHM_IREF: string | null = null;
  GCHM_ITYP: string | null = null;
  GCHM_SIZE: number | null = null;
  GCHM_PERP: number | null = null;
  GCHM_RDEV: string | null = null;
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
  GEOL_BGS: string | null = null;
  GEOL_FORM: string | null = null;
  GEOL_REM: string | null = null;
  FILE_FSET: string | null = null;
  GEOL_BNDF: string | null = null;
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
  SPEC_BASE: number | null = null;
  GRAG_DEV: string | null = null;
  GRAG_PDEN: string | null = null;
  GRAG_PRET: string | null = null;
  GRAG_SUFF: boolean | null = null;
  GRAG_EXCL: string | null = null;
  GRAG_CC: number | null = null;
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

export class ICBR extends AgsGroup {
  static readonly code = "ICBR";
  LOCA_ID: string | null = null;
  ICBR_DPTH: number | null = null;
  ICBR_TESN: string | null = null;
  ICBR_ICBR: number | null = null;
  ICBR_MC: string | null = null;
  ICBR_DATE: Date | null = null;
  ICBR_KENT: string | null = null;
  ICBR_SEAT: number | null = null;
  ICBR_SURC: number | null = null;
  ICBR_TYPE: string | null = null;
  ICBR_REM: string | null = null;
  ICBR_ENV: string | null = null;
  ICBR_METH: string | null = null;
  ICBR_CONT: string | null = null;
  ICBR_CRED: string | null = null;
  TEST_STAT: string | null = null;
  GEOL_STAT: string | null = null;
  FILE_FSET: string | null = null;
  ICBR_OPER: string | null = null;
  ICBR_BASE: number | null = null;
  constructor(init: Partial<ICBR> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class IDEN extends AgsGroup {
  static readonly code = "IDEN";
  LOCA_ID: string | null = null;
  IDEN_DPTH: number | null = null;
  IDEN_TESN: string | null = null;
  IDEN_DATE: Date | null = null;
  IDEN_TYPE: string | null = null;
  IDEN_IDEN: number | null = null;
  IDEN_MC: string | null = null;
  IDEN_STAB: number | null = null;
  IDEN_STYP: string | null = null;
  IDEN_REM: string | null = null;
  IDEN_ENV: string | null = null;
  IDEN_METH: string | null = null;
  IDEN_CONT: string | null = null;
  IDEN_CRED: string | null = null;
  TEST_STAT: string | null = null;
  GEOL_STAT: string | null = null;
  FILE_FSET: string | null = null;
  IDEN_OPER: string | null = null;
  constructor(init: Partial<IDEN> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class IFID extends AgsGroup {
  static readonly code = "IFID";
  LOCA_ID: string | null = null;
  IFID_DPTH: number | null = null;
  IFID_TESN: string | null = null;
  IFID_DATE: Date | null = null;
  IFID_RES: string | null = null;
  IFID_REM: string | null = null;
  IFID_ENV: string | null = null;
  IFID_METH: string | null = null;
  IFID_CONT: string | null = null;
  IFID_CRED: string | null = null;
  TEST_STAT: string | null = null;
  GEOL_STAT: string | null = null;
  FILE_FSET: string | null = null;
  IFID_OPER: string | null = null;
  constructor(init: Partial<IFID> = {}) {
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
  IPEN_OPER: string | null = null;
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
  IPID_RES: string | null = null;
  IPID_REM: string | null = null;
  IPID_ENV: string | null = null;
  IPID_METH: string | null = null;
  IPID_CONT: string | null = null;
  IPID_CRED: string | null = null;
  TEST_STAT: string | null = null;
  GEOL_STAT: string | null = null;
  FILE_FSET: string | null = null;
  IPID_OPER: string | null = null;
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
  IPRG_PRWL: number | null = null;
  IPRG_SWAL: number | null = null;
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
  iprts: IPRT[] = [];
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
  IPRT_DPTH: number | null = null;
  IPRT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<IPRT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class IRDX extends AgsGroup {
  static readonly code = "IRDX";
  LOCA_ID: string | null = null;
  IRDX_DPTH: number | null = null;
  IRDX_TESN: string | null = null;
  IRDX_DATE: Date | null = null;
  IRDX_PH: number | null = null;
  IRDX_MPOT: number | null = null;
  IRDX_IRDX: number | null = null;
  IRDX_REM: string | null = null;
  IRDX_ENV: string | null = null;
  IRDX_METH: string | null = null;
  IRDX_CONT: string | null = null;
  IRDX_CRED: string | null = null;
  TEST_STAT: string | null = null;
  GEOL_STAT: string | null = null;
  FILE_FSET: string | null = null;
  IRDX_OPER: string | null = null;
  constructor(init: Partial<IRDX> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class IRES extends AgsGroup {
  static readonly code = "IRES";
  LOCA_ID: string | null = null;
  IRES_DPTH: number | null = null;
  IRES_TESN: string | null = null;
  IRES_BASE: number | null = null;
  IRES_TYPE: string | null = null;
  IRES_DATE: Date | null = null;
  IRES_IRES: number | null = null;
  IRES_RES1: number | null = null;
  IRES_RES2: number | null = null;
  IRES_REM: string | null = null;
  IRES_ENV: string | null = null;
  IRES_METH: string | null = null;
  IRES_CONT: string | null = null;
  IRES_CRED: string | null = null;
  TEST_STAT: string | null = null;
  GEOL_STAT: string | null = null;
  FILE_FSET: string | null = null;
  IRES_OPER: string | null = null;
  constructor(init: Partial<IRES> = {}) {
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
  ISAG_OPER: string | null = null;
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
  ISAT_DPTH: number | null = null;
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
  ISPT_N60: number | null = null;
  constructor(init: Partial<ISPT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ISTA extends AgsGroup {
  static readonly code = "ISTA";
  LOCA_ID: string | null = null;
  ISTG_TESN: string | null = null;
  ISTA_TOP: number | null = null;
  ISTA_BASE: number | null = null;
  ISTA_ANYN: string | null = null;
  ISTA_DPTH: number | null = null;
  ISTA_RECT: number | null = null;
  ISTA_RECB: number | null = null;
  ISTA_RCOM: string | null = null;
  ISTA_MIVL: string | null = null;
  ISTA_WVTY: string | null = null;
  ISTA_UPSR: number | null = null;
  ISTA_FTU: string | null = null;
  ISTA_FMIN: number | null = null;
  ISTA_FMAX: number | null = null;
  ISTA_WATT: number | null = null;
  ISTA_WATB: number | null = null;
  ISTA_WATM: string | null = null;
  ISTA_ITM: string | null = null;
  ISTA_WVL: number | null = null;
  ISTA_WVLM: string | null = null;
  ISTA_STAC: boolean | null = null;
  ISTA_IVAL: boolean | null = null;
  ISTA_REM: string | null = null;
  ISTA_ANBY: string | null = null;
  ISTA_CONT: string | null = null;
  ISTA_DATE: Date | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<ISTA> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ISTG extends AgsGroup {
  static readonly code = "ISTG";
  LOCA_ID: string | null = null;
  ISTG_TESN: string | null = null;
  ISTG_TYPE: string | null = null;
  ISTG_LINK: number | null = null;
  ISTG_STAR: Date | null = null;
  ISTG_END: Date | null = null;
  ISTG_REF: string | null = null;
  ISTG_RECC: string | null = null;
  ISTG_RECD: string | null = null;
  ISTG_SOUR: string | null = null;
  ISTG_RORD: string | null = null;
  ISTG_SHOF: number | null = null;
  ISTG_ORNT: number | null = null;
  ISTG_SVOF: number | null = null;
  ISTG_OTOP: number | null = null;
  ISTG_OBOT: number | null = null;
  ISTG_BHCP: string | null = null;
  ISTG_MTO: string | null = null;
  ISTG_OPER: string | null = null;
  ISTG_ANBY: string | null = null;
  ISTG_REM: string | null = null;
  ISTG_ENV: string | null = null;
  ISTG_METH: string | null = null;
  ISTG_CONT: string | null = null;
  ISTG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  istas: ISTA[] = [];
  istss: ISTS[] = [];
  constructor(init: Partial<ISTG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ISTR extends AgsGroup {
  static readonly code = "ISTR";
  LOCA_ID: string | null = null;
  ISTG_TESN: string | null = null;
  ISTS_SGLN: string | null = null;
  ISTR_DPTH: number | null = null;
  ISTR_REF: string | null = null;
  ISTR_SSD: number | null = null;
  ISTR_QUAL: string | null = null;
  ISTR_QUAM: string | null = null;
  ISTR_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<ISTR> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ISTS extends AgsGroup {
  static readonly code = "ISTS";
  LOCA_ID: string | null = null;
  ISTG_TESN: string | null = null;
  ISTS_SGLN: string | null = null;
  ISTS_TYPE: string | null = null;
  ISTS_DTIM: Date | null = null;
  ISTS_RATE: number | null = null;
  ISTS_PTRT: number | null = null;
  ISTS_TTLY: number | null = null;
  ISTS_REM: string | null = null;
  FILE_FSET: string | null = null;
  istrs: ISTR[] = [];
  constructor(init: Partial<ISTS> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class ITCH extends AgsGroup {
  static readonly code = "ITCH";
  LOCA_ID: string | null = null;
  ITCH_DPTH: number | null = null;
  ITCH_TESN: string | null = null;
  ITCH_DATE: Date | null = null;
  ITCH_TCON: number | null = null;
  ITCH_TRES: number | null = null;
  ITCH_TEMP: number | null = null;
  ITCH_REM: string | null = null;
  ITCH_ENV: string | null = null;
  ITCH_METH: string | null = null;
  ITCH_OPER: string | null = null;
  ITCH_CONT: string | null = null;
  ITCH_CRED: string | null = null;
  TEST_STAT: string | null = null;
  GEOL_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<ITCH> = {}) {
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
  IVAN_OPER: string | null = null;
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
  LDEN_MC: string | null = null;
  LDEN_BDEN: number | null = null;
  LDEN_DDEN: number | null = null;
  LDEN_REM: string | null = null;
  LDEN_METH: string | null = null;
  LDEN_LAB: string | null = null;
  LDEN_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  LDEN_DEV: string | null = null;
  constructor(init: Partial<LDEN> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LDYN extends AgsGroup {
  static readonly code = "LDYN";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  LDYN_PWAV: number | null = null;
  LDYN_SWAV: number | null = null;
  LDYN_EMOD: number | null = null;
  LDYN_SG: number | null = null;
  LDYN_REM: string | null = null;
  LDYN_METH: string | null = null;
  LDYN_LAB: string | null = null;
  LDYN_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  LDYN_DEV: string | null = null;
  constructor(init: Partial<LDYN> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LFCN extends AgsGroup {
  static readonly code = "LFCN";
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
  LFCN_DEV: string | null = null;
  LFCN_CMAS: number | null = null;
  LFCN_CANG: number | null = null;
  LFCN_PENA: number | null = null;
  LFCN_PEN1: number | null = null;
  LFCN_PEN2: number | null = null;
  LFCN_PEN3: number | null = null;
  LFCN_PEN4: number | null = null;
  LFCN_CONF: boolean | null = null;
  LFCN_FCPK: number | null = null;
  LFCN_FCRM: number | null = null;
  LFCN_WC: string | null = null;
  LFCN_WCST: string | null = null;
  LFCN_REM: string | null = null;
  LFCN_METH: string | null = null;
  LFCN_LAB: string | null = null;
  LFCN_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<LFCN> = {}) {
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
  SPEC_BASE: number | null = null;
  LLIN_DEV: string | null = null;
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
  SPEC_PREP: string | null = null;
  LLPL_LL: number | null = null;
  LLPL_PL: string | null = null;
  LLPL_PI: number | null = null;
  LLPL_425: number | null = null;
  LLPL_PREP: string | null = null;
  LLPL_STAB: number | null = null;
  LLPL_STYP: string | null = null;
  LLPL_REM: string | null = null;
  LLPL_METH: string | null = null;
  LLPL_LAB: string | null = null;
  LLPL_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  LLPL_DEV: string | null = null;
  LLPL_TYPE: string | null = null;
  LLPL_POIN: string | null = null;
  LLPL_CONE: string | null = null;
  LLPL_1PRE: number | null = null;
  LLPL_1PCF: number | null = null;
  LLPL_SIZE: string | null = null;
  LLPL_PASS: number | null = null;
  LLPL_WC: string | null = null;
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
  LNMC_MC: string | null = null;
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
  SPEC_BASE: number | null = null;
  LNMC_DEV: string | null = null;
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
  LOCA_REM: string | null = null;
  LOCA_FDEP: number | null = null;
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
  LOCA_LAT: string | null = null;
  LOCA_LON: string | null = null;
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
  LOCA_NATD: string | null = null;
  LOCA_ORID: string | null = null;
  LOCA_ORJO: string | null = null;
  LOCA_ORCO: string | null = null;
  LOCA_GLDT: Date | null = null;
  LOCA_VSSL: string | null = null;
  LOCA_NSRI: number | null = null;
  LOCA_LSRI: number | null = null;
  LOCA_LLSI: number | null = null;
  bkfls: BKFL[] = [];
  cdias: CDIA[] = [];
  chiss: CHIS[] = [];
  cores: CORE[] = [];
  cptgs: CPTG[] = [];
  cptms: CPTM[] = [];
  cptps: CPTP[] = [];
  dcpgs: DCPG[] = [];
  detls: DETL[] = [];
  discs: DISC[] = [];
  dlogs: DLOG[] = [];
  dmtgs: DMTG[] = [];
  dobss: DOBS[] = [];
  dprgs: DPRG[] = [];
  drems: DREM[] = [];
  fghgs: FGHG[] = [];
  flshs: FLSH[] = [];
  fracs: FRAC[] = [];
  geols: GEOL[] = [];
  hdias: HDIA[] = [];
  hdphs: HDPH[] = [];
  horns: HORN[] = [];
  icbrs: ICBR[] = [];
  idens: IDEN[] = [];
  ifids: IFID[] = [];
  ipens: IPEN[] = [];
  ipids: IPID[] = [];
  iprgs: IPRG[] = [];
  irdxs: IRDX[] = [];
  iress: IRES[] = [];
  isags: ISAG[] = [];
  ispts: ISPT[] = [];
  istgs: ISTG[] = [];
  itchs: ITCH[] = [];
  ivans: IVAN[] = [];
  mongs: MONG[] = [];
  pipes: PIPE[] = [];
  pltgs: PLTG[] = [];
  pmmgs: PMMG[] = [];
  pmtgs: PMTG[] = [];
  ptims: PTIM[] = [];
  pumgs: PUMG[] = [];
  samps: SAMP[] = [];
  scpgs: SCPG[] = [];
  trems: TREM[] = [];
  wadds: WADD[] = [];
  weths: WETH[] = [];
  wgpgs: WGPG[] = [];
  winss: WINS[] = [];
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
  SPEC_BASE: number | null = null;
  LPDN_DEV: string | null = null;
  LPDN_PVOL: number | null = null;
  LPDN_GAS: string | null = null;
  constructor(init: Partial<LPDN> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LPEN extends AgsGroup {
  static readonly code = "LPEN";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  LPEN_PPEN: number | null = null;
  LPEN_MC: string | null = null;
  LPEN_REM: string | null = null;
  LPEN_METH: string | null = null;
  LPEN_LAB: string | null = null;
  LPEN_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  LPEN_DEV: string | null = null;
  constructor(init: Partial<LPEN> = {}) {
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
  LRES_MC: string | null = null;
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
  SPEC_BASE: number | null = null;
  LRES_DEV: string | null = null;
  constructor(init: Partial<LRES> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LSLT extends AgsGroup {
  static readonly code = "LSLT";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  LSLT_SLIM: number | null = null;
  LSLT_SHRA: number | null = null;
  LSLT_IDEN: number | null = null;
  LSLT_MCI: string | null = null;
  LSLT_425: number | null = null;
  LSLT_REM: string | null = null;
  LSLT_METH: string | null = null;
  LSLT_LAB: string | null = null;
  LSLT_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  LSLT_DEV: string | null = null;
  constructor(init: Partial<LSLT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LSTG extends AgsGroup {
  static readonly code = "LSTG";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  LSTG_ICL: number | null = null;
  LSTG_PH: number | null = null;
  LSTG_LIME: string | null = null;
  LSTG_SUIT: number | null = null;
  LSTG_425: number | null = null;
  LSTG_REM: string | null = null;
  LSTG_METH: string | null = null;
  LSTG_LAB: string | null = null;
  LSTG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  LSTG_DEV: string | null = null;
  lstts: LSTT[] = [];
  constructor(init: Partial<LSTG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LSTT extends AgsGroup {
  static readonly code = "LSTT";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  LSTT_TESN: string | null = null;
  LSTT_LCON: number | null = null;
  LSTT_PH: number | null = null;
  LSTT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<LSTT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LSWL extends AgsGroup {
  static readonly code = "LSWL";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  LSWL_SWPR: number | null = null;
  LSWL_SWSI: number | null = null;
  LSWL_MCI: number | null = null;
  LSWL_SDIA: number | null = null;
  LSWL_THCK: number | null = null;
  LSWL_BDEN: number | null = null;
  LSWL_DDEN: number | null = null;
  LSWL_REM: string | null = null;
  LSWL_METH: string | null = null;
  LSWL_LAB: string | null = null;
  LSWL_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  LSWL_DEV: string | null = null;
  constructor(init: Partial<LSWL> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LTCH extends AgsGroup {
  static readonly code = "LTCH";
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
  LTCH_COND: string | null = null;
  LTCH_BDEN: number | null = null;
  LTCH_DDEN: number | null = null;
  LTCH_MC: string | null = null;
  LTCH_TCON: number | null = null;
  LTCH_TRES: number | null = null;
  LTCH_TEMP: number | null = null;
  LTCH_PDIA: number | null = null;
  LTCH_PSPA: number | null = null;
  LTCH_PPEN: number | null = null;
  LTCH_PRBE: string | null = null;
  LTCH_PART: string | null = null;
  LTCH_DEV: string | null = null;
  LTCH_REM: string | null = null;
  LTCH_METH: string | null = null;
  LTCH_LAB: string | null = null;
  LTCH_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<LTCH> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class LUCT extends AgsGroup {
  static readonly code = "LUCT";
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
  LUCT_DEV: string | null = null;
  LUCT_TYPE: string | null = null;
  LUCT_DIA: number | null = null;
  LUCT_SLEN: number | null = null;
  LUCT_IWC: string | null = null;
  LUCT_BDEN: number | null = null;
  LUCT_DDEN: number | null = null;
  LUCT_RATE: number | null = null;
  LUCT_UCS: number | null = null;
  LUCT_STRA: number | null = null;
  LUCT_MODE: string | null = null;
  LUCT_REM: string | null = null;
  LUCT_METH: string | null = null;
  LUCT_LAB: string | null = null;
  LUCT_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<LUCT> = {}) {
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
  LVAN_VNPK: string | null = null;
  LVAN_VNRM: string | null = null;
  LVAN_MC: string | null = null;
  LVAN_SIZE: number | null = null;
  LVAN_VLEN: number | null = null;
  LVAN_REM: string | null = null;
  LVAN_METH: string | null = null;
  LVAN_LAB: string | null = null;
  LVAN_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  LVAN_DEV: string | null = null;
  LVAN_TYPE: string | null = null;
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
  MCVG_NMC: string | null = null;
  MCVG_STAB: number | null = null;
  MCVG_STYP: string | null = null;
  MCVG_REM: string | null = null;
  MCVG_METH: string | null = null;
  MCVG_LAB: string | null = null;
  MCVG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  MCVG_DEV: string | null = null;
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
  MCVT_MC: string | null = null;
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
  MOND_LIM: string | null = null;
  MOND_ULIM: string | null = null;
  MOND_NAME: string | null = null;
  MOND_CRED: string | null = null;
  MOND_CONT: string | null = null;
  MOND_REM: string | null = null;
  FILE_FSET: string | null = null;
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
  monss: MONS[] = [];
  constructor(init: Partial<MONG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class MONS extends AgsGroup {
  static readonly code = "MONS";
  LOCA_ID: string | null = null;
  MONG_ID: string | null = null;
  MONG_DIS: number | null = null;
  MONS_STAR: Date | null = null;
  MONS_ENDD: Date | null = null;
  MONS_BY: string | null = null;
  MONS_TYPE: string | null = null;
  MONS_STAT: string | null = null;
  MONS_RPLO: string | null = null;
  MONS_RPID: string | null = null;
  MONS_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<MONS> = {}) {
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
  PLTG_OPER: string | null = null;
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

export class PMMC extends AgsGroup {
  static readonly code = "PMMC";
  LOCA_ID: string | null = null;
  PMMG_DPTH: number | null = null;
  PMMG_TESN: string | null = null;
  PMMC_CYNO: string | null = null;
  PMMC_P1CY: number | null = null;
  PMMC_P2CY: number | null = null;
  PMMC_EMCY: number | null = null;
  PMMC_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<PMMC> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PMMD extends AgsGroup {
  static readonly code = "PMMD";
  LOCA_ID: string | null = null;
  PMMG_DPTH: number | null = null;
  PMMG_TESN: string | null = null;
  PMMD_SEQ: number | null = null;
  PMMD_P01S: number | null = null;
  PMMD_P15S: number | null = null;
  PMMD_P30S: number | null = null;
  PMMD_P60S: number | null = null;
  PMMD_V01S: number | null = null;
  PMMD_V15S: number | null = null;
  PMMD_V30S: number | null = null;
  PMMD_V60S: number | null = null;
  PMMD_CP: number | null = null;
  PMMD_CVOL: number | null = null;
  PMMD_SLOP: number | null = null;
  PMMD_CREP: number | null = null;
  PMMD_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<PMMD> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PMMG extends AgsGroup {
  static readonly code = "PMMG";
  LOCA_ID: string | null = null;
  PMMG_DPTH: number | null = null;
  PMMG_TESN: string | null = null;
  PMMG_DATE: Date | null = null;
  PMMG_DCU: number | null = null;
  PMMG_PRWL: number | null = null;
  PMMG_REF: string | null = null;
  PMMG_TYPE: string | null = null;
  PMMG_DIAM: number | null = null;
  PMMG_PRC: number | null = null;
  PMMG_TC: string | null = null;
  PMMG_P1: number | null = null;
  PMMG_P2: number | null = null;
  PMMG_EM: number | null = null;
  PMMG_MPL: number | null = null;
  PMMG_MPLM: string | null = null;
  PMMG_PF: number | null = null;
  PMMG_METH: string | null = null;
  PMMG_CREM: string | null = null;
  PMMG_REM: string | null = null;
  PMMG_CRDT: Date | null = null;
  PMMG_OPER: string | null = null;
  PMMG_ANBY: string | null = null;
  PMMG_CONT: string | null = null;
  PMMG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  PMMG_ENV: string | null = null;
  FILE_FSET: string | null = null;
  pmmcs: PMMC[] = [];
  pmmds: PMMD[] = [];
  constructor(init: Partial<PMMG> = {}) {
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
  PMTD_TPC: number | null = null;
  PMTD_PPA: number | null = null;
  PMTD_PPB: number | null = null;
  PMTD_VOL: number | null = null;
  PMTD_REM: string | null = null;
  FILE_FSET: string | null = null;
  PMTD_AX1: number | null = null;
  PMTD_AX2: number | null = null;
  PMTD_AX3: number | null = null;
  PMTD_SA1: number | null = null;
  PMTD_SA2: number | null = null;
  PMTD_SA3: number | null = null;
  PMTD_SA4: number | null = null;
  PMTD_SA5: number | null = null;
  PMTD_SA6: number | null = null;
  PMTD_SAME: number | null = null;
  PMTD_TIME: number | null = null;
  PMTD_ARM1: number | null = null;
  PMTD_ARM2: number | null = null;
  PMTD_ARM3: number | null = null;
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
  PMTG_NUAR: number | null = null;
  PMTG_ORNT: number | null = null;
  PMTG_AXIS: string | null = null;
  PMTG_PRWL: number | null = null;
  PMTG_TC: string | null = null;
  PMTG_STAD: Date | null = null;
  PMTG_ENDD: Date | null = null;
  PMTG_TOPP: number | null = null;
  PMTG_BOTP: number | null = null;
  PMTG_SBHT: string | null = null;
  PMTG_SBCS: number | null = null;
  PMTG_SBCT: string | null = null;
  PMTG_SBCD: number | null = null;
  PMTG_SBCP: number | null = null;
  PMTG_FLFT: string | null = null;
  PMTG_FLFP: number | null = null;
  PMTG_TRST: number | null = null;
  PMTG_PPRD: boolean | null = null;
  PMTG_CMT: string | null = null;
  PMTG_CREM: string | null = null;
  PMTG_CRDT: Date | null = null;
  PMTG_ANBY: string | null = null;
  pmtds: PMTD[] = [];
  pmtls: PMTL[] = [];
  pmtps: PMTP[] = [];
  pmtzs: PMTZ[] = [];
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
  PMTL_AXIS: string | null = null;
  PMTL_HP: number | null = null;
  PMTL_HT: number | null = null;
  PMTL_CR: number | null = null;
  PMTD_SEQ: number | null = null;
  constructor(init: Partial<PMTL> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PMTP extends AgsGroup {
  static readonly code = "PMTP";
  LOCA_ID: string | null = null;
  PMTG_DPTH: number | null = null;
  PMTG_TESN: string | null = null;
  PMTP_U0: number | null = null;
  PMTP_STO: number | null = null;
  PMTP_HO: number | null = null;
  PMTP_HOM: string | null = null;
  PMTP_GI: number | null = null;
  PMTP_SU: number | null = null;
  PMTP_SUM: string | null = null;
  PMTP_AF: number | null = null;
  PMTP_AD: number | null = null;
  PMTP_AFDM: string | null = null;
  PMTP_AFCV: number | null = null;
  PMTP_DC: number | null = null;
  PMTP_DCM: string | null = null;
  PMTP_PL: number | null = null;
  PMTP_PF: number | null = null;
  PMTP_PFM: string | null = null;
  PMTP_YM: number | null = null;
  PMTP_YMM: string | null = null;
  PMTP_MU: number | null = null;
  PMTP_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<PMTP> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PMTZ extends AgsGroup {
  static readonly code = "PMTZ";
  LOCA_ID: string | null = null;
  PMTG_DPTH: number | null = null;
  PMTG_TESN: string | null = null;
  PMTZ_PARM: string | null = null;
  PMTZ_MRS: string | null = null;
  PMTZ_ZC: string | null = null;
  PMTZ_ZB: string | null = null;
  PMTZ_ZH: string | null = null;
  PMTZ_ZA: string | null = null;
  PMTZ_ZD: string | null = null;
  PMTZ_EGUT: string | null = null;
  PMTZ_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<PMTZ> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PREM extends AgsGroup {
  static readonly code = "PREM";
  PREM_DTIM: Date | null = null;
  PREM_COMP: string | null = null;
  PREM_REM: string | null = null;
  PREM_DURN: string | null = null;
  PREM_ETIM: Date | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<PREM> = {}) {
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
  PTST_MC: string | null = null;
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
  SPEC_BASE: number | null = null;
  PTST_DEV: string | null = null;
  PTST_WCIS: string | null = null;
  PTST_WCF: string | null = null;
  PTST_FSAT: number | null = null;
  PTST_TEMP: number | null = null;
  PTST_SOUR: string | null = null;
  PTST_BACK: number | null = null;
  PTST_BVAL: number | null = null;
  PTST_LOSS: string | null = null;
  constructor(init: Partial<PTST> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PUMG extends AgsGroup {
  static readonly code = "PUMG";
  LOCA_ID: string | null = null;
  PUMG_TEST: string | null = null;
  PUMG_CONT: string | null = null;
  PUMG_METH: string | null = null;
  PUMG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  PUMG_ENV: string | null = null;
  PUMG_REM: string | null = null;
  FILE_FSET: string | null = null;
  PUMG_OPER: string | null = null;
  pumts: PUMT[] = [];
  constructor(init: Partial<PUMG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class PUMT extends AgsGroup {
  static readonly code = "PUMT";
  LOCA_ID: string | null = null;
  PUMG_TEST: string | null = null;
  PUMT_DTIM: Date | null = null;
  PUMT_DPTH: number | null = null;
  PUMT_QUAT: number | null = null;
  PUMT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<PUMT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RCAG extends AgsGroup {
  static readonly code = "RCAG";
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
  RCAG_DEV: string | null = null;
  RCAG_DATE: Date | null = null;
  RCAG_COND: string | null = null;
  RCAG_GSIZ: number | null = null;
  RCAG_ANIS: string | null = null;
  RCAG_MACH: string | null = null;
  RCAG_MMTD: string | null = null;
  RCAG_CAIM: number | null = null;
  RCAG_CAIS: number | null = null;
  RCAG_ABCL: string | null = null;
  RCAG_REM: string | null = null;
  RCAG_METH: string | null = null;
  RCAG_LAB: string | null = null;
  RCAG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  rcats: RCAT[] = [];
  constructor(init: Partial<RCAG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RCAT extends AgsGroup {
  static readonly code = "RCAT";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  RCAT_TESN: string | null = null;
  RCAT_CUT: string | null = null;
  RCAT_SDIR: string | null = null;
  RCAT_STYH: number | null = null;
  RCAT_STYC: string | null = null;
  RCAT_CAI: number | null = null;
  RCAT_CAIS: number | null = null;
  RCAT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<RCAT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RCCV extends AgsGroup {
  static readonly code = "RCCV";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  RCCV_TESN: string | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  RCCV_MC: string | null = null;
  RCCV_CCV: number | null = null;
  RCCV_100: number | null = null;
  RCCV_REM: string | null = null;
  RCCV_METH: string | null = null;
  RCCV_LAB: string | null = null;
  RCCV_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  RCCV_DEV: string | null = null;
  constructor(init: Partial<RCCV> = {}) {
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
  RDEN_MC: string | null = null;
  RDEN_SMC: string | null = null;
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
  RDEN_IDEN: number | null = null;
  SPEC_BASE: number | null = null;
  RDEN_DEV: string | null = null;
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
  SPEC_BASE: number | null = null;
  RELD_DEV: string | null = null;
  constructor(init: Partial<RELD> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RESC extends AgsGroup {
  static readonly code = "RESC";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  RESC_TESN: string | null = null;
  RESC_SDIA: number | null = null;
  RESC_HIGH: number | null = null;
  RESC_CTYP: string | null = null;
  RESC_ELAP: string | null = null;
  RESC_CHGT: number | null = null;
  RESC_CDIA: number | null = null;
  RESC_CMC: string | null = null;
  RESC_CDDN: number | null = null;
  RESC_CRD: number | null = null;
  RESC_INCE: number | null = null;
  RESC_EASC: number | null = null;
  RESC_ERSC: number | null = null;
  RESC_DEVS: number | null = null;
  RESC_SHRS: number | null = null;
  RESC_MNES: number | null = null;
  RESC_AXSN: number | null = null;
  RESC_VLSN: number | null = null;
  RESC_RDSN: number | null = null;
  RESC_BESE: string | null = null;
  RESC_BEAX: string | null = null;
  RESC_DBTE: number | null = null;
  RESC_MAT: number | null = null;
  RESC_MATM: string | null = null;
  RESC_SWV: number | null = null;
  RESC_SMGM: number | null = null;
  RESC_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<RESC> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RESD extends AgsGroup {
  static readonly code = "RESD";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  RESD_TESN: string | null = null;
  RESD_MNUM: string | null = null;
  RESD_CNDS: string | null = null;
  RESD_SDIA: number | null = null;
  RESD_HIGH: number | null = null;
  RESD_CELL: number | null = null;
  RESD_BP: number | null = null;
  RESD_AXL: number | null = null;
  RESD_BPWP: number | null = null;
  RESD_MPWP: number | null = null;
  RESD_PPR: number | null = null;
  RESD_PWPM: number | null = null;
  RESD_EAS: number | null = null;
  RESD_VOL: number | null = null;
  RESD_DEV: number | null = null;
  RESD_MEES: number | null = null;
  RESD_MIPS: number | null = null;
  RESD_MAPS: number | null = null;
  RESD_AVSS: number | null = null;
  RESD_SM: number | null = null;
  RESD_DMP: number | null = null;
  RESD_REM: string | null = null;
  FILE_FSET: string | null = null;
  resps: RESP[] = [];
  constructor(init: Partial<RESD> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RESG extends AgsGroup {
  static readonly code = "RESG";
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
  RESG_COND: string | null = null;
  RESG_CONS: string | null = null;
  RESG_DRAG: string | null = null;
  RESG_ORNT: string | null = null;
  RESG_SDIA: number | null = null;
  RESG_HIGT: number | null = null;
  RESG_MCI: string | null = null;
  RESG_MCF: string | null = null;
  RESG_BDEN: number | null = null;
  RESG_DDEN: number | null = null;
  RESG_MIDD: number | null = null;
  RESG_MADD: number | null = null;
  RESG_IRDI: number | null = null;
  RESG_IVR: number | null = null;
  RESG_ISAT: number | null = null;
  RESG_PDEN: string | null = null;
  RESG_DAMP: string | null = null;
  RESG_DEV: string | null = null;
  RESG_REM: string | null = null;
  RESG_METH: string | null = null;
  RESG_LAB: string | null = null;
  RESG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  rescs: RESC[] = [];
  resds: RESD[] = [];
  resss: RESS[] = [];
  constructor(init: Partial<RESG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RESP extends AgsGroup {
  static readonly code = "RESP";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  RESD_TESN: string | null = null;
  RESD_MNUM: string | null = null;
  RESP_CTYP: string | null = null;
  RESP_CSTG: number | null = null;
  RESP_CELL: number | null = null;
  RESP_BACK: number | null = null;
  RESP_ERSC: number | null = null;
  RESP_EASC: number | null = null;
  RESP_DEV: number | null = null;
  RESP_VOLS: number | null = null;
  RESP_STRN: number | null = null;
  RESP_SMOD: number | null = null;
  RESP_SSTR: number | null = null;
  RESP_DAMP: number | null = null;
  RESP_SMRA: number | null = null;
  RESP_SR: number | null = null;
  RESP_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<RESP> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RESS extends AgsGroup {
  static readonly code = "RESS";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  RESS_TESN: string | null = null;
  RESS_INC: number | null = null;
  RESS_DIFF: number | null = null;
  RESS_CELL: number | null = null;
  RESS_BPWP: number | null = null;
  RESS_STRN: number | null = null;
  RESS_MCF: string | null = null;
  RESS_BDEN: number | null = null;
  RESS_DDEN: number | null = null;
  RESS_FVR: number | null = null;
  RESS_FSAT: number | null = null;
  RESS_B: number | null = null;
  RESS_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<RESS> = {}) {
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
  SPEC_BASE: number | null = null;
  RPLT_DEV: string | null = null;
  constructor(init: Partial<RPLT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RSCH extends AgsGroup {
  static readonly code = "RSCH";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  RSCH_SCHV: number | null = null;
  RSCH_AXIS: string | null = null;
  RSCH_CLAM: string | null = null;
  RSCH_REM: string | null = null;
  RSCH_METH: string | null = null;
  RSCH_LAB: string | null = null;
  RSCH_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  RSCH_DEV: string | null = null;
  RSCH_STYP: string | null = null;
  RSCH_EXCV: string | null = null;
  RSCH_DIAM: number | null = null;
  RSCH_LEN: number | null = null;
  RSCH_WC: number | null = null;
  RSCH_WCTX: string | null = null;
  RSCH_HTYP: string | null = null;
  RSCH_ORN: string | null = null;
  RSCH_MEAN: number | null = null;
  RSCH_MED: number | null = null;
  RSCH_MODE: number | null = null;
  RSCH_RANG: number | null = null;
  RSCH_NUM: string | null = null;
  constructor(init: Partial<RSCH> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RSHR extends AgsGroup {
  static readonly code = "RSHR";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  RSHR_SHOR: number | null = null;
  RSHR_AXIS: string | null = null;
  RSHR_NUM: number | null = null;
  RSHR_REM: string | null = null;
  RSHR_METH: string | null = null;
  RSHR_LAB: string | null = null;
  RSHR_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  RSHR_DEV: string | null = null;
  constructor(init: Partial<RSHR> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class RTEN extends AgsGroup {
  static readonly code = "RTEN";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  RTEN_SDIA: number | null = null;
  RTEN_LEN: number | null = null;
  RTEN_MC: number | null = null;
  RTEN_COND: string | null = null;
  RTEN_DURN: string | null = null;
  RTEN_STRA: number | null = null;
  RTEN_TENS: number | null = null;
  RTEN_MODE: string | null = null;
  RTEN_MACH: string | null = null;
  RTEN_REM: string | null = null;
  RTEN_METH: string | null = null;
  RTEN_LAB: string | null = null;
  RTEN_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  RTEN_DEV: string | null = null;
  constructor(init: Partial<RTEN> = {}) {
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
  RUCS_MACH: string | null = null;
  RUCS_REM: string | null = null;
  RUCS_METH: string | null = null;
  RUCS_LAB: string | null = null;
  RUCS_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  RUCS_DEV: string | null = null;
  RUCS_ESEC: number | null = null;
  RUCS_ETAN: number | null = null;
  RUCS_EAVG: number | null = null;
  RUCS_SSEC: string | null = null;
  RUCS_STAN: string | null = null;
  RUCS_SAVG: string | null = null;
  RUCS_MUS: number | null = null;
  RUCS_MUT: number | null = null;
  RUCS_MUAV: number | null = null;
  RUCS_E: number | null = null;
  RUCS_MU: number | null = null;
  RUCS_ESTR: string | null = null;
  RUCS_ETYP: string | null = null;
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
  RWCO_MC: string | null = null;
  RWCO_TEMP: number | null = null;
  RWCO_REM: string | null = null;
  RWCO_METH: string | null = null;
  RWCO_LAB: string | null = null;
  RWCO_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  RWCO_DEV: string | null = null;
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
  SAMP_UBLO: number | null = null;
  SAMP_CONT: string | null = null;
  SAMP_PREP: string | null = null;
  SAMP_SDIA: number | null = null;
  SAMP_WDEP: number | null = null;
  SAMP_RECV: number | null = null;
  SAMP_TECH: string | null = null;
  SAMP_MATX: string | null = null;
  SAMP_TYPC: string | null = null;
  SAMP_WHO: string | null = null;
  SAMP_WHY: string | null = null;
  SAMP_REM: string | null = null;
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
  SAMP_LINK: number | null = null;
  GEOL_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SAMP_RECL: number | null = null;
  aavts: AAVT[] = [];
  acvts: ACVT[] = [];
  aelos: AELO[] = [];
  aflks: AFLK[] = [];
  aivts: AIVT[] = [];
  aloss: ALOS[] = [];
  apsvs: APSV[] = [];
  artws: ARTW[] = [];
  asdis: ASDI[] = [];
  asnss: ASNS[] = [];
  awads: AWAD[] = [];
  cbrgs: CBRG[] = [];
  chocs: CHOC[] = [];
  cmpgs: CMPG[] = [];
  congs: CONG[] = [];
  ctrgs: CTRG[] = [];
  ectns: ECTN[] = [];
  elrgs: ELRG[] = [];
  eress: ERES[] = [];
  escgs: ESCG[] = [];
  frsts: FRST[] = [];
  gchms: GCHM[] = [];
  grags: GRAG[] = [];
  ldens: LDEN[] = [];
  ldyns: LDYN[] = [];
  lfcns: LFCN[] = [];
  llins: LLIN[] = [];
  llpls: LLPL[] = [];
  lnmcs: LNMC[] = [];
  lpdns: LPDN[] = [];
  lpens: LPEN[] = [];
  lress: LRES[] = [];
  lslts: LSLT[] = [];
  lstgs: LSTG[] = [];
  lswls: LSWL[] = [];
  ltchs: LTCH[] = [];
  lucts: LUCT[] = [];
  lvans: LVAN[] = [];
  mcvgs: MCVG[] = [];
  ptsts: PTST[] = [];
  rcags: RCAG[] = [];
  rccvs: RCCV[] = [];
  rdens: RDEN[] = [];
  relds: RELD[] = [];
  resgs: RESG[] = [];
  rplts: RPLT[] = [];
  rschs: RSCH[] = [];
  rshrs: RSHR[] = [];
  rtens: RTEN[] = [];
  rucss: RUCS[] = [];
  rwcos: RWCO[] = [];
  shbgs: SHBG[] = [];
  sucts: SUCT[] = [];
  tnpcs: TNPC[] = [];
  tregs: TREG[] = [];
  trigs: TRIG[] = [];
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
  SCDG_OPER: string | null = null;
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
  SCPG_OPER: string | null = null;
  scdgs: SCDG[] = [];
  scpps: SCPP[] = [];
  scpts: SCPT[] = [];
  constructor(init: Partial<SCPG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class SCPP extends AgsGroup {
  static readonly code = "SCPP";
  LOCA_ID: string | null = null;
  SCPG_TESN: string | null = null;
  SCPP_TOP: number | null = null;
  SCPP_BASE: number | null = null;
  SCPP_REF: string | null = null;
  SCPP_REM: string | null = null;
  SCPP_CSBT: string | null = null;
  SCPP_CSU: number | null = null;
  SCPP_CRD: number | null = null;
  SCPP_CPHI: number | null = null;
  SCPP_CIC: number | null = null;
  SCPP_CSPT: number | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<SCPP> = {}) {
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
  SPEC_BASE: number | null = null;
  SHBG_DEV: string | null = null;
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
  SHBT_MCI: string | null = null;
  SHBT_MCF: string | null = null;
  SHBT_DIA1: number | null = null;
  SHBT_DIA2: number | null = null;
  SHBT_HGT: number | null = null;
  SHBT_CRIT: string | null = null;
  SHBT_REM: string | null = null;
  FILE_FSET: string | null = null;
  SHBT_PVST: number | null = null;
  SHBT_RVST: number | null = null;
  constructor(init: Partial<SHBT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class STND extends AgsGroup {
  static readonly code = "STND";
  STND_REF: string | null = null;
  STND_TTLE: string | null = null;
  STND_SCPE: string | null = null;
  STND_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<STND> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class SUCT extends AgsGroup {
  static readonly code = "SUCT";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  SUCT_DIAM: number | null = null;
  SUCT_LEN: number | null = null;
  SUCT_COND: string | null = null;
  SUCT_BDEN: number | null = null;
  SUCT_DDEN: number | null = null;
  SUCT_MC: number | null = null;
  SUCT_VAL: number | null = null;
  SUCT_REM: string | null = null;
  SUCT_METH: string | null = null;
  SUCT_LAB: string | null = null;
  SUCT_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  SUCT_DEV: string | null = null;
  constructor(init: Partial<SUCT> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class TNPC extends AgsGroup {
  static readonly code = "TNPC";
  LOCA_ID: string | null = null;
  SAMP_TOP: number | null = null;
  SAMP_REF: string | null = null;
  SAMP_TYPE: string | null = null;
  SAMP_ID: string | null = null;
  SPEC_REF: string | null = null;
  SPEC_DPTH: number | null = null;
  SPEC_DESC: string | null = null;
  SPEC_PREP: string | null = null;
  TNPC_TESN: string | null = null;
  TNPC_DRY: string | null = null;
  TNPC_WET: string | null = null;
  TNPC_REM: string | null = null;
  TNPC_METH: string | null = null;
  TNPC_LAB: string | null = null;
  TNPC_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  TNPC_DEV: string | null = null;
  constructor(init: Partial<TNPC> = {}) {
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
  TREG_TYPE: string | null = null;
  TREG_COND: string | null = null;
  TREG_COH: number | null = null;
  TREG_PHI: number | null = null;
  TREG_FCR: string | null = null;
  TREG_REM: string | null = null;
  TREG_METH: string | null = null;
  TREG_LAB: string | null = null;
  TREG_CRED: string | null = null;
  TEST_STAT: string | null = null;
  FILE_FSET: string | null = null;
  SPEC_BASE: number | null = null;
  TREG_DEV: string | null = null;
  trets: TRET[] = [];
  constructor(init: Partial<TREG> = {}) {
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
  TRET_SDIA: number | null = null;
  TRET_LEN: number | null = null;
  TRET_IMC: string | null = null;
  TRET_FMC: string | null = null;
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
  TRET_REM: string | null = null;
  FILE_FSET: string | null = null;
  TRET_BACK: number | null = null;
  TRET_VERT: number | null = null;
  TRET_VOLM: number | null = null;
  TRET_RATE: number | null = null;
  TRET_BVAL: number | null = null;
  TRET_DRN: string | null = null;
  TRET_MEMB: number | null = null;
  TRET_FILC: number | null = null;
  TRET_IVR: number | null = null;
  TRET_SATR: number | null = null;
  TRET_CVP: number | null = null;
  TRET_CRP: number | null = null;
  TRET_MEAN: number | null = null;
  TRET_CU: number | null = null;
  TRET_EP50: number | null = null;
  TRET_E50: number | null = null;
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
  SPEC_BASE: number | null = null;
  TRIG_DEV: string | null = null;
  trits: TRIT[] = [];
  constructor(init: Partial<TRIG> = {}) {
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
  TRIT_IMC: string | null = null;
  TRIT_FMC: string | null = null;
  TRIT_CELL: number | null = null;
  TRIT_DEVF: number | null = null;
  TRIT_BDEN: number | null = null;
  TRIT_DDEN: number | null = null;
  TRIT_STRN: number | null = null;
  TRIT_CU: number | null = null;
  TRIT_MODE: string | null = null;
  TRIT_REM: string | null = null;
  FILE_FSET: string | null = null;
  TRIT_FZWC: string | null = null;
  TRIT_RATE: number | null = null;
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

export class WGPG extends AgsGroup {
  static readonly code = "WGPG";
  LOCA_ID: string | null = null;
  WGPG_ID: string | null = null;
  WGPG_TOOL: string | null = null;
  WGPG_DATE: Date | null = null;
  WGPG_STRT: number | null = null;
  WGPG_STOP: number | null = null;
  WGPG_BHD: number | null = null;
  WGPG_WAT: string | null = null;
  WGPG_DETL: string | null = null;
  WGPG_CDIA: string | null = null;
  WGPG_REM: string | null = null;
  WGPG_ENV: string | null = null;
  WGPG_METH: string | null = null;
  WGPG_CONT: string | null = null;
  WGPG_CRED: string | null = null;
  WGPG_STAT: string | null = null;
  FILE_FSET: string | null = null;
  WGPG_OPER: string | null = null;
  WGPG_LIM: string | null = null;
  WGPG_ULIM: string | null = null;
  wgpts: WGPT[] = [];
  constructor(init: Partial<WGPG> = {}) {
    super();
    Object.assign(this, init);
  }
}

export class WGPT extends AgsGroup {
  static readonly code = "WGPT";
  LOCA_ID: string | null = null;
  WGPG_ID: string | null = null;
  WGPG_TOOL: string | null = null;
  WGPT_PARA: string | null = null;
  WGPT_UNIT: string | null = null;
  WGPT_DPTH: number | null = null;
  WGPT_RDNG: string | null = null;
  WGPT_CAS: string | null = null;
  WGPT_REM: string | null = null;
  FILE_FSET: string | null = null;
  constructor(init: Partial<WGPT> = {}) {
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
  WSTD_POST: number | null = null;
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
