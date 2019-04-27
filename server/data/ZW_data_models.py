import mongoengine
import datetime

class zwDocument(mongoengine.EmbeddedDocument):
    """ Represents user imported text for a given session.Only exists within zwUser """
    body_text = mongoengine.StringField(required=True)
    context_title = mongoengine.StringField(required=True)
    context_URL = mongoengine.StringField(required=True)

    comment = mongoengine.StringField()


class vocabItem(mongoengine.EmbeddedDocument):
    """ Represents 中文 phrase saved by user. Only exists within zwUser."""
    phrase = mongoengine.StringField(required=True)
    pinyin = mongoengine.StringField(required=True)
    definition = mongoengine.StringField(required=True)
    context_title = mongoengine.StringField()
    context_URL = mongoengine.StringField()

    comment = mongoengine.StringField()

class zwUser(mongoengine.Document):
    name = mongoengine.StringField(required=True)
    email = mongoengine.StringField(required=True)
    pw_hash = mongoengine.StringField(required=True) # Hash of user password
    registered_date = mongoengine.DateTimeField(default=datetime.datetime.now) # passing now function, not now value

    documents = mongoengine.EmbeddedDocumentListField(zwDocument)
    phrase_dict = mongoengine.EmbeddedDocumentListField(vocabItem) # user's phrase dictionary

    meta = {
        'db_alias':'core',
        'collection':'ZW_Users'
    }