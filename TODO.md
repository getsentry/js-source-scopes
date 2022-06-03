# TODO

## Internals

- should we rather use usize for internals that are not written to disk?
- ^ similarly, should we use more strict types rather than sentinel values?

## SmCache

- find a better name ;-)
- figure out if we need to save `sourcesContent` as well
  - adapt the `SourceContext` type and persist that into the binary
  - the raw contents can be appended to the string table
  - the `line_offsets` can be persisted into another table and has offsets into the string table of each line start
  - then we would need a third table for `files`, which consist of:
    - name: offset into string table
    - line_offsets_idx: index into the offsets table
    - lines: number of line offsets this file has (minimum of 1 for zero-length file)
  - the `files` table can be sorted by name to make lookup a bit faster

# file UUIDs

goals:

1. skip uploads of existing/knows files by uuid
2. match stack trace to uuid "somehow"

## brainstorm:

"file uuid" = hash of the file, or whatever else…

"release" = {
$filename => uuid,
$filename2 => uuid2,
…
}
