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

def format_defn_html(defn):
    """ Takes /-delimited definition and generates corresponding HTML """
    # Input: /The first definition/The second definition/ ...
    # Output: 1. The first definition\n2. The second definition\n ...
    res = ''
    clean_defn = defn.replace('\"', '\'') # some entries in CEDICT use " character
    defns = clean_defn.split('/')[1:-1] # removes first and last splits, which are '' in this case
    for (i, d) in enumerate(defns):
        res += f'{i+1}. {d}<br/>'
    return res

def render_phrase_table_html(trad, simp, raw_pinyin, formatted_pinyin, defn):
    """ Takes CEDICT entry information and generates corresponding HTML """
    download_icon_loc = 'https://icons.getbootstrap.com/icons/box-arrow-down.svg'
    pinyin_html = ''.join([f'<td style="visibility: visible" class="pinyin" name="{d}">{d}</td>' for d in formatted_pinyin.split(' ')])
    pinyin_html = f'<tr>{pinyin_html}</tr>'
    def perform_render(phrase):
        res = ''
        span_start = f'<span tabindex="0" data-bs-toggle="popover" data-bs-content="{format_defn_html(defn)}" \
            title="{phrase} [{raw_pinyin}] <a role=&quot;button&quot; href=&quot;#{formatted_pinyin.replace(" ", "_")}&quot;>\
            <img src=&quot;{download_icon_loc}&quot;></img></a>" data-bs-html="true">' # ... oh neptune...
        res += span_start.replace('            ', '')
        res += '<table style="display: inline-table;">'
        res += pinyin_html
        phrase_html = ''.join([f'<td>{w}</td>' for w in phrase])
        phrase_html = f'<tr>{phrase_html}</tr>'
        res += phrase_html
        res += '</table>'
        res += '</span>'
        return res
    res_trad = perform_render(trad)
    res_simp = perform_render(simp)
    return (res_trad, res_simp)

if __name__ == '__main__':
    # Load CEDICT from file to mongoDB
    cedict_path = 'static/cedict_ts.u8'
    if coll.estimated_document_count() > 100000:
        print('CEDICT is already loaded -- skipping operation...')
    else:
        # Track lines with identical pinyin + phrase
        prev_trad, prev_simp, prev_formatted_pinyin = None, None, None
        curr_matches_prev = lambda t, s, fp: (prev_trad == t) and (prev_simp == s) and (prev_formatted_pinyin == fp)

        # Parse file
        print('Loading CEDICT - this takes a few seconds...')
        with open(cedict_path, encoding='utf8') as f:
            entry_list = []
            for line in f:
                # skip no-data lines
                line = line.strip()
                if len(line) == 0 or line[0] == '#':
                    continue

                # get CEDICT components
                trad, simp, rest = [token for token in line.split(' ', 2)]
                close_bracket = rest.find(']')  # close bracket on pinyin
                raw_pinyin = rest[1:close_bracket]
                defn = rest[close_bracket+2:]

                # get formatted pinyin
                formatted_pinyin = pfmt.get(simp, delimiter=' ')
                formatted_pinyin = convert_digits_to_pinyin(formatted_pinyin)

                # handle case where lines can be merged (based on formatted pinyin)
                if curr_matches_prev(trad, simp, formatted_pinyin):
                    last_entry = entry_list.pop()
                    defn = last_entry['def'] + defn[1:]

                # render html
                trad_html, simp_html = render_phrase_table_html(trad, simp, raw_pinyin, formatted_pinyin, defn)

                # append entry
                entry_list.append({
                    'trad': trad,
                    'simp': simp,
                    'raw_pinyin': raw_pinyin,
                    'formatted_pinyin': formatted_pinyin,
                    'def': defn,
                    'trad_html': trad_html,
                    'simp_html': simp_html
                })
            
                # update prev items
                prev_trad, prev_simp, prev_formatted_pinyin = trad, simp, formatted_pinyin

            print('Loaded. Sending to db...')
            coll.insert_many(entry_list)
            print('Completed')

