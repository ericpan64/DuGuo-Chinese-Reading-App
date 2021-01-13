""" 
This file is a "dummy" config file provided for reference and if you want to deploy the app locally.
All confidential information is absent from this document.
"""

# cedict_to_csv.py
CEDICT_ORIG_PATH = 'static/cedict_ts.u8'
CEDICT_CSV_PATH = 'static/delimited_cedict_ts.txt'
SORTED_CEDICT_CSV_PATH = 'static/sorted_cedict_ts.csv'
N_COMMENTS = 30 # number of commented lines on top of original CEDICT file

# loadcedict.py
DB_NAME = 'duguo'
USER_COLL_NAME = 'users'
USER_DOC_COLL_NAME = 'docs'
USER_VOCAB_COLL_NAME = 'vocab'
USER_VOCAB_LIST_COLL_NAME = 'vocab-list'
CEDICT_COLL_NAME = 'cedict'
DB_URI = 'mongodb://root:example@mongodb:27017/'

# tokenserver.py
TOKENIZER_HOST = '0.0.0.0' # Opt for numeric address when possible
TOKENIZER_PORT = 8881
MAX_BUF = 1024000 # 1MB