"""
Author: Eric Pan
Description: Contains website views

The sections marked as === Views (previous...) === are code written by Martin Kess and adjusted to match the NoSQL schema
The import statement isn't working, so the __init__ file is housing the views for the demo

TODO: reimplement adding schema
"""

from __init__ import app
from flask import render_template, redirect, flash, url_for, request, jsonify, make_response
from forms import LoginForm, SignupForm, EditDocumentForm, VocabFileForm
from models import zwUser,zwDocument
from models import zwChars as z
from lib.zhongwen import annotate_text, query_cedict

from flask_login import login_user,login_required,logout_user,current_user

# === Views (previous, adjusted accordingly) ===
# == User Login Handling ==
@app.route('/login', methods=['GET', 'POST'])
def login():
    form = LoginForm()

    # Validates user
    if form.validate_on_submit():
        user = zwUser.objects(email=form.email.data).first()
        # user = User.query.filter_by(email=form.email.data).scalar()
        if user and user.check_password(form.password.data):
            login_user(user, remember=form.remember_me.data)
            flash('Welcome back, {}'.format(form.email.data))
            return redirect(url_for('home'))

        flash('Invalid username or password', 'error')

    return render_template(
        'login.html',
        form=form)

@app.route('/signup', methods=['GET', 'POST'])
def signup():
    form = SignupForm()

    if form.validate_on_submit():
        user = zwUser(form.email.data, form.password.data)
        user.save() # add user to zwUser collection
        login_user(user, remember=form.remember_me.data)
        flash('Account created - welcome {}'.format(form.email.data))
        return redirect(url_for('home'))

    return render_template('signup.html', form=form)

@app.route('/logout')
def logout():
    logout_user()
    flash('You have been logged out')
    return redirect(url_for('index'))


# == Document Handling ==
@app.route('/home')
@login_required
def home():
    ''' Display the user's documents. '''
    docs = current_user.documents.all()
    return render_template('home.html', documents=docs)

@app.route('/edit', methods=['GET', 'POST'])
@app.route('/edit/<document_id>', methods=['GET', 'POST'])
@login_required
def edit(document_id=None):
    ''' Edit the given document ID '''
    form = EditDocumentForm()

    if form.validate_on_submit():
        title_markup = annotate_text(form.title.data)
        markup = annotate_text(form.contents.data)

        if document_id:
            document = zwDocument.objects(id=document_id)
            # document = current_user.documents.filter_by(id=document_id).scalar()
            if document is None:
                flash('Could not load document {}'.format(document_id), 'error')
                return redirect(url_for('home'))
            document.title = form.title.data
            document.title_markup = title_markup
            document.context_url = form.original_url.data
            document.contents = form.contents.data
            document.markup = markup
            flash('Document updated')
        else:
            flash('Created new document')
            document = zwDocument(user_id=current_user.id,contents=form.contents.data,context_title=form.title.data, context_url=form.original_url.data)

        # Add document to User's list
        #   Adjusted schema to accomodate queries
        # Ref: https://stackoverflow.com/questions/14742513/mongoengine-how-to-append-a-new-document-to-an-embedded-listfield-document
        document.save()
        return redirect(url_for('view', document_id=document.id))

    if document_id and request.method == 'GET':
        document = zwDocument.objects(id=document_id)
        # document = current_user.documents.filter_by(id=document_id).scalar()
        if not document:
            flash('Could not look up document {}'.format(document_id), 'error')
            return redirect(url_for('home'))

        form.title.data = document.title
        form.original_url.data = document.context_url
        form.contents.data = document.contents

    return render_template('edit.html', form=form)

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
    form = VocabFileForm()

    if form.validate_on_submit():
        for line in form.vocab_file.data.stream:
            s = line.strip()
            if len(s) == 0:
                continue

            # strip out comments
            if s[0] == '#' or (len(s) > 1 and s[0:2] == '//'):
                continue

            # Three formats:
            # 什么
            # 什么<tab>definition
            # 什么<tab>pinyin<tab>definition
            # For now though, we limit to words in the user's vocabulary

            tokens = s.split('\t')
            if len(tokens) > 0:
                entries = query_cedict(tokens).as_pymongo()[0]
                if len(entries) == 0:
                    pass
                else: 
                    for entry in entries:
                        # http://docs.mongoengine.org/guide/document-instances.html
                        current_user.update(push__phrase_dict=entry)

                # else:
                #     # Some words have 2-3 definitions - the user here gets all of them.
                #     for entry in entries:
                #         # Filter out duplicates
                #         if current_user.vocab.filter_by(user_id=current_user.id, cedict_id=entry.id).count() == 0:
                #             db.session.add(Vocab(current_user, entry))
            else:
                # unknown format
                continue
        current_user.save()
    return render_template('vocab.html', form=form)

# == API Views == 
@app.route('/api/define')
@app.route('/api/define/<word>')
def define(word=None):

    ''' Return the CEDICT definition(s) for the word. '''
    if word is None:
        word = request.args.get('word', None)

    entry = query_cedict(word).as_pymongo()[0]

    if entry is None:
        response = jsonify({'error': 'could not find definition for {}'.format(word)})
        response.status_code = 404
        return response

    definitions = []
    definitions.append({
        'pinyin' : entry['pinyin'],
        'definitions' : [e for e in entry['definition'].split('/') if len(e) > 0]
        })

    return jsonify({'definitions' : definitions})


@app.route('/api/vocab/contains')
@app.route('/api/vocab/contains/<phrase>')
def vocab_contains(phrase=None):
    if phrase is None:
        phrase = request.args.get('phrase', None)

    return jsonify({
        'contains': current_user.objects(phrase_dict__phrase=phrase)
        # 'contains': CEDICT.query.filter_by(user_id=current_user.id, phrase=phrase).count() > 0
        })


# TODO: fix vocab adding hierarchy and convert queries to NoSQL
"""
@app.route('/api/vocab/add', methods=['POST'])
def vocab_add():
    ''' Add a phrase to the current user's vocabulary list '''
    if not current_user.is_authenticated:
        response = jsonify({'error': 'User not logged in'})
        response.status_code = 400
        return response

    phrase = request.form.get('phrase')

    definitions = CEDICT.objects(simplified=phrase).as_pymongo()[0]
    # definitions = CEDICT.query.filter_by(simplified=phrase).all()

    
    if len(definitions) > 0:
        for definition in definitions:
            v = zwPhrase(phrase,)
            v.save()

        db.session.commit()

        return jsonify({
            'success': True
            })
    else:
        response = jsonify({'error' : 'Could not find \'{}\' in CEDICT dictionary - you won\'t be able to add it to your vocabulary.'.format(phrase.encode('utf-8'))})
        response.status_code = 404
        return response


@app.route('/api/vocab/delete', methods=['POST'])
@login_required
def vocab_delete():
    phrase = request.form.get('phrase')
    for entry in current_user.vocab.filter_by(simplified=phrase).all():
        db.session.delete(entry)
    db.session.commit()

    return jsonify({'success': True})

@app.route('/api/vocab/all')
def vocab_all():
    ''' Get all of a user's vocab '''
    # TODO(mkess): Should this be paged? And if so... how?
    return jsonify({
        'vocab' : current_user.vocab.all()
    })

@app.route('/api/vocab/download_txt')
@login_required
def vocab_download_txt():
    txt = '\n'.join(entry.cedict.simplified for entry in current_user.vocab.all())
    response = make_response(txt)
    response.headers['Content-Disposition'] = 'attachment; filename=vocab.txt;'

    return response
"""