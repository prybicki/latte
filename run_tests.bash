#!/bin/bash

LATC=./target/debug/latte

for FILE in ./lattests/good/core*.lat ; do
  OUTPUT=${FILE//.lat/.output}
  ${LATC} "${FILE}"

  if ! lli simplest.bc > simplest.out; then
    echo "LLVM error @ ${FILE}"
  fi
  DIFF=$(diff simplest.out ${OUTPUT})
  if [ "$DIFF" ]; then
    echo "${FILE} ERROR"
    echo "${DIFF}"
  else
    echo "${FILE} OK"
  fi
done