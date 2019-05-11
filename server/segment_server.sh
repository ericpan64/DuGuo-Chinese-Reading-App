#!/bin/bash
set -e
set -x

SERVER_PORT=8084

echo "Starting segment server on port $SERVER_PORT"

/usr/bin/java -mx2g -cp stanford-nlp/stanford-corenlp-3.5.2.jar \
    edu.stanford.nlp.ie.NERServer \
    -port $SERVER_PORT \
    -sighanCorporaDict stanford-nlp/data/ \
    -sighanPostProcessing true \
    -keepAllWhitespaces false \
    -loadClassifier stanford-nlp/data/pku.gz \
    -serDictionary stanford-nlp/data/dict-chris6.ser.gz \
    -inputEncoding UTF-8

