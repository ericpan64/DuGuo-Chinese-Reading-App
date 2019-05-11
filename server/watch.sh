#!/bin/bash

# Run this script to automatically compile coffeescript for you!

set -e
set -x

coffee --watch --compile --output app/static/ app/static/
