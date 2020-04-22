"""
Author: Eric Pan, Francis Lin
Description: Contains website views + code to start server

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

# # Connect and load data to database
connectToMongo(alias='db', name='crm_main')
loadCEDICT()

# Start-up website
web = Flask(__name__)
web.config.from_object('config')

# === Views ===
@web.route('/') # Default landing page
@web.route('/index')
def landing():
    return render_template('index.html')


# # needed to load User
# @login_manager.user_loader
# def load_user(user_id):
#     return zwUser.objects(id=user_id).first()

@web.route('/login')
def login():
    pass

@web.route('/signup')
def signup():
    pass

# @web.route('/logout')
# def logout():
#     logout_user()
#     flash('You have been logged out')
#     return redirect(url_for('index'))
#
# @web.route('/home')
# @login_required
# def home():
#     ''' Display the user's documents. '''
#     pass
#
# @web.route('/edit', methods=['GET', 'POST'])
# @web.route('/edit/<document_id>', methods=['GET', 'POST'])
# @login_required
# def edit(document_id=None):
#     ''' Edit the given document ID '''
#     pass
#
# @web.route('/view/<document_id>')
# @login_required
# def view(document_id):
#     ''' View given document. '''
#     document = zwDocument.objects(id=document_id)
#     # document = current_user.documents.filter_by(id=document_id).scalar()
#     if not document:
#         flash('Could not look up document {}'.format(document_id), 'error')
#         return redirect(url_for('home'))
#
#     return render_template('view.html', document=document)
#
# @web.route('/delete/<document_id>', methods=['POST'])
# @login_required
# def delete_doc(document_id):
#     document = zwDocument.objects(id=document_id)
#     # document = current_user.documents.filter_by(id=document_id).scalar()
#     if not document:
#         response = jsonify({
#                 'Could not look up document {}'.format(document_id)
#                 })
#         response.status_code = 404
#         return response
#     document.delete() # deletes document
#     # db.session.delete(document)
#     # db.session.commit()
#     return jsonify({'success' : True })
#
# # == Vocab List Display ==
# @web.route('/vocab', methods=['GET', 'POST'])
# @login_required
# def vocab():
#     # Three token formats:
#         # 什么
#         # 什么<tab>definition
#         # 什么<tab>pinyin<tab>definition
#         # For now though, we limit to words in the user's vocabulary
#     pass
#
# # == API Views ==
# @web.route('/api/define')
# @web.route('/api/define/<word>')
# def define(word=None):
#
#     ''' Return the CEDICT definition(s) for the word. '''
#     pass
#
#
# @web.route('/api/vocab/contains')
# @web.route('/api/vocab/contains/<phrase>')
# def vocab_contains(phrase=None):
#     pass
#
# @web.route('/api/vocab/add', methods=['POST'])
# def vocab_add():
#     """
#     Adds value to current User's phrase dictionary
#     :input: POST request
#         'phrase' --> 1 or more phrases
#     :return: True if succeeds, False otherwise
#     """
#     pass
#
#
# @web.route('/api/vocab/delete', methods=['POST'])
# @login_required
# def vocab_delete():
#     pass


if __name__ == '__main__':
    web.run(debug=True,use_reloader=False)