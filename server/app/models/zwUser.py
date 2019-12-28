"""
Author: Eric Pan
Description: User data model

User -> List(Phrases) -> List(Words)
Document (linked to user via user_id)
CEDICT reference dictionary used for independent lookup
"""

from mongoengine import Document, StringField, DateTimeField, BooleanField, EmbeddedDocumentListField, DictField, ListField
from models import zwChars as z
from models.zwDocument import zwDocument
from flask_login import UserMixin
import datetime

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
    phrase_dict = EmbeddedDocumentListField(z.zwPhrase) # user's "phrase dictionary"

    context_titles = DictField() # maps URL to Title of context
