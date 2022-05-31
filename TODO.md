# TODO

## SmCache

- find a better name ;-)
- make a true binary format out of it
  - experiment with some new ideas:
  - uleb128 for string table length
  - compress file/line/scope to a u64 (can be 3 \* 21 bits)

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
