"""
Author: Martin Kess
Description: Runs CoreNLP/PoS tagger. Calls utils.py
"""

# -*- coding: utf-8 -*-

from utils import recv_all, socketcontext
from app import app
import socket

SEGMENT_SERVER_PORT = app.config['SEGMENT_SERVER_PORT']
POS_SERVER_PORT     = app.config['POS_SERVER_PORT']

def segment_text(text):
    ''' Given a sentence, insert spaces as words '''
    with socketcontext(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.connect(('localhost', SEGMENT_SERVER_PORT))
        sock.sendall(text.encode('utf-8') + '\n')
        return recv_all(sock).decode('utf-8').strip()

def get_parts_of_speech(sentence):
    ''' Given a segmented sentence, receive the sentence back with parts of
        speech tagged in XML form. '''
    with socketcontext(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.connect(('localhost', POS_SERVER_PORT))
        sock.sendall(sentence.encode('utf-8') + '\n')
        return recv_all(sock).decode('utf-8').strip()


