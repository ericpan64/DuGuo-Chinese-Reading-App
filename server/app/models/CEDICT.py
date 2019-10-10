from mongoengine import Document, EmbeddedDocumentField, StringField
import zwPhrase

class CEDICT(Document):
    """CC-CEDICT mapping in database. This gets loaded when the application starts"""
    # Required
    traditional = EmbeddedDocumentField(zwPhrase,required=True)
    simplified = EmbeddedDocumentField(zwPhrase, required=True)
    pinyin = StringField(required=True)
    definition = StringField(required=True)
