from pymongo import MongoClient
from pypinyin import pinyin as pfmt
from pypinyin import Style
import pandas as pd
import redis
from config import DB_NAME, DB_URI, USER_COLL_NAME, USER_DOC_COLL_NAME, USER_VOCAB_COLL_NAME, USER_VOCAB_LIST_COLL_NAME
from config import REDIS_HOST, REDIS_PORT, REDIS_PASSWORD, SORTED_CEDICT_CSV_PATH, RADICALS_OUTPUT_PATH

def init_mongodb():
    """
    Connects to mongoDB, creates indices, and returns collection for CEDICT load
    """
    client = MongoClient(DB_URI)
    db = client[DB_NAME]
    colls = {
        'user': db[USER_COLL_NAME],
        'user_docs': db[USER_DOC_COLL_NAME],
        'user_vocab': db[USER_VOCAB_COLL_NAME],
        'user_vocab_list': db[USER_VOCAB_LIST_COLL_NAME]
    }
    # User Indices
    colls['user'].create_index([ ("username", 1)], unique=True)
    colls['user'].create_index([ ("email", 1)], unique=True)
    colls['user'].create_index([ ("username", 1), ("email", 1)], unique=True)
    # User Doc Indices
    colls['user_docs'].create_index([ ("username", 1)])
    colls['user_docs'].create_index([ ("username", 1), ("cn_type", 1), ("cn_phonetics", 1)])
    colls['user_docs'].create_index([ ("username", 1), ("title", 1), ("cn_type", 1), ("cn_phonetics", 1)], unique=True)
    # User Vocab Indices
    colls['user_vocab'].create_index([ ("username", 1)])
    colls['user_vocab'].create_index([ ("username", 1), ("phrase", 1)])
    colls['user_vocab'].create_index([ ("username", 1), ("cn_type", 1), ("cn_phonetics", 1)])
    colls['user_vocab'].create_index([ ("username", 1), ("phrase", 1), ("cn_type", 1), ("cn_phonetics", 1)], unique=True)
    # User Vocab List Index
    colls['user_vocab_list'].create_index([ ("username", 1) ])
    colls['user_vocab_list'].create_index([ ("username", 1), ("cn_type", 1) ], unique=True)
    return

def convert_digits_to_chars(s):
    """
    Formats edge-case characters from pypinyin (pfmt) library
    """
    repl_dict = {
        '1': '一',
        '2': '二',
        '3': '三',
        '4': '四',
        '5': '五',
        '6': '六',
        '7': '七',
        '8': '八',
        '9': '九',
        '0': '零',
        '%': '啪', # FYI: same pinyin, not actually the same word
    }
    for k, v in repl_dict.items():
        s = s.replace(k, v)
    return s

def generate_uid(simp, raw_pinyin):
    return simp.replace(' ', '') + raw_pinyin.replace(' ', '')

def load_cedict(conn):
    """
    Performs the CEDICT load into the specified collection
    """
    print(f'Loading CEDICT to Redis from {SORTED_CEDICT_CSV_PATH} - this takes a few seconds...')
    print(f'Using radical information from {RADICALS_OUTPUT_PATH}')
    cedict_df = pd.read_csv(SORTED_CEDICT_CSV_PATH, index_col=0) # line_in_original
    radical_df = pd.read_csv(RADICALS_OUTPUT_PATH, index_col=1) # char
    entry_list = []
    prev_trad, prev_simp, prev_raw_pinyin = None, None, None # Track lines with identical pinyin + phrase
    prev_defn = '$' # Track when one definition is superset of other, init to dummy value
    flatten_list = lambda l: [i for j in l for i in j] # [[a], [b], [c]] => [a, b, c]
    uid_set = set()
    for _, row in cedict_df.iterrows():
        trad, simp, raw_pinyin, defn = row
        uid = generate_uid(simp, raw_pinyin)
        # skip cases where definition is a "variant of" another CEDICT entry
        if "variant of" in defn:
            continue
        # skip case where previous entry is superset of current
        if (prev_raw_pinyin == raw_pinyin) and defn in prev_defn:
            continue
        # get formatted pinyin
        formatted_simp = convert_digits_to_chars(simp)
        formatted_pinyin = ' '.join(flatten_list(pfmt(formatted_simp)))
        # get zhuyin (BOPOMOFO)
        zhuyin = ' '.join(flatten_list(pfmt(formatted_simp, style=Style.BOPOMOFO)))
        # handle case where current entry is superset of previous
        if (prev_raw_pinyin == raw_pinyin) and (prev_defn in defn):
            last_entry = entry_list.pop()
        # handle case where lines can be merged (based on raw_pinyin)
        # NOTE: this does cause some data loss for traditional entries with matching simplified phrases, treating as negligible
        elif (prev_raw_pinyin == raw_pinyin) and ((prev_simp == simp) or (prev_trad == trad)):
            last_entry = entry_list.pop()
            defn = last_entry['defn'] + '$' + defn
        # get radical information
        lookup_radical = lambda char: "NA" if char not in radical_df.index else radical_df.loc[char].radical_char
        radical_map = { c: lookup_radical(c) for c in simp }
        radical_map = str(radical_map).replace("'", '')
        radical_map = radical_map.replace(',', ',\n') # make newlines for readability
        # append entry
        entry_list.append({
            'uid': uid,
            'trad': trad,
            'simp': simp,
            'raw_pinyin': raw_pinyin,
            'defn': defn,
            'formatted_pinyin': formatted_pinyin,
            'zhuyin': zhuyin,
            'radical_map': radical_map
        })
        # update prev items
        prev_trad, prev_simp, prev_raw_pinyin = trad, simp, raw_pinyin
        prev_defn = defn
    # add to Redis
    print("Loading CEDICT to Redis, this takes a few minutes...")
    for entry in entry_list:
        uid = entry['uid']
        assert(uid not in uid_set)
        uid_set.add(uid)
        for k in entry:
            assert(conn.hset(uid, k, entry[k]) == 1)
    return

if __name__ == '__main__':
    try:
        init_mongodb()
        print('Generated indices in mongoDB')
    except:
        print('Skipping mongoDB index generation...')
    conn = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, password=REDIS_PASSWORD)
    print("Waiting for Redis load...")
    redis_loaded = False
    while not redis_loaded:
        try:
            # conn.flushdb() # local testing
            size = conn.dbsize()
            redis_loaded = True
        except:
            pass
    if size > 110000:
        print('Redis has >110k documents - thus assuming CEDICT is loaded, skipping operation...')
    else:
        load_cedict(conn)