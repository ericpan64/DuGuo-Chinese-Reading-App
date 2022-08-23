""" 
This file is a "dummy" config file provided for reference and if you want to deploy the app locally.
All confidential information is absent from this document.
"""

STATIC_DIR = "static"
OUTPUT_DIR = "scripts/output"

# radicals_to_csv.py
RADICALS_SOURCE_PATH = f'{STATIC_DIR}/radicals.csv'
RADICALS_OUTPUT_PATH = f'{OUTPUT_DIR}/radical_char_map.csv'

# cedict_to_csv.py
CEDICT_ORIG_PATH = f'{STATIC_DIR}/cedict_ts.u8'
CEDICT_CSV_PATH = f'{OUTPUT_DIR}/delimited_cedict_ts.txt'
PROCESSED_CEDICT_CSV_PATH = f'{OUTPUT_DIR}/processed_cedict_ts.csv'
N_COMMENTS = 30 # number of commented lines on top of original CEDICT file

