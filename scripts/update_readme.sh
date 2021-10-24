#! /bin/bash

# set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

cd "$(dirname "$0")"/..

cargo readme > README.md

# smoelius: Fix intra-doc links. This is a modification of:
# https://github.com/livioribeiro/cargo-readme/issues/70#issuecomment-907867904
sed -i 's/\[\(`SafeJoin::[^`]*`\)\]/\1/g' README.md

# smoelius: Fix reference-style links.
sed -i 's,^\(\[components\]\): .*$,\1: https://doc.rust-lang.org/std/path/enum.Component.html,' README.md
sed -i 's,^\(\[`io::Result`\]\): .*$,\1: https://doc.rust-lang.org/std/io/type.Result.html,' README.md
sed -i 's,^\(\[`Path::join`\]\): .*$,\1: https://doc.rust-lang.org/std/path/struct.Path.html#method.join,' README.md
