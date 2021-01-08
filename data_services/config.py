""" 
This file is a "dummy" config file provided for reference and if you want to deploy the app locally.
All confidential information is absent from this document.
"""

# loadcedict.py
DB_NAME = 'duguo'
COLL_NAME = 'cedict'
DB_URI = 'mongodb://root:example@mongodb:27017/'

# tokenserver.py
TOKENIZER_HOST = '0.0.0.0' # Opt for numeric address when possible
TOKENIZER_PORT = 8881