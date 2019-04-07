# Coding: UTF-8

from utils import recv_all, socketcontext
from app import app
import socket

SEGMENTER_SERVER_PORT = app.config['SEGMENTER_SERVER_PORT']
PoS_SERVER_PORT = app.config['PoS_SERVER_PORT']