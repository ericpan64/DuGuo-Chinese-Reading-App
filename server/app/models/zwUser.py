"""
Author: Eric Pan
Description: User data model

User -> List(Phrases) -> List(Words)
Document (linked to user via user_id)
CEDICT reference dictionary used for independent lookup
"""

from mongoengine import Document, StringField, DateTimeField, EmbeddedDocumentListField, DictField
from models import zwChars as z
import datetime

class zwUser(Document):
    """ Represents user """
    # Required
    email = StringField(required=True)
    pw_hash = StringField(required=True) # Hash of user password
    pw_salt = StringField(required=True)

    # Optional
    name = StringField()
    registered_date = DateTimeField(default=datetime.datetime.now) # passing now function, not now value
    phrase_dict = EmbeddedDocumentListField(z.zwPhrase) # user's "phrase dictionary"
    context_titles = DictField()

    meta = {
        'db_alias':'core',
        'collection':'ZW_Users' # Sets name of collection
    }