import spacy
from spacy.tokenizer import Tokenizer
import socket
import selectors
import types

# NLP import from: https://spacy.io/models/zh
nlp = spacy.load("zh_core_web_lg")
tokenizer = nlp.Defaults.create_tokenizer(nlp)

# Socket code adapted from: https://realpython.com/python-sockets
sel = selectors.DefaultSelector()
HOST = '127.0.0.1' # Opt for numeric address
PORT = 8881
IPV4 = socket.AF_INET
TCP = socket.SOCK_STREAM
MAX_BUF = 102400 # 1MB

def tokenize_str(s):
    tokens = tokenizer(s)
    comma_delimited_str = ','.join([str(t) for t in tokens])
    return comma_delimited_str

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
    lsock  = socket.socket(IPV4, TCP)
    lsock.bind((HOST, PORT))
    lsock.listen()
    print('listening on', (HOST, PORT))
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
    #     s.bind((HOST, PORT))
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