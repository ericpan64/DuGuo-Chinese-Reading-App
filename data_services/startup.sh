#!/bin/bash
cd src
python cedict_to_csv.py
python loadcedict.py
python tokenserver.py
cd ..