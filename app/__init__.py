# Initial tutorial: https://blog.miguelgrinberg.com/post/the-flask-mega-tutorial-part-i-hello-world

from flask import Flask

mFlask = Flask(__name__)

from app import routes
