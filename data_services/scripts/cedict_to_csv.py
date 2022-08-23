
"""
Generates CEDICT .csv for Redis load
"""
from pypinyin import pinyin as pfmt
from pypinyin import Style
import pandas as pd
from os import remove
from os.path import exists
from config import CEDICT_ORIG_PATH, CEDICT_CSV_PATH, PROCESSED_CEDICT_CSV_PATH, N_COMMENTS, RADICALS_OUTPUT_PATH

def convert_digits_to_chars(s):
    """
    Formats edge-case characters from pypinyin (pfmt) library
    """
    repl_dict = {
        '1': '一', '2': '二', '3': '三',
        '4': '四', '5': '五', '6': '六',
        '7': '七', '8': '八', '9': '九',
        '0': '零',
        '%': '啪', # FYI: same pinyin, not actually the same word
    }
    for k, v in repl_dict.items():
        s = s.replace(k, v)
    return s

def generate_delimited_cedict(input_path, output_path):
    """ 
    Generates $-delimited file from original cedict_ts text file

    This is done so there are headers for the data

    $ is used since it does not appear in the original CEDICT file
    """
    csv_header = "trad$simp$raw_pinyin$defn\n"
    with open(output_path, 'w') as fout:
        fout.write(csv_header)
        with open(input_path, encoding='utf8') as fin:
            for line in fin:
                # skip no-data lines
                line = line.strip()
                if len(line) == 0 or line[0] == '#':
                    continue

                # get CEDICT components
                trad, simp, rest = [token for token in line.split(' ', 2)]
                close_bracket = rest.find(']')  # close bracket on pinyin
                raw_pinyin = rest[1:close_bracket]
                defn = rest[close_bracket+2:]

                # write to csv
                saved_line = f"{trad}${simp}${raw_pinyin}${defn}\n"
                fout.write(saved_line)
    
def sort_delimited_cedict(input_path, output_path):
    """ Generates sorted .csv file """
    df = pd.read_csv(input_path, delimiter='$')
    df.index = range(N_COMMENTS + 1, df.index.size + N_COMMENTS + 1)
    df.index.name = 'line_in_original'
    df = df.sort_values(['simp', 'raw_pinyin', 'trad'], ascending=True)

    radical_df = pd.read_csv(RADICALS_OUTPUT_PATH, index_col="char")
    res_df = pd.DataFrame(df, columns=[
        'trad', 'simp', 'raw_pinyin',
        'defn', 'formatted_pinyin',
        'zhuyin', 'radical_map'
    ])
    flatten_list = lambda l: [i for j in l for i in j] # [[a], [b], [c]] => [a, b, c]
    lookup_radical = lambda char: "" if char not in radical_df.index else radical_df.loc[char].radical_char
    res_df.loc[:, 'formatted_pinyin'] = res_df.loc[:, 'simp'].apply(
        lambda simp: ' '.join(flatten_list(pfmt(convert_digits_to_chars(simp))))
    )
    res_df.loc[:, "zhuyin"] = res_df.loc[:, "simp"].apply(
        lambda simp: ' '.join(flatten_list(pfmt(convert_digits_to_chars(simp), style=Style.BOPOMOFO)))
    )
    res_df.loc[:, "radical_map"] = res_df.loc[:, "simp"].apply(
        lambda simp: { c: lookup_radical(c) for c in simp }
    )
    res_df.to_csv(output_path)


if __name__ == '__main__':
    if exists(CEDICT_CSV_PATH):
        remove(CEDICT_CSV_PATH)
    if exists (PROCESSED_CEDICT_CSV_PATH):
        remove(PROCESSED_CEDICT_CSV_PATH)

    generate_delimited_cedict(CEDICT_ORIG_PATH, CEDICT_CSV_PATH)
    sort_delimited_cedict(CEDICT_CSV_PATH, PROCESSED_CEDICT_CSV_PATH)