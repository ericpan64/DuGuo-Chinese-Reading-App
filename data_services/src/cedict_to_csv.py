
"""
Generates CEDICT .csv for Redis load
"""
import pandas as pd
from os import remove
from os.path import exists
from config import CEDICT_ORIG_PATH, CEDICT_CSV_PATH, SORTED_CEDICT_CSV_PATH, N_COMMENTS

def generate_delimited_cedict(input_path, output_path):
    """ 
    Generates $-delimited file from original cedict_ts text file
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

def fix_cedict_edge_cases(df):
    """ 
    Fix significant CEDICT edge cases that don't match with tokenizer. 
    Expand and document as-appropriate 
    """
    # Line  : 2451,不是,不是,bu4 shi4,/no/is not/not/ 
    # Change: bu4 shi4 => bu2 shi4
    df.loc[2451, 'raw_pinyin'] = 'bu2 shi4'
    
def sort_delimited_cedict(input_path, output_path):
    """ Generates sorted .csv file """
    df = pd.read_csv(input_path, delimiter='$')
    df.index = range(N_COMMENTS + 1, df.index.size + N_COMMENTS + 1)
    df.index.name = 'line_in_original'
    df = df.sort_values(['simp', 'raw_pinyin', 'trad'], ascending=True)
    fix_cedict_edge_cases(df)
    df.to_csv(output_path)

if __name__ == '__main__':
    if exists(CEDICT_CSV_PATH):
        remove(CEDICT_CSV_PATH)
    if exists (SORTED_CEDICT_CSV_PATH):
        remove(SORTED_CEDICT_CSV_PATH)

    generate_delimited_cedict(CEDICT_ORIG_PATH, CEDICT_CSV_PATH)
    sort_delimited_cedict(CEDICT_CSV_PATH, SORTED_CEDICT_CSV_PATH)