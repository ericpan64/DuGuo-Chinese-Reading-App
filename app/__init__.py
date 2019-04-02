# Initial tutorial: https://blog.miguelgrinberg.com/post/the-flask-mega-tutorial-part-i-hello-world

from flask import Flask
from flask.ext.login import LoginManager

app = Flask(__name__)
