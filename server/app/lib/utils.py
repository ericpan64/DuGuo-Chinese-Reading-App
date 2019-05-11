"""
Author: Martin Kess
Description: Socket manager for CoreNLP/PoS taggers
"""

from contextlib import contextmanager
import socket

# from: http://stackoverflow.com/a/16772520
@contextmanager
def socketcontext(*args, **kw):
    s = socket.socket(*args, **kw)
    try:
        yield s
    finally:
        s.close()

def recv_all(sock):
    data = []
    while True:
        r = sock.recv(4096)
        if not r:
            break
        data.append(r)
    return ''.join(data)

