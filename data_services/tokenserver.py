import spacy
from spacy.tokenizer import Tokenizer
import socket
from socket import error as SocketError
import errno
import selectors
import types
import time
import pandas as pd
from pypinyin import pinyin as pfmt
from pypinyin import Style
import opencc
from config import TOKENIZER_HOST, TOKENIZER_PORT, MAX_BUF, SORTED_CEDICT_CSV_PATH

# NLP import from: https://spacy.io/models/zh
nlp = spacy.load("zh_core_web_sm")
tokenizer = nlp.Defaults.create_tokenizer(nlp)

# Traditional -> Simplified Converter (~67% less lossy than Simp->Trad for CEDICT, though only affects ~1% of entries)
trad_converter = opencc.OpenCC('t2s.json')

# Socket code adapted from: https://realpython.com/python-sockets
sel = selectors.DefaultSelector()
IPV4 = socket.AF_INET
TCP = socket.SOCK_STREAM

# Create CEDICT_DF, CEDICT_SET (used for more accurate pinyin lookup)
CEDICT_DF = pd.read_csv(SORTED_CEDICT_CSV_PATH)
CEDICT_SET = set(CEDICT_DF.loc[:, 'simp'])
CEDICT_DF['raw_pinyin_lower'] = CEDICT_DF.loc[:, 'raw_pinyin'].apply(str.lower)
CEDICT_DF.set_index(['simp', 'raw_pinyin_lower'], inplace=True)

# Chinese chars use 3 bytes, so a phrase with Chinese chars will return false.
# Ex. len('京报') = 2
#     len(bytes('京报', 'utf-8')) = 6
entire_phrase_is_english = lambda p: len(p) == len(bytes(p, 'utf-8'))
entire_phrase_is_chinese = lambda p: (3 * len(p)) == len(bytes(p, 'utf-8'))
flatten_list = lambda l: [i for j in l for i in j] # [[a], [b], [c]] => [a, b, c]

def break_down_large_token_into_subtoken_list(t):
    """
    Applies divide & conquer approach to break-down a larger token into list of CEDICT-only components
    """
    # Base cases: found in CEDICT, or Chinese punctuation, or English phrase
    if t in CEDICT_SET or len(t) == 1 or entire_phrase_is_english(t):
        return [t]
    n = len(t)
    mid = int(n/2) # round down for odd #'s
    left_tokens = break_down_large_token_into_subtoken_list(t[0:mid])
    right_tokens = break_down_large_token_into_subtoken_list(t[mid: n])
    return left_tokens + right_tokens

