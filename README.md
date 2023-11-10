# file-utils

Doing some experiments with rayon and data-parallelism in Rust, with the overall
goal being finding duplicates in my Photo library, and ultimately using exif
meta to rebuild the collection without duplicates and according to a date based
layout.

## extract-metadata

This command will walk a directory tree, hash and extract exif metadata from 
all files and output the result as a stream of JSON lines to stdout. Requires
the exif command in `$PATH` (should work if you use nix flakes + direnv).

```
cargo run --bin extract-metadata --dir /path/to/photos | tee -a my-fotos.json
```

## find-duplicates

This command expects JSON input in the format from above, and outputs a list 
of content hashes and the files that match it. Only duplicates will be output.

```
cargo run --bin find-duplicates --input-file /path/to/my-fotos.json
```
