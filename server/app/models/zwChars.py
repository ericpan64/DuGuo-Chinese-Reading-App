"""
Author: Eric Pan
Description: Data model for chinese characters

Contains Words (atomic level), Phrases (sets of words), and CEDICT dictionary (sets of phrases)
"""

from mongoengine import Document, EmbeddedDocumentField, StringField
from mongoengine import EmbeddedDocument, BooleanField, DictField, EmbeddedDocumentListField, DateTimeField
import datetime

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

class CEDICT(Document):
    """CC-CEDICT mapping in database. This gets loaded when the application starts"""
    # Required
    traditional = EmbeddedDocumentField(zwPhrase,required=True)
    simplified = EmbeddedDocumentField(zwPhrase, required=True)
    pinyin = StringField(required=True)
    definition = StringField(required=True)
