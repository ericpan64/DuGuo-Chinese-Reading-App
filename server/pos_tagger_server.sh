#!/bin/bash
set -e
set -x

SERVER_PORT=8083

echo "Starting POS Tagger server on port $SERVER_PORT"

/usr/bin/java -mx500m -cp stanford-nlp/stanford-postagger-3.5.2.jar edu.stanford.nlp.tagger.maxent.MaxentTaggerServer \
    -port $SERVER_PORT \
    -model stanford-nlp/models/chinese-distsim.tagger \
    -outputFormat xml

