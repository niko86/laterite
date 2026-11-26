// The one test for "is this a file the app will load?", shared by every picker
// so the main loader and the two-file tools cannot drift apart on it.
//
// By NAME, checked after the file is chosen — never an `accept` list on the
// input: `.ags` has no registered MIME type, so a mobile picker handed one
// greys out the very files it is meant to allow.
export const isAgsFileName = (name: string): boolean =>
  name.toLowerCase().endsWith(".ags");
