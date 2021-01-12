import spacy
from spacy.tokenizer import Tokenizer
import socket
import selectors
import types
from pypinyin import pinyin as pfmt
from pypinyin import Style
from config import TOKENIZER_HOST, TOKENIZER_PORT

# NLP import from: https://spacy.io/models/zh
nlp = spacy.load("zh_core_web_sm")
tokenizer = nlp.Defaults.create_tokenizer(nlp)

# Socket code adapted from: https://realpython.com/python-sockets
sel = selectors.DefaultSelector()
IPV4 = socket.AF_INET
TCP = socket.SOCK_STREAM
MAX_BUF = 102400 # 1MB

def tokenize_str(s):
    """
    Given the input text s, tokenize and return as $-delimited phrases,
        where each phrase is `-delimited in the format: phrase`formatted_pinyin
        Pinyin is space-delimited for a multi-word phrase

    Example:
        Input : "祝你有美好的天！"
        Output: "祝`zhù$你`nǐ$有`yǒu$美好`měi hǎo$的`de$天`tiān$！`！"
    """
    flatten_list = lambda l: [i for j in l for i in j] # [[a], [b], [c]] => [a, b, c]
    reversed_pinyin_list = flatten_list(pfmt(s, style=Style.TONE3, neutral_tone_with_five=True))[::-1]
    tokens = tokenizer(s)
    delimited_list = [''] * len(tokens) # pre-allocate size
    for i in range(len(tokens)):
        pyin_list = [''] * len(tokens[i])
        for j in range(len(tokens[i])):
            pyin_list[j] = reversed_pinyin_list.pop()
        pyin = ' '.join(pyin_list)
        delimited_list[i] = f"{tokens[i]}`{pyin}"
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