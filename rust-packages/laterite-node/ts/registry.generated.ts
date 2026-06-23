// AUTO-GENERATED from ags_dictionary.json by tools/generate-typed-graph.mjs.
// DO NOT EDIT — re-run the generator after a dictionary change.

export type HeadingStatus = "KEY" | "REQUIRED" | "OTHER" | "KEY+REQUIRED" | "DEPRECATED";

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
  readonly headings: readonly GeneratedHeading[];
}

export const GROUPS_DATA: readonly GeneratedGroup[] = [
  {
    "code": "PROJ",
    "contents": "Project Information",
    "parent": null,
    "headings": [
      {
        "name": "PROJ_ID",
        "status": "KEY+REQUIRED",
        "type": "ID",
        "unit": null,
        "description": "Project identifier"
      },
      {
        "name": "PROJ_NAME",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Project title"
      },
      {
        "name": "PROJ_LOC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Location of site"
      },
      {
        "name": "PROJ_CLNT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Client organisation name"
      },
      {
        "name": "PROJ_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Contractor organisation name"
      },
      {
        "name": "PROJ_ENG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Project engineer/consultant/designer organisation name"
      },
      {
        "name": "PROJ_MEMO",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "General project comments"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. project specification, site location drawings)"
      }
    ]
  },
  {
    "code": "ABBR",
    "contents": "Abbreviation Definitions",
    "parent": null,
    "headings": [
      {
        "name": "ABBR_HDNG",
        "status": "KEY+REQUIRED",
        "type": "X",
        "unit": null,
        "description": "Field heading in group"
      },
      {
        "name": "ABBR_CODE",
        "status": "KEY+REQUIRED",
        "type": "X",
        "unit": null,
        "description": "Abbreviation used"
      },
      {
        "name": "ABBR_DESC",
        "status": "REQUIRED",
        "type": "X",
        "unit": null,
        "description": "Description of abbreviation"
      },
      {
        "name": "ABBR_LIST",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Source of abbreviation"
      },
      {
        "name": "ABBR_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. contract data specification)"
      }
    ]
  },
  {
    "code": "DICT",
    "contents": "User Defined Groups and Headings",
    "parent": null,
    "headings": [
      {
        "name": "DICT_TYPE",
        "status": "KEY+REQUIRED",
        "type": "PA",
        "unit": null,
        "description": "Flag to indicate definition is a GROUP or HEADING (i.e. can be either of GROUP or HEADING)"
      },
      {
        "name": "DICT_GRP",
        "status": "KEY+REQUIRED",
        "type": "X",
        "unit": null,
        "description": "Group name"
      },
      {
        "name": "DICT_HDNG",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Heading name (Note: This data is REQUIRED where DICT_TYPE='HEADING')"
      },
      {
        "name": "DICT_STAT",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Heading status KEY, REQUIRED or OTHER  (Note: This data is REQUIRED where DICT_TYPE='HEADING')"
      },
      {
        "name": "DICT_DTYP",
        "status": "OTHER",
        "type": "PT",
        "unit": null,
        "description": "Type of data and format  (Note: This data is REQUIRED where DICT_TYPE='HEADING')"
      },
      {
        "name": "DICT_DESC",
        "status": "REQUIRED",
        "type": "X",
        "unit": null,
        "description": "Description"
      },
      {
        "name": "DICT_UNIT",
        "status": "OTHER",
        "type": "PU",
        "unit": null,
        "description": "Units  (Note: This data is REQUIRED where DICT_TYPE='HEADING')"
      },
      {
        "name": "DICT_EXMP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Example"
      },
      {
        "name": "DICT_PGRP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Parent group name  (Note: This data is REQUIRED where DICT_TYPE='GROUP')"
      },
      {
        "name": "DICT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "FILE",
    "contents": "Associated Files",
    "parent": null,
    "headings": [
      {
        "name": "FILE_FSET",
        "status": "KEY+REQUIRED",
        "type": "X",
        "unit": null,
        "description": "File set reference"
      },
      {
        "name": "FILE_NAME",
        "status": "KEY+REQUIRED",
        "type": "X",
        "unit": null,
        "description": "File name"
      },
      {
        "name": "FILE_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Description of content"
      },
      {
        "name": "FILE_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "File type"
      },
      {
        "name": "FILE_PROG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Parent program and version number"
      },
      {
        "name": "FILE_DOCT",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Document type"
      },
      {
        "name": "FILE_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "File date"
      },
      {
        "name": "FILE_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Comments on file"
      }
    ]
  },
  {
    "code": "TRAN",
    "contents": "Data File Transmission Information / Data Status",
    "parent": null,
    "headings": [
      {
        "name": "TRAN_ISNO",
        "status": "KEY+REQUIRED",
        "type": "X",
        "unit": null,
        "description": "Issue sequence reference"
      },
      {
        "name": "TRAN_DATE",
        "status": "REQUIRED",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date of production of data file"
      },
      {
        "name": "TRAN_PROD",
        "status": "REQUIRED",
        "type": "X",
        "unit": null,
        "description": "Data file producer"
      },
      {
        "name": "TRAN_STAT",
        "status": "REQUIRED",
        "type": "X",
        "unit": null,
        "description": "Status of data within submission"
      },
      {
        "name": "TRAN_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Description of data transferred"
      },
      {
        "name": "TRAN_AGS",
        "status": "REQUIRED",
        "type": "X",
        "unit": null,
        "description": "AGS Edition Reference"
      },
      {
        "name": "TRAN_RECV",
        "status": "REQUIRED",
        "type": "X",
        "unit": null,
        "description": "Data file recipient"
      },
      {
        "name": "TRAN_DLIM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Record Link data type Delimiter"
      },
      {
        "name": "TRAN_RCON",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Concatenator"
      },
      {
        "name": "TRAN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. data file QA check records)"
      }
    ]
  },
  {
    "code": "TYPE",
    "contents": "Definition of Data Types",
    "parent": null,
    "headings": [
      {
        "name": "TYPE_TYPE",
        "status": "KEY+REQUIRED",
        "type": "X",
        "unit": null,
        "description": "Data type code"
      },
      {
        "name": "TYPE_DESC",
        "status": "REQUIRED",
        "type": "X",
        "unit": null,
        "description": "Description"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "UNIT",
    "contents": "Definition of Units",
    "parent": null,
    "headings": [
      {
        "name": "UNIT_UNIT",
        "status": "KEY+REQUIRED",
        "type": "X",
        "unit": null,
        "description": "Unit"
      },
      {
        "name": "UNIT_DESC",
        "status": "REQUIRED",
        "type": "X",
        "unit": null,
        "description": "Description"
      },
      {
        "name": "UNIT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "AAVT",
    "contents": "Aggregate Abrasion Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "AAVT_AAV",
        "status": "OTHER",
        "type": "2SF",
        "unit": null,
        "description": "Aggregate Abrasion Value"
      },
      {
        "name": "AAVT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "AAVT_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "AAVT_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "AAVT_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "AAVT_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "ACVT",
    "contents": "Aggregate Crushing Value Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "ACVT_ACV",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Aggregate Crushing Value"
      },
      {
        "name": "ACVT_FRAC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Size fraction from which test portion was obtained"
      },
      {
        "name": "ACVT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "ACVT_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "ACVT_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "ACVT_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "ACVT_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "AELO",
    "contents": "Aggregate Elongation Index Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "AELO_EI",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Aggregate elongation index"
      },
      {
        "name": "AELO_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "AELO_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "AELO_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "AELO_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "AELO_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "AFLK",
    "contents": "Aggregate Flakiness Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "AFLK_FI",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Aggregate flakiness index"
      },
      {
        "name": "AFLK_MASS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kg",
        "description": "Mass of test portion"
      },
      {
        "name": "AFLK_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "AFLK_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "AFLK_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "AFLK_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "AFLK_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "AIVT",
    "contents": "Aggregate Impact Value Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "AIVT_AIV1",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Aggregate impact value test 1"
      },
      {
        "name": "AIVT_AIV2",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Aggregate impact value test 2"
      },
      {
        "name": "AIVT_AIV",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Mean aggregate impact value"
      },
      {
        "name": "AIVT_FRAC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Size fraction from which test portion was obtained"
      },
      {
        "name": "AIVT_PDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Particle density of size fraction between 8 mm and 12.5mm"
      },
      {
        "name": "AIVT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "AIVT_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "AIVT_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "AIVT_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "AIVT_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "ALOS",
    "contents": "Los Angeles Abrasion Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "ALOS_LOSA",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Los Angeles coefficient"
      },
      {
        "name": "ALOS_LOPW",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Los Angeles percentage wear"
      },
      {
        "name": "ALOS_LOWR",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Los Angeles wear ratio"
      },
      {
        "name": "ALOS_FRAC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Size fraction from which test portion was obtained"
      },
      {
        "name": "ALOS_CHAR",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Ball load or charge grading"
      },
      {
        "name": "ALOS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "ALOS_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "ALOS_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "ALOS_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "ALOS_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "APSV",
    "contents": "Aggregate Polished Stone Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "APSV_AAV",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Aggregate polished stone value"
      },
      {
        "name": "APSV_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "APSV_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "APSV_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "APSV_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "APSV_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "ARTW",
    "contents": "Aggregate Determination of the Resistance to Wear (micro-Deval)",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "ARTW_FRAC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Size fraction on which sample obtained"
      },
      {
        "name": "ARTW_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of test"
      },
      {
        "name": "ARTW_MD1",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "Micro-Deval coefficient for test specimen one"
      },
      {
        "name": "ARTW_MD2",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "Micro-Deval coefficient for test specimen two"
      },
      {
        "name": "ARTW_MDE",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Mean micro-Deval value (dry)"
      },
      {
        "name": "ARTW_MDS",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Mean micro-Deval value (wet)"
      },
      {
        "name": "ARTW_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date control 2 polished stone value first run"
      },
      {
        "name": "ARTW_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "ARTW_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "ARTW_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "ARTW_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "ARTW_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "ASDI",
    "contents": "Slake Durability Index Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "ASDI_SDI1",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "First cycle slake durability index (if ASDI_SDI1 or ASDI_SDI2 is between 0% and 10%)"
      },
      {
        "name": "ASDI_SDI2",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Second cycle slake durability index"
      },
      {
        "name": "ASDI_SOLN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Nature and temperature of slaking fluid"
      },
      {
        "name": "ASDI_INDR",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Appearance of fragments retained in the drum"
      },
      {
        "name": "ASDI_PADR",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Appearance of fragments passing through the drum"
      },
      {
        "name": "ASDI_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "ASDI_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "ASDI_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "ASDI_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "ASDI_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "ASNS",
    "contents": "Aggregate Soundness Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "ASNS_SOUN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Aggregate soundness test"
      },
      {
        "name": "ASNS_FRAC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Size fraction from which test portion was obtained"
      },
      {
        "name": "ASNS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "ASNS_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "ASNS_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "ASNS_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "ASNS_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "AWAD",
    "contents": "Aggregate Water Absorption Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "AWAD_WTAB",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Aggregate water absorption"
      },
      {
        "name": "AWAD_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "AWAD_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "AWAD_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "AWAD_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "AWAD_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "BKFL",
    "contents": "Exploratory Hole Backfill Details",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "BKFL_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of section"
      },
      {
        "name": "BKFL_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of section"
      },
      {
        "name": "BKFL_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Backfill description"
      },
      {
        "name": "BKFL_LEG",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Backfill legend abbreviation"
      },
      {
        "name": "BKFL_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date of completion of backfill"
      },
      {
        "name": "BKFL_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Backfill remarks including how it was placed"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. drilling journals)"
      }
    ]
  },
  {
    "code": "CBRG",
    "contents": "California Bearing Ratio Tests - General",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "CBRG_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Sample condition"
      },
      {
        "name": "CBRG_NMC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Natural water/moisture content of specimen prior to test"
      },
      {
        "name": "CBRG_200",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Weight percent retained on 20mm sieve"
      },
      {
        "name": "CBRG_STAB",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Amount of stabiliser added"
      },
      {
        "name": "CBRG_STYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of stabiliser added"
      },
      {
        "name": "CBRG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks including commentary on effect of specimen disturbance on test result"
      },
      {
        "name": "CBRG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method including remoulding"
      },
      {
        "name": "CBRG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "CBRG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "CBRG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "CBRT",
    "contents": "California Bearing Ratio Tests - Data",
    "parent": "CBRG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "CBRT_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "CBRT_TOP",
        "status": "OTHER",
        "type": "U",
        "unit": "%",
        "description": "CBR at top"
      },
      {
        "name": "CBRT_BASE",
        "status": "OTHER",
        "type": "U",
        "unit": "%",
        "description": "CBR at bottom"
      },
      {
        "name": "CBRT_MCT",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water/moisture content at top after test"
      },
      {
        "name": "CBRT_MCBT",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water/moisture content at bottom after test"
      },
      {
        "name": "CBRT_IMC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Initial water/moisture content"
      },
      {
        "name": "CBRT_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial bulk density"
      },
      {
        "name": "CBRT_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial Dry density"
      },
      {
        "name": "CBRT_SURC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Surcharge pressure applied"
      },
      {
        "name": "CBRT_SKDT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of soaking"
      },
      {
        "name": "CBRT_SWEL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Amount of swell recorded during soaking (if applicable)"
      },
      {
        "name": "CBRT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test specific remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "CDIA",
    "contents": "Casing Diameter by Depth",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "CDIA_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of base of casing recorded in CDIA_DIAM"
      },
      {
        "name": "CDIA_DIAM",
        "status": "KEY",
        "type": "0DP",
        "unit": "mm",
        "description": "Casing diameter"
      },
      {
        "name": "CDIA_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. casing cement records)"
      }
    ]
  },
  {
    "code": "CHIS",
    "contents": "Chiselling Details",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "CHIS_FROM",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth at start of chiselling"
      },
      {
        "name": "CHIS_TO",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth at end of chiselling"
      },
      {
        "name": "CHIS_TIME",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm",
        "description": "Time taken"
      },
      {
        "name": "CHIS_STAR",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Start time"
      },
      {
        "name": "CHIS_TOOL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Chiselling tool used"
      },
      {
        "name": "CHIS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Notes on chiselling"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. drilling journals)"
      }
    ]
  },
  {
    "code": "CHOC",
    "contents": "Chain of Custody Information",
    "parent": "SAMP",
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
        "name": "CHOC_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Chain of custody reference"
      },
      {
        "name": "CHOC_FROM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Samples despatched from"
      },
      {
        "name": "CHOC_TO",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Samples despatched to"
      },
      {
        "name": "CHOC_DDIS",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date dispatched"
      },
      {
        "name": "CHOC_BTCH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Batch reference"
      },
      {
        "name": "CHOC_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "CHOC_CONT",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Number of sample containers"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (chain of custody sheets)"
      }
    ]
  },
  {
    "code": "CMPG",
    "contents": "Compaction Tests - General",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "CMPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test number"
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Specimen description"
      },
      {
        "name": "CMPG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Compaction test type"
      },
      {
        "name": "CMPG_MOLD",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Compaction mould type"
      },
      {
        "name": "CMPG_375",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Weight percent of material retained on 37.5mm sieve"
      },
      {
        "name": "CMPG_200",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Weight percent of material retained on 20mm sieve"
      },
      {
        "name": "CMPG_PDEN",
        "status": "OTHER",
        "type": "XN",
        "unit": "Mg/m3",
        "description": "Particle density with prefix # if value assumed"
      },
      {
        "name": "CMPG_MAXD",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Maximum dry density"
      },
      {
        "name": "CMPG_MCOP",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Water/moisture content at maximum dry density (Optimum)"
      },
      {
        "name": "CMPG_STAB",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Amount of stabiliser added"
      },
      {
        "name": "CMPG_STYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of stabiliser added"
      },
      {
        "name": "CMPG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks including commentary on effect of specimen disturbance on test result"
      },
      {
        "name": "CMPG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "CMPG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "CMPG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "CMPG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      },
      {
        "name": "CMPG_ZONE",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Grading zone"
      }
    ]
  },
  {
    "code": "CMPT",
    "contents": "Compaction Tests - Data",
    "parent": "CMPG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "CMPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test number"
      },
      {
        "name": "CMPT_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Compaction point number"
      },
      {
        "name": "CMPT_MC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water/moisture content"
      },
      {
        "name": "CMPT_DDEN",
        "status": "OTHER",
        "type": "3DP",
        "unit": "Mg/m3",
        "description": "Dry density at CMPT_MC water/moisture content"
      },
      {
        "name": "CMPT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "CONG",
    "contents": "Consolidation Tests - General",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "CONG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of consolidation test"
      },
      {
        "name": "CONG_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Sample condition"
      },
      {
        "name": "CONG_SDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Test specimen diameter"
      },
      {
        "name": "CONG_HIGT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Test specimen height"
      },
      {
        "name": "CONG_MCI",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Initial water/moisture content"
      },
      {
        "name": "CONG_MCF",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Final water/moisture content"
      },
      {
        "name": "CONG_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial bulk density"
      },
      {
        "name": "CONG_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial dry density"
      },
      {
        "name": "CONG_PDEN",
        "status": "OTHER",
        "type": "XN",
        "unit": "Mg/m3",
        "description": "Particle density with prefix # if value assumed"
      },
      {
        "name": "CONG_SATR",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Initial degree of saturation"
      },
      {
        "name": "CONG_SPRS",
        "status": "OTHER",
        "type": "2SF",
        "unit": "kPa",
        "description": "Swelling pressure"
      },
      {
        "name": "CONG_SATH",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Height change of specimen on saturation, or flooding as percentage of original height (BS1377 Settlement on saturation test)"
      },
      {
        "name": "CONG_IVR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Initial voids ratio"
      },
      {
        "name": "CONG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks including commentary on effect of specimen disturbance on test result"
      },
      {
        "name": "CONG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "CONG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "CONG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "CONG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviations from the test method"
      },
      {
        "name": "CONG_MCIS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Initial water/moisture content source"
      },
      {
        "name": "CONG_CORR",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Results corrected for equipment deformation"
      }
    ]
  },
  {
    "code": "CONS",
    "contents": "Consolidation Tests - Data",
    "parent": "CONG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "CONS_INCN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Oedometer stress increment"
      },
      {
        "name": "CONS_IVR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Voids ratio at start of increment"
      },
      {
        "name": "CONS_INCF",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Stress at end of stress increment/decrement"
      },
      {
        "name": "CONS_INCE",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Voids ratio at end of stress increment"
      },
      {
        "name": "CONS_INMV",
        "status": "OTHER",
        "type": "2SF",
        "unit": "m2/MN",
        "description": "Reported coefficient of volume compressibility over stress increment"
      },
      {
        "name": "CONS_INSC",
        "status": "OTHER",
        "type": "2SF",
        "unit": null,
        "description": "Coefficient of secondary compression over stress increment"
      },
      {
        "name": "CONS_CVRT",
        "status": "OTHER",
        "type": "2SF",
        "unit": "m2/yr",
        "description": "Coefficient of consolidation over stress increment determined by the root time method"
      },
      {
        "name": "CONS_CVLG",
        "status": "OTHER",
        "type": "2SF",
        "unit": "m2/yr",
        "description": "Coefficient of consolidation over stress increment determined by the log time method"
      },
      {
        "name": "CONS_TEMP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "DegC",
        "description": "Average temperature over stress increment"
      },
      {
        "name": "CONS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "CORE",
    "contents": "Coring Information",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
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
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of core run"
      },
      {
        "name": "CORE_PREC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Percentage of core recovered in core run (TCR)"
      },
      {
        "name": "CORE_SREC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Percentage of solid core recovered in core run (SCR)"
      },
      {
        "name": "CORE_RQD",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Rock Quality Designation for core run (RQD)"
      },
      {
        "name": "CORE_DIAM",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Core diameter"
      },
      {
        "name": "CORE_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm",
        "description": "Time taken to drill core run"
      },
      {
        "name": "CORE_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. photographs of rock cores)"
      }
    ]
  },
  {
    "code": "DCPG",
    "contents": "Dynamic Cone Penetrometer Tests - General",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "DCPG_DATE",
        "status": "KEY",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "DCPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "DCPG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth from surface to start of test"
      },
      {
        "name": "DCPG_ZERO",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Zero reading"
      },
      {
        "name": "DCPG_LREM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of surface and base layers removed prior to/during the test (if applicable)"
      },
      {
        "name": "DCPG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test remarks"
      },
      {
        "name": "DCPG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "DCPG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "DCPG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing organization"
      },
      {
        "name": "DCPG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. field record sheets)"
      },
      {
        "name": "DCPG_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      }
    ]
  },
  {
    "code": "DCPT",
    "contents": "Dynamic Cone Penetrometer Tests - Data",
    "parent": "DCPG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "DCPG_DATE",
        "status": "KEY",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "DCPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "DCPG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth from surface to start of test"
      },
      {
        "name": "DCPT_CBLO",
        "status": "KEY",
        "type": "0DP",
        "unit": null,
        "description": "Cumulative blows"
      },
      {
        "name": "DCPT_PEN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Penetration at DCPT_CBLO"
      },
      {
        "name": "DCPT_DEL",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm",
        "description": "Delay before increment started"
      },
      {
        "name": "DCPT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test reading remarks"
      }
    ]
  },
  {
    "code": "DETL",
    "contents": "Stratum Detail Descriptions",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "DETL_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of detail description"
      },
      {
        "name": "DETL_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of detail description"
      },
      {
        "name": "DETL_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Detail description"
      },
      {
        "name": "DETL_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. logging field sheets)"
      }
    ]
  },
  {
    "code": "DISC",
    "contents": "Discontinuity Data",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "DISC_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top in hole, or distance to start on traverse, of discontinuity zone, or discontinuity"
      },
      {
        "name": "DISC_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base in hole, or distance to end on traverse, of discontinuity zone"
      },
      {
        "name": "FRAC_SET",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Discontinuity set reference"
      },
      {
        "name": "DISC_NUMB",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Discontinuity reference"
      },
      {
        "name": "DISC_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of discontinuity"
      },
      {
        "name": "DISC_DIP",
        "status": "OTHER",
        "type": "X",
        "unit": "deg",
        "description": "Dip of discontinuity"
      },
      {
        "name": "DISC_DIR",
        "status": "OTHER",
        "type": "X",
        "unit": "deg",
        "description": "Dip direction of discontinuity"
      },
      {
        "name": "DISC_RGH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Small scale roughness"
      },
      {
        "name": "DISC_PLAN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Medium scale roughness"
      },
      {
        "name": "DISC_WAVE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "m",
        "description": "Large scale roughness, wavelength"
      },
      {
        "name": "DISC_AMP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "m",
        "description": "Large scale roughness, amplitude"
      },
      {
        "name": "DISC_JRC",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Joint Roughness Coefficient"
      },
      {
        "name": "DISC_APP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Surface appearance"
      },
      {
        "name": "DISC_APT",
        "status": "OTHER",
        "type": "XN",
        "unit": "mm",
        "description": "Discontinuity aperture measurement"
      },
      {
        "name": "DISC_APOB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Discontinuity aperture observation"
      },
      {
        "name": "DISC_INFM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Infilling material"
      },
      {
        "name": "DISC_TERM",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Discontinuity termination (lower, upper)"
      },
      {
        "name": "DISC_PERS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "m",
        "description": "Persistence measurement"
      },
      {
        "name": "DISC_STR",
        "status": "OTHER",
        "type": "0DP",
        "unit": "MPa",
        "description": "Discontinuity wall strength"
      },
      {
        "name": "DISC_WETH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Discontinuity wall weathering"
      },
      {
        "name": "DISC_SEEP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Seepage rating"
      },
      {
        "name": "DISC_FLOW",
        "status": "OTHER",
        "type": "0DP",
        "unit": "l/s",
        "description": "Water flow estimate"
      },
      {
        "name": "DISC_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. logging field sheets)"
      },
      {
        "name": "DISC_MID",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to mid-point in hole, or distance to mid-point on traverse, of discontinuity zone"
      }
    ]
  },
  {
    "code": "DOBS",
    "contents": "Drilling/Advancement Observations & Parameters",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "DOBS_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of reported section"
      },
      {
        "name": "DOBS_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of reported section"
      },
      {
        "name": "DOBS_SET",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Readings set reference"
      },
      {
        "name": "DOBS_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": "Duration to advance reported section"
      },
      {
        "name": "DOBS_STIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Date and time of start of reported section"
      },
      {
        "name": "DOBS_ETIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Date and time at end of reported section"
      },
      {
        "name": "DOBS_DHRT",
        "status": "OTHER",
        "type": "1DP",
        "unit": "Nm",
        "description": "Drill head rotational torque"
      },
      {
        "name": "DOBS_DHRS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "rpm",
        "description": "Drill head rotational speed"
      },
      {
        "name": "DOBS_PENR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "m/hr",
        "description": "Penetration rate"
      },
      {
        "name": "DOBS_HAMM",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Hammering used during section"
      },
      {
        "name": "DOBS_THRP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "bar",
        "description": "Pressure of downthrust system"
      },
      {
        "name": "DOBS_RESP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "bar",
        "description": "Pressure of restraining (holdback) system"
      },
      {
        "name": "DOBS_TORP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "bar",
        "description": "Torque pressure"
      },
      {
        "name": "DOBS_TORQ",
        "status": "OTHER",
        "type": "1DP",
        "unit": "Nm",
        "description": "Torque applied to top of drill rods"
      },
      {
        "name": "DOBS_THST",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kN",
        "description": "Downward thrust on bit"
      },
      {
        "name": "DOBS_REST",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kN",
        "description": "Restraining (holdback) force"
      },
      {
        "name": "DOBS_HAMP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "bar",
        "description": "Supply pressure to downhole hammer"
      },
      {
        "name": "DOBS_SPEN",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MJ/m3",
        "description": "Specific energy"
      },
      {
        "name": "DOBS_FMPO",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Flushing medium pressure at the output of the pump over flush zone"
      },
      {
        "name": "DOBS_FMCR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "l/min",
        "description": "Flushing medium circulation rate (input) over flush zone"
      },
      {
        "name": "DOBS_FMRR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "l/min",
        "description": "Flushing medium recovery rate over flush zone"
      },
      {
        "name": "DOBS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. drilling journals or log files)"
      },
      {
        "name": "DOBS_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Measurement method"
      }
    ]
  },
  {
    "code": "DPRB",
    "contents": "Dynamic Probe Tests - Data",
    "parent": "DPRG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "DPRG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "DPRB_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to start of dynamic probe increment"
      },
      {
        "name": "DPRB_BLOW",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Dynamic probe blows for increment DPRB_INC"
      },
      {
        "name": "DPRB_CBLW",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Cumulative blows for test"
      },
      {
        "name": "DPRB_TORQ",
        "status": "OTHER",
        "type": "0DP",
        "unit": "Nm",
        "description": "Maximum torque required to rotate rods"
      },
      {
        "name": "DPRB_DEL",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm",
        "description": "Delay before increment started"
      },
      {
        "name": "DPRB_INC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Dynamic probe increment"
      },
      {
        "name": "DPRB_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Notes on events during increment"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "DPRG",
    "contents": "Dynamic Probe Tests - General",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "DPRG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "DPRG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "DPRG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Dynamic probe type"
      },
      {
        "name": "DPRG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "DPRG_MASS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kg",
        "description": "Hammer mass"
      },
      {
        "name": "DPRG_DROP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Standard drop"
      },
      {
        "name": "DPRG_CONE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Cone base diameter"
      },
      {
        "name": "DPRG_ROD",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Rod diameter"
      },
      {
        "name": "DPRG_TANV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of anvil"
      },
      {
        "name": "DPRG_DAMP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of anvil damper"
      },
      {
        "name": "DPRG_TIP",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of cone if left in ground"
      },
      {
        "name": "DPRG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "General remarks"
      },
      {
        "name": "DPRG_ANG",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": "Cone angle"
      },
      {
        "name": "DPRG_RMSS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kg/m",
        "description": "Rod mass"
      },
      {
        "name": "DPRG_PARF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Precautions against rod friction"
      },
      {
        "name": "DPRG_PDIU",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Pre-drilling if used"
      },
      {
        "name": "DPRG_BCF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Blow count frequency"
      },
      {
        "name": "DPRG_GW",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Groundwater level"
      },
      {
        "name": "DPRG_REET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Reasons for early end of test"
      },
      {
        "name": "DPRG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "DPRG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing organization"
      },
      {
        "name": "DPRG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "DPRG_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      }
    ]
  },
  {
    "code": "DREM",
    "contents": "Depth Related Remarks",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "DREM_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of remark (DREM_REM)"
      },
      {
        "name": "DREM_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Base depth"
      },
      {
        "name": "DREM_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Depth related remark"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. drilling journals)"
      }
    ]
  },
  {
    "code": "ERES",
    "contents": "Environmental Contaminant Testing",
    "parent": "SAMP",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location Identifier"
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
        "description": "Laboratory specimen reference or Laboratory ID"
      },
      {
        "name": "SPEC_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of test specimen"
      },
      {
        "name": "ERES_CODE",
        "status": "KEY+REQUIRED",
        "type": "PA",
        "unit": null,
        "description": "Chemical code"
      },
      {
        "name": "ERES_METH",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "ERES_MATX",
        "status": "KEY+REQUIRED",
        "type": "PA",
        "unit": null,
        "description": "Laboratory test matrix"
      },
      {
        "name": "ERES_RTYP",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Run type (Initial or Reanalysis)"
      },
      {
        "name": "ERES_TESN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "ERES_NAME",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Chemical name"
      },
      {
        "name": "ERES_TNAM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Laboratory analytical test name"
      },
      {
        "name": "ERES_RVAL",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Result value"
      },
      {
        "name": "ERES_RUNI",
        "status": "REQUIRED",
        "type": "PU",
        "unit": null,
        "description": "Result unit"
      },
      {
        "name": "ERES_RTXT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Reported result"
      },
      {
        "name": "ERES_RTCD",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Result type"
      },
      {
        "name": "ERES_RRES",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Reportable result"
      },
      {
        "name": "ERES_DETF",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Detect flag"
      },
      {
        "name": "ERES_ORG",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Organic"
      },
      {
        "name": "ERES_IQLF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Interpreted qualifiers"
      },
      {
        "name": "ERES_LQLF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Laboratory qualifiers"
      },
      {
        "name": "ERES_RDLM",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Reporting detection limit"
      },
      {
        "name": "ERES_MDLM",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Method detection limit"
      },
      {
        "name": "ERES_QLM",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Quantification limit"
      },
      {
        "name": "ERES_DUNI",
        "status": "OTHER",
        "type": "PU",
        "unit": null,
        "description": "Unit of detection/quantification limits"
      },
      {
        "name": "ERES_TICP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Tentatively Identified Compound (TIC) probability"
      },
      {
        "name": "ERES_TICT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "s",
        "description": "Tentatively Identified Compound (TIC) retention time"
      },
      {
        "name": "ERES_RDAT",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Sample receipt date at laboratory"
      },
      {
        "name": "ERES_SGRP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Sample delivery or batch code"
      },
      {
        "name": "SPEC_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "SPEC_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Specimen description"
      },
      {
        "name": "ERES_DTIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Analysis date and time date"
      },
      {
        "name": "ERES_TEST",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test Name as defined in LBST_TEST during electronic scheduling"
      },
      {
        "name": "ERES_TORD",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Total or dissolved"
      },
      {
        "name": "ERES_LOCN",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Analysis location"
      },
      {
        "name": "ERES_BAS",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Basis"
      },
      {
        "name": "ERES_DIL",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Dilution factor"
      },
      {
        "name": "ERES_LMTH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Leachate preparation method"
      },
      {
        "name": "ERES_LDTM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Leachate preparation date and time"
      },
      {
        "name": "ERES_IREF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Instrument Reference No or Identifier"
      },
      {
        "name": "ERES_SIZE",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Size of material removed prior to test; value given indicates lowest sized material removed"
      },
      {
        "name": "ERES_PERP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Percentage of material removed"
      },
      {
        "name": "ERES_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "ERES_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "ERES_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "ESCG",
    "contents": "Effective Stress Consolidation Tests - General",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "ESCG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Test type"
      },
      {
        "name": "ESCG_CELL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of equipment used"
      },
      {
        "name": "ESCG_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Sample condition"
      },
      {
        "name": "ESCG_SDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Test specimen diameter"
      },
      {
        "name": "ESCG_HIGT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Test specimen height"
      },
      {
        "name": "ESCG_MCI",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Initial water/moisture content"
      },
      {
        "name": "ESCG_MCF",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Final water/moisture content"
      },
      {
        "name": "ESCG_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial bulk density"
      },
      {
        "name": "ESCG_BDEF",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Final bulk density"
      },
      {
        "name": "ESCG_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial dry density"
      },
      {
        "name": "ESCG_PDEN",
        "status": "OTHER",
        "type": "XN",
        "unit": "Mg/m3",
        "description": "Particle density with prefix # if value assumed"
      },
      {
        "name": "ESCG_IVR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Initial voids ratio"
      },
      {
        "name": "ESCG_SATR",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Initial degree of saturation"
      },
      {
        "name": "ESCG_LOAD",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of loading ( strain )"
      },
      {
        "name": "ESCG_DRAG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of drainage"
      },
      {
        "name": "ESCG_PPM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Pore pressure measurement location"
      },
      {
        "name": "ESCG_SPRS",
        "status": "OTHER",
        "type": "2SF",
        "unit": "kPa",
        "description": "Swelling pressure, if measured"
      },
      {
        "name": "ESCG_SATM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of saturation"
      },
      {
        "name": "ESCG_SINC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Saturation increments"
      },
      {
        "name": "ESCG_SDIF",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Differential pressure during saturation"
      },
      {
        "name": "ESCG_CELF",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Cell or diaphragm pressure at end of saturation"
      },
      {
        "name": "ESCG_BACF",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Back pressure at end of saturation"
      },
      {
        "name": "ESCG_BVAL",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "B value at end of saturation"
      },
      {
        "name": "ESCG_SVOL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "ml",
        "description": "Volume of water taken in during saturation"
      },
      {
        "name": "ESCG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks including commentary on effect of specimen disturbance on test result"
      },
      {
        "name": "ESCG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "ESCG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "ESCG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "ESCG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviations from the test method"
      },
      {
        "name": "ESCG_ISVR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Voids ratio at in situ vertical stress"
      },
      {
        "name": "ESCG_ISVS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "In situ vertical effective stress"
      },
      {
        "name": "ESCG_ISST",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Axial strain at in situ vertical effective stress"
      },
      {
        "name": "ESCG_PCP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Preconsolidation stress (yield stress)"
      },
      {
        "name": "ESCG_YSR",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "Yield stress ratio (based on Casagrande Method)"
      },
      {
        "name": "ESCG_CC",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Compression index over stress increment"
      },
      {
        "name": "ESCG_CS",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Swelling index over stress increment"
      }
    ]
  },
  {
    "code": "ESCT",
    "contents": "Effective Stress Consolidation Tests - Data",
    "parent": "ESCG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "ESCT_INCN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Consolidation stage number"
      },
      {
        "name": "ESCT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Additional stage specific details"
      },
      {
        "name": "ESCT_INCC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Cell or diaphragm pressure applied during stage"
      },
      {
        "name": "ESCT_INCB",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Back pressure applied during stage"
      },
      {
        "name": "ESCT_PWP0",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Pore pressure at end of undrained loading"
      },
      {
        "name": "ESCT_PWPF",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Pore pressure at end of consolidation stage"
      },
      {
        "name": "ESCT_INCF",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Effective stress at end of consolidation stage"
      },
      {
        "name": "ESCT_VR0",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Voids ratio at start of increment"
      },
      {
        "name": "ESCT_VRE",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Voids ratio at end of stress increment"
      },
      {
        "name": "ESCT_DISS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Percentage pore pressure dissipation at end of stage"
      },
      {
        "name": "ESCT_DSET",
        "status": "OTHER",
        "type": "3DP",
        "unit": "mm",
        "description": "Settlement measured during consolidation stage"
      },
      {
        "name": "ESCT_DVOL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "ml",
        "description": "Volume change measured during consolidation stage"
      },
      {
        "name": "ESCT_INMV",
        "status": "OTHER",
        "type": "2SF",
        "unit": "m2/MN",
        "description": "Reported coefficient of volume compressibility over stress increment"
      },
      {
        "name": "ESCT_INCV",
        "status": "OTHER",
        "type": "2SF",
        "unit": "m2/yr",
        "description": "Reported coefficient of consolidation over stress increment"
      },
      {
        "name": "ESCT_INSC",
        "status": "OTHER",
        "type": "2SF",
        "unit": null,
        "description": "Coefficient of secondary compression over stress increment"
      },
      {
        "name": "ESCT_CVME",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method used for deriving Cv"
      },
      {
        "name": "ESCT_TEMP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "DegC",
        "description": "Average temperature over stress increment"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "ESCT_INK",
        "status": "OTHER",
        "type": "XN",
        "unit": "m/s",
        "description": "Permeability over stress increment (t90)"
      }
    ]
  },
  {
    "code": "FLSH",
    "contents": "Drilling Flush Details",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "FLSH_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of flush zone"
      },
      {
        "name": "FLSH_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to bottom of flush zone"
      },
      {
        "name": "FLSH_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of flush"
      },
      {
        "name": "FLSH_RETN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Flush return minimum (as percentage)"
      },
      {
        "name": "FLSH_RETX",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Flush return maximum (as percentage)"
      },
      {
        "name": "FLSH_COL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Colour of flush return"
      },
      {
        "name": "FLSH_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. drilling journal, mud logging or test records)"
      }
    ]
  },
  {
    "code": "FRAC",
    "contents": "Fracture Spacing",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "FRAC_FROM",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top in hole, or distance to start on traverse, of the zone"
      },
      {
        "name": "FRAC_TO",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base in hole, or distance to end on traverse, of the zone"
      },
      {
        "name": "FRAC_SET",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Discontinuity set reference"
      },
      {
        "name": "FRAC_IMAX",
        "status": "OTHER",
        "type": "XN",
        "unit": "mm",
        "description": "Maximum fracture spacing over zone"
      },
      {
        "name": "FRAC_IAVE",
        "status": "OTHER",
        "type": "XN",
        "unit": "mm",
        "description": "Average fracture (modal) spacing over zone"
      },
      {
        "name": "FRAC_IMIN",
        "status": "OTHER",
        "type": "XN",
        "unit": "mm",
        "description": "Minimum fracture spacing over zone"
      },
      {
        "name": "FRAC_FI",
        "status": "OTHER",
        "type": "XN",
        "unit": null,
        "description": "Fracture Index / frequency over zone (fractures per metre)"
      },
      {
        "name": "FRAC_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Comments on fracture set"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. logging field sheets)"
      }
    ]
  },
  {
    "code": "FRST",
    "contents": "Frost Susceptibility Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "FRST_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Sample condition"
      },
      {
        "name": "FRST_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Dry density of specimens after preparation"
      },
      {
        "name": "FRST_MC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "water/moisture content of specimens at preparation"
      },
      {
        "name": "FRST_HVE1",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Frost heave, first specimen"
      },
      {
        "name": "FRST_HVE2",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Frost heave, second specimen"
      },
      {
        "name": "FRST_HVE3",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Frost heave, third specimen"
      },
      {
        "name": "FRST_HVE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Mean heave of 3 specimens"
      },
      {
        "name": "FRST_STAB",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Amount of stabiliser added"
      },
      {
        "name": "FRST_STYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of stabiliser added"
      },
      {
        "name": "FRST_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Notes on frost susceptibility testing as per TRRL SR 829"
      },
      {
        "name": "FRST_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "FRST_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "FRST_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "FRST_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviations from the test method"
      }
    ]
  },
  {
    "code": "GCHM",
    "contents": "Geotechnical Chemistry Testing",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "GCHM_CODE",
        "status": "KEY+REQUIRED",
        "type": "PA",
        "unit": null,
        "description": "Determinand"
      },
      {
        "name": "GCHM_METH",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "GCHM_TTYP",
        "status": "KEY+REQUIRED",
        "type": "PA",
        "unit": null,
        "description": "Test type"
      },
      {
        "name": "GCHM_RESL",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Test result"
      },
      {
        "name": "GCHM_UNIT",
        "status": "REQUIRED",
        "type": "PU",
        "unit": null,
        "description": "Test result units"
      },
      {
        "name": "GCHM_NAME",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Client/laboratory preferred name of determinand"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "GCHM_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks on test"
      },
      {
        "name": "GCHM_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "GCHM_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "GCHM_RTXT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Reported result"
      },
      {
        "name": "GCHM_DLM",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Lower detection limit"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "GCHM_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviations from the test method"
      },
      {
        "name": "GCHM_SGRP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Sample delivery or batch code"
      },
      {
        "name": "GCHM_LSID",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Laboratory sample ID"
      },
      {
        "name": "GCHM_RDAT",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Sample receipt date/time at laboratory"
      },
      {
        "name": "GCHM_DTIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Analysis date and time"
      },
      {
        "name": "GCHM_TEST",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test of Suite name"
      },
      {
        "name": "GCHM_IREF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Instrument reference no or identifier"
      },
      {
        "name": "GCHM_ITYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Instrument type"
      },
      {
        "name": "GCHM_SIZE",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Size of material removed prior to test; value given indicates lowest sized material removed"
      },
      {
        "name": "GCHM_PERP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Percentage of material removed"
      },
      {
        "name": "GCHM_RDEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Result deviation description(s)"
      }
    ]
  },
  {
    "code": "GEOL",
    "contents": "Field Geological Descriptions",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "GEOL_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to the top of stratum"
      },
      {
        "name": "GEOL_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to the base of description"
      },
      {
        "name": "GEOL_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "General description of stratum"
      },
      {
        "name": "GEOL_LEG",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Legend code"
      },
      {
        "name": "GEOL_GEOL",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Geology code"
      },
      {
        "name": "GEOL_GEO2",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Second geology code"
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stratum reference shown on trial pit or traverse sketch"
      },
      {
        "name": "GEOL_BGS",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "BGS Lexicon code"
      },
      {
        "name": "GEOL_FORM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Geological unit or stratum name"
      },
      {
        "name": "GEOL_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. logging field sheets, photographs of exposures)"
      },
      {
        "name": "GEOL_BNDF",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Geology base boundary definition"
      }
    ]
  },
  {
    "code": "GRAG",
    "contents": "Particle Size Distribution Analysis - General",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "GRAG_UC",
        "status": "OTHER",
        "type": "1SF",
        "unit": null,
        "description": "Uniformity coefficient D60/D10"
      },
      {
        "name": "GRAG_VCRE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Percentage of material tested greater than 63mm (cobbles)"
      },
      {
        "name": "GRAG_GRAV",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Percentage of material tested in range 63mm to 2mm (gravel)"
      },
      {
        "name": "GRAG_SAND",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Percentage of material tested in range 2mm to 63um (sand)"
      },
      {
        "name": "GRAG_SILT",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Percentage of material tested in range 63um to 2um (silt)"
      },
      {
        "name": "GRAG_CLAY",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Percentage of material tested less than 2um (clay)"
      },
      {
        "name": "GRAG_FINE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Percentage less than 63um"
      },
      {
        "name": "GRAG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks including commentary on effect of specimen disturbance on test result"
      },
      {
        "name": "GRAG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "GRAG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "GRAG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "GRAG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Any deviation from the specified test procedure, and any other information that could be important for interpreting the test results."
      },
      {
        "name": "GRAG_PDEN",
        "status": "OTHER",
        "type": "XN",
        "unit": "Mg/m3",
        "description": "Particle density used in calculations with prefix # if value assumed"
      },
      {
        "name": "GRAG_PRET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of pre-treatment, when applied"
      },
      {
        "name": "GRAG_SUFF",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Amount of soil tested was sufficient to comply with recommended minimum mass"
      },
      {
        "name": "GRAG_EXCL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remark if the size of the fractions is not expressed as percentage of total dry mass, together with the nature and amount of fractions excluded."
      },
      {
        "name": "GRAG_CC",
        "status": "OTHER",
        "type": "1SF",
        "unit": null,
        "description": "Coefficient of curvature"
      }
    ]
  },
  {
    "code": "GRAT",
    "contents": "Particle Size Distribution Analysis - Data",
    "parent": "GRAG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "GRAT_SIZE",
        "status": "KEY",
        "type": "3SF",
        "unit": "mm",
        "description": "Sieve or particle size"
      },
      {
        "name": "GRAT_PERP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Percentage passing/finer than GRAT_SIZE"
      },
      {
        "name": "GRAT_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Test type"
      },
      {
        "name": "GRAT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "HDIA",
    "contents": "Hole Diameter by Depth",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "HDIA_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of base of hole at the diameter recorded in HDIA_DIAM"
      },
      {
        "name": "HDIA_DIAM",
        "status": "KEY",
        "type": "0DP",
        "unit": "mm",
        "description": "Hole diameter"
      },
      {
        "name": "HDIA_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. drilling journals)"
      }
    ]
  },
  {
    "code": "HDPH",
    "contents": "Depth Related Exploratory Hole Construction Information",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "HDPH_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of section"
      },
      {
        "name": "HDPH_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of section"
      },
      {
        "name": "HDPH_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Type of depth related information"
      },
      {
        "name": "HDPH_STAR",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Date and time of start of section"
      },
      {
        "name": "HDPH_ENDD",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Date and time of end of section"
      },
      {
        "name": "HDPH_CREW",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of rig/drill crew/operator"
      },
      {
        "name": "HDPH_EXC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Plant used"
      },
      {
        "name": "HDPH_SHOR",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Shoring/support used"
      },
      {
        "name": "HDPH_STAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stability of trial pit / trial trench or logged traverse length"
      },
      {
        "name": "HDPH_DIML",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Trial pit / trial trench or logged traverse length"
      },
      {
        "name": "HDPH_DIMW",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Trial pit / trial trench or logged traverse width"
      },
      {
        "name": "HDPH_DBIT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Drill bit used"
      },
      {
        "name": "HDPH_BCON",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Bit condition"
      },
      {
        "name": "HDPH_BTYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Barrel type"
      },
      {
        "name": "HDPH_BLEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Barrel length"
      },
      {
        "name": "HDPH_LOG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Definitive person responsible for logging the section"
      },
      {
        "name": "HDPH_LOGD",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Start date of hole section logging"
      },
      {
        "name": "HDPH_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "HDPH_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during hole section construction"
      },
      {
        "name": "HDPH_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of method of hole section construction"
      },
      {
        "name": "HDPH_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Drilling contractor"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. drilling journals, hole orientation data)"
      }
    ]
  },
  {
    "code": "HORN",
    "contents": "Exploratory Hole Orientation and Inclination",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "HORN_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of exploratory hole section"
      },
      {
        "name": "HORN_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of exploratory hole section"
      },
      {
        "name": "HORN_ORNT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": "Orientation of exploratory hole section or traverse (degrees from north)"
      },
      {
        "name": "HORN_INCL",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": "Inclination of exploratory hole section or traverse (measured positively down from horizontal)"
      },
      {
        "name": "HORN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks relating to orientation and inclination of hole section"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. contract data specification)"
      }
    ]
  },
  {
    "code": "ICBR",
    "contents": "In Situ California Bearing Ratio Tests",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "ICBR_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of CBR test"
      },
      {
        "name": "ICBR_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "ICBR_ICBR",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "CBR value"
      },
      {
        "name": "ICBR_MC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water/moisture content relating to test"
      },
      {
        "name": "ICBR_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "ICBR_KENT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of kentledge (reaction load)"
      },
      {
        "name": "ICBR_SEAT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "N",
        "description": "Seating force"
      },
      {
        "name": "ICBR_SURC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Surcharge pressure"
      },
      {
        "name": "ICBR_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of CBR"
      },
      {
        "name": "ICBR_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "ICBR_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "ICBR_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "ICBR_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing organization"
      },
      {
        "name": "ICBR_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test status"
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stratum reference shown on trial pit or traverse sketch"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "ICBR_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      },
      {
        "name": "ICBR_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of derived CBR layer (DCP)"
      }
    ]
  },
  {
    "code": "IDEN",
    "contents": "In Situ Density Tests",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "IDEN_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of in situ density test"
      },
      {
        "name": "IDEN_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "IDEN_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "IDEN_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of density test performed"
      },
      {
        "name": "IDEN_IDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "In situ bulk density (after any calibration / corrections applied, i.e. reported value)"
      },
      {
        "name": "IDEN_MC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water/moisture content relating to in situ test (after any calibration / corrections applied, i.e. reported value)"
      },
      {
        "name": "IDEN_STAB",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Amount of stabiliser added"
      },
      {
        "name": "IDEN_STYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of stabiliser added"
      },
      {
        "name": "IDEN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "IDEN_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "IDEN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "IDEN_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing organization"
      },
      {
        "name": "IDEN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test status"
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stratum reference shown on trial pit or traverse sketch"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "IDEN_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      }
    ]
  },
  {
    "code": "IFID",
    "contents": "On Site Volatile Headspace Testing Using Flame Ionisation Detector",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "IFID_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of headspace test sample"
      },
      {
        "name": "IFID_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "IFID_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "IFID_RES",
        "status": "OTHER",
        "type": "XN",
        "unit": "ppmv",
        "description": "Result of FID analysis"
      },
      {
        "name": "IFID_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks on test"
      },
      {
        "name": "IFID_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "IFID_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of FID used and method description"
      },
      {
        "name": "IFID_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing organization"
      },
      {
        "name": "IFID_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test status"
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stratum reference shown on trial pit or traverse sketch"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "IFID_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      }
    ]
  },
  {
    "code": "IPEN",
    "contents": "In Situ Hand Penetrometer Tests",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "IPEN_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of test"
      },
      {
        "name": "IPEN_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "IPEN_IPEN",
        "status": "OTHER",
        "type": "XN",
        "unit": "kPa",
        "description": "Hand penetrometer result"
      },
      {
        "name": "IPEN_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "IPEN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks on test"
      },
      {
        "name": "IPEN_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "IPEN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "IPEN_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing organization"
      },
      {
        "name": "IPEN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test status"
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stratum reference shown on trial pit or traverse sketch"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "IPEN_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      }
    ]
  },
  {
    "code": "IPID",
    "contents": "On Site Volatile Headspace Testing by Photo Ionisation Detector",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "IPID_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of headspace test sample"
      },
      {
        "name": "IPID_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "IPID_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "IPID_TEMP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "DegC",
        "description": "Ambient temperature at time of test"
      },
      {
        "name": "IPID_RES",
        "status": "OTHER",
        "type": "XN",
        "unit": "ppmv",
        "description": "Result of PID analysis"
      },
      {
        "name": "IPID_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks on test"
      },
      {
        "name": "IPID_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "IPID_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of PID used and method description"
      },
      {
        "name": "IPID_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing organization"
      },
      {
        "name": "IPID_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test status"
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stratum reference shown on trial pit or traverse sketch"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "IPID_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      }
    ]
  },
  {
    "code": "IPRG",
    "contents": "In Situ Permeability Tests - General",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "IPRG_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of test zone"
      },
      {
        "name": "IPRG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "IPRG_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of test zone"
      },
      {
        "name": "IPRG_STG",
        "status": "KEY",
        "type": "0DP",
        "unit": null,
        "description": "Stage number of multistage test"
      },
      {
        "name": "IPRG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of test"
      },
      {
        "name": "IPRG_PRWL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to water in test zone immediately prior to test"
      },
      {
        "name": "IPRG_SWAL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to water at start of test"
      },
      {
        "name": "IPRG_TDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Diameter of test zone"
      },
      {
        "name": "IPRG_SDIA",
        "status": "OTHER",
        "type": "3DP",
        "unit": "m",
        "description": "Diameter of test installation (e.g. standpipe or casing)"
      },
      {
        "name": "IPRG_IPRM",
        "status": "OTHER",
        "type": "1SCI",
        "unit": "m/s",
        "description": "Permeability"
      },
      {
        "name": "IPRG_FLOW",
        "status": "OTHER",
        "type": "1DP",
        "unit": "l/s",
        "description": "Average flow during test stage"
      },
      {
        "name": "IPRG_AWL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to assumed standing water level"
      },
      {
        "name": "IPRG_HEAD",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Applied total head of water during test stage at centre of test zone"
      },
      {
        "name": "IPRG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "IPRG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test remarks"
      },
      {
        "name": "IPRG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "IPRG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "IPRG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing organization"
      },
      {
        "name": "IPRG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      }
    ]
  },
  {
    "code": "IPRT",
    "contents": "In Situ Permeability Tests - Data",
    "parent": "IPRG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "IPRG_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of test zone"
      },
      {
        "name": "IPRG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "IPRG_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of test zone"
      },
      {
        "name": "IPRG_STG",
        "status": "KEY",
        "type": "0DP",
        "unit": null,
        "description": "Stage number of multistage packer test"
      },
      {
        "name": "IPRT_TIME",
        "status": "KEY",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": "Elapsed time"
      },
      {
        "name": "IPRT_DPTH",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to water at time IPRT_TIME"
      },
      {
        "name": "IPRT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test reading remark"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "IRDX",
    "contents": "In Situ Redox Tests",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "IRDX_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of redox test"
      },
      {
        "name": "IRDX_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "IRDX_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "IRDX_PH",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "pH"
      },
      {
        "name": "IRDX_MPOT",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Mean value of the potential of the two platinum probes"
      },
      {
        "name": "IRDX_IRDX",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mV",
        "description": "Redox potential"
      },
      {
        "name": "IRDX_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of redox test and probe type"
      },
      {
        "name": "IRDX_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "IRDX_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "IRDX_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing organization"
      },
      {
        "name": "IRDX_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test status"
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stratum reference shown on trial pit or traverse sketch"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "IRDX_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      }
    ]
  },
  {
    "code": "IRES",
    "contents": "In Situ Resistivity Tests",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "IRES_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to which in situ resistivity test relates"
      },
      {
        "name": "IRES_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "IRES_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Base depth to which in-situ resistivity test relates"
      },
      {
        "name": "IRES_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of resistivity test"
      },
      {
        "name": "IRES_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "IRES_IRES",
        "status": "OTHER",
        "type": "2SF",
        "unit": "ohm m",
        "description": "Mean value of the apparent resistivity"
      },
      {
        "name": "IRES_RES1",
        "status": "OTHER",
        "type": "2SF",
        "unit": "ohm m",
        "description": "First value of apparent resistivity when more than 15% different to mean"
      },
      {
        "name": "IRES_RES2",
        "status": "OTHER",
        "type": "2SF",
        "unit": "ohm m",
        "description": "Second value of apparent resistivity when more than 15% different to mean"
      },
      {
        "name": "IRES_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of test e.g. electrode spacing and configuration"
      },
      {
        "name": "IRES_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "IRES_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "IRES_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing organization"
      },
      {
        "name": "IRES_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test status"
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stratum reference shown on trial pit or traverse sketch"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "IRES_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      }
    ]
  },
  {
    "code": "ISAG",
    "contents": "Soakaway Tests - General",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "ISAG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "ISAG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "ISAG_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm",
        "description": "Test duration"
      },
      {
        "name": "ISAG_PWID",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Soakaway pit width"
      },
      {
        "name": "ISAG_PLEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Soakaway pit length"
      },
      {
        "name": "ISAG_PDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Soakaway pit diameter"
      },
      {
        "name": "ISAG_DPTS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Soakaway pit depth at start of test"
      },
      {
        "name": "ISAG_DPTE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Soakaway pit depth at end of test"
      },
      {
        "name": "ISAG_CONS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Description of soakaway construction"
      },
      {
        "name": "ISAG_SI",
        "status": "OTHER",
        "type": "2SCI",
        "unit": "m/s",
        "description": "Soil infiltration rate"
      },
      {
        "name": "ISAG_PORO",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Fill porosity"
      },
      {
        "name": "ISAG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "ISAG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "ISAG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "ISAG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing organization"
      },
      {
        "name": "ISAG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "ISAG_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of operator carrying out test"
      }
    ]
  },
  {
    "code": "ISAT",
    "contents": "Soakaway Tests - Data",
    "parent": "ISAG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "ISAG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "ISAT_TIME",
        "status": "KEY",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": "Elapsed time"
      },
      {
        "name": "ISAT_DPTH",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to water"
      },
      {
        "name": "ISAT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remark relating to test reading"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "ISPT",
    "contents": "Standard Penetration Test Results",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "ISPT_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of test"
      },
      {
        "name": "ISPT_SEAT",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Number of blows for seating drive"
      },
      {
        "name": "ISPT_MAIN",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Number of blows for main test drive"
      },
      {
        "name": "ISPT_NPEN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Total penetration for seating drive and test drive"
      },
      {
        "name": "ISPT_NVAL",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "SPT 'N' value"
      },
      {
        "name": "ISPT_REP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "SPT reported result"
      },
      {
        "name": "ISPT_CAS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Casing depth at time of test"
      },
      {
        "name": "ISPT_WAT",
        "status": "OTHER",
        "type": "XN",
        "unit": "m",
        "description": "Depth to water at time of test"
      },
      {
        "name": "ISPT_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of SPT test"
      },
      {
        "name": "ISPT_HAM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Hammer serial number from manufacturer"
      },
      {
        "name": "ISPT_ERAT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Energy ratio of the hammer"
      },
      {
        "name": "ISPT_SWP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Self-weight penetration"
      },
      {
        "name": "ISPT_INC1",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Number of blows for 1st Increment (Seating)"
      },
      {
        "name": "ISPT_INC2",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Number of blows for 2nd Increment (Seating)"
      },
      {
        "name": "ISPT_INC3",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Number of blows for 1st Increment (Test)"
      },
      {
        "name": "ISPT_INC4",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Number of blows for 2nd Increment (Test)"
      },
      {
        "name": "ISPT_INC5",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Number of blows for 3rd Increment (Test)"
      },
      {
        "name": "ISPT_INC6",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Number of blows for 4th Increment (Test)"
      },
      {
        "name": "ISPT_PEN1",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Penetration for 1st Increment (Seating Drive)"
      },
      {
        "name": "ISPT_PEN2",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Penetration for 2nd Increment (Seating Drive)"
      },
      {
        "name": "ISPT_PEN3",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Penetration for 1st Increment (Test)"
      },
      {
        "name": "ISPT_PEN4",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Penetration for 2nd Increment (Test)"
      },
      {
        "name": "ISPT_PEN5",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Penetration for 3rd Increment (Test)"
      },
      {
        "name": "ISPT_PEN6",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Penetration for 4th Increment (Test)"
      },
      {
        "name": "ISPT_ROCK",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "SPT carried out in soft rock"
      },
      {
        "name": "ISPT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "ISPT_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "ISPT_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "ISPT_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "ISPT_N60",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "SPT 'N' value (corrected by energy ratio ISPT_ERAT)"
      }
    ]
  },
  {
    "code": "IVAN",
    "contents": "In Situ Vane Tests",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "IVAN_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of vane test"
      },
      {
        "name": "IVAN_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "IVAN_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Vane type"
      },
      {
        "name": "IVAN_IVAN",
        "status": "OTHER",
        "type": "XN",
        "unit": "kPa",
        "description": "Vane test result"
      },
      {
        "name": "IVAN_IVAR",
        "status": "OTHER",
        "type": "XN",
        "unit": "kPa",
        "description": "Vane test remoulded result (residual)"
      },
      {
        "name": "IVAN_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "IVAN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of vane test, vane size"
      },
      {
        "name": "IVAN_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "IVAN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "IVAN_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing organization"
      },
      {
        "name": "IVAN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test status"
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stratum reference shown on trial pit or traverse sketch"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "IVAN_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      }
    ]
  },
  {
    "code": "LBSG",
    "contents": "Testing Schedule",
    "parent": null,
    "headings": [
      {
        "name": "LBSG_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Schedule reference"
      },
      {
        "name": "LBSG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date of issue"
      },
      {
        "name": "LBSG_FROM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Schedule prepared by"
      },
      {
        "name": "LBSG_TO",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Schedule issued to"
      },
      {
        "name": "LBSG_DUE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date schedule to be completed and reported"
      },
      {
        "name": "LBSG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Comments on schedule"
      },
      {
        "name": "LBSG_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Status of schedule"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. schedule sheets)"
      }
    ]
  },
  {
    "code": "LBST",
    "contents": "Testing Schedule Details",
    "parent": "LBSG",
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
        "name": "LBSG_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Testing schedule reference"
      },
      {
        "name": "LBST_TEST",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test Name"
      },
      {
        "name": "CHOC_REF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Chain of custody reference"
      },
      {
        "name": "LBST_TTYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Full test method or standard"
      },
      {
        "name": "LBST_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method and test parameters"
      },
      {
        "name": "LBST_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Preparation requirements"
      },
      {
        "name": "LBST_DEPN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Dependent test options"
      },
      {
        "name": "LBST_STAT",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Status of laboratory test"
      },
      {
        "name": "LBST_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "LBST_DUE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test results due date"
      },
      {
        "name": "LBST_DETL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of testing carried out or reasons for no testing possible"
      },
      {
        "name": "LBST_DONE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date test completed"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "LDEN",
    "contents": "Density Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "LDEN_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of test performed"
      },
      {
        "name": "LDEN_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Sample condition"
      },
      {
        "name": "LDEN_SMTY",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of sample"
      },
      {
        "name": "LDEN_MC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water/moisture content"
      },
      {
        "name": "LDEN_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Bulk density"
      },
      {
        "name": "LDEN_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Dry density"
      },
      {
        "name": "LDEN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "LDEN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "LDEN_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "LDEN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "LDEN_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Specimen size if less than 50cm3 and any deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "LDYN",
    "contents": "Dynamic Testing",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "LDYN_PWAV",
        "status": "OTHER",
        "type": "0DP",
        "unit": "m/s",
        "description": "P-wave velocity"
      },
      {
        "name": "LDYN_SWAV",
        "status": "OTHER",
        "type": "0DP",
        "unit": "m/s",
        "description": "S-wave velocity"
      },
      {
        "name": "LDYN_EMOD",
        "status": "OTHER",
        "type": "0DP",
        "unit": "GPa",
        "description": "Dynamic elastic modulus"
      },
      {
        "name": "LDYN_SG",
        "status": "OTHER",
        "type": "0DP",
        "unit": "GPa",
        "description": "Shear modulus derived from LDYN_SWAV"
      },
      {
        "name": "LDYN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "LDYN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "LDYN_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "LDYN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "LDYN_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "LLIN",
    "contents": "Linear Shrinkage Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "LLIN_LS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Linear shrinkage"
      },
      {
        "name": "LLIN_425",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Percentage passing 0.425mm sieve"
      },
      {
        "name": "LLIN_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of preparation"
      },
      {
        "name": "LLIN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "LLIN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "LLIN_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "LLIN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "LLIN_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "LLPL",
    "contents": "Liquid and Plastic Limit Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
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
        "type": "XN",
        "unit": "%",
        "description": "Plastic limit"
      },
      {
        "name": "LLPL_PI",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Plasticity Index"
      },
      {
        "name": "LLPL_425",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Percentage passing 0.425mm sieve"
      },
      {
        "name": "LLPL_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of preparation"
      },
      {
        "name": "LLPL_STAB",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Amount of stabiliser added"
      },
      {
        "name": "LLPL_STYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of stabiliser added"
      },
      {
        "name": "LLPL_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "LLPL_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "LLPL_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "LLPL_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "LLPL_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      },
      {
        "name": "LLPL_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of test"
      },
      {
        "name": "LLPL_POIN",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Number of points"
      },
      {
        "name": "LLPL_CONE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "For fall cone method, type of cone"
      },
      {
        "name": "LLPL_1PRE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Mean of test readings, if one-point test."
      },
      {
        "name": "LLPL_1PCF",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Correlation factor if one-point test."
      },
      {
        "name": "LLPL_SIZE",
        "status": "OTHER",
        "type": "U",
        "unit": "mm",
        "description": "Sieve size if other than 0.425mm"
      },
      {
        "name": "LLPL_PASS",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Percentage passing LLPL_SIZE sieve if other than 0.425mm"
      },
      {
        "name": "LLPL_WC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "The water content of the specimen before removal of particles prior to determination liquid or plastic limits, if measured"
      }
    ]
  },
  {
    "code": "LNMC",
    "contents": "Water/moisture Content Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "LNMC_MC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water/moisture content"
      },
      {
        "name": "LNMC_TEMP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "DegC",
        "description": "Temperature sample dried at"
      },
      {
        "name": "LNMC_STAB",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Amount of stabiliser added"
      },
      {
        "name": "LNMC_STYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of stabiliser added"
      },
      {
        "name": "LNMC_ISNT",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Is test result assumed to be a natural water/moisture content"
      },
      {
        "name": "LNMC_COMM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Reason water/moisture content is assumed to be other than natural"
      },
      {
        "name": "LNMC_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "LNMC_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "LNMC_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "LNMC_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "LNMC_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "LOCA",
    "contents": "Location Details",
    "parent": "PROJ",
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
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of activity"
      },
      {
        "name": "LOCA_STAT",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Status of information relating to this position"
      },
      {
        "name": "LOCA_NATE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "National Grid Easting of location or start of traverse"
      },
      {
        "name": "LOCA_NATN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "National Grid Northing of location or start of traverse"
      },
      {
        "name": "LOCA_GREF",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "National grid referencing system used"
      },
      {
        "name": "LOCA_GL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Ground level relative to datum of location or start of traverse"
      },
      {
        "name": "LOCA_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "General remarks"
      },
      {
        "name": "LOCA_FDEP",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Final depth"
      },
      {
        "name": "LOCA_STAR",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date of start of activity"
      },
      {
        "name": "LOCA_PURP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Purpose of activity at this location"
      },
      {
        "name": "LOCA_TERM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Reason for activity termination"
      },
      {
        "name": "LOCA_ENDD",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "End date of activity"
      },
      {
        "name": "LOCA_LETT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "OSGB letter grid reference"
      },
      {
        "name": "LOCA_LOCX",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Local grid x co-ordinate or start of traverse"
      },
      {
        "name": "LOCA_LOCY",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Local grid y co-ordinate or start of traverse"
      },
      {
        "name": "LOCA_LOCZ",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Level or start of traverse to local datum"
      },
      {
        "name": "LOCA_LREF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Local grid referencing system used"
      },
      {
        "name": "LOCA_DATM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Local vertical datum referencing system used"
      },
      {
        "name": "LOCA_ETRV",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "National Grid Easting of end of traverse"
      },
      {
        "name": "LOCA_NTRV",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "National Grid Northing of end of traverse"
      },
      {
        "name": "LOCA_LTRV",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Ground level relative to datum of end of traverse"
      },
      {
        "name": "LOCA_XTRL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Local grid easting of end of traverse"
      },
      {
        "name": "LOCA_YTRL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Local grid northing of end of traverse"
      },
      {
        "name": "LOCA_ZTRL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Local elevation of end of traverse"
      },
      {
        "name": "LOCA_LAT",
        "status": "OTHER",
        "type": "DMS",
        "unit": null,
        "description": "Latitude of location or start of traverse"
      },
      {
        "name": "LOCA_LON",
        "status": "OTHER",
        "type": "DMS",
        "unit": null,
        "description": "Longitude of location or start of traverse"
      },
      {
        "name": "LOCA_ELAT",
        "status": "OTHER",
        "type": "DMS",
        "unit": null,
        "description": "Latitude of end of traverse"
      },
      {
        "name": "LOCA_ELON",
        "status": "OTHER",
        "type": "DMS",
        "unit": null,
        "description": "Longitude of end of traverse"
      },
      {
        "name": "LOCA_LLZ",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Geodetic datum"
      },
      {
        "name": "LOCA_LOCM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of location"
      },
      {
        "name": "LOCA_LOCA",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Site location sub division (within project) code or description"
      },
      {
        "name": "LOCA_CLST",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Investigation phase grouping code or description"
      },
      {
        "name": "LOCA_ALID",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Alignment Identifier"
      },
      {
        "name": "LOCA_OFFS",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Offset"
      },
      {
        "name": "LOCA_CNGE",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Chainage"
      },
      {
        "name": "LOCA_TRAN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Reference to or details of algorithm used to calculate local grid reference, local ground levels or chainage"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. boring or pitting instructions, location photographs)"
      },
      {
        "name": "LOCA_NATD",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "National vertical datum referencing system used"
      },
      {
        "name": "LOCA_ORID",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Original Hole ID"
      },
      {
        "name": "LOCA_ORJO",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Original Job Reference"
      },
      {
        "name": "LOCA_ORCO",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Originating Company"
      },
      {
        "name": "LOCA_GLDT",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Date time of LOCA_GL measurement"
      },
      {
        "name": "LOCA_VSSL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Survey vessel"
      },
      {
        "name": "LOCA_NSRI",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Spatial reference identifier for national grid referencing system used (EPSG code)"
      },
      {
        "name": "LOCA_LSRI",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Spatial reference identifier for local grid referencing system used (EPSG code)"
      },
      {
        "name": "LOCA_LLSI",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Spatial reference identifier for latitude and longitude referencing system used (EPSG code)"
      }
    ]
  },
  {
    "code": "LPDN",
    "contents": "Particle Density Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "LPDN_PDEN",
        "status": "OTHER",
        "type": "XN",
        "unit": "Mg/m3",
        "description": "Particle density with prefix # if value assumed"
      },
      {
        "name": "LPDN_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of test"
      },
      {
        "name": "LPDN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "LPDN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "LPDN_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "LPDN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "LPDN_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Any deviation from the specified test procedure, and any other information that could be important for interpreting the test results."
      },
      {
        "name": "LPDN_PVOL",
        "status": "OTHER",
        "type": "0DP",
        "unit": "ml",
        "description": "Pycnometer volume if used and not 50ml"
      },
      {
        "name": "LPDN_GAS",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Identity of gas if gas pycnometer used"
      }
    ]
  },
  {
    "code": "LPEN",
    "contents": "Laboratory Hand Penetrometer Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "LPEN_PPEN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Hand penetrometer undrained shear strength"
      },
      {
        "name": "LPEN_MC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water/moisture content local to test, if measured"
      },
      {
        "name": "LPEN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "LPEN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "LPEN_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "LPEN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "LPEN_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "LRES",
    "contents": "Laboratory Resistivity Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "LRES_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Bulk density"
      },
      {
        "name": "LRES_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Dry density"
      },
      {
        "name": "LRES_MC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water/moisture content"
      },
      {
        "name": "LRES_COND",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Sample condition including details of remoulding"
      },
      {
        "name": "LRES_LRES",
        "status": "OTHER",
        "type": "0DP",
        "unit": "ohm m",
        "description": "Temperature corrected (20 DegC) resistivity"
      },
      {
        "name": "LRES_CDIA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Diameter of container"
      },
      {
        "name": "LRES_CCSA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm2",
        "description": "Container cross-sectional area"
      },
      {
        "name": "LRES_CLEN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Length of container"
      },
      {
        "name": "LRES_TEMP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "DegC",
        "description": "Temperature at which test performed"
      },
      {
        "name": "LRES_ELEC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of electrodes including material"
      },
      {
        "name": "LRES_PENT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Dimensions of probes, diameter, spacing, penetration into the soil specimen and whether inserted into ends or side"
      },
      {
        "name": "LRES_CSHP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Shape of container"
      },
      {
        "name": "LRES_WAT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "ml",
        "description": "Volume of water required to saturate the soil"
      },
      {
        "name": "LRES_WRES",
        "status": "OTHER",
        "type": "3SF",
        "unit": "ohm m",
        "description": "Water resistivity"
      },
      {
        "name": "LRES_PART",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Approximate percentage of large particles removed prior to test"
      },
      {
        "name": "LRES_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "LRES_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "LRES_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "LRES_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "LRES_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "LSLT",
    "contents": "Shrinkage Limit Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "LSLT_SLIM",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Shrinkage limit"
      },
      {
        "name": "LSLT_SHRA",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Shrinkage ratio"
      },
      {
        "name": "LSLT_IDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial density"
      },
      {
        "name": "LSLT_MCI",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Initial water/moisture content of test specimen"
      },
      {
        "name": "LSLT_425",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Percentage passing 0.425mm sieve"
      },
      {
        "name": "LSLT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "LSLT_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "LSLT_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "LSLT_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "LSLT_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "LSTG",
    "contents": "Initial Consumption of Lime Tests - General",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "LSTG_ICL",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Initial consumption of lime"
      },
      {
        "name": "LSTG_PH",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "pH value used for interpretation of LSTG_ICL"
      },
      {
        "name": "LSTG_LIME",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of lime used for test"
      },
      {
        "name": "LSTG_SUIT",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "pH of saturated lime solution (suitability)"
      },
      {
        "name": "LSTG_425",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Percentage passing 0.425mm sieve"
      },
      {
        "name": "LSTG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks including commentary on effect of specimen disturbance on test result"
      },
      {
        "name": "LSTG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "LSTG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "LSTG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "LSTG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "LSTT",
    "contents": "Initial Consumption of Lime Tests - Data",
    "parent": "LSTG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "LSTT_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "LSTT_LCON",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Percentage of lime added"
      },
      {
        "name": "LSTT_PH",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "pH of lime/soil suspension"
      },
      {
        "name": "LSTT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "LSWL",
    "contents": "Swelling Index Testing",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "LSWL_SWPR",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Swelling Pressure Index"
      },
      {
        "name": "LSWL_SWSI",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Swelling Strain Index"
      },
      {
        "name": "LSWL_MCI",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Initial water content of test specimen"
      },
      {
        "name": "LSWL_SDIA",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Specimen diameter"
      },
      {
        "name": "LSWL_THCK",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Specimen thickness"
      },
      {
        "name": "LSWL_BDEN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kg/m3",
        "description": "Initial bulk density"
      },
      {
        "name": "LSWL_DDEN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kg/m3",
        "description": "Initial dry density"
      },
      {
        "name": "LSWL_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "LSWL_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "LSWL_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "LSWL_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "LSWL_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "LVAN",
    "contents": "Laboratory Vane Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "LVAN_VNPK",
        "status": "OTHER",
        "type": "XN",
        "unit": "kPa",
        "description": "Vane undrained shear strength (peak)"
      },
      {
        "name": "LVAN_VNRM",
        "status": "OTHER",
        "type": "XN",
        "unit": "kPa",
        "description": "Vane undrained shear strength (remoulded)"
      },
      {
        "name": "LVAN_MC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water/moisture content local to the test"
      },
      {
        "name": "LVAN_SIZE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Equivalent diameter of vane"
      },
      {
        "name": "LVAN_VLEN",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Length of vane"
      },
      {
        "name": "LVAN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "LVAN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method, including type of vane"
      },
      {
        "name": "LVAN_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "LVAN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "LVAN_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      },
      {
        "name": "LVAN_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Vane type"
      }
    ]
  },
  {
    "code": "MCVG",
    "contents": "MCV Tests - General",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "MCVG_200",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Weight percent of sample retained on 20 mm sieve"
      },
      {
        "name": "MCVG_NMC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Natural water/moisture content below 20 mm"
      },
      {
        "name": "MCVG_STAB",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Amount of stabiliser added"
      },
      {
        "name": "MCVG_STYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of stabiliser added"
      },
      {
        "name": "MCVG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks including commentary on effect of specimen disturbance on test result"
      },
      {
        "name": "MCVG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "MCVG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "MCVG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "MCVG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "MCVT",
    "contents": "MCV Tests - Data",
    "parent": "MCVG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "MCVT_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "MCVT_MC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water/moisture content for MCVT_TESN"
      },
      {
        "name": "MCVT_CURV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of interpretation of the test curve"
      },
      {
        "name": "MCVT_RELK",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "MCV value for MCVT_TESN"
      },
      {
        "name": "MCVT_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "After test bulk density for MCVT_TESN"
      },
      {
        "name": "MCVT_DIFF",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Difference between initial (n) and final (3n) blows in rapid assessment test"
      },
      {
        "name": "MCVT_RAPD",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stronger or weaker than pre-calibrated standard"
      },
      {
        "name": "MCVT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "MOND",
    "contents": "Monitoring Readings",
    "parent": "MONG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "MONG_ID",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Monitoring point reference"
      },
      {
        "name": "MONG_DIS",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Initial distance of monitoring point from LOCA_ID datum"
      },
      {
        "name": "MOND_DTIM",
        "status": "KEY",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Date and time of reading"
      },
      {
        "name": "MOND_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Reading type"
      },
      {
        "name": "MOND_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Reading reference"
      },
      {
        "name": "MOND_INST",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Instrument reference / serial number"
      },
      {
        "name": "MOND_RDNG",
        "status": "OTHER",
        "type": "XN",
        "unit": null,
        "description": "Reading"
      },
      {
        "name": "MOND_UNIT",
        "status": "REQUIRED",
        "type": "PU",
        "unit": null,
        "description": "Units of reading"
      },
      {
        "name": "MOND_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Measurement method"
      },
      {
        "name": "MOND_LIM",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Instrument/method reading/detection limit"
      },
      {
        "name": "MOND_ULIM",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Instrument/method upper reading/detection (when appropriate)"
      },
      {
        "name": "MOND_NAME",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Client preferred name of measurement"
      },
      {
        "name": "MOND_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
      },
      {
        "name": "MOND_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Organization taking reading"
      },
      {
        "name": "MOND_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Comments on reading"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. monitoring field sheets, instrument logging file)"
      }
    ]
  },
  {
    "code": "MONG",
    "contents": "Monitoring Installations and Instruments",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "MONG_ID",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Monitoring point reference"
      },
      {
        "name": "MONG_DIS",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Initial distance of monitoring point from LOCA_ID"
      },
      {
        "name": "PIPE_REF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Pipe reference"
      },
      {
        "name": "MONG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Installation date"
      },
      {
        "name": "MONG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Instrument type"
      },
      {
        "name": "MONG_DETL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of instrument"
      },
      {
        "name": "MONG_TRZ",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Distance to start of response zone from LOCA_ID datum"
      },
      {
        "name": "MONG_BRZ",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Distance to end of response zone from LOCA_ID datum"
      },
      {
        "name": "MONG_BRGA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": "Bearing of monitoring axis A (compass bearing)"
      },
      {
        "name": "MONG_BRGB",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": "Bearing of monitoring axis B (compass bearing)"
      },
      {
        "name": "MONG_BRGC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": "Bearing of monitoring axis C (compass bearing)"
      },
      {
        "name": "MONG_INCA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": "Inclination of instrument axis A (measured positively down from horizontal)"
      },
      {
        "name": "MONG_INCB",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": "Inclination of instrument axis B (measured positively down from horizontal)"
      },
      {
        "name": "MONG_INCC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": "Inclination of instrument axis C (measured positively down from horizontal)"
      },
      {
        "name": "MONG_RSCA",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Reading sign convention in direction A"
      },
      {
        "name": "MONG_RSCB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Reading sign convention in direction B"
      },
      {
        "name": "MONG_RSCC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Reading sign convention in direction C"
      },
      {
        "name": "MONG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "MONG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Contractor who installed monitoring instrument"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. equipment calibrations)"
      }
    ]
  },
  {
    "code": "PIPE",
    "contents": "Monitoring Installation Pipe Work",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "PIPE_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Pipe reference"
      },
      {
        "name": "PIPE_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Top of construction zone"
      },
      {
        "name": "PIPE_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Base of construction zone"
      },
      {
        "name": "PIPE_DIAM",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Diameter of pipe"
      },
      {
        "name": "PIPE_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of pipe"
      },
      {
        "name": "PIPE_CONS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of pipe construction"
      },
      {
        "name": "PIPE_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. drilling journals)"
      }
    ]
  },
  {
    "code": "PLTG",
    "contents": "Plate Loading Tests - General",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "PLTG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Test depth"
      },
      {
        "name": "PLTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "PLTG_CYC",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Load cycle"
      },
      {
        "name": "PLTG_PDIA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Plate diameter"
      },
      {
        "name": "PLTG_SEAT",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kN",
        "description": "Seating load including apparatus mass"
      },
      {
        "name": "PLTG_FA0",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Factor a0"
      },
      {
        "name": "PLTG_FA1",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Factor a1"
      },
      {
        "name": "PLTG_FA2",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Factor a2"
      },
      {
        "name": "PLTG_SMOD",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Strain modulus"
      },
      {
        "name": "PLTG_EV2",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Elastic modulus for second loading cycle"
      },
      {
        "name": "PLTG_MOSR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa/m",
        "description": "Modulus of subgrade reaction"
      },
      {
        "name": "PLTG_EMOD",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Elastic modulus"
      },
      {
        "name": "PLTG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "PLTG_STAB",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Amount of stabiliser added"
      },
      {
        "name": "PLTG_STYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of stabiliser added"
      },
      {
        "name": "PLTG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "PLTG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "PLTG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "PLTG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing organization"
      },
      {
        "name": "PLTG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test status"
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stratum reference shown on trial pit or traverse sketch"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "PLTG_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      }
    ]
  },
  {
    "code": "PLTT",
    "contents": "Plate Loading Tests - Data",
    "parent": "PLTG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "PLTG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Test depth"
      },
      {
        "name": "PLTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "PLTG_CYC",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Load cycle"
      },
      {
        "name": "PLTT_STG",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Load stage"
      },
      {
        "name": "PLTT_TIME",
        "status": "KEY",
        "type": "1DP",
        "unit": "min",
        "description": "Stage elapsed time"
      },
      {
        "name": "PLTT_LOAD",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kN",
        "description": "Applied load"
      },
      {
        "name": "PLTT_SET1",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Settlement Gauge 1"
      },
      {
        "name": "PLTT_SET2",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Settlement Gauge 2"
      },
      {
        "name": "PLTT_SET3",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Settlement Gauge 3"
      },
      {
        "name": "PLTT_SET4",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Settlement Gauge 4"
      },
      {
        "name": "PLTT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Comments on reading"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "PMTD",
    "contents": "Pressuremeter Test Data",
    "parent": "PMTG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "PMTG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of test"
      },
      {
        "name": "PMTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "PMTD_SEQ",
        "status": "KEY",
        "type": "0DP",
        "unit": null,
        "description": "Sequence number"
      },
      {
        "name": "PMTD_TPC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Total pressure"
      },
      {
        "name": "PMTD_PPA",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Pore pressure cell A"
      },
      {
        "name": "PMTD_PPB",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Pore pressure cell B"
      },
      {
        "name": "PMTD_VOL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "cm3",
        "description": "Volume change in test cell"
      },
      {
        "name": "PMTD_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "PMTD_AX1",
        "status": "OTHER",
        "type": "4DP",
        "unit": "mm",
        "description": "Axis 1 displacement"
      },
      {
        "name": "PMTD_AX2",
        "status": "OTHER",
        "type": "4DP",
        "unit": "mm",
        "description": "Axis 2 displacement"
      },
      {
        "name": "PMTD_AX3",
        "status": "OTHER",
        "type": "4DP",
        "unit": "mm",
        "description": "Axis 3 displacement"
      },
      {
        "name": "PMTD_SA1",
        "status": "OTHER",
        "type": "4DP",
        "unit": "mm",
        "description": "Arm 1 displacement"
      },
      {
        "name": "PMTD_SA2",
        "status": "OTHER",
        "type": "4DP",
        "unit": "mm",
        "description": "Arm 2 displacement"
      },
      {
        "name": "PMTD_SA3",
        "status": "OTHER",
        "type": "4DP",
        "unit": "mm",
        "description": "Arm 3 displacement"
      },
      {
        "name": "PMTD_SA4",
        "status": "OTHER",
        "type": "4DP",
        "unit": "mm",
        "description": "Arm 4 displacement"
      },
      {
        "name": "PMTD_SA5",
        "status": "OTHER",
        "type": "4DP",
        "unit": "mm",
        "description": "Arm 5 displacement"
      },
      {
        "name": "PMTD_SA6",
        "status": "OTHER",
        "type": "4DP",
        "unit": "mm",
        "description": "Arm 6 displacement"
      },
      {
        "name": "PMTD_SAME",
        "status": "OTHER",
        "type": "4DP",
        "unit": "mm",
        "description": "Mean arm displacement"
      },
      {
        "name": "PMTD_TIME",
        "status": "OTHER",
        "type": "0DP",
        "unit": "s",
        "description": "Time elapsed since start of test"
      },
      {
        "name": "PMTD_ARM1",
        "status": "DEPRECATED",
        "type": "3DP",
        "unit": "mm",
        "description": "Axis 1 displacement"
      },
      {
        "name": "PMTD_ARM2",
        "status": "DEPRECATED",
        "type": "3DP",
        "unit": "mm",
        "description": "Axis 2 displacement"
      },
      {
        "name": "PMTD_ARM3",
        "status": "DEPRECATED",
        "type": "3DP",
        "unit": "mm",
        "description": "Axis 3 displacement"
      }
    ]
  },
  {
    "code": "PMTG",
    "contents": "Pressuremeter Test Results - General",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "PMTG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of test"
      },
      {
        "name": "PMTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "PMTG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date of test"
      },
      {
        "name": "PMTG_WAT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Measured or assumed ground water level"
      },
      {
        "name": "PMTG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Subcontractors name"
      },
      {
        "name": "PMTG_CREW",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Operators details"
      },
      {
        "name": "PMTG_REF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Instrument reference / serial number"
      },
      {
        "name": "PMTG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Pressuremeter type"
      },
      {
        "name": "PMTG_DIAM",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Uninflated diameter of pressuremeter"
      },
      {
        "name": "PMTG_HO",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Estimated in situ horizontal stress"
      },
      {
        "name": "PMTG_GI",
        "status": "OTHER",
        "type": "0DP",
        "unit": "MPa",
        "description": "Initial shear modulus"
      },
      {
        "name": "PMTG_CU",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Undrained shear strength"
      },
      {
        "name": "PMTG_PL",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Limit pressure"
      },
      {
        "name": "PMTG_AF",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Angle of friction"
      },
      {
        "name": "PMTG_AD",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": "Angle of dilation"
      },
      {
        "name": "PMTG_AFCV",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Angle of friction at constant volume (*cv) used"
      },
      {
        "name": "PMTG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method(s) used to determine derived soil parameters (including those in PMTL)."
      },
      {
        "name": "PMTG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test status"
      },
      {
        "name": "PMTG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "PMTG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "PMTG_NUAR",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Number of arms"
      },
      {
        "name": "PMTG_ORNT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": "Bearing of arm 1 (clockwise degrees from North)"
      },
      {
        "name": "PMTG_AXIS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Arm combination used for analysis"
      },
      {
        "name": "PMTG_PRWL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to water/fluid in borehole prior to test"
      },
      {
        "name": "PMTG_TC",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Method of test control"
      },
      {
        "name": "PMTG_STAD",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Start of drilling of SBPM"
      },
      {
        "name": "PMTG_ENDD",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "End of drilling of SBPM"
      },
      {
        "name": "PMTG_TOPP",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of test pocket for SBPM"
      },
      {
        "name": "PMTG_BOTP",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of test pocket for SBPM"
      },
      {
        "name": "PMTG_SBHT",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Self-boring head type of SBPM"
      },
      {
        "name": "PMTG_SBCS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Self-boring cutting shoe diameter, dp, of SBPM"
      },
      {
        "name": "PMTG_SBCT",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Cutter type of SBPM"
      },
      {
        "name": "PMTG_SBCD",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Cutter dimension of SBPM"
      },
      {
        "name": "PMTG_SBCP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Cutter position, h (+ve inside cutting shoe, -ve outside cutting shoe) of SBPM"
      },
      {
        "name": "PMTG_FLFT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Flushing fluid type of SBPM"
      },
      {
        "name": "PMTG_FLFP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Flushing or jetting fluid pressure of SBPM"
      },
      {
        "name": "PMTG_TRST",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kN",
        "description": "Thrust force of SBPM"
      },
      {
        "name": "PMTG_PPRD",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Pore pressures recorded during boring"
      },
      {
        "name": "PMTG_CMT",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Cone module type"
      },
      {
        "name": "PMTG_CREM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Describe corrections applied during processing"
      },
      {
        "name": "PMTG_CRDT",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date of last calibration of instrument"
      },
      {
        "name": "PMTG_ANBY",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name(s) of analyser / person responsible for data quality and correctness"
      }
    ]
  },
  {
    "code": "PMTL",
    "contents": "Pressuremeter Test Results - Individual Loops",
    "parent": "PMTG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "PMTG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of test"
      },
      {
        "name": "PMTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "PMTL_LNO",
        "status": "KEY",
        "type": "0DP",
        "unit": null,
        "description": "Unload/reload loop number"
      },
      {
        "name": "PMTL_GAA",
        "status": "OTHER",
        "type": "3SF",
        "unit": "MPa",
        "description": "Unload/reload shear modulus, average"
      },
      {
        "name": "PMTL_SINC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Mean strain"
      },
      {
        "name": "PMTL_PINC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Mean pressure"
      },
      {
        "name": "PMTL_STRA",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Strain range or amplitude"
      },
      {
        "name": "PMTL_PRSA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Pressure range or amplitude"
      },
      {
        "name": "PMTL_NLSA",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Shear stress coefficient (from Bolton and Whittle, 1999)"
      },
      {
        "name": "PMTL_NLSB",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Linearity exponent (from Bolton and Whittle, 1999)"
      },
      {
        "name": "PMTL_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "PMTL_AXIS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Arm combination used for analysis"
      },
      {
        "name": "PMTL_HP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Hold pressure"
      },
      {
        "name": "PMTL_HT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "s",
        "description": "Hold duration"
      },
      {
        "name": "PMTL_CR",
        "status": "OTHER",
        "type": "4DP",
        "unit": "%",
        "description": "Creep rate"
      },
      {
        "name": "PMTD_SEQ",
        "status": "KEY",
        "type": "0DP",
        "unit": null,
        "description": "Sequence number"
      }
    ]
  },
  {
    "code": "PREM",
    "contents": "Project Specific Time Related Remarks",
    "parent": null,
    "headings": [
      {
        "name": "PREM_DTIM",
        "status": "KEY",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Date and time of remark or start of event"
      },
      {
        "name": "PREM_COMP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Component or sub-activity"
      },
      {
        "name": "PREM_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Time related remark"
      },
      {
        "name": "PREM_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": "Duration of event or activity"
      },
      {
        "name": "PREM_ETIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Date and time of end of event"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. site journal records)"
      }
    ]
  },
  {
    "code": "PTIM",
    "contents": "Boring/Drilling Progress by Time",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "PTIM_DTIM",
        "status": "KEY",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Date and time of progress reading"
      },
      {
        "name": "PTIM_DPTH",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Hole depth"
      },
      {
        "name": "PTIM_CAS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of casing"
      },
      {
        "name": "PTIM_WAT",
        "status": "OTHER",
        "type": "XN",
        "unit": "m",
        "description": "Depth to water"
      },
      {
        "name": "PTIM_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. drilling journals)"
      }
    ]
  },
  {
    "code": "PTST",
    "contents": "Laboratory Permeability Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "PTST_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "PTST_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Sample condition"
      },
      {
        "name": "PTST_SZUN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Size cut off of material too coarse for testing"
      },
      {
        "name": "PTST_UNS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Proportion of material removed above PTST"
      },
      {
        "name": "PTST_DIAM",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen diameter"
      },
      {
        "name": "PTST_LEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen length"
      },
      {
        "name": "PTST_MC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Initial water/moisture content of test specimen"
      },
      {
        "name": "PTST_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial bulk density of test specimen"
      },
      {
        "name": "PTST_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial dry density"
      },
      {
        "name": "PTST_IDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Diameter of drain for radial permeability in hydraulic cell"
      },
      {
        "name": "PTST_DMET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of forming central drain"
      },
      {
        "name": "PTST_VOID",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Initial voids ratio"
      },
      {
        "name": "PTST_K",
        "status": "OTHER",
        "type": "1SCI",
        "unit": "m/s",
        "description": "Coefficient of permeability"
      },
      {
        "name": "PTST_TSTR",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Mean effective stress at which permeability measured (when measured in triaxial or hydraulic cell)."
      },
      {
        "name": "PTST_HYGR",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Hydraulic gradient at which permeability measured (for constant head test)."
      },
      {
        "name": "PTST_ISAT",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Initial degree of saturation"
      },
      {
        "name": "PTST_SAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of saturation, where appropriate"
      },
      {
        "name": "PTST_CONS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of consolidation, where appropriate"
      },
      {
        "name": "PTST_PDEN",
        "status": "OTHER",
        "type": "XN",
        "unit": "Mg/m3",
        "description": "Particle density with prefix # if value assumed"
      },
      {
        "name": "PTST_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of permeability measurement"
      },
      {
        "name": "PTST_CELL",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of permeameter"
      },
      {
        "name": "PTST_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks on test"
      },
      {
        "name": "PTST_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "PTST_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "PTST_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "PTST_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviations from the test method"
      },
      {
        "name": "PTST_WCIS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Initial water content source"
      },
      {
        "name": "PTST_WCF",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Final water content of test specimen"
      },
      {
        "name": "PTST_FSAT",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Final degree of saturation, if determined"
      },
      {
        "name": "PTST_TEMP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "DegC",
        "description": "Average laboratory temperature at which the test was performed"
      },
      {
        "name": "PTST_SOUR",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Source of permeameter water"
      },
      {
        "name": "PTST_BACK",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Back pressure"
      },
      {
        "name": "PTST_BVAL",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "B-value, if used"
      },
      {
        "name": "PTST_LOSS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Equipment head loss corrections applied to the measurements, if any, and the associated flow rates"
      }
    ]
  },
  {
    "code": "PUMG",
    "contents": "Pumping Tests - General",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "PUMG_TEST",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "PUMG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Contractor"
      },
      {
        "name": "PUMG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of testing"
      },
      {
        "name": "PUMG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test status"
      },
      {
        "name": "PUMG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "PUMG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks on test"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "PUMG_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      }
    ]
  },
  {
    "code": "PUMT",
    "contents": "Pumping Tests - Data",
    "parent": "PUMG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "PUMG_TEST",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "PUMT_DTIM",
        "status": "KEY",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Date and time of reading"
      },
      {
        "name": "PUMT_DPTH",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to water below ground"
      },
      {
        "name": "PUMT_QUAT",
        "status": "OTHER",
        "type": "1DP",
        "unit": "l/s",
        "description": "Pumping rate from hole"
      },
      {
        "name": "PUMT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "RCCV",
    "contents": "Chalk Crushing Value Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "RCCV_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "RCCV_MC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water content of specimen tested"
      },
      {
        "name": "RCCV_CCV",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "Chalk crushing value"
      },
      {
        "name": "RCCV_100",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Percentage larger than 10mm in original sample"
      },
      {
        "name": "RCCV_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "RCCV_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "RCCV_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "RCCV_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "RCCV_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "RDEN",
    "contents": "Rock Porosity and Density Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "RDEN_MC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water content of specimen"
      },
      {
        "name": "RDEN_SMC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Saturated water content"
      },
      {
        "name": "RDEN_BDEN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kg/m3",
        "description": "Bulk density"
      },
      {
        "name": "RDEN_DDEN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kg/m3",
        "description": "Dry density"
      },
      {
        "name": "RDEN_PORO",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Porosity"
      },
      {
        "name": "RDEN_PDEN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kg/m3",
        "description": "Apparent particle density"
      },
      {
        "name": "RDEN_TEMP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "DegC",
        "description": "Temperature sample dried at"
      },
      {
        "name": "RDEN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "RDEN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "RDEN_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "RDEN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "RDEN_IDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Intact dry density"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "RDEN_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "RELD",
    "contents": "Relative Density Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "RELD_DMAX",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Maximum dry density"
      },
      {
        "name": "RELD_375",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Weight percent of sample retained on 37.5mm sieve"
      },
      {
        "name": "RELD_063",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Weight percent of sample retained on 6.3mm sieve"
      },
      {
        "name": "RELD_020",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Weight percent of sample retained on 2mm sieve"
      },
      {
        "name": "RELD_DMIN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Minimum dry density"
      },
      {
        "name": "RELD_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks on test"
      },
      {
        "name": "RELD_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "RELD_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "RELD_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "RELD_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "RPLT",
    "contents": "Point Load Testing",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "RPLT_PLS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "MPa",
        "description": "Uncorrected point load (Is)"
      },
      {
        "name": "RPLT_PLSI",
        "status": "OTHER",
        "type": "2DP",
        "unit": "MPa",
        "description": "Size corrected point load index (Is 50)"
      },
      {
        "name": "RPLT_PLTF",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Point load test type"
      },
      {
        "name": "RPLT_MC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Water content of point load test specimen"
      },
      {
        "name": "RPLT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "RPLT_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "RPLT_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "RPLT_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "RPLT_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "RSCH",
    "contents": "Schmidt Rebound Hardness Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "RSCH_SCHV",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Schmidt hardness value"
      },
      {
        "name": "RSCH_AXIS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Orientation of the hammer axis in the test from horizontal (positive numbers downwards and negative numbers upward)"
      },
      {
        "name": "RSCH_CLAM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of clamping specimen"
      },
      {
        "name": "RSCH_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "RSCH_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "RSCH_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "RSCH_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "RSCH_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      },
      {
        "name": "RSCH_STYP",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Specimen type"
      },
      {
        "name": "RSCH_EXCV",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Method of excavation or block production"
      },
      {
        "name": "RSCH_DIAM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Specimen diameter"
      },
      {
        "name": "RSCH_LEN",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Specimen length"
      },
      {
        "name": "RSCH_WC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Water content of specimen, if measured"
      },
      {
        "name": "RSCH_WCTX",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Description of water content if not measured"
      },
      {
        "name": "RSCH_HTYP",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Hammer type"
      },
      {
        "name": "RSCH_ORN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Orientation of hammer axis with reference to intact rock anisotropy features (e.g. lamination, foliation, schistosity, lineation)"
      },
      {
        "name": "RSCH_MEAN",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Schmidt hardness mean (normalized to horizontal impact direction)"
      },
      {
        "name": "RSCH_MED",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Schmidt hardness median (normalized to horizontal impact direction)"
      },
      {
        "name": "RSCH_MODE",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Schmidt hardness mode (normalized to horizontal impact direction)"
      },
      {
        "name": "RSCH_RANG",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Schmidt hardness range (normalized to horizontal impact direction)"
      },
      {
        "name": "RSCH_NUM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Number of determinations if less than 20 and reason"
      }
    ]
  },
  {
    "code": "RSHR",
    "contents": "Shore Scleroscope Hardness Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "RSHR_SHOR",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "Average Shore hardness value"
      },
      {
        "name": "RSHR_AXIS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Orientation of the test surface relative to bedding"
      },
      {
        "name": "RSHR_NUM",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Number of tests conducted"
      },
      {
        "name": "RSHR_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "RSHR_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "RSHR_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "RSHR_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "RSHR_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "RTEN",
    "contents": "Tensile Strength Testing",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "RTEN_SDIA",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Specimen diameter"
      },
      {
        "name": "RTEN_LEN",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Specimen thickness"
      },
      {
        "name": "RTEN_MC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Water content of test specimen"
      },
      {
        "name": "RTEN_COND",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Condition of specimen as tested"
      },
      {
        "name": "RTEN_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "mm:ss",
        "description": "Test duration"
      },
      {
        "name": "RTEN_STRA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "N/s",
        "description": "Stress rate"
      },
      {
        "name": "RTEN_TENS",
        "status": "OTHER",
        "type": "3SF",
        "unit": "MPa",
        "description": "Tensile strength"
      },
      {
        "name": "RTEN_MODE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Mode of failure"
      },
      {
        "name": "RTEN_MACH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Testing machine"
      },
      {
        "name": "RTEN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "RTEN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "RTEN_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "RTEN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "RTEN_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "RUCS",
    "contents": "Rock Uniaxial Compressive Strength and Deformability Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "RUCS_SDIA",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Specimen diameter"
      },
      {
        "name": "RUCS_LEN",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Specimen length"
      },
      {
        "name": "RUCS_MC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Water content of specimen tested"
      },
      {
        "name": "RUCS_COND",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Condition of specimen as tested"
      },
      {
        "name": "RUCS_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "mm:ss",
        "description": "Test duration"
      },
      {
        "name": "RUCS_STRA",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa/s",
        "description": "Stress rate"
      },
      {
        "name": "RUCS_UCS",
        "status": "OTHER",
        "type": "3SF",
        "unit": "MPa",
        "description": "Uniaxial compressive strength"
      },
      {
        "name": "RUCS_MODE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Mode of failure"
      },
      {
        "name": "RUCS_MACH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of testing machine"
      },
      {
        "name": "RUCS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "RUCS_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "RUCS_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "RUCS_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "RUCS_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      },
      {
        "name": "RUCS_ESEC",
        "status": "OTHER",
        "type": "3SF",
        "unit": "GPa",
        "description": "Young's modulus, secant"
      },
      {
        "name": "RUCS_ETAN",
        "status": "OTHER",
        "type": "3SF",
        "unit": "GPa",
        "description": "Young's modulus, tangent"
      },
      {
        "name": "RUCS_EAVG",
        "status": "OTHER",
        "type": "3SF",
        "unit": "GPa",
        "description": "Young's modulus, average (mean)"
      },
      {
        "name": "RUCS_SSEC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stress level at which secant Young's modulus has been measured"
      },
      {
        "name": "RUCS_STAN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stress level at which tangent Young's modulus has been measured"
      },
      {
        "name": "RUCS_SAVG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stress level at which average (mean) Young's modulus has been measured"
      },
      {
        "name": "RUCS_MUS",
        "status": "OTHER",
        "type": "3SF",
        "unit": null,
        "description": "Poisson's ratio, secant"
      },
      {
        "name": "RUCS_MUT",
        "status": "OTHER",
        "type": "3SF",
        "unit": null,
        "description": "Poisson's ratio, tangent"
      },
      {
        "name": "RUCS_MUAV",
        "status": "OTHER",
        "type": "3SF",
        "unit": null,
        "description": "Poisson's ratio, average (mean)"
      },
      {
        "name": "RUCS_E",
        "status": "DEPRECATED",
        "type": "3SF",
        "unit": "GPa",
        "description": "Young's modulus"
      },
      {
        "name": "RUCS_MU",
        "status": "DEPRECATED",
        "type": "2DP",
        "unit": null,
        "description": "Poisson's ratio"
      },
      {
        "name": "RUCS_ESTR",
        "status": "DEPRECATED",
        "type": "X",
        "unit": null,
        "description": "Stress level at which modulus has been measured"
      },
      {
        "name": "RUCS_ETYP",
        "status": "DEPRECATED",
        "type": "PA",
        "unit": null,
        "description": "Method of determination of Young's modulus"
      }
    ]
  },
  {
    "code": "RWCO",
    "contents": "Water Content of Rock Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "RWCO_MC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water content"
      },
      {
        "name": "RWCO_TEMP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "DegC",
        "description": "Temperature sample dried at"
      },
      {
        "name": "RWCO_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "RWCO_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "RWCO_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "RWCO_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "RWCO_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "SAMP",
    "contents": "Sample Information",
    "parent": "LOCA",
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
        "name": "SAMP_UBLO",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Number of blows required to drive sampler"
      },
      {
        "name": "SAMP_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Sample container"
      },
      {
        "name": "SAMP_PREP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of sample preparation at time of sampling"
      },
      {
        "name": "SAMP_SDIA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Sample diameter"
      },
      {
        "name": "SAMP_WDEP",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to water below ground surface at time of sampling"
      },
      {
        "name": "SAMP_RECV",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Percentage of sample recovered"
      },
      {
        "name": "SAMP_TECH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Sampling technique/method"
      },
      {
        "name": "SAMP_MATX",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Sample matrix"
      },
      {
        "name": "SAMP_TYPC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Sample QA type (Normal, blank or spike)"
      },
      {
        "name": "SAMP_WHO",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Samplers initials or name"
      },
      {
        "name": "SAMP_WHY",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Reason for sampling"
      },
      {
        "name": "SAMP_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Sample remarks"
      },
      {
        "name": "SAMP_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Sample/specimen description"
      },
      {
        "name": "SAMP_DESD",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date sample described"
      },
      {
        "name": "SAMP_LOG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Person responsible for sample/specimen description"
      },
      {
        "name": "SAMP_COND",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Condition and representativeness of sample"
      },
      {
        "name": "SAMP_CLSS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Sample classification as required by EN ISO 14688-1"
      },
      {
        "name": "SAMP_BAR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "bar",
        "description": "Barometric pressure at time of sampling"
      },
      {
        "name": "SAMP_TEMP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "DegC",
        "description": "Sample temperature at time of sampling"
      },
      {
        "name": "SAMP_PRES",
        "status": "OTHER",
        "type": "1DP",
        "unit": "bar",
        "description": "Gas pressure (above barometric)"
      },
      {
        "name": "SAMP_FLOW",
        "status": "OTHER",
        "type": "1DP",
        "unit": "l/min",
        "description": "Gas flow rate"
      },
      {
        "name": "SAMP_ETIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Date and time sampling completed"
      },
      {
        "name": "SAMP_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": "Sampling duration"
      },
      {
        "name": "SAMP_CAPT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Caption used to describe sample"
      },
      {
        "name": "SAMP_LINK",
        "status": "OTHER",
        "type": "RL",
        "unit": null,
        "description": "Sample record link"
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stratum reference shown on trial pit or traverse sketch"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. sampling field sheets, sample description records)"
      },
      {
        "name": "SAMP_RECL",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Length of sample recovered"
      }
    ]
  },
  {
    "code": "SCDG",
    "contents": "Static Cone Dissipation Tests - General",
    "parent": "SCPG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "SCPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference or push number"
      },
      {
        "name": "SCDG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of dissipation test"
      },
      {
        "name": "SCDG_PWPI",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Measured or assumed initial pore water pressure"
      },
      {
        "name": "SCDG_PWPE",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Measured or assumed equilibrium pore water pressure"
      },
      {
        "name": "SCDG_DDIS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Degree of dissipation for analysis"
      },
      {
        "name": "SCDG_T",
        "status": "OTHER",
        "type": "1DP",
        "unit": "s",
        "description": "Time to achieve degree of dissipation stated in SCDG_DDIS"
      },
      {
        "name": "SCDG_CV",
        "status": "OTHER",
        "type": "2SCI",
        "unit": "m2/yr",
        "description": "Coefficient of consolidation (vertical)"
      },
      {
        "name": "SCDG_CVMT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method(s) used to determine vertical coefficient of consolidation"
      },
      {
        "name": "SCDG_CH",
        "status": "OTHER",
        "type": "2SCI",
        "unit": "m2/yr",
        "description": "Coefficient of consolidation (horizontal)"
      },
      {
        "name": "SCDG_CHMT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method(s) used to determine horizontal coefficient of consolidation"
      },
      {
        "name": "SCDG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "SCDG_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      }
    ]
  },
  {
    "code": "SCDT",
    "contents": "Static Cone Dissipation Tests - Data",
    "parent": "SCDG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "SCPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference or push number"
      },
      {
        "name": "SCDG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of dissipation test"
      },
      {
        "name": "SCDT_SECS",
        "status": "KEY",
        "type": "1DP",
        "unit": "s",
        "description": "Seconds elapsed since start of test"
      },
      {
        "name": "SCDT_RES",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Cone resistance"
      },
      {
        "name": "SCDT_PWP1",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Face porewater pressure (u1)"
      },
      {
        "name": "SCDT_PWP2",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Shoulder porewater pressure (u2)"
      },
      {
        "name": "SCDT_PWP3",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Top of sleeve porewater pressure (u3)"
      },
      {
        "name": "SCDT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Comments"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "SCPG",
    "contents": "Static Cone Penetration Tests - General",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "SCPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference or push number"
      },
      {
        "name": "SCPG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Cone test type"
      },
      {
        "name": "SCPG_REF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Cone reference"
      },
      {
        "name": "SCPG_CSA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "cm2",
        "description": "Surface area of cone tip"
      },
      {
        "name": "SCPG_RATE",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm/s",
        "description": "Nominal rate of penetration of the cone"
      },
      {
        "name": "SCPG_FILT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of filter material used"
      },
      {
        "name": "SCPG_FRIC",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Friction reducer used"
      },
      {
        "name": "SCPG_WAT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Groundwater level at time of test"
      },
      {
        "name": "SCPG_WATA",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Origin of water level in SCPG_WAT"
      },
      {
        "name": "SCPG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Comments on testing and basis of any interpreted parameters included in SCPT and SCPP"
      },
      {
        "name": "SCPG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "SCPG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Subcontractors name"
      },
      {
        "name": "SCPG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Standard followed for testing"
      },
      {
        "name": "SCPG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
      },
      {
        "name": "SCPG_CAR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Cone area ratio used to calculate qt"
      },
      {
        "name": "SCPG_SLAR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Sleeve area ratio used to calculate ft"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. cone calibration records)"
      },
      {
        "name": "SCPG_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      }
    ]
  },
  {
    "code": "SCPP",
    "contents": "Static Cone Penetration Tests - Derived Parameters",
    "parent": "SCPG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "SCPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference or push number"
      },
      {
        "name": "SCPP_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of layer"
      },
      {
        "name": "SCPP_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of layer"
      },
      {
        "name": "SCPP_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Interpretation reference"
      },
      {
        "name": "SCPP_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "SCPP_CSBT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Interpreted Soil Type"
      },
      {
        "name": "SCPP_CSU",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Undrained Shear Strength (Su); fine soils only"
      },
      {
        "name": "SCPP_CRD",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Relative density (Dr); coarse soils only"
      },
      {
        "name": "SCPP_CPHI",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Internal Friction Angle; coarse soils only"
      },
      {
        "name": "SCPP_CIC",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "Soil Behaviour Type Index (Ic)"
      },
      {
        "name": "SCPP_CSPT",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Equivalent SPT N60 value"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "SCPT",
    "contents": "Static Cone Penetration Tests - Data",
    "parent": "SCPG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "SCPG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference or push number"
      },
      {
        "name": "SCPT_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of result"
      },
      {
        "name": "SCPT_RES",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Cone resistance (qc)"
      },
      {
        "name": "SCPT_FRES",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Local unit side friction resistance (fs)"
      },
      {
        "name": "SCPT_PWP1",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Face porewater pressure (u1)"
      },
      {
        "name": "SCPT_PWP2",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Shoulder porewater pressure (u2)"
      },
      {
        "name": "SCPT_PWP3",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Top of sleeve porewater pressure (u3)"
      },
      {
        "name": "SCPT_CON",
        "status": "OTHER",
        "type": "4DP",
        "unit": "uS/cm",
        "description": "Conductivity"
      },
      {
        "name": "SCPT_TEMP",
        "status": "OTHER",
        "type": "4DP",
        "unit": "DegC",
        "description": "Temperature"
      },
      {
        "name": "SCPT_PH",
        "status": "OTHER",
        "type": "4DP",
        "unit": null,
        "description": "pH reading"
      },
      {
        "name": "SCPT_SLP1",
        "status": "OTHER",
        "type": "4DP",
        "unit": "deg",
        "description": "Slope indicator no. 1"
      },
      {
        "name": "SCPT_SLP2",
        "status": "OTHER",
        "type": "4DP",
        "unit": "deg",
        "description": "Slope indicator no. 2"
      },
      {
        "name": "SCPT_REDX",
        "status": "OTHER",
        "type": "4DP",
        "unit": "mV",
        "description": "Redox potential reading"
      },
      {
        "name": "SCPT_MAGT",
        "status": "OTHER",
        "type": "4DP",
        "unit": "nT",
        "description": "Magnetic flux - Total (calculated)"
      },
      {
        "name": "SCPT_MAGX",
        "status": "OTHER",
        "type": "4DP",
        "unit": "nT",
        "description": "Magnetic flux - X"
      },
      {
        "name": "SCPT_MAGY",
        "status": "OTHER",
        "type": "4DP",
        "unit": "nT",
        "description": "Magnetic flux - Y"
      },
      {
        "name": "SCPT_MAGZ",
        "status": "OTHER",
        "type": "4DP",
        "unit": "nT",
        "description": "Magnetic flux - Z"
      },
      {
        "name": "SCPT_SMP",
        "status": "OTHER",
        "type": "4DP",
        "unit": "%",
        "description": "Soil moisture"
      },
      {
        "name": "SCPT_NGAM",
        "status": "OTHER",
        "type": "4DP",
        "unit": "counts/s",
        "description": "Natural gamma radiation"
      },
      {
        "name": "SCPT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "SCPT_FRR",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Friction ratio (Rf)"
      },
      {
        "name": "SCPT_QT",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Corrected cone resistance (qt) piezocone only"
      },
      {
        "name": "SCPT_FT",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Corrected sleeve resistance (ft) piezocone only"
      },
      {
        "name": "SCPT_QE",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Effective cone resistance (qe) piezocone only"
      },
      {
        "name": "SCPT_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Bulk density of material (measured or assumed)"
      },
      {
        "name": "SCPT_CPO",
        "status": "OTHER",
        "type": "2DP",
        "unit": "kPa",
        "description": "Total vertical stress (based on SCPT_BDEN)"
      },
      {
        "name": "SCPT_CPOD",
        "status": "OTHER",
        "type": "2DP",
        "unit": "kPa",
        "description": "Effective vertical stress (calculated from SCPT_CPO and SCPT_ISPP or SCPG_WAT)"
      },
      {
        "name": "SCPT_QNET",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Net cone resistance (qn)"
      },
      {
        "name": "SCPT_FRRC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Corrected friction ratio (Rf') piezocone only"
      },
      {
        "name": "SCPT_EXPP",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Excess pore pressure (u-uo) piezocone only"
      },
      {
        "name": "SCPT_BQ",
        "status": "OTHER",
        "type": "4DP",
        "unit": null,
        "description": "Pore pressure ratio (Bq) piezocone only"
      },
      {
        "name": "SCPT_ISPP",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "In situ pore pressure (uo) (measured or assumed where not simple hydrostatic based on SCPG_WAT)"
      },
      {
        "name": "SCPT_NQT",
        "status": "OTHER",
        "type": "4DP",
        "unit": null,
        "description": "Normalised cone resistance (Qt)"
      },
      {
        "name": "SCPT_NFR",
        "status": "OTHER",
        "type": "4DP",
        "unit": "%",
        "description": "Normalised friction ratio (Fr)"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. raw field data)"
      }
    ]
  },
  {
    "code": "SHBG",
    "contents": "Shear Box Testing - General",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "SHBG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Test type"
      },
      {
        "name": "SHBG_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Sample condition"
      },
      {
        "name": "SHBG_CONS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Specific condition statements"
      },
      {
        "name": "SHBG_PCOH",
        "status": "OTHER",
        "type": "2SF",
        "unit": "kPa",
        "description": "Peak cohesion intercept"
      },
      {
        "name": "SHBG_PHI",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Peak angle of friction"
      },
      {
        "name": "SHBG_RCOH",
        "status": "OTHER",
        "type": "2SF",
        "unit": "kPa",
        "description": "Residual cohesion intercept"
      },
      {
        "name": "SHBG_RPHI",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Residual angle of friction"
      },
      {
        "name": "SHBG_ENCA",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of encapsulation of specimens tested"
      },
      {
        "name": "SHBG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks including commentary on effect of specimen disturbance on test result"
      },
      {
        "name": "SHBG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "SHBG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "SHBG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "SHBG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "SHBT",
    "contents": "Shear Box Testing - Data",
    "parent": "SHBG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "SHBT_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Shear box stage/specimen reference"
      },
      {
        "name": "SHBT_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial bulk density"
      },
      {
        "name": "SHBT_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial dry density"
      },
      {
        "name": "SHBT_NORM",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Normal stress applied"
      },
      {
        "name": "SHBT_DISP",
        "status": "OTHER",
        "type": "2SF",
        "unit": "mm/min",
        "description": "Displacement rate for peak stress stage"
      },
      {
        "name": "SHBT_DISR",
        "status": "OTHER",
        "type": "2SF",
        "unit": "mm/min",
        "description": "Displacement rate for residual stress stage"
      },
      {
        "name": "SHBT_REVS",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Number of traverses if residual test"
      },
      {
        "name": "SHBT_PEAK",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Peak shear stress"
      },
      {
        "name": "SHBT_RES",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Residual shear stress"
      },
      {
        "name": "SHBT_PDIS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Horizontal displacement at peak shear stress"
      },
      {
        "name": "SHBT_RDIS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Horizontal displacement at residual shear stress"
      },
      {
        "name": "SHBT_PDIN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Vertical displacement at peak shear stress"
      },
      {
        "name": "SHBT_RDIN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Vertical displacement at residual shear stress"
      },
      {
        "name": "SHBT_PDEN",
        "status": "OTHER",
        "type": "XN",
        "unit": "Mg/m3",
        "description": "Particle density with prefix # if value assumed"
      },
      {
        "name": "SHBT_IVR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Initial voids ratio"
      },
      {
        "name": "SHBT_MCI",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Initial water/moisture content"
      },
      {
        "name": "SHBT_MCF",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Final water/moisture content"
      },
      {
        "name": "SHBT_DIA1",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen diameter in direction of shear (rock joints)"
      },
      {
        "name": "SHBT_DIA2",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen diameter perpendicular to shear (rock joints)"
      },
      {
        "name": "SHBT_HGT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen height"
      },
      {
        "name": "SHBT_CRIT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Failure/residual strength criterion used"
      },
      {
        "name": "SHBT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SHBT_PVST",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Normal (vertical) stress at peak shear stress"
      },
      {
        "name": "SHBT_RVST",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Normal (vertical) stress at residual shear stress"
      }
    ]
  },
  {
    "code": "STND",
    "contents": "Standards / Specifications",
    "parent": null,
    "headings": [
      {
        "name": "STND_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Reference of standard"
      },
      {
        "name": "STND_TTLE",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Document Title"
      },
      {
        "name": "STND_SCPE",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Scope of data collected to this standard"
      },
      {
        "name": "STND_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Comments"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. contract specific specifications)"
      }
    ]
  },
  {
    "code": "SUCT",
    "contents": "Suction Tests",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "SUCT_DIAM",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen diameter"
      },
      {
        "name": "SUCT_LEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen length"
      },
      {
        "name": "SUCT_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Sample condition"
      },
      {
        "name": "SUCT_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial bulk density"
      },
      {
        "name": "SUCT_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial dry density"
      },
      {
        "name": "SUCT_MC",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Initial water/moisture content"
      },
      {
        "name": "SUCT_VAL",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Suction value"
      },
      {
        "name": "SUCT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "SUCT_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "SUCT_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "SUCT_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "SUCT_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "TNPC",
    "contents": "Ten Per Cent Fines",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "TNPC_TESN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "TNPC_DRY",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "10% fines values on dry aggregate"
      },
      {
        "name": "TNPC_WET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "10% fines values on wet aggregate"
      },
      {
        "name": "TNPC_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "TNPC_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "TNPC_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "TNPC_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "TNPC_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "TREG",
    "contents": "Triaxial Tests - Effective Stress - General",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "TREG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Test type"
      },
      {
        "name": "TREG_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Sample condition"
      },
      {
        "name": "TREG_COH",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Cohesion intercept associated with TREG_PHI"
      },
      {
        "name": "TREG_PHI",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Angle of friction for effective shear strength triaxial test"
      },
      {
        "name": "TREG_FCR",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Failure criterion"
      },
      {
        "name": "TREG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks including commentary on effect of specimen disturbance on test result"
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
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "TREG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "TREG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Any deviation from the procedure or specified test conditions"
      }
    ]
  },
  {
    "code": "TREM",
    "contents": "Location Specific Time Related Remarks",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "TREM_DTIM",
        "status": "KEY",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Date and time of remark or start of event"
      },
      {
        "name": "TREM_COMP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Component or sub-activity"
      },
      {
        "name": "TREM_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Time related remark"
      },
      {
        "name": "TREM_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": "Duration of event or activity"
      },
      {
        "name": "TREM_ETIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Date and time of end of event"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. site journal records)"
      }
    ]
  },
  {
    "code": "TRET",
    "contents": "Triaxial Tests - Effective Stress - Data",
    "parent": "TREG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "TRET_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Triaxial test/stage number"
      },
      {
        "name": "TRET_SDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen diameter"
      },
      {
        "name": "TRET_LEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen length"
      },
      {
        "name": "TRET_IMC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Specimen initial water/moisture content"
      },
      {
        "name": "TRET_FMC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Specimen final water/moisture content"
      },
      {
        "name": "TRET_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial bulk density"
      },
      {
        "name": "TRET_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial dry density"
      },
      {
        "name": "TRET_SAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of saturation"
      },
      {
        "name": "TRET_CONS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of consolidation stage"
      },
      {
        "name": "TRET_CONP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Effective stress at end of consolidation/ start of shear stage"
      },
      {
        "name": "TRET_CELL",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Total cell pressure during shearing stage"
      },
      {
        "name": "TRET_PWPI",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Porewater pressure at start of shear stage"
      },
      {
        "name": "TRET_STRR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%/hr",
        "description": "Rate of axial strain during shear"
      },
      {
        "name": "TRET_STRN",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Axial strain at failure"
      },
      {
        "name": "TRET_DEVF",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Deviator stress at failure"
      },
      {
        "name": "TRET_PWPF",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Porewater pressure at failure"
      },
      {
        "name": "TRET_STV",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Volumetric strain at failure (drained only)"
      },
      {
        "name": "TRET_MODE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Mode of failure"
      },
      {
        "name": "TRET_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Comments"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "TRET_BACK",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Final back pressure applied prior to shearing"
      },
      {
        "name": "TRET_VERT",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Vertical strain at end of consolidation"
      },
      {
        "name": "TRET_VOLM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Volumetric strain at end of consolidation"
      },
      {
        "name": "TRET_RATE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%/hr",
        "description": "Rate of volumetric strain immediately prior to shearing"
      },
      {
        "name": "TRET_BVAL",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Final B-value prior to shearing"
      },
      {
        "name": "TRET_DRN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of drainage conditions during shear"
      },
      {
        "name": "TRET_MEMB",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Membrane corrections applied at failure"
      },
      {
        "name": "TRET_FILC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Filter paper corrections applied at failure"
      },
      {
        "name": "TRET_IVR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Initial voids ratio"
      },
      {
        "name": "TRET_SATR",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Saturation percentage"
      },
      {
        "name": "TRET_CVP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Effective vertical pressure at end of consolidation"
      },
      {
        "name": "TRET_CRP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Effective radial pressure at end of consolidation"
      },
      {
        "name": "TRET_MEAN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Peak mean effective stress during shear"
      },
      {
        "name": "TRET_CU",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Undrained shear strength at failure"
      },
      {
        "name": "TRET_EP50",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Strain at 50 % peak deviator stress"
      },
      {
        "name": "TRET_E50",
        "status": "OTHER",
        "type": "2DP",
        "unit": "MPa",
        "description": "Secant modulus at 50 % peak deviator stress"
      }
    ]
  },
  {
    "code": "TRIG",
    "contents": "Triaxial Tests - Total Stress - General",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "TRIG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Test type"
      },
      {
        "name": "TRIG_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Sample condition"
      },
      {
        "name": "TRIG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks including commentary on effect of specimen disturbance on test result"
      },
      {
        "name": "TRIG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "TRIG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "TRIG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "TRIG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      }
    ]
  },
  {
    "code": "TRIT",
    "contents": "Triaxial Tests - Total Stress - Data",
    "parent": "TRIG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "TRIT_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Triaxial test/stage reference"
      },
      {
        "name": "TRIT_SDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen diameter"
      },
      {
        "name": "TRIT_SLEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen length"
      },
      {
        "name": "TRIT_IMC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Specimen initial water/moisture content"
      },
      {
        "name": "TRIT_FMC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Specimen final water/moisture content"
      },
      {
        "name": "TRIT_CELL",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Total cell pressure"
      },
      {
        "name": "TRIT_DEVF",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Corrected deviator stress at failure"
      },
      {
        "name": "TRIT_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial bulk density"
      },
      {
        "name": "TRIT_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial dry density"
      },
      {
        "name": "TRIT_STRN",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%",
        "description": "Axial strain at failure"
      },
      {
        "name": "TRIT_CU",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Undrained Shear Strength at failure"
      },
      {
        "name": "TRIT_MODE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Mode of failure"
      },
      {
        "name": "TRIT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Comments"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      },
      {
        "name": "TRIT_FZWC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Failure zone water content, if measured"
      },
      {
        "name": "TRIT_RATE",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%/min",
        "description": "Mean rate of shear"
      }
    ]
  },
  {
    "code": "WADD",
    "contents": "Water Added Records",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "WADD_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of reported section"
      },
      {
        "name": "WADD_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of reported section"
      },
      {
        "name": "WADD_VOLM",
        "status": "OTHER",
        "type": "0DP",
        "unit": "l",
        "description": "Amount of water added"
      },
      {
        "name": "WADD_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Boring/drilling method associated with addition of water (HDPH_TYPE abbreviation)"
      },
      {
        "name": "WADD_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks related to addition of water requirements, method"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. drilling journal)"
      }
    ]
  },
  {
    "code": "WETH",
    "contents": "Weathering",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "WETH_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of weathering subdivision"
      },
      {
        "name": "WETH_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of weathering subdivision"
      },
      {
        "name": "WETH_SCH",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Weathering scheme"
      },
      {
        "name": "WETH_SYS",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Material or mass weathering system"
      },
      {
        "name": "WETH_WETH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Weathering classifier for WETH_SCH and WETH_SYS"
      },
      {
        "name": "WETH_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. logging sheets)"
      }
    ]
  },
  {
    "code": "WINS",
    "contents": "Window or Windowless Sampling Run Details",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "WINS_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Sampler run reference"
      },
      {
        "name": "WINS_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Top of sampling run"
      },
      {
        "name": "WINS_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Base of sampling run"
      },
      {
        "name": "WINS_DIAM",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Internal diameter of sampler"
      },
      {
        "name": "WINS_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": "Duration of sampling run"
      },
      {
        "name": "WINS_REC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Sample recovery"
      },
      {
        "name": "WINS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks about sampling run"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. field records)"
      }
    ]
  },
  {
    "code": "WSTD",
    "contents": "Water Strike - Details",
    "parent": "WSTG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "WSTG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to water strike"
      },
      {
        "name": "WSTD_NMIN",
        "status": "KEY",
        "type": "0DP",
        "unit": "min",
        "description": "Minutes after strike"
      },
      {
        "name": "WSTD_POST",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to water after WSTD_NMIN minutes"
      },
      {
        "name": "WSTD_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "WSTG",
    "contents": "Water Strike - General",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "WSTG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to water strike"
      },
      {
        "name": "WSTG_DTIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Date and time of water strike"
      },
      {
        "name": "WSTG_SEAL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth at which water strike sealed by casing"
      },
      {
        "name": "WSTG_CAS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Casing depth at time of water strike"
      },
      {
        "name": "WSTG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "CTRC",
    "contents": "Cyclic Triaxial Tests - Consolidation",
    "parent": "CTRG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "CTRC_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test / Stage Number"
      },
      {
        "name": "CTRC_CELL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Final cell pressure"
      },
      {
        "name": "CTRC_BPWP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Base porewater pressure"
      },
      {
        "name": "CTRC_MPWP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Mid-height porewater pressure"
      },
      {
        "name": "CTRC_MPB",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Mid-height B value"
      },
      {
        "name": "CTRC_BB",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Base B value"
      },
      {
        "name": "CTRC_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of consolidation"
      },
      {
        "name": "CTRC_BACF",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Final back pressure"
      },
      {
        "name": "CTRC_ELAP",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": "Duration of test/stage number"
      },
      {
        "name": "CTRC_CHGT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen height at end of stage"
      },
      {
        "name": "CTRC_DIAE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen diameter at end of stage"
      },
      {
        "name": "CTRC_MCE",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water content at end of stage"
      },
      {
        "name": "CTRC_BDE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Bulk density at end of stage"
      },
      {
        "name": "CTRC_DDE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Dry density at end of stage"
      },
      {
        "name": "CTRC_RDE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Relative density index of sand at end of stage"
      },
      {
        "name": "CTRC_INCE",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Voids ratio at end of stage"
      },
      {
        "name": "CTRC_ASE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Effective axial stress at end of stage"
      },
      {
        "name": "CTRC_RSE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Effective radial stress at end of stage"
      },
      {
        "name": "CTRC_SSE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Shear stress at end of stage"
      },
      {
        "name": "CTRC_DEVE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Deviatoric stress at end of stage"
      },
      {
        "name": "CTRC_MNSE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Mean effective stress at end of stage"
      },
      {
        "name": "CTRC_RTOE",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Ratio of radial to axial effective stress at end of stage"
      },
      {
        "name": "CTRC_EASE",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "External axial strain at end of stage"
      },
      {
        "name": "CTRC_VLSE",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Volumetric strain from measured volume change at end of stage"
      },
      {
        "name": "CTRC_RDSE",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Radial strain from measured volume change at end of stage"
      },
      {
        "name": "CTRC_B",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "B value"
      },
      {
        "name": "CTRC_BETS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Bender element test sequence"
      },
      {
        "name": "CTRC_BEAX",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Bender element axis of measurement"
      },
      {
        "name": "CTRC_BEDS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Distance between bender elements"
      },
      {
        "name": "CTRC_MAT",
        "status": "OTHER",
        "type": "4DP",
        "unit": "s",
        "description": "Measured arrival time of propagated wave"
      },
      {
        "name": "CTRC_MATM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of measuring arrival time of propagated wave"
      },
      {
        "name": "CTRC_SWV",
        "status": "OTHER",
        "type": "0DP",
        "unit": "m/s",
        "description": "Calculated shear wave velocity"
      },
      {
        "name": "CTRC_SMGM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Shear modulus Gmax"
      },
      {
        "name": "CTRC_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "CTRD",
    "contents": "Cyclic Triaxial Tests - Data",
    "parent": "CTRP",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "CTRC_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test / Stage Number"
      },
      {
        "name": "CTRP_CYC",
        "status": "KEY",
        "type": "0DP",
        "unit": null,
        "description": "Cycle number"
      },
      {
        "name": "CTRD_TIME",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Date/time of reading"
      },
      {
        "name": "CTRD_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Test conditions"
      },
      {
        "name": "CTRD_SDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen diameter"
      },
      {
        "name": "CTRD_HIGH",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen height"
      },
      {
        "name": "CTRD_CELL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Cell pressure"
      },
      {
        "name": "CTRD_BPWP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Base porewater pressure"
      },
      {
        "name": "CTRD_MPWP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Mid-plane porewater pressure"
      },
      {
        "name": "CTRD_EAS",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "External axial strain"
      },
      {
        "name": "CTRD_LAS1",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Local axial strain 1"
      },
      {
        "name": "CTRD_LAS2",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Local axial strain 2"
      },
      {
        "name": "CTRD_VOL",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Volumetric strain"
      },
      {
        "name": "CTRD_RAD",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Radial strain"
      },
      {
        "name": "CTRD_SHSN",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Shear strain"
      },
      {
        "name": "CTRD_SHST",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Shear stress"
      },
      {
        "name": "CTRD_DEV",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Deviatoric stress"
      },
      {
        "name": "CTRD_PSD",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Principal stress difference"
      },
      {
        "name": "CTRD_MEES",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Mean effective stress"
      },
      {
        "name": "CTRD_SECE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Secant Young's Modulus (Local)"
      },
      {
        "name": "CTRD_TANE",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Tangent Young's Modulus"
      },
      {
        "name": "CTRD_FREQ",
        "status": "OTHER",
        "type": "2SF",
        "unit": "Hz",
        "description": "Loading frequency"
      },
      {
        "name": "CTRD_CSTS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Cyclic amplitude"
      },
      {
        "name": "CTRD_ACVS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Average cyclic axial stress"
      },
      {
        "name": "CTRD_DAVS",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Double amplitude axial strain"
      },
      {
        "name": "CTRD_CESR",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Compression/Extension stress ratio"
      },
      {
        "name": "CTRD_EMPR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Excess mid-plane pore pressure ratio"
      },
      {
        "name": "CTRD_EBPR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Excess base pore pressure ratio"
      },
      {
        "name": "CTRD_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "CTRG",
    "contents": "Cyclic Triaxial Test - General",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Specimen preparation technique used"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "CTRG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of test"
      },
      {
        "name": "CTRG_MCI",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Initial water/moisture content"
      },
      {
        "name": "CTRG_MCF",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Final water/moisture content"
      },
      {
        "name": "CTRG_H2O",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Description of type of water used for filter flushing, and salt content if relevant"
      },
      {
        "name": "CTRG_SBP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Saturation back pressure"
      },
      {
        "name": "CTRG_SATR",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Initial degree of saturation after back pressure"
      },
      {
        "name": "CTRG_IRD",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Initial sample relative density"
      },
      {
        "name": "CTRG_SDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Initial specimen diameter"
      },
      {
        "name": "CTRG_HIGT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Initial height of specimen"
      },
      {
        "name": "CTRG_TMSS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "g",
        "description": "Total mass of installed specimen"
      },
      {
        "name": "CTRG_PDEN",
        "status": "OTHER",
        "type": "XN",
        "unit": "Mg/m3",
        "description": "Particle density with prefix # if value assumed"
      },
      {
        "name": "CTRG_MADD",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Maximum density of sand"
      },
      {
        "name": "CTRG_MIDD",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Minimum density of sand"
      },
      {
        "name": "CTRG_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial dry density"
      },
      {
        "name": "CTRG_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial bulk density"
      },
      {
        "name": "CTRG_IVR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Initial voids ratio"
      },
      {
        "name": "CTRG_SAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of saturation"
      },
      {
        "name": "CTRG_DURN",
        "status": "OTHER",
        "type": "1DP",
        "unit": "day",
        "description": "Test Duration"
      },
      {
        "name": "CTRG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "CTRG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "CTRG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviations from the test method"
      },
      {
        "name": "CTRG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "CTRG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      }
    ]
  },
  {
    "code": "CTRP",
    "contents": "Cyclic Triaxial Test - Derived Parameters",
    "parent": "CTRC",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "CTRC_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test / Stage Number"
      },
      {
        "name": "CTRP_CYC",
        "status": "KEY",
        "type": "0DP",
        "unit": null,
        "description": "Cycle number"
      },
      {
        "name": "CTRP_CYCF",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Cycle number of failure"
      },
      {
        "name": "CTRP_PWPM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Maximum excess porewater pressure"
      },
      {
        "name": "CTRP_MNPP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Minimum excess porewater pressure"
      },
      {
        "name": "CTRP_MXSS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Maximum shear stress"
      },
      {
        "name": "CTRP_MNSS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Minimum shear stress"
      },
      {
        "name": "CTRP_AVSS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Mean shear stress"
      },
      {
        "name": "CTRP_CSS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Cyclic shear stress ((Max-Min)/2)"
      },
      {
        "name": "CTRP_ACVS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Average cyclic axial stress"
      },
      {
        "name": "CTRP_ASF",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Axial strain at failure"
      },
      {
        "name": "CTRP_FPWP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Porewater pressure at failure"
      },
      {
        "name": "CTRP_QMAX",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Maximum deviatoric stress"
      },
      {
        "name": "CTRP_QMIN",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Minimum deviatoric stress"
      },
      {
        "name": "CTRP_MNES",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Mean effective stress at end of CTRD_CYC"
      },
      {
        "name": "CTRP_EAMX",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Maximum axial strain"
      },
      {
        "name": "CTRP_EAMN",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Minimum axial strain"
      },
      {
        "name": "CTRP_FVR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Final voids ratio"
      },
      {
        "name": "CTRP_QEMX",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Deviatoric stress at maximum axial strain"
      },
      {
        "name": "CTRP_QEMN",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Deviatoric stress at minimum axial strain"
      },
      {
        "name": "CTRP_ESEC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Secant modulus"
      },
      {
        "name": "CTRP_DAMP",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Damping ratio"
      },
      {
        "name": "CTRP_MODE",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Mode of failure"
      },
      {
        "name": "CTRP_DIPL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Percent Difference from Programmed Load"
      },
      {
        "name": "CTRP_OBP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Observed Performance (Visual)"
      },
      {
        "name": "CTRP_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "CTRS",
    "contents": "Cyclic Triaxial Tests - Saturation",
    "parent": "CTRG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "CTRS_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test / Stage Number"
      },
      {
        "name": "CTRS_CELL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Saturation cell pressure"
      },
      {
        "name": "CTRS_BPWP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Saturation base porewater pressure"
      },
      {
        "name": "CTRS_MPWP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Saturation mid-height porewater pressure"
      },
      {
        "name": "CTRS_MPB",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Saturation mid-height B value"
      },
      {
        "name": "CTRS_BB",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Saturation base B value"
      },
      {
        "name": "CTRS_SAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Saturation method"
      },
      {
        "name": "CTRS_FSAT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Final saturation"
      },
      {
        "name": "CTRS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "DLOG",
    "contents": "Driller Geological Description",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "DLOG_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of drillers stratum description"
      },
      {
        "name": "DLOG_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of drillers stratum description"
      },
      {
        "name": "DLOG_DESC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Drillers description of stratum"
      },
      {
        "name": "DLOG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. sampling field sheets)"
      }
    ]
  },
  {
    "code": "ECTN",
    "contents": "Sample Container Details",
    "parent": "SAMP",
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
        "name": "ECTN_ID",
        "status": "KEY+REQUIRED",
        "type": "ID",
        "unit": null,
        "description": "Container unique identifier"
      },
      {
        "name": "ECTN_TYPE",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Sample container type"
      },
      {
        "name": "ECTN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Sample container remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. sampling field sheets)"
      }
    ]
  },
  {
    "code": "ELRG",
    "contents": "Environmental Laboratory Reporting",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "ELRG_CODE",
        "status": "KEY+REQUIRED",
        "type": "PA",
        "unit": null,
        "description": "Determinand code"
      },
      {
        "name": "ELRG_METH",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "ELRG_MATX",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Laboratory test matrix"
      },
      {
        "name": "ELRG_RTYP",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Run type (initial or reanalysis)"
      },
      {
        "name": "ELRG_TADE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Test additional descriptor"
      },
      {
        "name": "ELRG_TICN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Tentatively identified compound (TIC)"
      },
      {
        "name": "ELRG_RUNI",
        "status": "KEY+REQUIRED",
        "type": "PU",
        "unit": null,
        "description": "Result unit"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "ELRG_LSID",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Laboratory sample ID"
      },
      {
        "name": "ELRG_RTCD",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Result type"
      },
      {
        "name": "ELRG_IQLF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Interpreted qualifiers"
      },
      {
        "name": "ELRG_LQLF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Laboratory qualifiers"
      },
      {
        "name": "ELRG_RVAL",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Result value"
      },
      {
        "name": "ELRG_RTXT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Reported result"
      },
      {
        "name": "ELRG_NAME",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Determinand name"
      },
      {
        "name": "ELRG_TNAM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Laboratory analytical name"
      },
      {
        "name": "ELRG_DCAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Determinand category"
      },
      {
        "name": "ELRG_TESN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "ELRG_FDEV",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Flagged deviation"
      },
      {
        "name": "ELRG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Result deviation description(s)"
      },
      {
        "name": "ELRG_RRES",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Reportable result"
      },
      {
        "name": "ELRG_DETF",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Detect flag"
      },
      {
        "name": "ELRG_ORG",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Organic"
      },
      {
        "name": "ELRG_RDLM",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Reporting detection limit"
      },
      {
        "name": "ELRG_MDLM",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Method detection limit"
      },
      {
        "name": "ELRG_QLM",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Quantification limit"
      },
      {
        "name": "ELRG_DUNI",
        "status": "OTHER",
        "type": "PU",
        "unit": null,
        "description": "Unit of detection/quantification limits"
      },
      {
        "name": "ELRG_CASC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "CAS code"
      },
      {
        "name": "ELRG_TICP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Tentatively identified compound (TIC) probability"
      },
      {
        "name": "ELRG_TICT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "s",
        "description": "Tentatively identified compound (TIC) retention time"
      },
      {
        "name": "ELRG_RDAT",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Sample receipt date/time at laboratory"
      },
      {
        "name": "ELRG_SGRP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Sample delivery or batch code"
      },
      {
        "name": "ELRG_DTIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Analysis date and time"
      },
      {
        "name": "ELRG_TEST",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test or Suite Name"
      },
      {
        "name": "ELRG_TORD",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Total or dissolved"
      },
      {
        "name": "ELRG_LOCN",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Analysis location"
      },
      {
        "name": "ELRG_BAS",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Basis"
      },
      {
        "name": "ELRG_DIL",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Dilution factor"
      },
      {
        "name": "ELRG_LMTH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Leachate preparation method"
      },
      {
        "name": "ELRG_LDTM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm",
        "description": "Leachate preparation date and time"
      },
      {
        "name": "ELRG_IREF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Instrument reference number or identifier"
      },
      {
        "name": "ELRG_ITYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Instrument type"
      },
      {
        "name": "ELRG_SIZE",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Size of material removed prior to test; value given indicates lowest sized material removed"
      },
      {
        "name": "ELRG_PERP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Percentage of material removed"
      },
      {
        "name": "ELRG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "ELRG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "ELRG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "FGHG",
    "contents": "Field Geohydraulic Testing - General",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "FGHG_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of test zone"
      },
      {
        "name": "FGHG_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of test zone"
      },
      {
        "name": "FGHG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "FGHG_TDIA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Diameter of test zone"
      },
      {
        "name": "FGHG_SDIA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Inside diameter of installation standpipe or borehole casing"
      },
      {
        "name": "FGHG_ODIA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Outside diameter of installation standpipe or borehole casing"
      },
      {
        "name": "FGHG_HBAS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of borehole during test (excluding tests in installations)"
      },
      {
        "name": "FGHG_CAS",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of casing during test (excluding tests in installations)"
      },
      {
        "name": "FGHG_SFAC",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Shape factor for test zone"
      },
      {
        "name": "FGHG_SFRF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Shape factor reference"
      },
      {
        "name": "FGHG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "FGHG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of test"
      },
      {
        "name": "FGHG_CNFG",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Test configuration"
      },
      {
        "name": "FGHG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "FGHG_PRWL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to water in borehole or installation prior to test"
      },
      {
        "name": "FGHG_AWL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to assumed standing water level used for calculations of head during test"
      },
      {
        "name": "FGHG_HEAD",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Applied total head of water at centre of test zone"
      },
      {
        "name": "FGHG_FLOW",
        "status": "OTHER",
        "type": "1DP",
        "unit": "l/s",
        "description": "Average flow rate during test"
      },
      {
        "name": "FGHG_IPRM",
        "status": "OTHER",
        "type": "1SCI",
        "unit": "m/s",
        "description": "Representative permeability for test"
      },
      {
        "name": "FGHG_ILUG",
        "status": "OTHER",
        "type": "XN",
        "unit": null,
        "description": "Representative Lugeon value for water pressure test"
      },
      {
        "name": "FGHG_FTYP",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Flow type for water pressure test"
      },
      {
        "name": "FGHG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test remarks"
      },
      {
        "name": "FGHG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "FGHG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing organization"
      },
      {
        "name": "FGHG_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      },
      {
        "name": "FGHG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      }
    ]
  },
  {
    "code": "FGHI",
    "contents": "Field Geohydraulic Testing - Instrumentation Details",
    "parent": "FGHG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "FGHG_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of test zone"
      },
      {
        "name": "FGHG_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of test zone"
      },
      {
        "name": "FGHG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "FGHI_INST",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Instrument reference / serial number"
      },
      {
        "name": "FGHI_TYPE",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Instrument measured parameters"
      },
      {
        "name": "FGHI_DETL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of instrument"
      },
      {
        "name": "FGHI_LOCT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Instrument position"
      },
      {
        "name": "FGHI_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. equipment calibrations)"
      }
    ]
  },
  {
    "code": "FGHS",
    "contents": "Field Geohydraulic Testing - Test Results (per stage)",
    "parent": "FGHG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "FGHG_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of test zone"
      },
      {
        "name": "FGHG_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of test zone"
      },
      {
        "name": "FGHG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "FGHS_STG",
        "status": "KEY",
        "type": "0DP",
        "unit": null,
        "description": "Stage number of multistage test"
      },
      {
        "name": "FGHS_STTM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Start of stage date / time"
      },
      {
        "name": "FGHS_ENTM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "End of stage date / time"
      },
      {
        "name": "FGHS_HEAD",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Applied head of water during test stage at centre of test zone"
      },
      {
        "name": "FGHS_FLOW",
        "status": "OTHER",
        "type": "1DP",
        "unit": "l/s",
        "description": "Average flow rate during test stage"
      },
      {
        "name": "FGHS_IPRM",
        "status": "OTHER",
        "type": "1SCI",
        "unit": "m/s",
        "description": "Permeability for test stage"
      },
      {
        "name": "FGHS_ILUG",
        "status": "OTHER",
        "type": "XN",
        "unit": null,
        "description": "Lugeon value for test stage"
      },
      {
        "name": "FGHS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. equipment calibrations)"
      }
    ]
  },
  {
    "code": "FGHT",
    "contents": "Field Geohydraulic Testing - Data",
    "parent": "FGHI",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "FGHG_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of test zone"
      },
      {
        "name": "FGHG_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of test zone"
      },
      {
        "name": "FGHG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "FGHI_INST",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Instrument reference / serial number"
      },
      {
        "name": "FGHT_TIME",
        "status": "KEY",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Test date / clock time of reading"
      },
      {
        "name": "FGHT_TYPE",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Test record type"
      },
      {
        "name": "FGHS_STG",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Stage number of multistage test"
      },
      {
        "name": "FGHT_DURN",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": "Elapsed time of reading during test or test stage"
      },
      {
        "name": "FGHT_RDNG",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Test record (reading)"
      },
      {
        "name": "FGHT_UNIT",
        "status": "OTHER",
        "type": "PU",
        "unit": null,
        "description": "Reading units"
      },
      {
        "name": "FGHT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test record remark"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. equipment calibrations)"
      }
    ]
  },
  {
    "code": "LFCN",
    "contents": "Laboratory Fall Cone Test",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "LFCN_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviations from the procedure"
      },
      {
        "name": "LFCN_CMAS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "g",
        "description": "Mass of cone used"
      },
      {
        "name": "LFCN_CANG",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": "Angle of cone tip"
      },
      {
        "name": "LFCN_PENA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Average cone penetration"
      },
      {
        "name": "LFCN_PEN1",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Individual penetration point 1 if values differ by more than 0.5mm from the average, for undisturbed tests."
      },
      {
        "name": "LFCN_PEN2",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Individual penetration point 2 if values differ by more than 0.5mm from the average, for undisturbed tests."
      },
      {
        "name": "LFCN_PEN3",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Individual penetration point 3 if values differ by more than 0.5mm from the average, for undisturbed tests."
      },
      {
        "name": "LFCN_PEN4",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Individual penetration point 4 if values differ by more than 0.5mm from the average, for undisturbed tests."
      },
      {
        "name": "LFCN_CONF",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Non-conforming test (due to penetration range)"
      },
      {
        "name": "LFCN_FCPK",
        "status": "OTHER",
        "type": "2SF",
        "unit": "kPa",
        "description": "Estimated undrained fall cone shear strength"
      },
      {
        "name": "LFCN_FCRM",
        "status": "OTHER",
        "type": "2SF",
        "unit": "kPa",
        "description": "Estimated undrained fall cone shear strength, remoulded"
      },
      {
        "name": "LFCN_WC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water content of specimen"
      },
      {
        "name": "LFCN_WCST",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Water content determined on specimen trimmings or other if applicable."
      },
      {
        "name": "LFCN_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test remarks"
      },
      {
        "name": "LFCN_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "LFCN_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "LFCN_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      }
    ]
  },
  {
    "code": "LTCH",
    "contents": "Laboratory Thermal Conductivity",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "LTCH_COND",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Sample condition"
      },
      {
        "name": "LTCH_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Bulk density"
      },
      {
        "name": "LTCH_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Dry density"
      },
      {
        "name": "LTCH_MC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water/moisture content"
      },
      {
        "name": "LTCH_TCON",
        "status": "OTHER",
        "type": "2DP",
        "unit": "W/(K-m)",
        "description": "Thermal Conductivity"
      },
      {
        "name": "LTCH_TRES",
        "status": "OTHER",
        "type": "2DP",
        "unit": "(K-m)/W",
        "description": "Thermal Resistivity"
      },
      {
        "name": "LTCH_TEMP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "DegC",
        "description": "Ambient temperature at which test is performed"
      },
      {
        "name": "LTCH_PDIA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Probe diameter"
      },
      {
        "name": "LTCH_PSPA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Probe spacing"
      },
      {
        "name": "LTCH_PPEN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Probe penetration"
      },
      {
        "name": "LTCH_PRBE",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of probe insertion"
      },
      {
        "name": "LTCH_PART",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Particle grain size removed"
      },
      {
        "name": "LTCH_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the procedure"
      },
      {
        "name": "LTCH_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "LTCH_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "LTCH_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "LTCH_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      }
    ]
  },
  {
    "code": "LUCT",
    "contents": "Laboratory Unconfined Compression Test",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "LUCT_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the procedure"
      },
      {
        "name": "LUCT_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Test type"
      },
      {
        "name": "LUCT_DIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen diameter"
      },
      {
        "name": "LUCT_SLEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen length"
      },
      {
        "name": "LUCT_IWC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Specimen initial water content"
      },
      {
        "name": "LUCT_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial bulk density"
      },
      {
        "name": "LUCT_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial dry density"
      },
      {
        "name": "LUCT_RATE",
        "status": "OTHER",
        "type": "2SF",
        "unit": "%/min",
        "description": "Mean rate of compression"
      },
      {
        "name": "LUCT_UCS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Unconfined compressive strength"
      },
      {
        "name": "LUCT_STRA",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Strain at failure"
      },
      {
        "name": "LUCT_MODE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Mode of failure"
      },
      {
        "name": "LUCT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "LUCT_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "LUCT_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "LUCT_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      }
    ]
  },
  {
    "code": "RCAG",
    "contents": "Rock Abrasiveness Tests - General",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "RCAG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      },
      {
        "name": "RCAG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date of test"
      },
      {
        "name": "RCAG_COND",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Condition of specimen as tested (saturated, as received, air dried, oven dried, etc)"
      },
      {
        "name": "RCAG_GSIZ",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Maximum grain size"
      },
      {
        "name": "RCAG_ANIS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Planes of weakness or anisotropy present (bedding, schistosity, etc)"
      },
      {
        "name": "RCAG_MACH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of apparatus"
      },
      {
        "name": "RCAG_MMTD",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Measurement method (side view, top view, optical, digital)"
      },
      {
        "name": "RCAG_CAIM",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "CAI mean value"
      },
      {
        "name": "RCAG_CAIS",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "CAI standard deviation"
      },
      {
        "name": "RCAG_ABCL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Abrasiveness classification"
      },
      {
        "name": "RCAG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "RCAG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "RCAG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "RCAG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "RCAT",
    "contents": "Rock Abrasiveness Tests - Data",
    "parent": "RCAG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "RCAT_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Measurement number"
      },
      {
        "name": "RCAT_CUT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Surface condition (rough, saw-cut)"
      },
      {
        "name": "RCAT_SDIR",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Direction of scratching with respect to planes of weakness or anisotropy"
      },
      {
        "name": "RCAT_STYH",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Rockwell hardness HRC of stylus"
      },
      {
        "name": "RCAT_STYC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stylus condition (new or re-sharpened)"
      },
      {
        "name": "RCAT_CAI",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "As measured CAI value"
      },
      {
        "name": "RCAT_CAIS",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "Equivalent CAI value at standard stylus hardness HRC 55"
      },
      {
        "name": "RCAT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "RESC",
    "contents": "Resonant Column Test - Consolidation",
    "parent": "RESG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "RESC_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test / Stage Number"
      },
      {
        "name": "RESC_SDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen diameter"
      },
      {
        "name": "RESC_HIGH",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen height"
      },
      {
        "name": "RESC_CTYP",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of consolidation"
      },
      {
        "name": "RESC_ELAP",
        "status": "OTHER",
        "type": "T",
        "unit": "hh:mm:ss",
        "description": "Duration of stage"
      },
      {
        "name": "RESC_CHGT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen height at end of test/stage"
      },
      {
        "name": "RESC_CDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen diameter at end of test/stage"
      },
      {
        "name": "RESC_CMC",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Water content at end of test/stage"
      },
      {
        "name": "RESC_CDDN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Dry density at end of test/stage"
      },
      {
        "name": "RESC_CRD",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Relative density at end of test/stage"
      },
      {
        "name": "RESC_INCE",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Voids ratio at end of test/stage"
      },
      {
        "name": "RESC_EASC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Effective axial stress during consolidation at end of test/stage"
      },
      {
        "name": "RESC_ERSC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Effective radial stress during consolidation at end of test/stage"
      },
      {
        "name": "RESC_DEVS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Deviatoric stress at end of test/stage"
      },
      {
        "name": "RESC_SHRS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Shear stress at end of test/stage"
      },
      {
        "name": "RESC_MNES",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Mean effective stress at end of test/stage"
      },
      {
        "name": "RESC_AXSN",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Axial strain at end of test/stage"
      },
      {
        "name": "RESC_VLSN",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Volumetric strain from measured volume change at end of test/stage"
      },
      {
        "name": "RESC_RDSN",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Radial strain from measured volume change"
      },
      {
        "name": "RESC_BESE",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Bender element test sequence"
      },
      {
        "name": "RESC_BEAX",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Bender element axis of measurement"
      },
      {
        "name": "RESC_DBTE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Distance between bender elements"
      },
      {
        "name": "RESC_MAT",
        "status": "OTHER",
        "type": "4DP",
        "unit": "s",
        "description": "Measured arrival time of propagated wave"
      },
      {
        "name": "RESC_MATM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of measuring arrival time of propagated wave"
      },
      {
        "name": "RESC_SWV",
        "status": "OTHER",
        "type": "0DP",
        "unit": "m/s",
        "description": "Calculated shear wave velocity"
      },
      {
        "name": "RESC_SMGM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Shear modulus Gmax from bender elements"
      },
      {
        "name": "RESC_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "RESD",
    "contents": "Resonant Column Test - Data",
    "parent": "RESG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "RESD_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test / Stage Number"
      },
      {
        "name": "RESD_MNUM",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Measurement Number"
      },
      {
        "name": "RESD_CNDS",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Test Conditions"
      },
      {
        "name": "RESD_SDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen Diameter"
      },
      {
        "name": "RESD_HIGH",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Specimen Height"
      },
      {
        "name": "RESD_CELL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Cell Pressure"
      },
      {
        "name": "RESD_BP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Back Pressure"
      },
      {
        "name": "RESD_AXL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Axial Stress"
      },
      {
        "name": "RESD_BPWP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Base Pore Water Pressure"
      },
      {
        "name": "RESD_MPWP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Mid-height Pore Water Pressure"
      },
      {
        "name": "RESD_PPR",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Pore Pressure Ratio"
      },
      {
        "name": "RESD_PWPM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Maximum Excess Pore Water Pressure"
      },
      {
        "name": "RESD_EAS",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "External Axial Strain"
      },
      {
        "name": "RESD_VOL",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Volumetric Strain"
      },
      {
        "name": "RESD_DEV",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Principal Stress Difference"
      },
      {
        "name": "RESD_MEES",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Mean Effective Stress"
      },
      {
        "name": "RESD_MIPS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Minor Principal Stress (sigma 3)"
      },
      {
        "name": "RESD_MAPS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Major Principal Stress (sigma 1)"
      },
      {
        "name": "RESD_AVSS",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Average Shear Strain"
      },
      {
        "name": "RESD_SM",
        "status": "OTHER",
        "type": "2DP",
        "unit": "MPa",
        "description": "Shear Modulus"
      },
      {
        "name": "RESD_DMP",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Damping"
      },
      {
        "name": "RESD_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "RESG",
    "contents": "Resonant Column Test - General",
    "parent": "SAMP",
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
        "description": "Depth to top of test specimen"
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
        "description": "Details of specimen preparation including time between preparation and testing"
      },
      {
        "name": "SPEC_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of specimen"
      },
      {
        "name": "RESG_COND",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Sample condition"
      },
      {
        "name": "RESG_CONS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Specific condition statements"
      },
      {
        "name": "RESG_DRAG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of Drainage"
      },
      {
        "name": "RESG_ORNT",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Orientation of Specimen"
      },
      {
        "name": "RESG_SDIA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Initial specimen diameter"
      },
      {
        "name": "RESG_HIGT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Initial specimen Height"
      },
      {
        "name": "RESG_MCI",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Initial Water/moisture Content"
      },
      {
        "name": "RESG_MCF",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Final Water/moisture Content"
      },
      {
        "name": "RESG_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial Bulk Density"
      },
      {
        "name": "RESG_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Initial Dry Density"
      },
      {
        "name": "RESG_MIDD",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Minimum dry density for sand"
      },
      {
        "name": "RESG_MADD",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Maximum dry density for sand"
      },
      {
        "name": "RESG_IRDI",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Initial relative density index"
      },
      {
        "name": "RESG_IVR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Initial void ratio"
      },
      {
        "name": "RESG_ISAT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Initial degree of saturation"
      },
      {
        "name": "RESG_PDEN",
        "status": "OTHER",
        "type": "XN",
        "unit": "Mg/m3",
        "description": "Particle density with prefix # if value assumed"
      },
      {
        "name": "RESG_DAMP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Damping measurement method"
      },
      {
        "name": "RESG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviation from the specified procedure"
      },
      {
        "name": "RESG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "RESG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "RESG_LAB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing laboratory/organization"
      },
      {
        "name": "RESG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number ({when appropriate)"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test Status"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "RESP",
    "contents": "Resonant Column Test - Derived Parameters",
    "parent": "RESD",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "RESD_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test / Stage Number"
      },
      {
        "name": "RESD_MNUM",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Measurement Number"
      },
      {
        "name": "RESP_CTYP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of Consolidation"
      },
      {
        "name": "RESP_CSTG",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Consolidation Stage"
      },
      {
        "name": "RESP_CELL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Isotropic/Anisotropic Consolidation Cell Pressure"
      },
      {
        "name": "RESP_BACK",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Isotropic/Anisotropic Consolidation Back Pressure"
      },
      {
        "name": "RESP_ERSC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Effective Radial Stress During Consolidation"
      },
      {
        "name": "RESP_EASC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Effective Axial Stress During Consolidation"
      },
      {
        "name": "RESP_DEV",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Deviator Stress at End of Isotropic/Anisotropic Consolidation"
      },
      {
        "name": "RESP_VOLS",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Change to Volumetric Strain During Isotropic/Anisotropic Consolidation"
      },
      {
        "name": "RESP_STRN",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Axial Strain After Isotropic/Anisotropic Consolidation"
      },
      {
        "name": "RESP_SMOD",
        "status": "OTHER",
        "type": "2DP",
        "unit": "MPa",
        "description": "Shear Modulus G0"
      },
      {
        "name": "RESP_SSTR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Mean Effective Stress"
      },
      {
        "name": "RESP_DAMP",
        "status": "OTHER",
        "type": "4DP",
        "unit": "%",
        "description": "Damping Ratio"
      },
      {
        "name": "RESP_SMRA",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Normalised Shear Modulus by Maximum Shear Modulus"
      },
      {
        "name": "RESP_SR",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Slippage Ratio"
      },
      {
        "name": "RESP_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "RESS",
    "contents": "Resonant Column Test - Saturation",
    "parent": "RESG",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "RESS_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test / Stage Number"
      },
      {
        "name": "RESS_INC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Pressure increment"
      },
      {
        "name": "RESS_DIFF",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Differential pressure used"
      },
      {
        "name": "RESS_CELL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Final cell pressure"
      },
      {
        "name": "RESS_BPWP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Final base porewater pressure"
      },
      {
        "name": "RESS_STRN",
        "status": "OTHER",
        "type": "3DP",
        "unit": "%",
        "description": "Final axial strain"
      },
      {
        "name": "RESS_MCF",
        "status": "OTHER",
        "type": "X",
        "unit": "%",
        "description": "Final water content"
      },
      {
        "name": "RESS_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Final bulk density"
      },
      {
        "name": "RESS_DDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Final dry density"
      },
      {
        "name": "RESS_FVR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Final voids ratio"
      },
      {
        "name": "RESS_FSAT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Final degree of saturation"
      },
      {
        "name": "RESS_B",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Final B value"
      },
      {
        "name": "RESS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "WGPG",
    "contents": "Wireline Geophysics - General",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "WGPG_ID",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "WGPG_TOOL",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Tool used"
      },
      {
        "name": "WGPG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "WGPG_STRT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Test start depth"
      },
      {
        "name": "WGPG_STOP",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Test stop depth"
      },
      {
        "name": "WGPG_BHD",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of borehole"
      },
      {
        "name": "WGPG_WAT",
        "status": "OTHER",
        "type": "XN",
        "unit": "m",
        "description": "Depth of water in borehole"
      },
      {
        "name": "WGPG_DETL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of instrument"
      },
      {
        "name": "WGPG_CDIA",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Casing internal diameter as reported by drillers"
      },
      {
        "name": "WGPG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "WGPG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "WGPG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Measurement method"
      },
      {
        "name": "WGPG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Contractor who undertook testing"
      },
      {
        "name": "WGPG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (Where appropriate)"
      },
      {
        "name": "WGPG_STAT",
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      },
      {
        "name": "WGPG_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      },
      {
        "name": "WGPG_LIM",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Instrument/method reading/detection limit"
      },
      {
        "name": "WGPG_ULIM",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Instrument/method upper reading detection (when appropriate)"
      }
    ]
  },
  {
    "code": "WGPT",
    "contents": "Wireline Geophysics - Readings",
    "parent": "WGPG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "WGPG_ID",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "WGPG_TOOL",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Tool used"
      },
      {
        "name": "WGPT_PARA",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Parameter recorded by tool WGPG_TOOL"
      },
      {
        "name": "WGPT_UNIT",
        "status": "KEY+REQUIRED",
        "type": "PU",
        "unit": null,
        "description": "Test result units"
      },
      {
        "name": "WGPT_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of reading"
      },
      {
        "name": "WGPT_RDNG",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Reading"
      },
      {
        "name": "WGPT_CAS",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Borehole casing details at depth of reading"
      },
      {
        "name": "WGPT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "CBRP",
    "contents": "California Bearing Ratio Tests - Readings",
    "parent": "CBRT",
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
        "description": "Depth to top of test specimen"
      },
      {
        "name": "CBRT_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "CBRP_END",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Sample end"
      },
      {
        "name": "CBRP_PEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Penetration"
      },
      {
        "name": "CBRP_LOAD",
        "status": "OTHER",
        "type": "3DP",
        "unit": "kN",
        "description": "Force/Load"
      }
    ]
  },
  {
    "code": "CPDG",
    "contents": "Pore Pressure Dissipation Tests (PPDT) - General",
    "parent": "CPTG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "CPTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference or push number"
      },
      {
        "name": "CPDG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Inclination corrected depth of dissipation test"
      },
      {
        "name": "CPDG_IR",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "Rigidity index used in analysis"
      },
      {
        "name": "CPDG_RCMP",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Were the rods clamped during test?"
      },
      {
        "name": "CPDG_UI",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Measured, corrected or assumed initial pore water pressure (u_i)"
      },
      {
        "name": "CPDG_UIP",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Procedure to define initial pore water pressure"
      },
      {
        "name": "CPDG_M",
        "status": "OTHER",
        "type": "3DP",
        "unit": "kPa/s",
        "description": "Gradient of extrapolation line on square root time graph"
      },
      {
        "name": "CPDG_UEQ",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Measured or assumed equilibrium pore water pressure (u_o)"
      },
      {
        "name": "CPDG_UEP",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Procedure to define equilibrium pore water"
      },
      {
        "name": "CPDG_DDIS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Degree of dissipation for analysis"
      },
      {
        "name": "CPDG_T",
        "status": "OTHER",
        "type": "1DP",
        "unit": "s",
        "description": "Time to achieve degree of dissipation stated in CPDG_DDIS"
      },
      {
        "name": "CPDG_CH",
        "status": "OTHER",
        "type": "2SCI",
        "unit": "m2/yr",
        "description": "Coefficient of consolidation (horizontal), c_h"
      },
      {
        "name": "CPDG_CHMT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method(s) used to determine horizontal coefficient of consolidation"
      },
      {
        "name": "CPDG_CV",
        "status": "OTHER",
        "type": "2SCI",
        "unit": "m2/yr",
        "description": "Coefficient of consolidation (vertical), c_v"
      },
      {
        "name": "CPDG_CVMT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method(s) used to determine vertical coefficient of consolidation"
      },
      {
        "name": "CPDG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks, note if data is recorded as whole seconds"
      },
      {
        "name": "CPDG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Test date and time"
      },
      {
        "name": "CPDG_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      },
      {
        "name": "CPDG_ANBY",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name(s) of analyser / person responsible for data quality and correctness"
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
        "description": "Associated file reference (e.g. equipment calibrations)"
      }
    ]
  },
  {
    "code": "CPDT",
    "contents": "Pore Pressure Dissipation Tests (PPDT) - Data",
    "parent": "CPDG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "CPTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference or push number"
      },
      {
        "name": "CPDG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Inclination corrected depth of dissipation test"
      },
      {
        "name": "CPDT_TIME",
        "status": "KEY",
        "type": "1DP",
        "unit": "s",
        "description": "Elapsed time since start of test"
      },
      {
        "name": "CPDT_QC",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Cone resistance (q_c)"
      },
      {
        "name": "CPDT_TF",
        "status": "OTHER",
        "type": "3DP",
        "unit": "kN",
        "description": "Total force or thrust"
      },
      {
        "name": "CPDT_FS",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Sleeve friction (f_s)"
      },
      {
        "name": "CPDT_U1",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Face porewater pressure (u_1)"
      },
      {
        "name": "CPDT_U2",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Shoulder porewater pressure (u_2)"
      },
      {
        "name": "CPDT_U3",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Top of sleeve porewater pressure (u_3)"
      },
      {
        "name": "CPDT_TMPI",
        "status": "OTHER",
        "type": "1DP",
        "unit": "DegC",
        "description": "Cone internal temperature. If multiple temperature sensors exist, then the sensor closest to the pore pressure sensors should be used."
      },
      {
        "name": "CPDT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Comments"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "CPTG",
    "contents": "Cone Penetration Test (CPT/CPTu) - General",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "CPTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference or push number"
      },
      {
        "name": "CPTG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Cone test type"
      },
      {
        "name": "CPTG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Date time at beginning of test or push"
      },
      {
        "name": "CPTG_PED",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Pre-drilled depth"
      },
      {
        "name": "CPTG_RATE",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm/s",
        "description": "Nominal rate of penetration of the cone"
      },
      {
        "name": "CPTG_ORNT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": "Orientation of inclination X from North"
      },
      {
        "name": "CPTG_RLOC",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "The location where the reference reading (pretest zero) of the test was performed"
      },
      {
        "name": "CPTG_WAT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to groundwater level, z_w at time of test. Negative for water levels above the location where the reference reading was performed."
      },
      {
        "name": "CPTG_WATA",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Origin of groundwater level in CPTG_WAT"
      },
      {
        "name": "CPTG_TERM",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Termination reason(s)"
      },
      {
        "name": "CPTG_REF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Cone reference"
      },
      {
        "name": "CPTG_MAN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Manufacturer of cone penetrometer"
      },
      {
        "name": "CPTG_FILL",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Filter location(s)"
      },
      {
        "name": "CPTG_CSA",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm2",
        "description": "Cross sectional area of cone tip, Ball and T-Bar"
      },
      {
        "name": "CPTG_CSAN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm2",
        "description": "Nominal cross sectional area of cone tip, Ball and T-Bar"
      },
      {
        "name": "CPTG_CAR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Cone area ratio used to calculate qt, also use for Ball and T-Bar"
      },
      {
        "name": "CPTG_SLA",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm2",
        "description": "Friction sleeve area"
      },
      {
        "name": "CPTG_SLAN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm2",
        "description": "Nominal friction sleeve area"
      },
      {
        "name": "CPTG_SHA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm2",
        "description": "Cross-sectional area of the connecting shaft for Ball and T-Bar"
      },
      {
        "name": "CPTG_SLAR",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Sleeve area ratio used to calculate ft"
      },
      {
        "name": "CPTG_CFOS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Shoulder of cone to centre of friction sleeve offset (physical measurement)"
      },
      {
        "name": "CPTG_CFOA",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Shoulder of cone to centre of friction sleeve offset used by analysis"
      },
      {
        "name": "CPTG_TBL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "T-Bar length"
      },
      {
        "name": "CPTG_TBD",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "T-Bar diameter"
      },
      {
        "name": "CPTG_CPC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Nominal cone maximum tip pressure capacity (assumed zero load on sleeve for subtraction cones and purely axial load)"
      },
      {
        "name": "CPTG_FPC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Nominal friction maximum pressure capacity"
      },
      {
        "name": "CPTG_UPC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Nominal porewater pressure maximum pressure capacity"
      },
      {
        "name": "CPTG_CPCL",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Cone penetrometer class"
      },
      {
        "name": "CPTG_CRDT",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date of last calibration of cone"
      },
      {
        "name": "CPTG_CDDT",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date of last calibration of data logger (applicable to analogue cones)"
      },
      {
        "name": "CPTG_LCA",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Load cell arrangement"
      },
      {
        "name": "CPTG_FILT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of filter material used"
      },
      {
        "name": "CPTG_FRIC",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Friction reducer used"
      },
      {
        "name": "CPTG_FRID",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Friction reducer distance behind the shoulder of the cone tip"
      },
      {
        "name": "CPTG_FRIS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Friction reducer diameter"
      },
      {
        "name": "CPTG_SAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of saturation of pore pressure system and type of fluid used"
      },
      {
        "name": "CPTG_EQPT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Mass, reaction and equipment geometry"
      },
      {
        "name": "CPTG_APCL",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Test application class or category described in the standard"
      },
      {
        "name": "CPTG_DAZV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Description of application of zero values"
      },
      {
        "name": "CPTG_CORR",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Corrections applied during data processing (e.g. depth corrections, removal of rod change spikes, zeros)"
      },
      {
        "name": "CPTG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Comments on testing and basis of any interpreted parameters included in CPTT"
      },
      {
        "name": "CPTG_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      },
      {
        "name": "CPTG_ANBY",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name(s) of analyser / person responsible for data quality and correctness"
      },
      {
        "name": "CPTG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "CPTG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Standard followed for testing"
      },
      {
        "name": "CPTG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviations from the standard followed"
      },
      {
        "name": "CPTG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Subcontractor name"
      },
      {
        "name": "CPTG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
        "description": "Associated file reference (e.g. cone calibration records)"
      }
    ]
  },
  {
    "code": "CPTM",
    "contents": "Cone Penetration Test (CPT/CPTu) - Methods/references for Correlated Parameters",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "CPTM_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of Method"
      },
      {
        "name": "CPTM_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of Method, optional, if empty then method applies to maximum depth of LOCA_ID"
      },
      {
        "name": "CPTM_SBT1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Soil Behaviour Type"
      },
      {
        "name": "CPTM_SU1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Undrained Shear Strength (s_u) 1, could be used for lower estimate; fine soils"
      },
      {
        "name": "CPTM_SU2",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Undrained Shear Strength (s_u) 2, could be used for upper estimate; fine soils"
      },
      {
        "name": "CPTM_DR1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Relative density (D_r) 1, could be used for lower estimate; coarse soils"
      },
      {
        "name": "CPTM_DR2",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Relative density (D_r) 2, could be used for upper estimate; coarse soils"
      },
      {
        "name": "CPTM_PHI1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Internal Friction Angle; coarse soils"
      },
      {
        "name": "CPTM_IC1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Soil Behaviour Type Index (I_c)"
      },
      {
        "name": "CPTM_N601",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Equivalent SPT N_60 value"
      },
      {
        "name": "CPTM_E1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Young's Modulus, E"
      },
      {
        "name": "CPTM_MV1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Coefficient of Volume Change, m_v"
      },
      {
        "name": "CPTM_G01",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Small strain shear modulus, G_0"
      },
      {
        "name": "CPTM_VS1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Shear wave velocity (correlated), V_s"
      },
      {
        "name": "CPTM_DUW1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Dry unit weight, gamma_d"
      },
      {
        "name": "CPTM_SUW1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Saturated unit weight, gamma_s"
      },
      {
        "name": "CPTM_M1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Constrained modulus, M"
      },
      {
        "name": "CPTM_CC1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Compression index, C_C"
      },
      {
        "name": "CPTM_P01",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Preconsolidation stress, p_0'"
      },
      {
        "name": "CPTM_ST1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Sensitivity, S_t"
      },
      {
        "name": "CPTM_K01",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Coefficient of lateral earth pressure, K_0"
      },
      {
        "name": "CPTM_IR1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Rigidity index, I_r"
      },
      {
        "name": "CPTM_K1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Permeability, k"
      },
      {
        "name": "CPTM_FC1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Fines content, FC"
      },
      {
        "name": "CPTM_CSR1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Cyclic stress ratio, CSR"
      },
      {
        "name": "CPTM_CRR1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for Cyclic resistance ratio, CRR"
      },
      {
        "name": "CPTM_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "CPTP",
    "contents": "Cone Penetration Test (CPT/CPTu) - Correlated Parameters",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "CPTP_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of each reading (CPTT_DPTH) or depth to top of layer"
      },
      {
        "name": "CPTP_BASE",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of layer, optional"
      },
      {
        "name": "CPTP_SBT1",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Soil Behaviour Type"
      },
      {
        "name": "CPTP_SU1",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Undrained Shear Strength (s_u) 1, could be used for lower estimate; fine soils"
      },
      {
        "name": "CPTP_SU2",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Undrained Shear Strength (s_u) 2, could be used for upper estimate; fine soils"
      },
      {
        "name": "CPTP_DR1",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Relative density (D_r) 1, could be used for lower estimate; coarse soils"
      },
      {
        "name": "CPTP_DR2",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Relative density (D_r) 2, could be used for upper estimate; coarse soils"
      },
      {
        "name": "CPTP_PHI1",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Internal Friction Angle; coarse soils"
      },
      {
        "name": "CPTP_IC1",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "Soil Behaviour Type Index (I_c)"
      },
      {
        "name": "CPTP_N601",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Equivalent SPT N_60 value"
      },
      {
        "name": "CPTP_E1",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Young's Modulus, E"
      },
      {
        "name": "CPTP_MV1",
        "status": "OTHER",
        "type": "2SCI",
        "unit": "m2/MN",
        "description": "Coefficient of Volume Change, m_v"
      },
      {
        "name": "CPTP_G01",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Small strain shear modulus, G_0"
      },
      {
        "name": "CPTP_VS1",
        "status": "OTHER",
        "type": "1DP",
        "unit": "m/s",
        "description": "Shear wave velocity (correlated), V_s"
      },
      {
        "name": "CPTP_DUW1",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kN/m3",
        "description": "Dry unit weight, gamma_d"
      },
      {
        "name": "CPTP_SUW1",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kN/m3",
        "description": "Saturated unit weight, gamma_s"
      },
      {
        "name": "CPTP_M1",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Constrained modulus, M"
      },
      {
        "name": "CPTP_CC1",
        "status": "OTHER",
        "type": "2SCI",
        "unit": null,
        "description": "Compression index, C_C"
      },
      {
        "name": "CPTP_P01",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Preconsolidation stress, p_0'"
      },
      {
        "name": "CPTP_ST1",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "Sensitivity, S_t"
      },
      {
        "name": "CPTP_K01",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "Coefficient of lateral earth pressure, K_0"
      },
      {
        "name": "CPTP_IR1",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "Rigidity index, I_r"
      },
      {
        "name": "CPTP_K1",
        "status": "OTHER",
        "type": "1SCI",
        "unit": "m/s",
        "description": "Permeability, k"
      },
      {
        "name": "CPTP_FC1",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Fines content, FC"
      },
      {
        "name": "CPTP_CSR1",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Cyclic stress ratio, CSR"
      },
      {
        "name": "CPTP_CRR1",
        "status": "OTHER",
        "type": "3DP",
        "unit": null,
        "description": "Cyclic resistance ratio, CRR"
      },
      {
        "name": "CPTP_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "CPTT",
    "contents": "Cone Penetration Test (CPT/CPTu) - Data",
    "parent": "CPTG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "CPTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference or push number"
      },
      {
        "name": "CPTT_REDN",
        "status": "KEY",
        "type": "0DP",
        "unit": null,
        "description": "Sequence number, incrementing number defining order of records"
      },
      {
        "name": "CPTT_DPTH",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of result corrected for inclination"
      },
      {
        "name": "CPTT_PLEN",
        "status": "OTHER",
        "type": "U",
        "unit": "m",
        "description": "Recorded penetration length; Recommend types: 2DP or 3DP"
      },
      {
        "name": "CPTT_QC",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Cone resistance (q_c), or measured Ball and T-Bar resistance (q_m)"
      },
      {
        "name": "CPTT_FS",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Sleeve friction (f_s)"
      },
      {
        "name": "CPTT_U1",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Face porewater pressure (u_1)"
      },
      {
        "name": "CPTT_U2",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Shoulder porewater pressure (u_2)"
      },
      {
        "name": "CPTT_U3",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Top of sleeve porewater pressure (u_3)"
      },
      {
        "name": "CPTT_INCX",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Inclination X"
      },
      {
        "name": "CPTT_INCY",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Inclination Y"
      },
      {
        "name": "CPTT_TIME",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss.sss",
        "description": "Clock time during the test"
      },
      {
        "name": "CPTT_DUR",
        "status": "OTHER",
        "type": "1DP",
        "unit": "s",
        "description": "Duration since start of test"
      },
      {
        "name": "CPTT_TF",
        "status": "OTHER",
        "type": "3DP",
        "unit": "kN",
        "description": "Total force or thrust"
      },
      {
        "name": "CPTT_RF",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Friction ratio (R_f)"
      },
      {
        "name": "CPTT_BDEN",
        "status": "OTHER",
        "type": "2DP",
        "unit": "Mg/m3",
        "description": "Bulk density of material (measured or assumed)"
      },
      {
        "name": "CPTT_CPO",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Total vertical stress (based on CPTT_BDEN)"
      },
      {
        "name": "CPTT_ISPP",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "In situ pore pressure (u_o) (measured or assumed where not simple hydrostatic based on CPTG_WAT)"
      },
      {
        "name": "CPTT_CPOD",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Effective vertical stress (calculated from CPTT_CPO and CPTT_ISPP or CPTG_WAT)"
      },
      {
        "name": "CPTT_QT",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Corrected cone resistance (q_t) piezocone only"
      },
      {
        "name": "CPTT_FT",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Corrected sleeve resistance (f_t) piezocone only"
      },
      {
        "name": "CPTT_QNET",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Net cone resistance (q_n), or net Ball and T-bar resistance (q_ball and q_T-bar)"
      },
      {
        "name": "CPTT_QE",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Effective cone resistance (qe) piezocone only"
      },
      {
        "name": "CPTT_RFT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Corrected friction ratio (R_ft) piezocone only"
      },
      {
        "name": "CPTT_EXPP",
        "status": "OTHER",
        "type": "4DP",
        "unit": "MPa",
        "description": "Excess pore pressure (u-u_o) piezocone only"
      },
      {
        "name": "CPTT_BQ",
        "status": "OTHER",
        "type": "4DP",
        "unit": null,
        "description": "Pore pressure ratio (B_q) piezocone only"
      },
      {
        "name": "CPTT_NQT",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Normalised cone resistance (Q_t)"
      },
      {
        "name": "CPTT_NFR",
        "status": "OTHER",
        "type": "2DP",
        "unit": "%",
        "description": "Normalised friction ratio (F_r)"
      },
      {
        "name": "CPTT_MAGX",
        "status": "OTHER",
        "type": "0DP",
        "unit": "nT",
        "description": "Magnetic flux - X"
      },
      {
        "name": "CPTT_MAGY",
        "status": "OTHER",
        "type": "0DP",
        "unit": "nT",
        "description": "Magnetic flux - Y"
      },
      {
        "name": "CPTT_MAGZ",
        "status": "OTHER",
        "type": "0DP",
        "unit": "nT",
        "description": "Magnetic flux - Z"
      },
      {
        "name": "CPTT_MAGT",
        "status": "OTHER",
        "type": "1DP",
        "unit": "nT",
        "description": "Magnetic flux - Total (calculated)"
      },
      {
        "name": "CPTT_MAGG",
        "status": "OTHER",
        "type": "1DP",
        "unit": "nT/cm",
        "description": "Magnetic flux - Gradient (calculated)"
      },
      {
        "name": "CPTT_CON",
        "status": "OTHER",
        "type": "0DP",
        "unit": "uS/cm",
        "description": "Conductivity"
      },
      {
        "name": "CPTT_TEMP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "DegC",
        "description": "Soil temperature"
      },
      {
        "name": "CPTT_TPQC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "DegC",
        "description": "Temperature associated with tip sensor. Use this heading if there is one temperature sensor."
      },
      {
        "name": "CPTT_TPFS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "DegC",
        "description": "Temperature associated with sleeve sensor"
      },
      {
        "name": "CPTT_TPU",
        "status": "OTHER",
        "type": "1DP",
        "unit": "DegC",
        "description": "Temperature associated with pore pressure sensor"
      },
      {
        "name": "CPTT_PH",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "pH reading"
      },
      {
        "name": "CPTT_REDX",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mV",
        "description": "Redox potential"
      },
      {
        "name": "CPTT_SMP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "%",
        "description": "Soil moisture"
      },
      {
        "name": "CPTT_NGAM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "counts/s",
        "description": "Natural gamma radiation"
      },
      {
        "name": "CPTT_FFD1",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Fluorescence intensity 1"
      },
      {
        "name": "CPTT_FFD2",
        "status": "OTHER",
        "type": "0DP",
        "unit": "%",
        "description": "Fluorescence intensity 2"
      },
      {
        "name": "CPTT_PID",
        "status": "OTHER",
        "type": "0DP",
        "unit": "uV",
        "description": "Photo ionization detector"
      },
      {
        "name": "CPTT_FID",
        "status": "OTHER",
        "type": "0DP",
        "unit": "uV",
        "description": "Flame ionization detector"
      },
      {
        "name": "CPTT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. raw field data)"
      }
    ]
  },
  {
    "code": "CPTY",
    "contents": "Cone Penetration Test (CPT/CPTu) - Cyclic Tests",
    "parent": "CPTG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "CPTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference or push number"
      },
      {
        "name": "CPTY_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Cyclic test number"
      },
      {
        "name": "CPTY_DPTH",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Top depth of cyclic test corrected for inclination"
      },
      {
        "name": "CPTY_DINT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth Interval of cyclic test"
      },
      {
        "name": "CPTY_NUMC",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Number of full cycles in cyclic test"
      },
      {
        "name": "CPTY_REDI",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Initial reading number of cyclic test"
      },
      {
        "name": "CPTY_REDF",
        "status": "OTHER",
        "type": "0DP",
        "unit": null,
        "description": "Final reading number of cyclic test"
      },
      {
        "name": "CPTY_TIMI",
        "status": "OTHER",
        "type": "1DP",
        "unit": "s",
        "description": "Initial elapsed time (CPTT_TIME) of cyclic test"
      },
      {
        "name": "CPTY_TIMF",
        "status": "OTHER",
        "type": "1DP",
        "unit": "s",
        "description": "Final elapsed time (CPTT_TIME) of cyclic test"
      },
      {
        "name": "CPTY_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks, including early termination reason"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. raw field data)"
      }
    ]
  },
  {
    "code": "CPTZ",
    "contents": "Cone Penetration Test (CPT/CPTu) - Zeros",
    "parent": "CPTG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "CPTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference or push number"
      },
      {
        "name": "CPTZ_PARM",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Measured Parameter"
      },
      {
        "name": "CPTZ_ZBD",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Zero before at deck/surface (for over water testing where CPTG_RLOC is BB or SB)"
      },
      {
        "name": "CPTZ_ZB",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Zero before at reference/test level"
      },
      {
        "name": "CPTZ_ZA",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Zero after at reference/test level"
      },
      {
        "name": "CPTZ_ZAD",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Zero after at deck/surface (for over water testing where CPTG_RLOC is BB or SB)"
      },
      {
        "name": "CPTZ_ZAC",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Zero after test when cone has been cleaned at test level/deck/surface"
      },
      {
        "name": "CPTZ_ZD",
        "status": "OTHER",
        "type": "4DP",
        "unit": null,
        "description": "Zero drift between reference readings, CPTZ_ZA - CPTZ_ZB"
      },
      {
        "name": "CPTZ_ZDD",
        "status": "OTHER",
        "type": "4DP",
        "unit": null,
        "description": "Zero drift between deck/surface readings, CPTZ_ZAD - CPTZ_ZBD"
      },
      {
        "name": "CPTZ_ZDC",
        "status": "OTHER",
        "type": "4DP",
        "unit": null,
        "description": "Zero drift between before and cleaned, CPTZ_ZAC - first of CPTZ_ZBD or CPTZ_ZB"
      },
      {
        "name": "CPTZ_CD",
        "status": "OTHER",
        "type": "4DP",
        "unit": null,
        "description": "Calibration drift between calibration or first test, and first of CPTZ_ZBD or CPTZ_ZB"
      },
      {
        "name": "CPTZ_ZS",
        "status": "OTHER",
        "type": "4DP",
        "unit": null,
        "description": "Zero output stability, the difference between maximum and minimum values recorded for one minute"
      },
      {
        "name": "CPTZ_ZSS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Origin of zero output stability"
      },
      {
        "name": "CPTZ_ZVUC",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Zero value used in calculation"
      },
      {
        "name": "CPTZ_EGUT",
        "status": "OTHER",
        "type": "PU",
        "unit": null,
        "description": "Engineering unit for CPTZ_ZBD, CPTZ_ZB, CPTZ_ZA, CPTZ_ZAD, CPTZ_ZAC, CPTG_ZD, CPTG_CD and CPTG_ZS"
      },
      {
        "name": "CPTZ_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "DMDG",
    "contents": "Flat Dilatometer Dissipation Test - General",
    "parent": "DMTG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "DMTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "DMDG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of dissipation test"
      },
      {
        "name": "DMDG_TFLX",
        "status": "OTHER",
        "type": "2DP",
        "unit": "min",
        "description": "Time to point of inflection on dissipation curve (T_flex)"
      },
      {
        "name": "DMDG_CH",
        "status": "OTHER",
        "type": "2SCI",
        "unit": "m2/yr",
        "description": "Coefficient of consolidation (c_h), (horizontal), calculated from DMDG_TFLX"
      },
      {
        "name": "DMDG_CHMT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method(s) used to determine horizontal coefficient of consolidation"
      },
      {
        "name": "DMDG_MH",
        "status": "OTHER",
        "type": "2DP",
        "unit": "MPa",
        "description": "Horizontal constrained modulus, M_h"
      },
      {
        "name": "DMDG_MHMT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method(s) used to determine horizontal constrained modulus"
      },
      {
        "name": "DMDG_KH",
        "status": "OTHER",
        "type": "1SCI",
        "unit": "m/s",
        "description": "Horizontal coefficient of permeability (k_h), calculated from DMDG_TFLX"
      },
      {
        "name": "DMDG_KHMT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method(s) used to determine horizontal coefficient of permeability"
      },
      {
        "name": "DMDG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Test start date and time"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test status"
      },
      {
        "name": "DMDG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Note on set up conditions"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated File Reference"
      }
    ]
  },
  {
    "code": "DMDT",
    "contents": "Flat Dilatometer Dissipation Test - Data",
    "parent": "DMDG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "DMTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "DMDG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of dissipation test"
      },
      {
        "name": "DMDT_TIME",
        "status": "KEY",
        "type": "1DP",
        "unit": "s",
        "description": "Elapsed time since start of test"
      },
      {
        "name": "DMDT_A",
        "status": "OTHER",
        "type": "2DP",
        "unit": "kPa",
        "description": "A-pressure test reading"
      },
      {
        "name": "DMDT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Note on individual record"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated File Reference"
      }
    ]
  },
  {
    "code": "DMTG",
    "contents": "Flat Dilatometer Test - General",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "DMTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "DMTG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Test date and time"
      },
      {
        "name": "DMTG_ORNT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": "Angle that the membrane is pointing to"
      },
      {
        "name": "DMTG_PED",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Pre-drilled depth"
      },
      {
        "name": "DMTG_WAT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to groundwater level, z_w at time of test"
      },
      {
        "name": "DMTG_WATA",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Origin of groundwater level in DMTG_WAT"
      },
      {
        "name": "DMTG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Specific details on type of DMT equipment"
      },
      {
        "name": "DMTG_REFB",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Serial number of blade (if applicable)"
      },
      {
        "name": "DMTG_REFA",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Serial number of the acquisition unit (if applicable)"
      },
      {
        "name": "DMTG_MAN",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Manufacturer of the dilatometer"
      },
      {
        "name": "DMTG_RIG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type of penetration rig"
      },
      {
        "name": "DMTG_EQPT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Mass, reaction and equipment geometry"
      },
      {
        "name": "DMTG_COT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method and calibration of thrust measurement"
      },
      {
        "name": "DMTG_TDR",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Type and diameter of penetration rods"
      },
      {
        "name": "DMTG_DIMS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Geometry and dimensions of the dilatometer, as measured"
      },
      {
        "name": "DMTG_PRSG",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Measuring range of the pressure gauges and zero offset when vented"
      },
      {
        "name": "DMTG_FRIC",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of any rod friction reducer, including diameter"
      },
      {
        "name": "DMTG_DITH",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Membrane thickness"
      },
      {
        "name": "DMTG_BCVA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "kPa",
        "description": "Blade calibration value used, delta A"
      },
      {
        "name": "DMTG_BCVB",
        "status": "OTHER",
        "type": "2DP",
        "unit": "kPa",
        "description": "Blade calibration value used, delta B"
      },
      {
        "name": "DMTG_FAED",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Dilatometer modulus (Ed) factor"
      },
      {
        "name": "DMTG_FAS0",
        "status": "OTHER",
        "type": "1DP",
        "unit": "mm",
        "description": "Membrane displacement"
      },
      {
        "name": "DMTG_TERM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Termination reason(s)"
      },
      {
        "name": "DMTG_CORR",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Corrections applied during data processing (e.g. depth corrections, zeros)"
      },
      {
        "name": "DMTG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Note on set up conditions, comments on testing, types of materials encountered if possible"
      },
      {
        "name": "DMTG_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      },
      {
        "name": "DMTG_ANBY",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name(s) of analyser / person responsible for data quality and correctness"
      },
      {
        "name": "DMTG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "DMTG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Standard followed for testing"
      },
      {
        "name": "DMTG_DEV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Deviations from the standard followed"
      },
      {
        "name": "DMTG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Subcontractors name"
      },
      {
        "name": "DMTG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
      }
    ]
  },
  {
    "code": "DMTP",
    "contents": "Flat Dilatometer Test - Derived Parameters",
    "parent": "DMTT",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "DMTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "DMTT_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of result"
      },
      {
        "name": "DMTP_BUW",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kN/m3",
        "description": "Estimated bulk unit weight of soil, gamma (can be custom or correlation from software)"
      },
      {
        "name": "DMTP_TVS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Estimated total vertical stress, sigma_v, (based on DMTP_BUW)"
      },
      {
        "name": "DMTP_EVS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Estimated effective vertical stress, sigma'_v (Calculated from DMTP_TVS and DMTP_U0 or DMTG_WAT)"
      },
      {
        "name": "DMTP_U0",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "In situ pore pressure, u_o (can be custom or based on depth below DMTG_WAT)"
      },
      {
        "name": "DMTP_ID",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Material index, I_D"
      },
      {
        "name": "DMTP_KD",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "Horizontal stress index, K_D"
      },
      {
        "name": "DMTP_ED",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Dilatometer modulus, E_D"
      },
      {
        "name": "DMTP_UD",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Pore pressure index u_D"
      },
      {
        "name": "DMTP_VS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "m/s",
        "description": "Shear wave velocity (correlated), V_s"
      },
      {
        "name": "DMTP_VDM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Vertical drained constrained modulus, M"
      },
      {
        "name": "DMTP_SU",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Undrained shear strength, s_u, fine soils only"
      },
      {
        "name": "DMTP_PHI",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Effective angle of internal friction, phi', coarse soils only"
      },
      {
        "name": "DMTP_K0",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Coefficient of lateral earth pressure at rest, K_0, fine soils only"
      },
      {
        "name": "DMTP_THS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Estimated total horizontal stress, sigma_h"
      },
      {
        "name": "DMTP_EHS",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Estimated effective horizontal stress, sigma'_h"
      },
      {
        "name": "DMTP_OCR",
        "status": "OTHER",
        "type": "1DP",
        "unit": null,
        "description": "Over-consolidation ratio, OCR, fine soils only"
      },
      {
        "name": "DMTP_MPS",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Estimated maximum preconsolidation stress, sigma'_p, (calculated from DMTP_OCR and DMTP_EVS)"
      },
      {
        "name": "DMTP_DSD",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Interpreted soil description"
      },
      {
        "name": "DMTP_BUWM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for estimated bulk unit weight of soil, gamma_b"
      },
      {
        "name": "DMTP_TVSM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for estimated total vertical stress, sigma_v"
      },
      {
        "name": "DMTP_EVSM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for estimated effective vertical stress, sigma'_v"
      },
      {
        "name": "DMTP_U0M",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for in situ pore pressure, u_o"
      },
      {
        "name": "DMTP_IDM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for material index, I_D"
      },
      {
        "name": "DMTP_KDM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for horizontal stress index, K_D"
      },
      {
        "name": "DMTP_EDM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for dilatometer modulus, E_D"
      },
      {
        "name": "DMTP_UDM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for pore pressure index u_D"
      },
      {
        "name": "DMTP_VSM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for shear wave velocity (correlated), V_s"
      },
      {
        "name": "DMTP_VDMM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for vertical drained constrained modulus, M"
      },
      {
        "name": "DMTP_SUM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for undrained shear strength, s_u, fine soils only"
      },
      {
        "name": "DMTP_PHIM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for effective angle of internal friction, phi', coarse soils only"
      },
      {
        "name": "DMTP_K0M",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for coefficient of lateral earth pressure at rest, K_0, fine soils only"
      },
      {
        "name": "DMTP_THSM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for estimated total horizontal stress, sigma_h"
      },
      {
        "name": "DMTP_EHSM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for estimated effective horizontal stress, sigma'_h"
      },
      {
        "name": "DMTP_OCRM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for over-consolidation ratio, OCR, fine soils only"
      },
      {
        "name": "DMTP_MPSM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for estimated maximum preconsolidation stress, sigma'_p, (calculated from DMTP_OCR and DMTP_EVS)"
      },
      {
        "name": "DMTP_DSDM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method for interpreted soil description"
      },
      {
        "name": "DMTP_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "DMTT",
    "contents": "Flat Dilatometer Test - Data",
    "parent": "DMTG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "DMTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "DMTT_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of result"
      },
      {
        "name": "DMTT_MTH",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kg",
        "description": "Thrust"
      },
      {
        "name": "DMTT_BCVA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "kPa",
        "description": "Blade calibration value for specific depth record, delta A"
      },
      {
        "name": "DMTT_BCVB",
        "status": "OTHER",
        "type": "2DP",
        "unit": "kPa",
        "description": "Blade calibration value for specific depth record, delta B"
      },
      {
        "name": "DMTT_TMST",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Pressurisation start time"
      },
      {
        "name": "DMTT_A",
        "status": "OTHER",
        "type": "2DP",
        "unit": "kPa",
        "description": "A-pressure test reading"
      },
      {
        "name": "DMTT_TMA",
        "status": "OTHER",
        "type": "1DP",
        "unit": "s",
        "description": "A-position time since start of pressurisation"
      },
      {
        "name": "DMTT_B",
        "status": "OTHER",
        "type": "2DP",
        "unit": "kPa",
        "description": "B-pressure test reading"
      },
      {
        "name": "DMTT_TMB",
        "status": "OTHER",
        "type": "1DP",
        "unit": "s",
        "description": "B-position time since start of pressurisation"
      },
      {
        "name": "DMTT_C",
        "status": "OTHER",
        "type": "2DP",
        "unit": "kPa",
        "description": "C-pressure test reading"
      },
      {
        "name": "DMTT_TMC",
        "status": "OTHER",
        "type": "1DP",
        "unit": "s",
        "description": "C-position time since start of pressurisation"
      },
      {
        "name": "DMTT_P0",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Corrected test reading A, p_0"
      },
      {
        "name": "DMTT_P1",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Corrected test reading B, p_1"
      },
      {
        "name": "DMTT_P2",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Corrected test reading C, p_2"
      },
      {
        "name": "DMTT_INCX",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Inclination 1 (the axis through the membrane)"
      },
      {
        "name": "DMTT_INCY",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Inclination 2 (the axis across the width of the blade)"
      },
      {
        "name": "DMTT_RATE",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm/s",
        "description": "Penetration rate"
      },
      {
        "name": "DMTT_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks on specific depth readings"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "DMTZ",
    "contents": "Flat Dilatometer Test - Zeros",
    "parent": "DMTG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "DMTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "DMTZ_DATE",
        "status": "KEY",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Test date and time of zero readings"
      },
      {
        "name": "DMTZ_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "When were the zeros performed"
      },
      {
        "name": "DMTZ_BCVA",
        "status": "OTHER",
        "type": "2DP",
        "unit": "kPa",
        "description": "Blade zero value, delta A"
      },
      {
        "name": "DMTZ_BCVB",
        "status": "OTHER",
        "type": "2DP",
        "unit": "kPa",
        "description": "Blade zero value, delta B"
      },
      {
        "name": "DMTZ_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks on the zero values"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "ISTA",
    "contents": "In Situ Seismic Test - Analysis",
    "parent": "ISTG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "ISTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Setup reference"
      },
      {
        "name": "ISTA_TOP",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to top of analysis range"
      },
      {
        "name": "ISTA_BASE",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to base of analysis range"
      },
      {
        "name": "ISTA_ANYN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Analysis Reference"
      },
      {
        "name": "ISTA_DPTH",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Midpoint of analysis depth range"
      },
      {
        "name": "ISTA_RECT",
        "status": "OTHER",
        "type": "RL",
        "unit": null,
        "description": "Link(s) to signal receiver top record"
      },
      {
        "name": "ISTA_RECB",
        "status": "OTHER",
        "type": "RL",
        "unit": null,
        "description": "Link(s) to signal receiver bottom record"
      },
      {
        "name": "ISTA_RCOM",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Selected receiver component"
      },
      {
        "name": "ISTA_MIVL",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Selected method for interval velocity"
      },
      {
        "name": "ISTA_WVTY",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Wave type"
      },
      {
        "name": "ISTA_UPSR",
        "status": "OTHER",
        "type": "3DP",
        "unit": "ms",
        "description": "Up-sample rate"
      },
      {
        "name": "ISTA_FTU",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Filter type used"
      },
      {
        "name": "ISTA_FMIN",
        "status": "OTHER",
        "type": "0DP",
        "unit": "Hz",
        "description": "Minimum filter frequency"
      },
      {
        "name": "ISTA_FMAX",
        "status": "OTHER",
        "type": "0DP",
        "unit": "Hz",
        "description": "Maximum filter frequency"
      },
      {
        "name": "ISTA_WATT",
        "status": "OTHER",
        "type": "3DP",
        "unit": "ms",
        "description": "Wave arrival time top signal"
      },
      {
        "name": "ISTA_WATB",
        "status": "OTHER",
        "type": "3DP",
        "unit": "ms",
        "description": "Wave arrival time bottom signal"
      },
      {
        "name": "ISTA_WATM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method to assess wave arrival time"
      },
      {
        "name": "ISTA_ITM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method to assess interval time"
      },
      {
        "name": "ISTA_WVL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "m/s",
        "description": "Final wave velocity"
      },
      {
        "name": "ISTA_WVLM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method to assess wave velocity"
      },
      {
        "name": "ISTA_STAC",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Final shear wave velocity is based on stacked traces"
      },
      {
        "name": "ISTA_IVAL",
        "status": "OTHER",
        "type": "YN",
        "unit": null,
        "description": "Result invalid?"
      },
      {
        "name": "ISTA_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "ISTA_ANBY",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of analyser/ person responsible for data QAQC"
      },
      {
        "name": "ISTA_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Analysis subcontractors name"
      },
      {
        "name": "ISTA_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Analysis date"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Analysis status"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. ASCII .csv file containing logged instrumentation data)"
      }
    ]
  },
  {
    "code": "ISTG",
    "contents": "In Situ Seismic Test - General",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "ISTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Setup reference"
      },
      {
        "name": "ISTG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Seismic test type"
      },
      {
        "name": "ISTG_LINK",
        "status": "OTHER",
        "type": "RL",
        "unit": null,
        "description": "Record Link to pushing test, such as CPT"
      },
      {
        "name": "ISTG_STAR",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Date and time at the beginning of seismic setup reference"
      },
      {
        "name": "ISTG_END",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Date and time at the end of seismic setup reference"
      },
      {
        "name": "ISTG_REF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Seismic receiver module reference"
      },
      {
        "name": "ISTG_RECC",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Seismic receiver configuration. SINGLE for single receiver, DUAL for dual receiver test"
      },
      {
        "name": "ISTG_RECD",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Seismic receiver details, such as model"
      },
      {
        "name": "ISTG_SOUR",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Source, such as Hammer system"
      },
      {
        "name": "ISTG_RORD",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Recording equipment details"
      },
      {
        "name": "ISTG_SHOF",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Horizontal offset between centre of hole and source"
      },
      {
        "name": "ISTG_ORNT",
        "status": "OTHER",
        "type": "0DP",
        "unit": "deg",
        "description": "Orientation of source relative to the receivers in plan view. Ideally 90 deg, such that the waves are directed at the receivers"
      },
      {
        "name": "ISTG_SVOF",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Source measured vertical offset from ground level/seafloor (positive down)"
      },
      {
        "name": "ISTG_OTOP",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Offset between centre of the top receiver and the pushing device (CPT/DMT) tip. (Use this entry for both SINGLE and DUAL receiver setup.)"
      },
      {
        "name": "ISTG_OBOT",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Offset between centre of the bottom receiver and the pushing device (CPT/DMT) tip. (Only use this entry for DUAL receiver setup.)"
      },
      {
        "name": "ISTG_BHCP",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Borehole, how receiver is clamped in place"
      },
      {
        "name": "ISTG_MTO",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Method of determination of trigger time latency"
      },
      {
        "name": "ISTG_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      },
      {
        "name": "ISTG_ANBY",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of analyser/ person responsible for data QAQC"
      },
      {
        "name": "ISTG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks, including borehole state at time of test if needed"
      },
      {
        "name": "ISTG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "ISTG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "ISTG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Subcontractors name"
      },
      {
        "name": "ISTG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
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
      }
    ]
  },
  {
    "code": "ISTR",
    "contents": "In Situ Seismic Test - Signal Receiver",
    "parent": "ISTS",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "ISTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Setup reference"
      },
      {
        "name": "ISTS_SGLN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Signal reference"
      },
      {
        "name": "ISTR_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to receiver"
      },
      {
        "name": "ISTR_REF",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Receiver reference"
      },
      {
        "name": "ISTR_SSD",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Source slant distance"
      },
      {
        "name": "ISTR_QUAL",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Quality of received signal"
      },
      {
        "name": "ISTR_QUAM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method of quality assessment"
      },
      {
        "name": "ISTR_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "ISTS",
    "contents": "In Situ Seismic Test - Signal",
    "parent": "ISTG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "ISTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Setup reference"
      },
      {
        "name": "ISTS_SGLN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Signal reference"
      },
      {
        "name": "ISTS_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Source type"
      },
      {
        "name": "ISTS_DTIM",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Date and time of signal"
      },
      {
        "name": "ISTS_RATE",
        "status": "OTHER",
        "type": "3DP",
        "unit": "ms",
        "description": "Raw sampling rate"
      },
      {
        "name": "ISTS_PTRT",
        "status": "OTHER",
        "type": "1DP",
        "unit": "ms",
        "description": "Pre-trigger recording time"
      },
      {
        "name": "ISTS_TTLY",
        "status": "OTHER",
        "type": "3DP",
        "unit": "ms",
        "description": "Trigger time latency (positive for late recording)"
      },
      {
        "name": "ISTS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "ITCH",
    "contents": "In Situ Thermal Conductivity",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "ITCH_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of thermal conductivity test"
      },
      {
        "name": "ITCH_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "ITCH_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Test date"
      },
      {
        "name": "ITCH_TCON",
        "status": "OTHER",
        "type": "2DP",
        "unit": "W/(K-m)",
        "description": "Thermal Conductivity"
      },
      {
        "name": "ITCH_TRES",
        "status": "OTHER",
        "type": "2DP",
        "unit": "(K-m)/W",
        "description": "Thermal Resistivity"
      },
      {
        "name": "ITCH_TEMP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "DegC",
        "description": "Ambient temperature at which test is performed"
      },
      {
        "name": "ITCH_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of probe used and method description"
      },
      {
        "name": "ITCH_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "ITCH_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "ITCH_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      },
      {
        "name": "ITCH_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of testing organisation"
      },
      {
        "name": "ITCH_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test status"
      },
      {
        "name": "GEOL_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Stratum reference shown on trial pit or traverse sketch"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "MONS",
    "contents": "Monitoring Installations and Instruments Status",
    "parent": "MONG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "MONG_ID",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Monitoring point reference"
      },
      {
        "name": "MONG_DIS",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Initial distance of monitoring point from LOCA_ID"
      },
      {
        "name": "MONS_STAR",
        "status": "KEY",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date and time of start of status"
      },
      {
        "name": "MONS_ENDD",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date and time of end of status"
      },
      {
        "name": "MONS_BY",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Who recorded status"
      },
      {
        "name": "MONS_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Type of status"
      },
      {
        "name": "MONS_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Status"
      },
      {
        "name": "MONS_RPLO",
        "status": "OTHER",
        "type": "ID",
        "unit": null,
        "description": "Location identifier this installation or instrument replaces"
      },
      {
        "name": "MONS_RPID",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Monitoring point reference this installation or instrument replaces"
      },
      {
        "name": "MONS_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference"
      }
    ]
  },
  {
    "code": "PMMC",
    "contents": "Menard Pressuremeter Test Results - Unload/Reload Cycles",
    "parent": "PMMG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "PMMG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of test"
      },
      {
        "name": "PMMG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "PMMC_CYNO",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Cycle number"
      },
      {
        "name": "PMMC_P1CY",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Corrected pressure at origin of cyclic pressure range"
      },
      {
        "name": "PMMC_P2CY",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Corrected pressure at end of cyclic pressure range"
      },
      {
        "name": "PMMC_EMCY",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Cyclic Menard modulus"
      },
      {
        "name": "PMMC_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "PMMD",
    "contents": "Menard Pressuremeter Test Results - Data",
    "parent": "PMMG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "PMMG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of test"
      },
      {
        "name": "PMMG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "PMMD_SEQ",
        "status": "KEY",
        "type": "0DP",
        "unit": null,
        "description": "Sequence number"
      },
      {
        "name": "PMMD_P01S",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Measured pressure at 1 s"
      },
      {
        "name": "PMMD_P15S",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Measured pressure at 15 s"
      },
      {
        "name": "PMMD_P30S",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Measured pressure at 30 s"
      },
      {
        "name": "PMMD_P60S",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Measured pressure at 60 s"
      },
      {
        "name": "PMMD_V01S",
        "status": "OTHER",
        "type": "1DP",
        "unit": "cm3",
        "description": "Measured volume at 1 s"
      },
      {
        "name": "PMMD_V15S",
        "status": "OTHER",
        "type": "1DP",
        "unit": "cm3",
        "description": "Measured volume at 15 s"
      },
      {
        "name": "PMMD_V30S",
        "status": "OTHER",
        "type": "1DP",
        "unit": "cm3",
        "description": "Measured volume at 30 s"
      },
      {
        "name": "PMMD_V60S",
        "status": "OTHER",
        "type": "1DP",
        "unit": "cm3",
        "description": "Measured volume at 60 s"
      },
      {
        "name": "PMMD_CP",
        "status": "OTHER",
        "type": "3DP",
        "unit": "MPa",
        "description": "Corrected pressure"
      },
      {
        "name": "PMMD_CVOL",
        "status": "OTHER",
        "type": "1DP",
        "unit": "cm3",
        "description": "Corrected volume"
      },
      {
        "name": "PMMD_SLOP",
        "status": "OTHER",
        "type": "0DP",
        "unit": "cm3/MPa",
        "description": "Slope"
      },
      {
        "name": "PMMD_CREP",
        "status": "OTHER",
        "type": "1DP",
        "unit": "cm3",
        "description": "Creep"
      },
      {
        "name": "PMMD_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. test result sheets)"
      }
    ]
  },
  {
    "code": "PMMG",
    "contents": "Menard Pressuremeter Test Results - General",
    "parent": "LOCA",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "PMMG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of test"
      },
      {
        "name": "PMMG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "PMMG_DATE",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-ddThh:mm:ss",
        "description": "Date and time of test"
      },
      {
        "name": "PMMG_DCU",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Distance of control unit above ground"
      },
      {
        "name": "PMMG_PRWL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "m",
        "description": "Depth to water/fluid in borehole prior to test"
      },
      {
        "name": "PMMG_REF",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Instrument reference / serial number"
      },
      {
        "name": "PMMG_TYPE",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Pressuremeter type"
      },
      {
        "name": "PMMG_DIAM",
        "status": "OTHER",
        "type": "0DP",
        "unit": "mm",
        "description": "Uninflated diameter of pressuremeter"
      },
      {
        "name": "PMMG_PRC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "MPa",
        "description": "Pressure capacity"
      },
      {
        "name": "PMMG_TC",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Method of test control"
      },
      {
        "name": "PMMG_P1",
        "status": "OTHER",
        "type": "2DP",
        "unit": "MPa",
        "description": "Start of linear section pressure, P_1"
      },
      {
        "name": "PMMG_P2",
        "status": "OTHER",
        "type": "2DP",
        "unit": "MPa",
        "description": "End of linear section pressure, P_2"
      },
      {
        "name": "PMMG_EM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Menard modulus, E_M"
      },
      {
        "name": "PMMG_MPL",
        "status": "OTHER",
        "type": "2DP",
        "unit": "MPa",
        "description": "Menard limit pressure"
      },
      {
        "name": "PMMG_MPLM",
        "status": "OTHER",
        "type": "PA",
        "unit": null,
        "description": "Menard limit pressure method"
      },
      {
        "name": "PMMG_PF",
        "status": "OTHER",
        "type": "2DP",
        "unit": "MPa",
        "description": "Creep pressure"
      },
      {
        "name": "PMMG_METH",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test method"
      },
      {
        "name": "PMMG_CREM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Describe corrections applied during processing"
      },
      {
        "name": "PMMG_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "PMMG_CRDT",
        "status": "OTHER",
        "type": "DT",
        "unit": "yyyy-mm-dd",
        "description": "Date of last calibration of instrument"
      },
      {
        "name": "PMMG_OPER",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name of test operator"
      },
      {
        "name": "PMMG_ANBY",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Name(s) of analyser / person responsible for data quality and correctness"
      },
      {
        "name": "PMMG_CONT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Subcontractors name"
      },
      {
        "name": "PMMG_CRED",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Accrediting body and reference number (when appropriate)"
      },
      {
        "name": "TEST_STAT",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Test status"
      },
      {
        "name": "PMMG_ENV",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Details of weather and environmental conditions during test"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. equipment calibrations)"
      }
    ]
  },
  {
    "code": "PMTP",
    "contents": "Pressuremeter Test Results - Parameters",
    "parent": "PMTG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "PMTG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of test"
      },
      {
        "name": "PMTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "PMTP_U0",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "In situ pore water pressure"
      },
      {
        "name": "PMTP_STO",
        "status": "OTHER",
        "type": "2DP",
        "unit": "mm",
        "description": "Strain origin"
      },
      {
        "name": "PMTP_HO",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Estimated in situ horizontal stress"
      },
      {
        "name": "PMTP_HOM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method remark for Estimated in situ horizontal stress"
      },
      {
        "name": "PMTP_GI",
        "status": "OTHER",
        "type": "3SF",
        "unit": "MPa",
        "description": "Shear modulus from first loading"
      },
      {
        "name": "PMTP_SU",
        "status": "OTHER",
        "type": "1DP",
        "unit": "kPa",
        "description": "Undrained shear strength"
      },
      {
        "name": "PMTP_SUM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method remark for Undrained Shear Strength (s_u)"
      },
      {
        "name": "PMTP_AF",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Peak angle of friction"
      },
      {
        "name": "PMTP_AD",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Angle of dilation"
      },
      {
        "name": "PMTP_AFDM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method remark for Peak angle of friction and Angle of dilation"
      },
      {
        "name": "PMTP_AFCV",
        "status": "OTHER",
        "type": "1DP",
        "unit": "deg",
        "description": "Angle of friction at constant volume (*cv) used"
      },
      {
        "name": "PMTP_DC",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Drained cohesion"
      },
      {
        "name": "PMTP_DCM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method remark for Drained cohesion"
      },
      {
        "name": "PMTP_PL",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Total limit pressure"
      },
      {
        "name": "PMTP_PF",
        "status": "OTHER",
        "type": "0DP",
        "unit": "kPa",
        "description": "Total yield stress"
      },
      {
        "name": "PMTP_PFM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method remark for Total yield stress"
      },
      {
        "name": "PMTP_YM",
        "status": "OTHER",
        "type": "1DP",
        "unit": "MPa",
        "description": "Yield modulus"
      },
      {
        "name": "PMTP_YMM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Method remark for Yield modulus"
      },
      {
        "name": "PMTP_MU",
        "status": "OTHER",
        "type": "2DP",
        "unit": null,
        "description": "Poisson's ratio"
      },
      {
        "name": "PMTP_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. equipment calibrations)"
      }
    ]
  },
  {
    "code": "PMTZ",
    "contents": "Pressuremeter Test Results - Zeros",
    "parent": "PMTG",
    "headings": [
      {
        "name": "LOCA_ID",
        "status": "KEY",
        "type": "ID",
        "unit": null,
        "description": "Location identifier"
      },
      {
        "name": "PMTG_DPTH",
        "status": "KEY",
        "type": "2DP",
        "unit": "m",
        "description": "Depth of test"
      },
      {
        "name": "PMTG_TESN",
        "status": "KEY",
        "type": "X",
        "unit": null,
        "description": "Test reference"
      },
      {
        "name": "PMTZ_PARM",
        "status": "KEY",
        "type": "PA",
        "unit": null,
        "description": "Measured Parameter"
      },
      {
        "name": "PMTZ_MRS",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Measuring ranges of the sensors, min to max unit"
      },
      {
        "name": "PMTZ_ZC",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Zero from calibration"
      },
      {
        "name": "PMTZ_ZB",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Zero before at surface"
      },
      {
        "name": "PMTZ_ZH",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Zero in hole before test at test depth"
      },
      {
        "name": "PMTZ_ZA",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Zero after at surface"
      },
      {
        "name": "PMTZ_ZD",
        "status": "OTHER",
        "type": "U",
        "unit": null,
        "description": "Zero drift"
      },
      {
        "name": "PMTZ_EGUT",
        "status": "OTHER",
        "type": "PU",
        "unit": null,
        "description": "Unit for PMTZ_ZD"
      },
      {
        "name": "PMTZ_REM",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Remarks"
      },
      {
        "name": "FILE_FSET",
        "status": "OTHER",
        "type": "X",
        "unit": null,
        "description": "Associated file reference (e.g. equipment calibrations)"
      }
    ]
  }
];
