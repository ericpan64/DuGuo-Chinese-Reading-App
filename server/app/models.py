"""
Author: Eric Pan
Description: Data models

User -> List(Phrases) -> List(Words)
Document (linked to user via user_id)
CEDICT reference dictionary used for independent lookup
"""

from mongoengine import Document,EmbeddedDocument,EmbeddedDocumentListField
from mongoengine import StringField,DateTimeField,BooleanField
import datetime

class zwDocument(Document):
    """ Represents user imported text for a given session. Linked to zwUser via user_id """
    # Mainly saves context. This could also be broken down into phrases for metadata purposes..
    user_id = StringField(required=True)
    body = StringField(required=True)
    context_title = StringField(required=True)
    context_url = StringField(required=True)
    title = StringField()
    markup = StringField()
    title_markup = StringField()
    date_saved = DateTimeField(default=datetime.datetime.now)

    comment = StringField() # Optional user-added comment

class zwWord(EmbeddedDocument):
    """ Represents single chinese character part of a phrase """
    word = StringField(required=True) # raw text received
    is_simplified = BooleanField(required=True) # True if simplified, else is traditional
    pinyin = StringField()
    definition = StringField()
    # radicals = StringField()

class zwPhrase(EmbeddedDocument):
    """ Represents Chinese phrase saved by user. Only exists within zwUser."""
    phrase = StringField(required=True) # Actual text
    part_of_speech = StringField() # Populated by NLP tagger
    word_list = EmbeddedDocumentListField(zwWord)
    pinyin = StringField(required=True)
    definition = StringField(required=True)
    context_title = StringField()
    context_URL = StringField()

    comment = StringField() # Optional user-added comment

class zwUser(Document):
    """ Represents user """
    name = StringField()
    email = StringField(required=True)
    pw_hash = StringField(required=True) # Hash of user password
    registered_date = DateTimeField(default=datetime.datetime.now) # passing now function, not now value

    phrase_dict = EmbeddedDocumentListField(zwPhrase) # user's phrase dictionary

    """
    meta = {
        'db_alias':'core',
        'collection':'ZW_Users'
    }
    """

class CEDICT(Document):
    """CC-CEDICT mapping in database. This gets loaded when the application starts"""
    traditional = StringField(required=True)
    simplified = StringField(required=True)
    pinyin = StringField(required=True)
    definition = StringField(required=True)
