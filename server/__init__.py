from flask import Flask
from flask_login import LoginManager # Need to automate this step with Oauth2 later

app = Flask(__name__)
app.config.from_object('config')

login_manager = LoginManager()
login_manager.session_protection = 'strong' # 
login_manager.login_view = 'login' # 

login_manager.init_app(app)

@app.context_processor
def jinja_functions(): # 
    pass
# Testing 'Hello World' code below
@app.route('/hello') # default here:localhost:5000 (port num)
def hello():
    return "Hello world!"

from jinja2 import Template
t = Template("Hello {{variable}}")

if __name__ == '__main__':
    print(t.render(variable="World!"))
    # app.run(debug=True)