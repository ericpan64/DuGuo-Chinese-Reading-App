import spacy
from spacy.tokenizer import Tokenizer
import socket
import selectors
import types
from pypinyin import pinyin as pfmt
from pypinyin import Style
from config import TOKENIZER_HOST, TOKENIZER_PORT, MAX_BUF

# NLP import from: https://spacy.io/models/zh
nlp = spacy.load("zh_core_web_sm")
tokenizer = nlp.Defaults.create_tokenizer(nlp)

# Socket code adapted from: https://realpython.com/python-sockets
sel = selectors.DefaultSelector()
IPV4 = socket.AF_INET
TCP = socket.SOCK_STREAM

TONE_CHAR_SET = {
    'ā','ē','ī','ō','ū','ǖ','Ā','Ē','Ī','Ō','Ū','Ǖ',
    'á','é','í','ó','ú','ǘ','Á','É','Í','Ó','Ú','Ǘ',
    'ǎ','ě','ǐ','ǒ','ǔ','ǚ','Ǎ','Ě','Ǐ','Ǒ','Ǔ','Ǚ',
    'à','è','ì','ò','ù','ǜ','À','È','Ì','Ò','Ù','Ǜ',
    'ü','Ü',
}

def is_english_phrase(p):
    """ 
    For given phrase, if figures-out if it is entirely alphanumeric characters.

    Chinese chars use 3 bytes, so a phrase with Chinese chars will return false.

    Ex. len('京报') = 2
        len(bytes('京报', 'utf-8')) = 6
    """
    return len(p) == len(bytes(p, 'utf-8'))

def tokenize_str(s):
    """
    Given the input text s, tokenize and return as $-delimited phrases,
        where each phrase is `-delimited in the format: phrase`raw_pinyin`formatted_pinyin
        Pinyin is space-delimited for a multi-word phrase
    Example:
        Input : "祝你有美好的天！"
        Output: "祝`zhu4`zhù$你`ni3`nǐ$有`you3`yǒu$美好`mei3 hao3`měi hǎo$的`de5`de$天`tian1`tiān$！`！`！"
    """
    flatten_list = lambda l: [i for j in l for i in j] # [[a], [b], [c]] => [a, b, c]
    tokens = tokenizer(s)
    n_pinyin  = len(s)
    # Handle special characters to match tokenizer output
    # for special characters within an alphanumeric phrase, tokenizer splits it but pfmt doesn't
    # for spaces, tokenizer ignores but pfmt doesn't
    is_fmt_pinyin = lambda py: len(set(py).intersection(TONE_CHAR_SET)) > 0
    def get_corrected_syntax_for_pinyin_list(style):
        init_pinyin_list = flatten_list(pfmt(s, style=style, neutral_tone_with_five=True))
        pinyin_list = [''] * n_pinyin # pre-allocate since known size
        i, j = 0, 0
        while i < len(init_pinyin_list) and j < n_pinyin:
            curr_pinyin = init_pinyin_list[i]
            # formatted pinyin needs separate handling since it's a alphanumeric str with special character
            # ... that we _don't_ want to tokenize!
            if is_fmt_pinyin(curr_pinyin):
                pinyin_list[j] = curr_pinyin
                j += 1
            else:
                curr_pinyin = [str(t) for t in list(tokenizer(curr_pinyin))]
                phrase_len = len(curr_pinyin)
                pinyin_list[j:j+phrase_len] = curr_pinyin
                j += phrase_len
            i += 1
        pinyin_list = [py for py in pinyin_list if py not in {'', ' '}]
        return pinyin_list
    reversed_raw_pinyin_list = get_corrected_syntax_for_pinyin_list(Style.TONE3)[::-1] # works
    reversed_fmt_pinyin_list = get_corrected_syntax_for_pinyin_list(None)[::-1] # breaks b/c tokenizer splits special chars
    # Generate delimited string
    n_tokens = len(tokens)
    delimited_list = [''] * n_tokens # pre-allocate since known size
    for i in range(n_tokens):
        if is_english_phrase(str(tokens[i])):
            raw_pinyin = reversed_raw_pinyin_list.pop()
            fmt_pinyin = reversed_fmt_pinyin_list.pop()
        else:
            raw_pyin_list = [''] * len(tokens[i])
            fmt_pyin_list = [''] * len(tokens[i])
            for j in range(len(tokens[i])):
                raw_pyin_list[j] = reversed_raw_pinyin_list.pop()
                fmt_pyin_list[j] = reversed_fmt_pinyin_list.pop()
            raw_pinyin = ' '.join(raw_pyin_list)
            fmt_pinyin = ' '.join(fmt_pyin_list)
        delimited_list[i] = f"{tokens[i]}`{raw_pinyin}`{fmt_pinyin}"
    delimited_str = '$'.join(delimited_list)
    return delimited_str


def accept_wrapper(sock):
    conn, addr = sock.accept()
    print('accepted connection from: ', addr)
    conn.setblocking(False)
    data = types.SimpleNamespace(addr=addr, inb=b'', outb=b'')
    events = selectors.EVENT_READ | selectors.EVENT_WRITE
    sel.register(conn, events, data=data)

def service_connection(key, mask):
    sock = key.fileobj
    data = key.data
    if mask & selectors.EVENT_READ:
        recv_data = sock.recv(MAX_BUF)  # Should be ready to read
        if recv_data:
            # run NLP parser, then send results back
            recv_data = tokenize_str(str(recv_data ,'utf-8'))
            data.outb += bytes(recv_data, 'utf-8')
        else:
            print('closing connection to', data.addr)
            sel.unregister(sock)
            sock.close()
    if mask & selectors.EVENT_WRITE:
        if data.outb:
            print('echoing', repr(data.outb), 'to', data.addr)
            sent = sock.send(data.outb)  # Should be ready to write
            data.outb = data.outb[sent:]

if __name__ == '__main__':
    # Multi-threaded connections
    print("Starting socket server...")
    lsock  = socket.socket(IPV4, TCP)
    lsock.bind((TOKENIZER_HOST, TOKENIZER_PORT))
    lsock.listen()
    print('listening on', (TOKENIZER_HOST, TOKENIZER_PORT))
    lsock.setblocking(False)
    sel.register(lsock, selectors.EVENT_READ, data=None)

    while True:
        events = sel.select(timeout=None)
        for key, mask in events:
            if key.data is None:
                accept_wrapper(key.fileobj)
            else:
                service_connection(key, mask)

    # # Single threaded connections
    # with socket.socket(IPV4, TCP) as s:
    #     s.bind((TOKENIZER_HOST, TOKENIZER_PORT))
    #     s.listen()
    #     conn, addr = s.accept()
    #     with conn:
    #         print('Connected by: ', addr)
    #         while True:
    #             data = conn.recv(MAX_BUF)
    #             if not data:
    #                 break
    #             data = tokenize_str(str(data, 'utf-8'))
    #             conn.sendall(bytes(data, 'utf-8'))