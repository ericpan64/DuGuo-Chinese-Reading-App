"""
Author: Eric Pan
Description: Data model for chinese characters

Contains Words (atomic level), Phrases (sets of words), and CEDICT dictionary (sets of phrases)
"""

from mongoengine import Document, StringField, EmbeddedDocumentListField
from mongoengine import EmbeddedDocument, BooleanField, DictField, DateTimeField
from flask_login import UserMixin
import datetime

class zwPhrase(EmbeddedDocument):
    """ Represents Chinese phrase saved by user. Only exists within zwUser."""
    # Required
    phrase = StringField(required=True) # Actual text
    pinyin = StringField(required=True)
    definition = StringField(required=True)
    is_simplified = BooleanField(required=True)

    # Optional
    part_of_speech = StringField() # Populated by NLP tagger
    context_title = DictField() # Store context for ALL occurrences of the phrase
    context_URL = DictField()
    date_saved = DateTimeField(default=datetime.datetime.now)
    radicals = DictField()

    comment = StringField() # Optional user-added comment

class CEDICT(Document):
    """CC-CEDICT mapping in database. This gets loaded when the application starts"""
    # Required
    traditional = StringField(required=True)
    simplified = StringField(required=True)
    pinyin = StringField(required=True)
    definition = StringField(required=True)

class zwDocument(EmbeddedDocument):
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


class zwUser(Document, UserMixin):
    """ Represents user """
    # Required
    email = StringField(required=True)
    pw_hash = StringField(required=True) # Hash of user password
    # pw_salt = StringField(required=True)
    documents = EmbeddedDocumentListField(zwDocument)

    # Optional
    name = StringField()
    registered_date = DateTimeField(default=datetime.datetime.now) # passing now function, not now value
    phrase_dict = EmbeddedDocumentListField(zwPhrase) # user's "phrase dictionary"

    context_titles = DictField() # maps URL to Title of context