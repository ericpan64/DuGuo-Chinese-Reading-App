from pymongo import MongoClient
from pypinyin import pinyin as pfmt
from pypinyin import Style
import pandas as pd
from config import DB_NAME, COLL_NAME, DB_URI # Note: this exists but is not published on this repo

# Connect to mongoDB
client = MongoClient(DB_URI)
db = client[DB_NAME]
coll = db[COLL_NAME]
coll.drop() # reload when testing

# Track set of Traditional/Simplified characters with duplicate entries.
# Skip them for now (seek to merge definitions in the future)
get_first_col_as_set = lambda fn: set(pd.read_csv(fn).iloc[:, 0])
simp_dups = get_first_col_as_set('static/simplified_duplicates.csv')
trad_dups = get_first_col_as_set('static/traditional_duplicates.csv')

def convert_digits_to_chars(s):
    """ Formats edge-case characters from pypinyin (pfmt) library """
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

def render_phrase_table_html(phrase, raw_pinyin, formatted_pinyin, defn, zhuyin):
    """ Takes CEDICT entry information and generates corresponding HTML """
    download_icon_loc = 'https://icons.getbootstrap.com/icons/download.svg'
    sound_icon_loc = 'https://icons.getbootstrap.com/icons/mic.svg'

    # Helper fn
    def get_phrase_data_as_lists():
        # get individual words (used in pinyin name)
        word_list = [w for w in phrase]
        pinyin_list = formatted_pinyin.split(' ')
        zhuyin_list = zhuyin.split(' ')
        # handle case for non-chinese character pinyin getting "stuck" (e.g. ['AA'] should be ['A', 'A'])
        if len(word_list) > len(pinyin_list):
            # from inspection, this is always first or last item. Hard-code for edge cases (['dǎ', 'call'], ['mǔ', 'tāi', 'solo'], ['kǎ', 'lā', 'OK'])
            split_first_non_chinese_phrase = lambda pylist: [c for c in pylist[0]] + pylist[1:]
            split_last_non_chinese_phrase = lambda pylist: pylist[:-1] + [c for c in pylist[-1]]
            if pinyin_list[0] not in {'dǎ', 'mǔ', 'kǎ'} and len(pinyin_list[0]) > 1:
                pinyin_list = split_first_non_chinese_phrase(pinyin_list)
                zhuyin_list = split_first_non_chinese_phrase(zhuyin_list)
            elif len(pinyin_list[-1]) > 1:
                pinyin_list = split_last_non_chinese_phrase(pinyin_list)
                zhuyin_list = split_last_non_chinese_phrase(zhuyin_list)
        # print(f"word_list  : {word_list}\npinyin_list: {pinyin_list}\nzhuyin_list: {zhuyin_list}")
        assert len(word_list) == len(pinyin_list)
        assert len(word_list) == len(zhuyin_list)
        return word_list, pinyin_list, zhuyin_list

    # Helper fn
    def perform_render(use_pinyin):
        # generate html
        word_list, pinyin_list, zhuyin_list = get_phrase_data_as_lists()
        n_words = len(word_list)
        res = ''
        phonetics = raw_pinyin if use_pinyin else zhuyin
        span_start = f'<span tabindex="0" data-bs-toggle="popover" data-bs-trigger="focus" data-bs-content="{format_defn_html(defn)}" \
            title="{phrase} [{phonetics}] \
            <a role=&quot;button&quot; href=&quot;#~{phrase}&quot;><img src=&quot;{sound_icon_loc}&quot;></img></a> \
            <a role=&quot;button&quot; href=&quot;#{phrase}&quot;><img src=&quot;{download_icon_loc}&quot;></img></a>" \
            data-bs-html="true">' # ... dear neptune...
        res += span_start.replace('            ', '')
        res += '<table style="display: inline-table; text-align: center;">'
        if use_pinyin:
            pinyin_html = ''.join([f'<td style="visibility: visible" class="pinyin" name="{word_list[i]}">{pinyin_list[i]}</td>' for i in range(n_words)])
            pinyin_html = f'<tr>{pinyin_html}</tr>'
            res += pinyin_html
        else:
            zhuyin_html = ''.join([f'<td style="visibility: visible" class="zhuyin" name="{word_list[i]}">{zhuyin_list[i]}</td>' for i in range(n_words)])
            zhuyin_html = f'<tr>{zhuyin_html}</tr>'
            res += zhuyin_html
        phrase_html = ''.join([f'<td class="phrase">{w}</td>' for w in phrase])
        phrase_html = f'<tr>{phrase_html}</tr>'
        res += phrase_html
        res += '</table>'
        res += '</span>'
        return res
        
    res_pinyin = perform_render(use_pinyin=True)
    res_zhuyin = perform_render(use_pinyin=False)
    return (res_pinyin, res_zhuyin)

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

                # skip cases with duplicate entries (identified in static/*.csv files)
                if (trad in trad_dups) or (simp in simp_dups):
                    continue

                # get formatted pinyin
                flatten_list = lambda l: [i for j in l for i in j] # [[a], [b], [c]] => [a, b, c]
                formatted_simp = convert_digits_to_chars(simp)
                formatted_pinyin = ' '.join(flatten_list(pfmt(formatted_simp)))

                # get zhuyin (BOPOMOFO)
                zhuyin = ' '.join(flatten_list(pfmt(formatted_simp, style=Style.BOPOMOFO)))
                
                # handle case where lines can be merged (based on formatted pinyin)
                if curr_matches_prev(trad, simp, formatted_pinyin):
                    last_entry = entry_list.pop()
                    defn = last_entry['def'] + defn[1:]

                # render html
                trad_html, trad_zhuyin_html = render_phrase_table_html(trad, raw_pinyin, formatted_pinyin, defn, zhuyin)
                simp_html, simp_zhuyin_html = render_phrase_table_html(simp, raw_pinyin, formatted_pinyin, defn, zhuyin)

                # append entry
                entry_list.append({
                    'trad': trad,
                    'simp': simp,
                    'raw_pinyin': raw_pinyin,
                    'formatted_pinyin': formatted_pinyin,
                    'def': defn,
                    'trad_html': trad_html,
                    'simp_html': simp_html,
                    'zhuyin': zhuyin,
                    'trad_zhuyin_html': trad_zhuyin_html,
                    'simp_zhuyin_html': simp_zhuyin_html
                })
            
                # update prev items
                prev_trad, prev_simp, prev_formatted_pinyin = trad, simp, formatted_pinyin

            print('Loaded. Sending to db...')
            coll.insert_many(entry_list)
        
        print('Creating an index on trad, simp phrases...')
        coll.create_index([ ("trad", 1) ], unique=True)
        coll.create_index([ ("simp", 1) ], unique=True)
        print('Completed')

