"""
Author: Eric Pan
Description: Data models

User -> List(Phrases) -> List(Words)
Document (linked to user via user_id)
CEDICT reference dictionary used for independent lookup
"""

from mongoengine import Document,EmbeddedDocument,EmbeddedDocumentField,EmbeddedDocumentListField
from mongoengine import StringField,DateTimeField,BooleanField,DictField
import datetime

class zwDocument(Document):
    """ Represents user imported text for a given session. Linked to zwUser via user_id """
    # Mainly saves context. This could also be broken down into phrases for metadata purposes
    # Required
    user_id = StringField(required=True)
    body = StringField(required=True)
    context_title = StringField(required=True)
    context_url = StringField(required=True)

    # Optional
    title = StringField()
    date_saved = DateTimeField(default=datetime.datetime.now)
    comment = StringField() # Optional user-added comment

    meta = {'allow_inheritance' : True}

class zwWord(EmbeddedDocument):
    """ Represents single chinese character part of a phrase """
    # Required
    word = StringField(required=True) # raw text received
    is_simplified = BooleanField(required=True) # True if simplified, else is traditional
    pinyin = StringField(required=True)

    # Optional
    definition = StringField()
    radicals = DictField()

class zwPhrase(EmbeddedDocument):
    """ Represents Chinese phrase saved by user. Only exists within zwUser."""
    # Required
    phrase = EmbeddedDocumentListField(zwWord, required=True) # Actual text
    pinyin = StringField(required=True)
    definition = StringField(required=True)
    is_simplified = BooleanField(required=True)

    # Optional
    part_of_speech = StringField() # Populated by NLP tagger
    context_title = DictField() # Store context for ALL occurrences of the phrase
    context_URL = DictField()
    date_saved = DateTimeField(default=datetime.datetime.now)

    comment = StringField() # Optional user-added comment

class zwUser(Document):
    """ Represents user """
    # Required
    email = StringField(required=True)
    pw_hash = StringField(required=True) # Hash of user password
    pw_salt = StringField(required=True)

    # Optional
    name = StringField()
    registered_date = DateTimeField(default=datetime.datetime.now) # passing now function, not now value
    phrase_dict = EmbeddedDocumentListField(zwPhrase) # user's "phrase dictionary"

    meta = {
        'db_alias':'core',
        'collection':'ZW_Users' # Sets name of collection
    }

class CEDICT(Document):
    """CC-CEDICT mapping in database. This gets loaded when the application starts"""
    # Required
    traditional = EmbeddedDocumentField(zwPhrase,required=True)
    simplified = EmbeddedDocumentField(zwPhrase, required=True)
    pinyin = StringField(required=True)
    definition = StringField(required=True)

