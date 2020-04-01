"""
Author: Eric Pan, Francis Lin
Description: Contains website views
"""

from app import app

# === Views (previous, adjusted accordingly) ===
@app.route('/login', methods=['GET', 'POST'])
def login():
    pass

@app.route('/signup', methods=['GET', 'POST'])
def signup():
    pass

@app.route('/logout')
def logout():
    logout_user()
    flash('You have been logged out')
    return redirect(url_for('index'))

@app.route('/home')
@login_required
def home():
    ''' Display the user's documents. '''
    pass

@app.route('/edit', methods=['GET', 'POST'])
@app.route('/edit/<document_id>', methods=['GET', 'POST'])
@login_required
def edit(document_id=None):
    ''' Edit the given document ID '''
    pass

@app.route('/view/<document_id>')
@login_required
def view(document_id):
    ''' View given document. '''
    document = zwDocument.objects(id=document_id)
    # document = current_user.documents.filter_by(id=document_id).scalar()
    if not document:
        flash('Could not look up document {}'.format(document_id), 'error')
        return redirect(url_for('home'))

    return render_template('view.html', document=document)

@app.route('/delete/<document_id>', methods=['POST'])
@login_required
def delete_doc(document_id):
    document = zwDocument.objects(id=document_id)
    # document = current_user.documents.filter_by(id=document_id).scalar()
    if not document:
        response = jsonify({
                'Could not look up document {}'.format(document_id)
                })
        response.status_code = 404
        return response
    document.delete() # deletes document
    # db.session.delete(document)
    # db.session.commit()
    return jsonify({'success' : True })

# == Vocab List Display ==
@app.route('/vocab', methods=['GET', 'POST'])
@login_required
def vocab():
    # Three token formats:
        # 什么
        # 什么<tab>definition
        # 什么<tab>pinyin<tab>definition
        # For now though, we limit to words in the user's vocabulary
    pass

# == API Views == 
@app.route('/api/define')
@app.route('/api/define/<word>')
def define(word=None):

    ''' Return the CEDICT definition(s) for the word. '''
    pass


@app.route('/api/vocab/contains')
@app.route('/api/vocab/contains/<phrase>')
def vocab_contains(phrase=None):
    pass

@app.route('/api/vocab/add', methods=['POST'])
def vocab_add():
    """
    Adds value to current User's phrase dictionary
    :input: POST request
        'phrase' --> 1 or more phrases
    :return: True if succeeds, False otherwise
    """
    pass


@app.route('/api/vocab/delete', methods=['POST'])
@login_required
def vocab_delete():
    pass