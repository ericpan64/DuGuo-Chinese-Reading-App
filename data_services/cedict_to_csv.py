
"""
Pseudocode:
- Open CEDICT file
- Grab the following data:
    trad: up to first space
    simp: up to second space
    raw_pinyin: between the []
    defn: the /-split definition surrounded by double-quotes
- Write to static/cedict.csv

Test:
- Import csv with pandas
- Sort by raw_pinyin, then simp, then trad
- Export as_csv to static/cedict_sorted.csv

Manually verify:
- Order makes sense (want to create unique index on raw_pinyin + simp)

Afterwards:
- Modify CEDICT loading code to use sorted csv

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

def sort_delimited_cedict(input_path, output_path):
    """ Generates sorted .csv file """
    df = pd.read_csv(input_path, delimiter='$')
    df.index = range(N_COMMENTS + 1, df.index.size + N_COMMENTS + 1)
    df.index.name = 'line_in_original'
    df = df.sort_values(['simp', 'raw_pinyin', 'trad'], ascending=True)
    df.to_csv(output_path)

if __name__ == '__main__':
    if exists(CEDICT_CSV_PATH):
        remove(CEDICT_CSV_PATH)
    if exists (SORTED_CEDICT_CSV_PATH):
        remove(SORTED_CEDICT_CSV_PATH)

    generate_delimited_cedict(CEDICT_ORIG_PATH, CEDICT_CSV_PATH)
    sort_delimited_cedict(CEDICT_CSV_PATH, SORTED_CEDICT_CSV_PATH)