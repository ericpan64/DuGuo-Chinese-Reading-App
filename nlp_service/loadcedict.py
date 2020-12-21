from pymongo import MongoClient
import pinyin as pfmt
from config import DB_NAME, COLL_NAME, DB_PORT, DB_HOSTNAME # Note: this exists but is not published on this repo

client = MongoClient(DB_HOSTNAME, DB_PORT)
db = client[DB_NAME]
coll = db[COLL_NAME]

def convert_digits_to_pinyin(s):
    repl_dict = {
        '0': 'líng',
        '1': 'yī',
        '2': 'èr', 
        '3': 'sān',
        '4': 'sì',
        '5': 'wǔ',
        '6': 'liù',
        '7': 'qī',
        '8': 'bā',
        '9': 'jiǔ',
        '%': 'pā',
    }
    for k, v in repl_dict.items():
        s = s.replace(k, v)
    return s

if __name__ == '__main__':
    # Load CEDICT from file to mongoDB
    cedict_path='static/cedict_ts.u8'
    if coll.estimated_document_count() > 100000:
        print("CEDICT is already loaded -- skipping operation...")

    print("Loading CEDICT - this takes a few seconds...")
    with open(cedict_path, encoding="utf8") as f:
        entry_list = []
        for line in f:
            line = line.strip()
            if len(line) == 0 or line[0] == '#':
                continue

            trad, simp, rest = [tok for tok in line.split(' ', 2)]
            # print(trad,len(trad),simp,rest)
            close_bracket = rest.find(']')  # close bracket on pinyin
            pinyin_raw = rest[1:close_bracket]
            defn = rest[close_bracket+2:]

            pinyin_formatted = pfmt.get(simp, delimiter=" ")
            pinyin_formatted = convert_digits_to_pinyin(pinyin_formatted)

            entry_list.append({
                "trad": trad,
                "simp": simp,
                "pinyin_raw": pinyin_raw,
                "pinyin_formatted": pinyin_formatted,
                "def": defn
            })
        print("Loaded. Sending to db...")
        coll.insert_many(entry_list)
        print("Completed")

