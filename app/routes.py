from app import mFlask

@app.route('/') # example of decorator (modifies function following @ variable)
@app.route('/index')
def index():
    return "Hello, World!"