def tokenize_str(s):
    """
    Given the input text s, tokenize and return as $-delimited phrases,
        where each phrase is `-delimited in the format: simp_phrase`raw_pinyin`formatted_pinyin
        Pinyin is space-delimited for a multi-word phrase
    Example:
        Input : "祝你有美好的天！"
        Output: "祝`zhu4$你`ni3$有`you3$美好`mei3 hao3$的`de5$天`tian1$！`!"
    """
    # Convert to Simplified, then tokenize
    s = trad_converter.convert(s)
    s = s.replace(' ', '') # remove whitespace (large whitespace is inconsistently tokenized)
    tokens = tokenizer(s)
    # Get NER component as set (if any)
    token_entities = nlp(' '.join([str(t) for t in tokens])).ents
    token_entities = set([str(t) for t in token_entities])
    # Break-down phrases that are not in CEDICT to lesser components
    str_tokens = [''] * max([len(t) for t in tokens]) * len(tokens) # pre-allocate upper-bound, clean-up after
    j = 0
    for i in range(len(tokens)):
        t = str(tokens[i])
        if t in CEDICT_SET or not entire_phrase_is_chinese(t):
            str_tokens[j] = t
            j += 1
        else:
            # use divide-and-conquer approach: recursively split until all tokens are accounted for
            subtokens = break_down_large_token_into_subtoken_list(t)
            n_st = len(subtokens)
            str_tokens[j: j+n_st] = subtokens
            j += n_st
    while str_tokens[-1] == '':
        str_tokens.pop()
    # Handle special characters to match tokenizer output
    # for special characters within an alphanumeric phrase, tokenizer splits it but pfmt doesn't
    n_pinyin  = len(s)
    init_pinyin_list = flatten_list(pfmt(s, style=Style.TONE3, neutral_tone_with_five=True))
    raw_pinyin_list = [''] * n_pinyin # pre-allocate since known size
    i, j = 0, 0
    while i < len(init_pinyin_list) and j < n_pinyin:
        curr_pinyin = init_pinyin_list[i]
        curr_pinyin = [str(t) for t in list(tokenizer(curr_pinyin))]
        phrase_len = len(curr_pinyin)
        raw_pinyin_list[j:j+phrase_len] = curr_pinyin
        j += phrase_len
        i += 1
    raw_pinyin_list = [py for py in raw_pinyin_list if py not in {'', ' '}]
    reversed_raw_pinyin_list = raw_pinyin_list[::-1]
    # Generate delimited string
    n_tokens = len(str_tokens)
    delimited_list = [''] * n_tokens # pre-allocate since known size
    # In this case, no conversion -> no pinyin -> not chinese (catches Chinese punctuation)
    not_chinese_phrase = lambda x: x == reversed_raw_pinyin_list[-1] 
    for i in range(n_tokens):
        phrase = str_tokens[i]
        if not_chinese_phrase(phrase):
            raw_pinyin = reversed_raw_pinyin_list.pop()
        else:
            pypinyin_list = [''] * len(phrase)
            for j in range(len(phrase)):
                pypinyin_list[j] = reversed_raw_pinyin_list.pop()
            # try lookup raw_pinyin in CEDICT_DF as source of truth, if not, default to pypinyin
            # lookup logic tries: 
            # 1) unique phrase match
            # 2) For multiple matches, first raw_pinyin_lower match
            #    Sort so in NER case, capitalized pinyin comes first.
            # Otherwise, uncapitalized pinyin comes first.
            # Default to pypinyin result
            try:
                cedict_phrase_df = CEDICT_DF.loc[phrase, :]
                if cedict_phrase_df.shape[0] == 1:
                    raw_pinyin = cedict_phrase_df.raw_pinyin.iloc[0]
                else:
                    raw_pinyin_lower = ' '.join(pypinyin_list).lower()
                    cedict_res_df = cedict_phrase_df.loc[raw_pinyin_lower, :]
                    if phrase in token_entities:
                        cedict_res_df.sort_values(acending=True, inplace=True)
                    else:
                        cedict_res_df.sort_values(acending=False, inplace=True)
                    raw_pinyin = cedict_res_df.raw_pinyin.iloc[0]
            except:
                raw_pinyin = ' '.join(pypinyin_list)
        delimited_list[i] = f"{phrase}`{raw_pinyin}"
    delimited_str = '$'.join(delimited_list)
    return delimited_str

if __name__ == '__main__':
    # Multi-threaded connections
    print("Starting socket server...")
    lsock = socket.socket(IPV4, TCP)
    lsock.bind((TOKENIZER_HOST, TOKENIZER_PORT))
    lsock.listen()
    print(f'listening on ({TOKENIZER_HOST}:{TOKENIZER_PORT})')
    lsock.setblocking(False)
    sel.register(lsock, selectors.EVENT_READ, data=None)

    def accept_wrapper(sock):
        conn, addr = sock.accept()
        print(f'accepted connection from: {addr}')
        conn.setblocking(False)
        data = types.SimpleNamespace(addr=addr, inb=b'', outb=b'')
        events = selectors.EVENT_READ | selectors.EVENT_WRITE
        sel.register(conn, events, data=data)

    def service_connection(key, mask):
        sock = key.fileobj
        data = key.data
        if mask & selectors.EVENT_READ:
            try:
                recv_data = sock.recv(MAX_BUF)  # Ready to read
                if recv_data:
                    # run NLP parser, then send results back
                    recv_data = tokenize_str(str(recv_data ,'utf-8'))
                    data.outb += bytes(recv_data, 'utf-8')
                else:
                    print(f'closing connection to: {data.addr}')
                    sel.unregister(sock)
                    sock.close()
            except SocketError as e:
                if e.errno != errno.ECONNRESET:
                    raise e
                print(f'connection to {data.addr} was reset by sender')
        if mask & selectors.EVENT_WRITE:
            if data.outb:
                header = bytes(str(len(data.outb)), 'utf-8')
                header += b' ' * (64 - len(header)) # 64-byte header
                sock.sendall(header)
                sock.sendall(data.outb)  # Ready to write
                data.outb = []

    while True:
        events = sel.select(timeout=None)
        for key, mask in events:
            if key.data is None:
                accept_wrapper(key.fileobj)
            else:
                service_connection(key, mask)