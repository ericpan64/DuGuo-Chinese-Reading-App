"""
Authors: Eric Pan
Description: Runs CoreNLP/PoS tagger using sockets as context managers
    CoreNLP must be running on separate port on the server
    Socket sends data back/forth
        Socket calls are implemented as a context manager so each call is self-contained
        (i.e. connections are opened/closed properly)

Socket Reference:
    https://www.geeksforgeeks.org/socket-programming-python/
"""

# -*- coding: utf-8 -*-

from contextlib import contextmanager
import socket

# === Stanford NLP API Definition ===
# Adopted from blog post: https://www.khalidalnajjar.com/how-to-setup-and-use-stanford-corenlp-server-with-python/
# Run shell command first
from stanfordcorenlp import StanfordCoreNLP
import json

class StanfordNLP:
    def __init__(self, host='http://localhost', port=9000):
        self.nlp = StanfordCoreNLP(host, port=port,
                                   timeout=30000)  # , quiet=False, logging_level=logging.DEBUG)
        self.props = {
            'annotators': 'tokenize,ssplit,pos,lemma,ner,parse,depparse,dcoref,relation',
            'pipelineLanguage': 'zh',
            'outputFormat': 'json'
        }

    def word_tokenize(self, sentence):
        return self.nlp.word_tokenize(sentence)

    def pos(self, sentence):
        return self.nlp.pos_tag(sentence)

    def ner(self, sentence):
        return self.nlp.ner(sentence)

    def parse(self, sentence):
        return self.nlp.parse(sentence)

    def dependency_parse(self, sentence):
        return self.nlp.dependency_parse(sentence)

    def annotate(self, sentence):
        return json.loads(self.nlp.annotate(sentence, properties=self.props))

    @staticmethod
    def tokens_to_dict(_tokens):
        tokens = defaultdict(dict)
        for token in _tokens:
            tokens[int(token['index'])] = {
                'word': token['word'],
                'lemma': token['lemma'],
                'pos': token['pos'],
                'ner': token['ner']
            }
        return tokens


# === Port Definitions ===
SEGMENT_SERVER_PORT = 8083
POS_SERVER_PORT     = 8084

# === Context Manager Utility Functions  ===
# from:
@contextmanager
def socketcontext(*args, **kw):
    """
    Utility function to initiate/close a socket (this isn't implemented natively in Python's socket library)
    :returns: Generator for this function
    Ref on code: http://stackoverflow.com/a/16772520
    Ref on yield: https://stackoverflow.com/questions/231767/what-does-the-yield-keyword-do
    """
    s = socket.socket(*args, **kw)
    try:
        yield s
    finally:
        s.close()

def recv_all(sock):
    """
    Given server socket, receive ALL of the data! (on port 4096)
    :param sock: Socket object (server)
    :returns: data received by socket (string)
    """
    data = []
    while True:
        r = sock.recv(4096) # Receives data on this port
        if not r:
            break
        data.append(r)
    return ''.join(data)

# === CoreNLP Segmenter API Calls ===

# TODO Reimplement using CoreNLP API
def segment_text(text):
    pass
#     ''' Given a sentence, insert spaces as words '''
#     with socketcontext(socket.AF_INET, socket.SOCK_STREAM) as sock:
#         sock.connect(('localhost', SEGMENT_SERVER_PORT))
#         sock.sendall(text.encode('utf-8') + '\n')
#         return recv_all(sock).decode('utf-8').strip()
#
def get_parts_of_speech(sentence):
    pass
#     ''' Given a segmented sentence, receive the sentence back with parts of
#         speech tagged in XML form. '''
#     with socketcontext(socket.AF_INET, socket.SOCK_STREAM) as sock:
#         sock.connect(('localhost', POS_SERVER_PORT))
#         sock.sendall(sentence.encode('utf-8') + '\n')
#         return recv_all(sock).decode('utf-8').strip()


