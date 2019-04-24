from flask import Flask
from flask import request # Handles HTTP requests
from flask import jsonify
# import json

textList = [{'body':'notChinese'},{'body' : '中文'}] # Need to replace with MongoDB database

app = Flask(__name__)
app.config.from_object('config')

# Need to replace with OAuth
# login_manager = LoginManager()
# login_manager.session_protection = 'strong' # 
# login_manager.login_view = 'login' # 

# login_manager.init_app(app)

@app.context_processor
def jinja_functions(): # Template engine
    pass # need to add Chinese word rendering here

# Testing routing code (move to route.py later)
@app.route('/') # Default landing page
def landing():
    return jsonify({'message':'This is the landing page'})

@app.route('/print') # Returns list of all items 
def printAll():
    return jsonify(textList)

@app.route('/uploadText',methods=['POST'])
def postMethod(): # https://stackoverflow.com/questions/42893826/flask-listen-to-post-request
    data = request.get_json(force=True) # this is still NULL
    textList.append(data) # add to db
    # textList.append({"got":"here"})
    return jsonify(data)

# Add route to pull-up most recent document

# Handle file removal internally / through other user commands

if __name__ == '__main__':
    app.run(debug=True)