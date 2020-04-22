import argparse
import app.models as ce
from mongoengine import register_connection, connect

def connectToMongo(alias='db', name='crm-db-main'):
    """
    Sets-up mongoDB connection
    """
    register_connection(alias=alias, name=name)
    connect(name)  # http://docs.mongoengine.org/guide/connecting.html
    return

def loadCEDICT(reload=False):
    """
    Loads CEDICT collection into database
    """
    parser = argparse.ArgumentParser()
    parser.add_argument('--cedict', default='cedict_ts.u8')
    args = parser.parse_args()

    # Don't load if there are already records in the db
    if not reload:
        print("Checking if CEDICT is already loaded...")
        try:
            numItems = len(ce.CEDICT.objects(definition__exists=True))
            print(f"CEDICT already loaded with {numItems} items -- skipping reload.")
            return
        except ConnectionError:
            print(f"Connection failed. Are you sure mongoDB is on?")
            
            

    print("Loading CEDICT - this takes a few seconds...")
    with open(args.cedict, encoding="utf8") as f:
        entry_list = []
        for line in f:
            line = line.strip()
            if len(line) == 0 or line[0] == '#':
                continue

            trad, simp, rest = [tok for tok in line.split(' ', 2)]
            # print(trad,len(trad),simp,rest)
            close_bracket = rest.find(']')  # close bracket on pinyin
            pinyin = rest[1:close_bracket]
            defn = rest[close_bracket+2:]

            entry_list.append(ce.CEDICT(traditional=trad, simplified=simp, pinyin=pinyin, definition=defn))
        print("Loaded. Sending to db...")
        ce.CEDICT.objects.insert(entry_list)
        print("Completed")
    return