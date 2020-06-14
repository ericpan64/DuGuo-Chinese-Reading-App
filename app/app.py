"""
Author: Eric Pan
Description: Contains app startup code + views

Example post data
{
  "URL": "https://zh.wikipedia.org/wiki/Wikipedia:%E9%A6%96%E9%A1%B5",
  "body": "\uff0c\u7f8e\u56fd\u80af\u5854\u57fa\u5dde\u7684\u653f\u6cbb\u5bb6\uff0c\u66fe\u62c5\u4efb\u8be5\u5dde\u7b2c44\u548c49\u4efb\u5dde\u957f\u548c\u8054\u90a6\u53c2\u8bae\u5458\u3002",
  "title": "\u7ef4\u57fa\u767e\u79d1\uff0c\u81ea\u7531\u7684\u767e\u79d1\u5168\u4e66",
  "user": "108978819231638632466"
}
"""

from flask import Flask, render_template
from lib.mongo import connectToMongo, loadCEDICT

""" === Server Start-up === """
# Mongodb needs to be started on port 27017
connectToMongo(alias='db', name='crm_main')
loadCEDICT()

# Start-up website
app = Flask(__name__)
app.config.from_object('config')

""" === Views === """

@app.route('/')
@app.route('/index')
def landing():
    return render_template('index.html')


# # needed to load User
# @login_manager.user_loader
# def load_user(user_id):
#     return zwUser.objects(id=user_id).first()

@app.route('/login')
def login():
    pass

@app.route('/signup')
def signup():
    pass


# TODO
# Logout
# Home Page
# Edit Document page
# View Document page
# Delete Document
# Vocab List page
# Vocab List add
# CEDICT lookup


if __name__ == '__main__':
    app.run(debug=True,use_reloader=False)

    print("We got here")