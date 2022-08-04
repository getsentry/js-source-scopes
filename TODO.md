# TODO

## Internals

- should we rather use usize for internals that are not written to disk?
- ^ similarly, should we use more strict types rather than sentinel values?

## SmCache

- find a better name ;-)

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
