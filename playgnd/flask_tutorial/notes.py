from flask import Flask, request, url_for, redirect, render_template
import os
app = Flask("Flasky", template_folder='templates') # Initializes flask object

@app.route('/') # route function modifies URL
def hello_world():

    HTML = render_template('hello.html',marks=100) # Render HTML using render_template
    # return '<html><body>Hello world</body></html>' # Return string / valid HTML
    return HTML

@app.route('/thisURL')
def finalResult():
    # Testing returning an object --> It broke
    return str([1, 2, 3]) # needs to return HTML text of some sort

@app.route('/<thisName>', methods=['POST','GET']) # can accept variable URL input
def test(thisName):
    # Example of handling POST / GET values for login functionality
    user = thisName
    if request.method == 'POST': # POST form comes from HTML file
        result = request.form
        return render_template("hello.html",marks=100,result=result)
        # user = request.form['Variable from POST']
        #return redirect(url_for('test',thisName = user))
    else:

        print(request.args) # This is EMPTY for just URL, GET values come from ? query, & as delimiter
        return user
        # user = request.args.get('Variable from GET') # gets value from MultiDict
        # return redirect(url_for('test', thisName=user))

    s = 'Printed %s' % thisName
    return s

if __name__ == '__main__':
    cwd = os.getcwd()
    print(cwd)
    app.run(debug=True) # debug=True auto reloads after code changes



"""
https://www.tutorialspoint.com/flask/flask_sending_form_data_to_template.htm

Notes:
- WSGI = Web server gateway interface -- Python standard for web dev
    Werkzeug is a WSGI toolkit
        Flask is built on top of Werkzeug
    Jinja2 = template engine -- framework for combining template with dynamic data
        Here, web pages will be generated from HTML templates
        HTML acts as backbone, then variable data is added dynamically

- GET Request (default) -- unencrypted, sent via URL. HEAD is GET without response body
- POST Request -- sends HTML form data, not cached
- PUT Request -- replaces all current instances of target with sent content
- DELTE Request -- given URL, deletes all target instances

Jinja2 Template Syntax:
- {% ... %} for statements (e.g. if-else and endif)
- {{ ... }} for Expressions to print
- {# ... #} for comments not included in template output
- # ... ## for Line statements

- Static files (e.g. javascript/CSS) are expected in /static directory
    I.e. you can call javascript code if it's in the /static folder

Request Object Definitions:
- Form -- dictionary object, key value pairs
- args -- parsed query after URL (after ?)
- Cookies -- dict of Cookie name and values
- files -- data on uploaded files
- method -- request method

"""