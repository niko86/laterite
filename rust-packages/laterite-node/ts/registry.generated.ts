// AUTO-GENERATED from ags5_dictionary.json by tools/generate-typed-graph.mjs.
// DO NOT EDIT — re-run the generator after a dictionary change.

export type HeadingStatus = "KEY" | "REQUIRED" | "OTHER";

export interface GeneratedHeading {
  readonly name: string;
  readonly status: HeadingStatus;
  readonly type: string;
  readonly unit: string | null;
  readonly description: string;
}

export interface GeneratedGroup {
  readonly code: string;
  readonly contents: string;
  readonly parent: string | null;
  readonly isHighVolume: boolean;
  readonly headings: readonly GeneratedHeading[];
}

export const GROUPS_DATA: readonly GeneratedGroup[] = [
  {
    "code": "PROJ",
    "contents": "Project Information",
    "parent": null,
    "isHighVolume": false,
    "headings": [
      {
        "name": "PROJ_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Project identifier"
      },
      {
        "name": "PROJ_NAME",
        "status": "REQUIRED",
        "type": "X",
        "unit": null,
        "description": "Project title"
      },
      {
        "name": "PROJ_LOC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Project location"
      },
      {
        "name": "PROJ_CLNT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Client"
      },
      {
        "name": "PROJ_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Contractor"
      },
      {
        "name": "PROJ_ENG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Project engineer"
      },
      {
        "name": "PROJ_MEMO",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "General project memo"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PROJ_OFFC",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "LOCA",
    "contents": "Location Details",
    "parent": "PROJ",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "LOCA_TYPE",
        "status": "REQUIRED",
        "type": "PA",
        "unit": null,
        "description": "Location type (CP, CPT, RC, TP)"
      },
      {
        "name": "LOCA_STAT",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Status of investigation point"
      },
      {
        "name": "LOCA_NATE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "National grid easting"
      },
      {
        "name": "LOCA_NATN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "National grid northing"
      },
      {
        "name": "LOCA_GREF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "National grid reference"
      },
      {
        "name": "LOCA_GL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Ground level"
      },
      {
        "name": "LOCA_LAT",
        "status": "OTHER",
        "type": "6DP",
        "unit": "deg",
        "description": "Latitude (decimal degrees)"
      },
      {
        "name": "LOCA_LON",
        "status": "OTHER",
        "type": "6DP",
        "unit": "deg",
        "description": "Longitude (decimal degrees)"
      },
      {
        "name": "LOCA_FDEP",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Final depth"
      },
      {
        "name": "LOCA_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "LOCA_STAR",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "LOCA_PURP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_TERM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_ENDD",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "LOCA_LETT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_LOCX",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "LOCA_LOCY",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "LOCA_LOCZ",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "LOCA_LREF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_DATM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_ETRV",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "LOCA_NTRV",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "LOCA_LTRV",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "LOCA_XTRL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "LOCA_YTRL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "LOCA_ZTRL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "LOCA_ELAT",
        "status": "OTHER",
        "type": "DMS",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_ELON",
        "status": "OTHER",
        "type": "DMS",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_LLZ",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_LOCM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_LOCA",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_CLST",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_ALID",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_OFFS",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_CNGE",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_TRAN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_CHKG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LOCA_APPG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "GEOL",
    "contents": "Field Geological Descriptions",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier (parent)"
      },
      {
        "name": "GEOL_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of stratum"
      },
      {
        "name": "GEOL_BASE",
        "status": "REQUIRED",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of stratum"
      },
      {
        "name": "GEOL_DESC",
        "status": "REQUIRED",
        "type": "X",
        "unit": null,
        "description": "Stratum description"
      },
      {
        "name": "GEOL_LEG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Legend code"
      },
      {
        "name": "GEOL_GEOL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Geology code"
      },
      {
        "name": "GEOL_GEO2",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Secondary geology code"
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stratum status"
      },
      {
        "name": "GEOL_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "GEOL_BGS",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "GEOL_FORM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "CORE",
    "contents": "Coring Information",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier (parent)"
      },
      {
        "name": "CORE_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of core run"
      },
      {
        "name": "CORE_BASE",
        "status": "REQUIRED",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of core run"
      },
      {
        "name": "CORE_PREC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Total core recovery"
      },
      {
        "name": "CORE_SREC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Solid core recovery"
      },
      {
        "name": "CORE_RQD",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Rock Quality Designation"
      },
      {
        "name": "CORE_DIAM",
        "status": "OTHER",
        "type": "X",
        "unit": "mm",
        "description": "Core diameter code"
      },
      {
        "name": "CORE_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "CORE_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm",
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "SAMP",
    "contents": "Sample Information",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier (parent)"
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of sample"
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Sample reference"
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Sample type (B, C, D, ES, EW, U)"
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Sample unique identifier"
      },
      {
        "name": "SAMP_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of sample"
      },
      {
        "name": "SAMP_DTIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Date and time sample taken"
      },
      {
        "name": "SAMP_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Sample remarks"
      },
      {
        "name": "SAMP_UBLO",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_SDIA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "SAMP_WDEP",
        "status": "OTHER",
        "type": "X",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_RECV",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "SAMP_TECH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_MATX",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_WHO",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_WHY",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_DESD",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "SAMP_LOG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_COND",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_CLSS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_BAR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "bar",
        "description": ""
      },
      {
        "name": "SAMP_TEMP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "degC",
        "description": ""
      },
      {
        "name": "SAMP_PRES",
        "status": "OTHER",
        "type": "1DP",
        "unit": "bar",
        "description": ""
      },
      {
        "name": "SAMP_FLOW",
        "status": "OTHER",
        "type": "1DP",
        "unit": "l/min",
        "description": ""
      },
      {
        "name": "SAMP_ETIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": ""
      },
      {
        "name": "SAMP_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": ""
      },
      {
        "name": "SAMP_CAPT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_LINK",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_RECL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      }
    ]
  },
  {
    "code": "LLPL",
    "contents": "Liquid and Plastic Limit Tests",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of sample"
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Sample reference"
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Sample type"
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Sample unique identifier"
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Specimen reference"
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of specimen"
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Specimen description"
      },
      {
        "name": "LLPL_LL",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Liquid limit"
      },
      {
        "name": "LLPL_PL",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Plastic limit"
      },
      {
        "name": "LLPL_PI",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Plasticity index"
      },
      {
        "name": "LLPL_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "LLPL_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LLPL_425",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": ""
      },
      {
        "name": "LLPL_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LLPL_STAB",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": ""
      },
      {
        "name": "LLPL_STYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LLPL_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LLPL_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "TREG",
    "contents": "Triaxial Tests - Effective Stress - General",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of sample"
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Sample reference"
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Sample type"
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Sample unique identifier"
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Specimen reference"
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of specimen"
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Specimen description"
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Specimen preparation method"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "TREG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Triaxial test type"
      },
      {
        "name": "TREG_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Specimen condition"
      },
      {
        "name": "TREG_COH",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Cohesion intercept"
      },
      {
        "name": "TREG_PHI",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Angle of shearing resistance"
      },
      {
        "name": "TREG_FCR",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Failure criterion"
      },
      {
        "name": "TREG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "TREG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Testing laboratory"
      },
      {
        "name": "TREG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test status"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      },
      {
        "name": "TREG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "TREG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviations from test method"
      }
    ]
  },
  {
    "code": "TRET",
    "contents": "Triaxial Tests - Effective Stress - Per Test",
    "parent": "TREG",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of sample"
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Sample reference"
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Sample type"
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Sample unique identifier"
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Specimen reference"
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of specimen"
      },
      {
        "name": "TRET_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Triaxial test number (multi-stage)"
      },
      {
        "name": "TRET_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "TRET_SDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "TRET_LEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "TRET_IMC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "TRET_FMC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "TRET_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "TRET_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "TRET_SAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRET_CONS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRET_CONP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "TRET_CELL",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "TRET_PWPI",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "TRET_STRR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%/hr",
        "description": ""
      },
      {
        "name": "TRET_STRN",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "TRET_DEVF",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "TRET_PWPF",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "TRET_STV",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "TRET_MODE",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "TREL",
    "contents": "Triaxial Tests - Logged Data (AGS-L draft, publish 2026)",
    "parent": "TRET",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of sample"
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Sample reference"
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Sample type"
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Sample unique identifier"
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Specimen reference"
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of specimen"
      },
      {
        "name": "TRET_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Triaxial test number"
      },
      {
        "name": "TREL_MNUM",
        "status": "KEY",
        "type": "0DP",
        "unit": null,
        "description": "Measurement number / record index"
      },
      {
        "name": "TREL_TTIM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "s",
        "description": "Elapsed time since start of test"
      },
      {
        "name": "TREL_TTDT",
        "status": "OTHER",
        "type": "DT",
        "unit": null,
        "description": "Test date time"
      },
      {
        "name": "TREL_STIM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "s",
        "description": "Elapsed time since start of stage"
      },
      {
        "name": "TREL_STGN",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Stage number"
      },
      {
        "name": "TREL_STGD",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stage description"
      },
      {
        "name": "TREL_CELL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Cell pressure"
      },
      {
        "name": "TREL_BACK",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Back pressure"
      },
      {
        "name": "TREL_PWP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Pore pressure (external instrumentation)"
      },
      {
        "name": "TREL_PWPM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Pore pressure (mid-height)"
      },
      {
        "name": "TREL_SZT",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Vertical total stress"
      },
      {
        "name": "TREL_SZE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Vertical effective stress"
      },
      {
        "name": "TREL_SRT",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Radial total stress"
      },
      {
        "name": "TREL_SRE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Radial effective stress"
      },
      {
        "name": "TREL_EZET",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Total vertical strain (external)"
      },
      {
        "name": "TREL_EZES",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Stage vertical strain (external)"
      },
      {
        "name": "TREL_EPET",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Total volumetric strain (external)"
      },
      {
        "name": "TREL_EPES",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Stage volumetric strain (external)"
      },
      {
        "name": "TREL_EZ1T",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Total vertical strain (local LVDT 1)"
      },
      {
        "name": "TREL_EZ1S",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Stage vertical strain (local LVDT 1)"
      },
      {
        "name": "TREL_EZ2T",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Total vertical strain (local LVDT 2)"
      },
      {
        "name": "TREL_EZ2S",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Stage vertical strain (local LVDT 2)"
      },
      {
        "name": "TREL_ER1T",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Total radial strain (local LDT 1)"
      },
      {
        "name": "TREL_ER1S",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Stage radial strain (local LDT 1)"
      },
      {
        "name": "TREL_CYCN",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Cycle number"
      }
    ]
  },
  {
    "code": "CONL",
    "contents": "Consolidation Tests - Lab Data (AGS-L draft, publish 2026)",
    "parent": "SAMP",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of sample"
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Sample reference"
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Sample type"
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Sample unique identifier"
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Specimen reference"
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of specimen"
      },
      {
        "name": "CONL_MNUM",
        "status": "KEY",
        "type": "0DP",
        "unit": null,
        "description": "Measurement number / record index"
      },
      {
        "name": "CONL_TTIM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "s",
        "description": "Elapsed time since start of test"
      },
      {
        "name": "CONL_TTDT",
        "status": "OTHER",
        "type": "DT",
        "unit": null,
        "description": "Test date time"
      },
      {
        "name": "CONL_STIM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "s",
        "description": "Elapsed time since start of stage"
      },
      {
        "name": "CONL_STGN",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Stage number"
      },
      {
        "name": "CONL_STGD",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stage description"
      },
      {
        "name": "CONL_SZT",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Applied vertical stress"
      },
      {
        "name": "CONL_HGHT",
        "status": "OTHER",
        "type": "3DP",
        "unit": "mm",
        "description": "Specimen height"
      },
      {
        "name": "CONL_EZET",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Total vertical strain"
      },
      {
        "name": "CONL_VR",
        "status": "OTHER",
        "type": "4DP",
        "unit": null,
        "description": "Void ratio"
      },
      {
        "name": "CONL_PWP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Pore pressure"
      }
    ]
  },
  {
    "code": "ABBR",
    "contents": "(scaffolded) ABBR",
    "parent": null,
    "isHighVolume": false,
    "headings": [
      {
        "name": "ABBR_HDNG",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ABBR_CODE",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ABBR_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ABBR_LIST",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ABBR_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "BKFL",
    "contents": "(scaffolded) BKFL",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "BKFL_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "BKFL_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "BKFL_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "BKFL_LEG",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "BKFL_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "BKFL_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "CBRG",
    "contents": "(scaffolded) CBRG",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CBRG_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "CBRG_NMC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CBRG_200",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CBRG_STAB",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CBRG_STYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CBRG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CBRG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CBRG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CBRG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CBRG_SIZE",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "CBRT",
    "contents": "(scaffolded) CBRT",
    "parent": "CBRG",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "CBRT_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CBRT_TOP",
        "status": "KEY",
        "type": "2SF",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CBRT_BASE",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CBRT_MCT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CBRT_MCBT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CBRT_IMC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CBRT_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "CBRT_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "CBRT_SURC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "CBRT_SKDT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CBRT_SWEL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "CBRT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "CDIA",
    "contents": "(scaffolded) CDIA",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "CDIA_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "CDIA_DIAM",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "CDIA_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "CHIS",
    "contents": "(scaffolded) CHIS",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "CHIS_FROM",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "CHIS_TO",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "CHIS_TIME",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm",
        "description": ""
      },
      {
        "name": "CHIS_STAR",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": ""
      },
      {
        "name": "CHIS_TOOL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CHIS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "CHOC",
    "contents": "(scaffolded) CHOC",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "CHOC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CHOC_FROM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CHOC_TO",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CHOC_DDIS",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "CHOC_BTCH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CHOC_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CHOC_CONT",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "CMPG",
    "contents": "(scaffolded) CMPG",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "CMPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CMPG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "CMPG_MOLD",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "CMPG_375",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CMPG_200",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CMPG_PDEN",
        "status": "OTHER",
        "type": "XN",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "CMPG_MAXD",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "CMPG_MCOP",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CMPG_STAB",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CMPG_STYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CMPG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CMPG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CMPG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CMPG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CMPG_SIZ1",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "CMPG_SIZ2",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "CMPT",
    "contents": "(scaffolded) CMPT",
    "parent": "CMPG",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "CMPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CMPT_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CMPT_MC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CMPT_DDEN",
        "status": "OTHER",
        "type": "3DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "CMPT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "CONG",
    "contents": "(scaffolded) CONG",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CONG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "CONG_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "CONG_SDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "CONG_HIGT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "CONG_MCI",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CONG_MCF",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CONG_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "CONG_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "CONG_PDEN",
        "status": "OTHER",
        "type": "XN",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "CONG_SATR",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CONG_SPRS",
        "status": "OTHER",
        "type": "2SF",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "CONG_SATH",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CONG_IVR",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "CONG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CONG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CONG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CONG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "CONS",
    "contents": "(scaffolded) CONS",
    "parent": "CONG",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "CONS_INCN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CONS_IVR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "CONS_INCF",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "CONS_INCE",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "CONS_INMV",
        "status": "OTHER",
        "type": "2SF",
        "unit": "m2/MN",
        "description": ""
      },
      {
        "name": "CONS_INSC",
        "status": "OTHER",
        "type": "2SF",
        "unit": null,
        "description": ""
      },
      {
        "name": "CONS_CVRT",
        "status": "OTHER",
        "type": "2SF",
        "unit": "m2/yr",
        "description": ""
      },
      {
        "name": "CONS_CVLG",
        "status": "OTHER",
        "type": "2SF",
        "unit": "m2/yr",
        "description": ""
      },
      {
        "name": "CONS_TEMP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "degC",
        "description": ""
      },
      {
        "name": "CONS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "DCPG",
    "contents": "(scaffolded) DCPG",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "DCPG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "DCPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DCPG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "DCPG_ZERO",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "DCPG_LREM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DCPG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DCPG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DCPG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DCPG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DCPG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "DCPT",
    "contents": "(scaffolded) DCPT",
    "parent": "DCPG",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "DCPG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "DCPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DCPG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "DCPT_CBLO",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "DCPT_PEN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "DCPT_DEL",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm",
        "description": ""
      },
      {
        "name": "DCPT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "DETL",
    "contents": "(scaffolded) DETL",
    "parent": "LOCA",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "DETL_TOP",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "DETL_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "DETL_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DETL_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "DICT",
    "contents": "(scaffolded) DICT",
    "parent": null,
    "isHighVolume": false,
    "headings": [
      {
        "name": "DICT_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "DICT_GRP",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DICT_HDNG",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DICT_STAT",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "DICT_DTYP",
        "status": "OTHER",
        "type": "PT",
        "unit": null,
        "description": ""
      },
      {
        "name": "DICT_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DICT_UNIT",
        "status": "OTHER",
        "type": "PU",
        "unit": null,
        "description": ""
      },
      {
        "name": "DICT_EXMP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DICT_PGRP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DICT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "DISC",
    "contents": "(scaffolded) DISC",
    "parent": "LOCA",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "DISC_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "DISC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "FRAC_SET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DISC_NUMB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DISC_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "DISC_DIP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "DISC_DIR",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "DISC_RGH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DISC_PLAN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DISC_WAVE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "DISC_AMP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "DISC_JRC",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "DISC_APP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DISC_APT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "DISC_APOB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DISC_INFM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DISC_TERM",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "DISC_PERS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "DISC_STR",
        "status": "OTHER",
        "type": "0DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "DISC_WETH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DISC_SEEP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DISC_FLOW",
        "status": "OTHER",
        "type": "0DP",
        "unit": "l/s",
        "description": ""
      },
      {
        "name": "DISC_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "DREM",
    "contents": "(scaffolded) DREM",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "DREM_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "DREM_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "DREM_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "ERES",
    "contents": "(scaffolded) ERES",
    "parent": "SAMP",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "ERES_CODE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_MATX",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_RTYP",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_TESN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_NAME",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_TNAM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_RVAL",
        "status": "OTHER",
        "type": "6DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_RUNI",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_RTXT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_RTCD",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_RRES",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_DETF",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_ORG",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_IQLF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_LQLF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_RDLM",
        "status": "OTHER",
        "type": "6DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_MDLM",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_QLM",
        "status": "OTHER",
        "type": "6DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_DUNI",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_TICP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "ERES_TICT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "s",
        "description": ""
      },
      {
        "name": "ERES_RDAT",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "ERES_SGRP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_DTIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": ""
      },
      {
        "name": "ERES_TEST",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_TORD",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_LOCN",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_BAS",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_DIL",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_LMTH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_LDTM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": ""
      },
      {
        "name": "ERES_IREF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_SIZE",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "ERES_PERP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "ERES_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ERES_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "FLSH",
    "contents": "(scaffolded) FLSH",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "FLSH_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "FLSH_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "FLSH_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "FLSH_RETN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "FLSH_RETX",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "FLSH_COL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FLSH_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "FRAC",
    "contents": "(scaffolded) FRAC",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "FRAC_FROM",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "FRAC_TO",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "FRAC_SET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FRAC_IMAX",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "FRAC_IAVE",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "FRAC_IMIN",
        "status": "OTHER",
        "type": "X",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "FRAC_FI",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FRAC_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "GCHM",
    "contents": "(scaffolded) GCHM",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "GCHM_CODE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "GCHM_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "GCHM_TTYP",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "GCHM_RESL",
        "status": "OTHER",
        "type": "XN",
        "unit": null,
        "description": ""
      },
      {
        "name": "GCHM_UNIT",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "GCHM_NAME",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "GCHM_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "GCHM_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "GCHM_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "GCHM_DLM",
        "status": "OTHER",
        "type": "4DP",
        "unit": "-",
        "description": ""
      }
    ]
  },
  {
    "code": "GRAG",
    "contents": "(scaffolded) GRAG",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "GRAG_UC",
        "status": "OTHER",
        "type": "1SF",
        "unit": null,
        "description": ""
      },
      {
        "name": "GRAG_VCRE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "GRAG_GRAV",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "GRAG_SAND",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "GRAG_SILT",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "GRAG_CLAY",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "GRAG_FINE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "GRAG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "GRAG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "GRAG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "GRAG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "GRAT",
    "contents": "(scaffolded) GRAT",
    "parent": "GRAG",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "GRAT_SIZE",
        "status": "OTHER",
        "type": "3SF",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "GRAT_PERP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "GRAT_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "GRAT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "HDIA",
    "contents": "(scaffolded) HDIA",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "HDIA_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "HDIA_DIAM",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "HDIA_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "HDPH",
    "contents": "(scaffolded) HDPH",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "HDPH_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "HDPH_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "HDPH_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "HDPH_STAR",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": ""
      },
      {
        "name": "HDPH_ENDD",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": ""
      },
      {
        "name": "HDPH_CREW",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "HDPH_EXC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "HDPH_SHOR",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "HDPH_STAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "HDPH_DIML",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "HDPH_DIMW",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "HDPH_DBIT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "HDPH_BCON",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "HDPH_BTYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "HDPH_BLEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "HDPH_LOG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "HDPH_LOGD",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "HDPH_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "HDPH_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "HDPH_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "HDPH_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "HORN",
    "contents": "(scaffolded) HORN",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "HORN_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "HORN_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "HORN_ORNT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "HORN_INCL",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "HORN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "IPEN",
    "contents": "(scaffolded) IPEN",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPEN_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "IPEN_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPEN_IPEN",
        "status": "OTHER",
        "type": "X",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "IPEN_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "IPEN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPEN_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPEN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPEN_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPEN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "IPID",
    "contents": "(scaffolded) IPID",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPID_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "IPID_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPID_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "IPID_TEMP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "DegC",
        "description": ""
      },
      {
        "name": "IPID_RES",
        "status": "OTHER",
        "type": "2DP",
        "unit": "ppmv",
        "description": ""
      },
      {
        "name": "IPID_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPID_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPID_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPID_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPID_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "IPRG",
    "contents": "(scaffolded) IPRG",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPRG_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "IPRG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPRG_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "IPRG_STG",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPRG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPRG_PRWL",
        "status": "OTHER",
        "type": "X",
        "unit": "m",
        "description": ""
      },
      {
        "name": "IPRG_SWAL",
        "status": "OTHER",
        "type": "X",
        "unit": "m",
        "description": ""
      },
      {
        "name": "IPRG_TDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "IPRG_SDIA",
        "status": "OTHER",
        "type": "3DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "IPRG_IPRM",
        "status": "OTHER",
        "type": "1SCI",
        "unit": "m/s",
        "description": ""
      },
      {
        "name": "IPRG_FLOW",
        "status": "OTHER",
        "type": "2DP",
        "unit": "l/s",
        "description": ""
      },
      {
        "name": "IPRG_AWL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "IPRG_HEAD",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "IPRG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "IPRG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPRG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPRG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPRG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPRG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "IPRT",
    "contents": "(scaffolded) IPRT",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPRG_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "IPRG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPRG_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "IPRG_STG",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "IPRT_TIME",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": ""
      },
      {
        "name": "IPRT_DPTH",
        "status": "KEY",
        "type": "X",
        "unit": "m",
        "description": ""
      },
      {
        "name": "IPRT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "ISPT",
    "contents": "(scaffolded) ISPT",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "ISPT_SEAT",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_MAIN",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_NPEN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "ISPT_NVAL",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_REP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_CAS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "ISPT_WAT",
        "status": "OTHER",
        "type": "X",
        "unit": "m",
        "description": ""
      },
      {
        "name": "ISPT_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_HAM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_ERAT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "ISPT_SWP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "ISPT_INC1",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_INC2",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_INC3",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_INC4",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_INC5",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_INC6",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_PEN1",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "ISPT_PEN2",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "ISPT_PEN3",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "ISPT_PEN4",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "ISPT_PEN5",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "ISPT_PEN6",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "ISPT_ROCK",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISPT_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "IVAN",
    "contents": "(scaffolded) IVAN",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "IVAN_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "IVAN_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IVAN_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "IVAN_IVAN",
        "status": "OTHER",
        "type": "X",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "IVAN_IVAR",
        "status": "OTHER",
        "type": "X",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "IVAN_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "IVAN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IVAN_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IVAN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IVAN_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "IVAN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "LBSG",
    "contents": "(scaffolded) LBSG",
    "parent": null,
    "isHighVolume": false,
    "headings": [
      {
        "name": "LBSG_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBSG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "LBSG_FROM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBSG_TO",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBSG_DUE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "LBSG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBSG_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBSG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "LBST",
    "contents": "(scaffolded) LBST",
    "parent": "LBSG",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBSG_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBST_TEST",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "CHOC_REF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBST_TTYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBST_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBST_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBST_DEPN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBST_STAT",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBST_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBST_DUE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "LBST_DETL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBST_DONE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LBST_TCNT",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "LDEN",
    "contents": "(scaffolded) LDEN",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LDEN_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "LDEN_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "LDEN_SMTY",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "LDEN_MC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "LDEN_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "LDEN_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "LDEN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LDEN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LDEN_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LDEN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "LLIN",
    "contents": "(scaffolded) LLIN",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LLIN_LS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "LLIN_425",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "LLIN_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LLIN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LLIN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LLIN_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LLIN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "LNMC",
    "contents": "(scaffolded) LNMC",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LNMC_MC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "LNMC_TEMP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "degC",
        "description": ""
      },
      {
        "name": "LNMC_STAB",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": ""
      },
      {
        "name": "LNMC_STYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LNMC_ISNT",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": ""
      },
      {
        "name": "LNMC_COMM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LNMC_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LNMC_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LNMC_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LNMC_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "MOND",
    "contents": "(scaffolded) MOND",
    "parent": "MONG",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "MONG_ID",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MONG_DIS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "MOND_DTIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": ""
      },
      {
        "name": "MOND_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "MOND_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MOND_INST",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MOND_RDNG",
        "status": "OTHER",
        "type": "XN",
        "unit": null,
        "description": ""
      },
      {
        "name": "MOND_UNIT",
        "status": "OTHER",
        "type": "PU",
        "unit": null,
        "description": ""
      },
      {
        "name": "MOND_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MOND_LIM",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "MOND_ULIM",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "MOND_NAME",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MOND_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MOND_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MOND_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MOND_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "MONG",
    "contents": "(scaffolded) MONG",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "MONG_ID",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MONG_DIS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "PIPE_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MONG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "MONG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "MONG_DETL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MONG_TRZ",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "MONG_BRZ",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "MONG_BRGA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "MONG_BRGB",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "MONG_BRGC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "MONG_INCA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "MONG_INCB",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "MONG_INCC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "MONG_RSCA",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MONG_RSCB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MONG_RSCC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MONG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MONG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "PIPE",
    "contents": "(scaffolded) PIPE",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "PIPE_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PIPE_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "PIPE_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "PIPE_DIAM",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PIPE_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "PIPE_CONS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PIPE_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "PLTG",
    "contents": "(scaffolded) PLTG",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "PLTG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "PLTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PLTG_CYC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PLTG_PDIA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PLTG_SEAT",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kN",
        "description": ""
      },
      {
        "name": "PLTG_FA0",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "PLTG_FA1",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "PLTG_FA2",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "PLTG_SMOD",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "PLTG_EV2",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "PLTG_MOSR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa/m",
        "description": ""
      },
      {
        "name": "PLTG_EMOD",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "PLTG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "PLTG_STAB",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": ""
      },
      {
        "name": "PLTG_STYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PLTG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PLTG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PLTG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PLTG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PLTG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "PLTT",
    "contents": "(scaffolded) PLTT",
    "parent": "PLTG",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "PLTG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "PLTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PLTG_CYC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PLTT_STG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PLTT_TIME",
        "status": "OTHER",
        "type": "1DP",
        "unit": "min",
        "description": ""
      },
      {
        "name": "PLTT_LOAD",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kN",
        "description": ""
      },
      {
        "name": "PLTT_SET1",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PLTT_SET2",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PLTT_SET3",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PLTT_SET4",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PLTT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "PMTD",
    "contents": "(scaffolded) PMTD",
    "parent": "PMTG",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "PMTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTD_SEQ",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTD_ARM1",
        "status": "OTHER",
        "type": "3DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PMTD_ARM2",
        "status": "OTHER",
        "type": "3DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PMTD_ARM3",
        "status": "OTHER",
        "type": "3DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PMTD_TPC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "PMTD_PPA",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "PMTD_PPB",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "PMTD_VOL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "cm3",
        "description": ""
      },
      {
        "name": "PMTD_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTD_ARM4",
        "status": "OTHER",
        "type": "3DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PMTD_ARM5",
        "status": "OTHER",
        "type": "3DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PMTD_ARM6",
        "status": "OTHER",
        "type": "3DP",
        "unit": "mm",
        "description": ""
      }
    ]
  },
  {
    "code": "PMTG",
    "contents": "(scaffolded) PMTG",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "PMTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "PMTG_WAT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "PMTG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTG_CREW",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTG_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTG_DIAM",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PMTG_HO",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "PMTG_GI",
        "status": "OTHER",
        "type": "0DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "PMTG_CU",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "PMTG_PL",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "PMTG_AF",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "PMTG_AD",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "PMTG_AFCV",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "PMTG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "PMTL",
    "contents": "(scaffolded) PMTL",
    "parent": "PMTG",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "PMTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTD_SEQ",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTL_LNO",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTL_GAA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "PMTL_SINC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "PMTL_PINC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "PMTL_STRA",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "PMTL_PRSA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "PMTL_NLSA",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "PMTL_NLSB",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "PMTL_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "PTIM",
    "contents": "(scaffolded) PTIM",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "PTIM_DTIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": ""
      },
      {
        "name": "PTIM_DPTH",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "PTIM_CAS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "PTIM_WAT",
        "status": "OTHER",
        "type": "X",
        "unit": "m",
        "description": ""
      },
      {
        "name": "PTIM_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "RPLT",
    "contents": "(scaffolded) RPLT",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RPLT_PLS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "RPLT_PLSI",
        "status": "OTHER",
        "type": "2DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "RPLT_PLTF",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "RPLT_MC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "RPLT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RPLT_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RPLT_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RPLT_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "RUCS",
    "contents": "(scaffolded) RUCS",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RUCS_SDIA",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "RUCS_LEN",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "RUCS_MC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "RUCS_COND",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RUCS_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "mm:ss",
        "description": ""
      },
      {
        "name": "RUCS_STRA",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa/s",
        "description": ""
      },
      {
        "name": "RUCS_UCS",
        "status": "OTHER",
        "type": "3SF",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "RUCS_MODE",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RUCS_E",
        "status": "OTHER",
        "type": "3SF",
        "unit": "GPa",
        "description": ""
      },
      {
        "name": "RUCS_MU",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "RUCS_ESTR",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RUCS_ETYP",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "RUCS_MACH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RUCS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RUCS_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RUCS_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RUCS_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "RWCO",
    "contents": "(scaffolded) RWCO",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RWCO_MC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "RWCO_TEMP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "degC",
        "description": ""
      },
      {
        "name": "RWCO_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RWCO_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RWCO_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RWCO_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "SCPG",
    "contents": "(scaffolded) SCPG",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_CSA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "cm2",
        "description": ""
      },
      {
        "name": "SCPG_RATE",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm/s",
        "description": ""
      },
      {
        "name": "SCPG_FILT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_FRIC",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_WAT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SCPG_WATA",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_CAR",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_SLAR",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "SCPT",
    "contents": "(scaffolded) SCPT",
    "parent": "SCPG",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPT_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SCPT_RES",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCPT_FRES",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCPT_PWP1",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCPT_PWP2",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCPT_PWP3",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCPT_CON",
        "status": "OTHER",
        "type": "4DP",
        "unit": "µS/cm",
        "description": ""
      },
      {
        "name": "SCPT_TEMP",
        "status": "OTHER",
        "type": "4DP",
        "unit": "DegC",
        "description": ""
      },
      {
        "name": "SCPT_PH",
        "status": "OTHER",
        "type": "4DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPT_SLP1",
        "status": "OTHER",
        "type": "4DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "SCPT_SLP2",
        "status": "OTHER",
        "type": "4DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "SCPT_REDX",
        "status": "OTHER",
        "type": "4DP",
        "unit": "mV",
        "description": ""
      },
      {
        "name": "SCPT_MAGT",
        "status": "OTHER",
        "type": "4DP",
        "unit": "nT",
        "description": ""
      },
      {
        "name": "SCPT_MAGX",
        "status": "OTHER",
        "type": "4DP",
        "unit": "nT",
        "description": ""
      },
      {
        "name": "SCPT_MAGY",
        "status": "OTHER",
        "type": "4DP",
        "unit": "nT",
        "description": ""
      },
      {
        "name": "SCPT_MAGZ",
        "status": "OTHER",
        "type": "4DP",
        "unit": "nT",
        "description": ""
      },
      {
        "name": "SCPT_SMP",
        "status": "OTHER",
        "type": "4DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "SCPT_NGAM",
        "status": "OTHER",
        "type": "4DP",
        "unit": "counts/s",
        "description": ""
      },
      {
        "name": "SCPT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPT_FRR",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "SCPT_QT",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCPT_FT",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCPT_QE",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCPT_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "SCPT_CPO",
        "status": "OTHER",
        "type": "2DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "SCPT_CPOD",
        "status": "OTHER",
        "type": "2DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "SCPT_QNET",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCPT_FRRC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "SCPT_EXPP",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCPT_BQ",
        "status": "OTHER",
        "type": "4DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPT_ISPP",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCPT_NQT",
        "status": "OTHER",
        "type": "4DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPT_NFR",
        "status": "OTHER",
        "type": "4DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "SHBG",
    "contents": "(scaffolded) SHBG",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SHBG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SHBG_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SHBG_CONS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SHBG_PCOH",
        "status": "OTHER",
        "type": "2SF",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "SHBG_PHI",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "SHBG_RCOH",
        "status": "OTHER",
        "type": "2SF",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "SHBG_RPHI",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "SHBG_ENCA",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SHBG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SHBG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SHBG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SHBG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "SHBT",
    "contents": "(scaffolded) SHBT",
    "parent": "SHBG",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SHBT_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SHBT_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "SHBT_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "SHBT_NORM",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "SHBT_DISP",
        "status": "OTHER",
        "type": "2SF",
        "unit": "mm/min",
        "description": ""
      },
      {
        "name": "SHBT_DISR",
        "status": "OTHER",
        "type": "2SF",
        "unit": "mm/min",
        "description": ""
      },
      {
        "name": "SHBT_REVS",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "SHBT_PEAK",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "SHBT_RES",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "SHBT_PDIS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "SHBT_RDIS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "SHBT_PDIN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "SHBT_RDIN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "SHBT_PDEN",
        "status": "OTHER",
        "type": "XN",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "SHBT_IVR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "SHBT_MCI",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "SHBT_MCF",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "SHBT_DIA1",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "SHBT_DIA2",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "SHBT_HGT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "SHBT_CRIT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SHBT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "TRAN",
    "contents": "(scaffolded) TRAN",
    "parent": null,
    "isHighVolume": false,
    "headings": [
      {
        "name": "TRAN_ISNO",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRAN_DATE",
        "status": "KEY",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "TRAN_PROD",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRAN_STAT",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRAN_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRAN_AGS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRAN_RECV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRAN_DLIM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRAN_RCON",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRAN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "TREM",
    "contents": "(scaffolded) TREM",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "TREM_DTIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": ""
      },
      {
        "name": "TREM_COMP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TREM_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TREM_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": ""
      },
      {
        "name": "TREM_ETIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "TRIG",
    "contents": "(scaffolded) TRIG",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRIG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRIG_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRIG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRIG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRIG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRIG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "TRIT",
    "contents": "(scaffolded) TRIT",
    "parent": "TRIG",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "TRIT_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRIT_SDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "TRIT_SLEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "TRIT_IMC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "TRIT_FMC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "TRIT_CELL",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "TRIT_DEVF",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "TRIT_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "TRIT_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "TRIT_STRN",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": ""
      },
      {
        "name": "TRIT_CU",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "TRIT_MODE",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRIT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "TRIL",
    "contents": "Triaxial Test Logged Data (AGS-L draft, publish 2026)",
    "parent": "TRIT",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "TRIT_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRIL_MNUM",
        "status": "KEY",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRIL_TTIM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "s",
        "description": ""
      },
      {
        "name": "TRIL_TTDT",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": ""
      },
      {
        "name": "TRIL_STIM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "s",
        "description": ""
      },
      {
        "name": "TRIL_STGN",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRIL_STGD",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TRIL_CELL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "TRIL_SDEV",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "TRIL_EZES",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": ""
      }
    ]
  },
  {
    "code": "TYPE",
    "contents": "(scaffolded) TYPE",
    "parent": null,
    "isHighVolume": false,
    "headings": [
      {
        "name": "TYPE_TYPE",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TYPE_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "UNIT",
    "contents": "(scaffolded) UNIT",
    "parent": null,
    "isHighVolume": false,
    "headings": [
      {
        "name": "UNIT_UNIT",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "UNIT_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "UNIT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "WADD",
    "contents": "(scaffolded) WADD",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "WADD_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "WADD_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "WADD_VOLM",
        "status": "OTHER",
        "type": "0DP",
        "unit": "l",
        "description": ""
      },
      {
        "name": "WADD_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "WADD_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "WINS",
    "contents": "(scaffolded) WINS",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "WINS_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "WINS_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "WINS_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "WINS_DIAM",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "WINS_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": ""
      },
      {
        "name": "WINS_REC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "WINS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "WSTD",
    "contents": "(scaffolded) WSTD",
    "parent": "WSTG",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "WSTG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "WSTD_NMIN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "min",
        "description": ""
      },
      {
        "name": "WSTD_POST",
        "status": "OTHER",
        "type": "X",
        "unit": "m",
        "description": ""
      },
      {
        "name": "WSTD_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "WSTG",
    "contents": "(scaffolded) WSTG",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "WSTG_DPTH",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "WSTG_DTIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": ""
      },
      {
        "name": "WSTG_SEAL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "WSTG_CAS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "WSTG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "ASDI",
    "contents": "(scaffolded) ASDI",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ASDI_SDI1",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "ASDI_SDI2",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "ASDI_SOLN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ASDI_INDR",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ASDI_PADR",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ASDI_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ASDI_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ASDI_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ASDI_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "DOBS",
    "contents": "(scaffolded) DOBS",
    "parent": "LOCA",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "DOBS_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "DOBS_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "DOBS_SET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DOBS_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": ""
      },
      {
        "name": "DOBS_STIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": ""
      },
      {
        "name": "DOBS_ETIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": ""
      },
      {
        "name": "DOBS_DHRT",
        "status": "OTHER",
        "type": "1DP",
        "unit": "Nm",
        "description": ""
      },
      {
        "name": "DOBS_DHRS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "rpm",
        "description": ""
      },
      {
        "name": "DOBS_PENR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "m/hr",
        "description": ""
      },
      {
        "name": "DOBS_HAMM",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": ""
      },
      {
        "name": "DOBS_THRP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "bar",
        "description": ""
      },
      {
        "name": "DOBS_RESP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "bar",
        "description": ""
      },
      {
        "name": "DOBS_TORP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "bar",
        "description": ""
      },
      {
        "name": "DOBS_TORQ",
        "status": "OTHER",
        "type": "1DP",
        "unit": "Nm",
        "description": ""
      },
      {
        "name": "DOBS_THST",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kN",
        "description": ""
      },
      {
        "name": "DOBS_REST",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kN",
        "description": ""
      },
      {
        "name": "DOBS_HAMP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "bar",
        "description": ""
      },
      {
        "name": "DOBS_SPEN",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MJ/m3",
        "description": ""
      },
      {
        "name": "DOBS_FMPO",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "DOBS_FMCR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "l/min",
        "description": ""
      },
      {
        "name": "DOBS_FMRR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "l/min",
        "description": ""
      },
      {
        "name": "DOBS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "DPRB",
    "contents": "(scaffolded) DPRB",
    "parent": "DPRG",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRB_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "DPRB_BLOW",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRB_CBLW",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRB_TORQ",
        "status": "OTHER",
        "type": "0DP",
        "unit": "Nm",
        "description": ""
      },
      {
        "name": "DPRB_DEL",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm",
        "description": ""
      },
      {
        "name": "DPRB_INC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "DPRB_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "DPRG",
    "contents": "(scaffolded) DPRG",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "DPRG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRG_MASS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kg",
        "description": ""
      },
      {
        "name": "DPRG_DROP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "DPRG_CONE",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "DPRG_ROD",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "DPRG_TANV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRG_DAMP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRG_TIP",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "DPRG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRG_ANG",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": ""
      },
      {
        "name": "DPRG_RMSS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kg/m",
        "description": ""
      },
      {
        "name": "DPRG_PARF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRG_PDIU",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRG_BCF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRG_GW",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "DPRG_REET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "DPRG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "ECTN",
    "contents": "(scaffolded) ECTN",
    "parent": "SAMP",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ECTN_ID",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ECTN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "ISAG",
    "contents": "(scaffolded) ISAG",
    "parent": "LOCA",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISAG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISAG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": ""
      },
      {
        "name": "ISAG_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm",
        "description": ""
      },
      {
        "name": "ISAG_PWID",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "ISAG_PLEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "ISAG_PDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "ISAG_DPTS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "ISAG_DPTE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "ISAG_CONS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISAG_SI",
        "status": "OTHER",
        "type": "2SCI",
        "unit": "m/s",
        "description": ""
      },
      {
        "name": "ISAG_PORO",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "ISAG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISAG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISAG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISAG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISAG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "ISAT",
    "contents": "(scaffolded) ISAT",
    "parent": "ISAG",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISAG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "ISAT_TIME",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": ""
      },
      {
        "name": "ISAT_DPTH",
        "status": "KEY",
        "type": "X",
        "unit": "m",
        "description": ""
      },
      {
        "name": "ISAT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "LPDN",
    "contents": "(scaffolded) LPDN",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LPDN_PDEN",
        "status": "OTHER",
        "type": "XN",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "LPDN_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "LPDN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LPDN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LPDN_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LPDN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "LRES",
    "contents": "(scaffolded) LRES",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LRES_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "LRES_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "LRES_MC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "LRES_COND",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LRES_LRES",
        "status": "OTHER",
        "type": "0DP",
        "unit": "ohm m",
        "description": ""
      },
      {
        "name": "LRES_CDIA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "LRES_CCSA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm2",
        "description": ""
      },
      {
        "name": "LRES_CLEN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "LRES_TEMP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "DegC",
        "description": ""
      },
      {
        "name": "LRES_ELEC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LRES_PENT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LRES_CSHP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LRES_WAT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "ml",
        "description": ""
      },
      {
        "name": "LRES_WRES",
        "status": "OTHER",
        "type": "3SF",
        "unit": "ohm m",
        "description": ""
      },
      {
        "name": "LRES_PART",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LRES_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LRES_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LRES_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LRES_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "LVAN",
    "contents": "(scaffolded) LVAN",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LVAN_VNPK",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "LVAN_VNRM",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "LVAN_MC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "LVAN_SIZE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "LVAN_VLEN",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "LVAN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LVAN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LVAN_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "LVAN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "MCVG",
    "contents": "(scaffolded) MCVG",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MCVG_200",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "MCVG_NMC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "MCVG_STAB",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": ""
      },
      {
        "name": "MCVG_STYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MCVG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MCVG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MCVG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MCVG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MCVG_SIZE",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "MCVT",
    "contents": "(scaffolded) MCVT",
    "parent": "MCVG",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "MCVT_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MCVT_MC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "MCVT_CURV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MCVT_RELK",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "MCVT_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "MCVT_DIFF",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "MCVT_RAPD",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "MCVT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "PTST",
    "contents": "(scaffolded) PTST",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "PTST_TESN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PTST_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "PTST_SZUN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PTST_UNS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "PTST_DIAM",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PTST_LEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PTST_MC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "PTST_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "PTST_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "PTST_IDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": ""
      },
      {
        "name": "PTST_DMET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PTST_VOID",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "PTST_K",
        "status": "OTHER",
        "type": "1SCI",
        "unit": "m/s",
        "description": ""
      },
      {
        "name": "PTST_TSTR",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": ""
      },
      {
        "name": "PTST_HYGR",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "PTST_ISAT",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": ""
      },
      {
        "name": "PTST_SAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PTST_CONS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PTST_PDEN",
        "status": "OTHER",
        "type": "XN",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "PTST_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "PTST_CELL",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "PTST_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PTST_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PTST_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "PTST_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "RDEN",
    "contents": "(scaffolded) RDEN",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RDEN_MC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "RDEN_SMC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "RDEN_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "RDEN_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "RDEN_PORO",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "RDEN_PDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "RDEN_TEMP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "degC",
        "description": ""
      },
      {
        "name": "RDEN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RDEN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RDEN_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RDEN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "RELD",
    "contents": "(scaffolded) RELD",
    "parent": "SAMP",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SAMP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "SAMP_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RELD_DMAX",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "RELD_375",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "RELD_063",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "RELD_020",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "RELD_DMIN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": ""
      },
      {
        "name": "RELD_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RELD_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RELD_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RELD_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "RELD_SIZ1",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "RELD_SIZ2",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": ""
      },
      {
        "name": "RELD_SIZ3",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "SCDG",
    "contents": "(scaffolded; parent-corrected) SCDG",
    "parent": "SCPG",
    "isHighVolume": false,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCDG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SCDG_PWPI",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCDG_PWPE",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCDG_DDIS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": ""
      },
      {
        "name": "SCDG_T",
        "status": "OTHER",
        "type": "0DP",
        "unit": "s",
        "description": ""
      },
      {
        "name": "SCDG_CV",
        "status": "OTHER",
        "type": "1DP",
        "unit": "m2/yr",
        "description": ""
      },
      {
        "name": "SCDG_CVMT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCDG_CH",
        "status": "OTHER",
        "type": "1DP",
        "unit": "m2/yr",
        "description": ""
      },
      {
        "name": "SCDG_CHMT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCDG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "SCDT",
    "contents": "(scaffolded; parent-corrected) SCDT",
    "parent": "SCDG",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "SCDG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "SCDT_SECS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "s",
        "description": ""
      },
      {
        "name": "SCDT_RES",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCDT_PWP1",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCDT_PWP2",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCDT_PWP3",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": ""
      },
      {
        "name": "SCDT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  },
  {
    "code": "WETH",
    "contents": "(scaffolded) WETH",
    "parent": "LOCA",
    "isHighVolume": true,
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": ""
      },
      {
        "name": "WETH_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "WETH_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": ""
      },
      {
        "name": "WETH_SCH",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "WETH_SYS",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": ""
      },
      {
        "name": "WETH_WETH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "WETH_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": ""
      }
    ]
  }
];
