from mongoengine import EmbeddedDocument, EmbeddedDocumentListField, StringField, BooleanField, DictField, DateTimeField
import zwWord
import datetime

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
