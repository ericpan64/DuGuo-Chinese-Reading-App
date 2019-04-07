from flask import Flask
from flask_login import LoginManager # Need to automate this step with Oauth2 later
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

# Testing code
@app.route('/') # Default landing page
def landing():
    return jsonify({'message':'This is the landing page'})

@app.route('/list') # Returns list of all items 
def returnAll():
    return jsonify(textList)

@app.route('/list/<string:body>', methods=['GET']) # Simple GET request
def returnOne(body):
    tx = [text for text in textList if text['body']==body]
    return jsonify({'body' : tx[0]['body']}) # Here: assuming text will be one item

@app.route('/list/upload',methods=['POST'])
def addOne():
    text = {'body' : request.json['body']} # The 'request' object defines structure of request
    textList.append(text) # adds to db
    return jsonify({'textList' : textList})

@app.route('/list/update/<string:body>',methods=['PUT'])
def updateOne(body):
    '''Updates only the first item in textList'''
    tx = [text for text in textList if text['body']==body] # related object to original textList
    tx[0]['body'] = request.json['body']
    return jsonify({'textList':textList})

@app.route('/list/delete/<string:body>',methods=['DELETE'])
def removeOne(body):
    tx = [text for text in textList if text['body']==body] # related object to original textList
    textList.remove(tx[0])
    return jsonify({'text':tx[0]})

# @app.route('/receiveParse', methods=['PUT'])
# def parse_request():
#     text = request.json['body']
#     return jsonify({''})


# # Testing 'Hello World' code below
# @app.route('/hello') # default here:localhost:5000 (port num)
# def hello():
#     return "Hello world!"

from jinja2 import Template
t = Template("{{variable}}")

if __name__ == '__main__':
    # print(t.render(variable="World!"))
    app.run(debug=True)