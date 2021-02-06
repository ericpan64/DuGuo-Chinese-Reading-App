"""
This script generates a char->radical map as a .csv.

Each character maps to {0, 1} radical(s), so there are no lookup conflicts to handle!

Data is sourced from: 
1. https://ltl-beijing.com/chinese-radicals/
2. http://ccdb.hemiola.com/
"""
import urllib.request
import pandas as pd
from config import RADICALS_SOURCE_PATH, RADICALS_OUTPUT_PATH

def api_url_to_list(url):
    """ 
    Given a URL to the API, makes the request and parses list from response.
    """
    request_url = urllib.request.urlopen(url) 
    resp = request_url.read().decode('utf-8')
    resp = resp.replace('"string":', '')
    resp = resp.replace('[', '')
    resp = resp.replace(']', '')
    resp = resp.replace('{', '')
    resp = resp.replace('}', '')
    resp = resp.replace('\"', '')
    return resp.split(',')

if __name__ == '__main__':
    print("Loading radical information, this takes a couple of minutes...")
    # Get set of chars (unicode)
    char_list = api_url_to_list('http://ccdb.hemiola.com/characters')
    char_map = {char: 0 for char in char_list} # interesting: each character only maps to 1 radical!
    # For each radical, add to corresponding lists
    print("Sourcing char: radical_no for all 214 radicals from ccdb API... ")
    for i in range(1, 215):
        radical_list = api_url_to_list(f'http://ccdb.hemiola.com/characters/radicals/{i}')
        for char in radical_list:
            char_map[char] = i
    # Save as .csv
    print(f"Appending definitions from {RADICALS_SOURCE_PATH}")
    char_map_w_cols = { 'char': [chr(int(c[2:], 16)) for c in char_map.keys()], 'radical_no': list(char_map.values())}
    char_df = pd.DataFrame.from_dict(char_map_w_cols)
    char_df.set_index('radical_no', inplace=True)
    radicals_df = pd.read_csv(RADICALS_SOURCE_PATH, index_col=0)
    char_df = char_df.join(radicals_df, on='radical_no', how='right', lsuffix='left')
    char_df.to_csv(RADICALS_OUTPUT_PATH)
    print(f"Saved to: {RADICALS_OUTPUT_PATH}")