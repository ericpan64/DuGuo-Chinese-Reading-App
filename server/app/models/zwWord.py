from mongoengine import EmbeddedDocument, StringField, BooleanField, DictField

class zwWord(EmbeddedDocument):
    """ Represents single chinese character part of a phrase """
    # Required
    word = StringField(required=True) # raw text received
    is_simplified = BooleanField(required=True) # True if simplified, else is traditional
    pinyin = StringField(required=True)

    # Optional
    definition = StringField()
    radicals = DictField()