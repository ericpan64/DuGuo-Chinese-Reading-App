import mongoengine
import datetime
from ZW_document import ZW_Document

class ZW_User(mongoengine.Document):
    name = mongoengine.StringField(required=True)
    email = mongoengine.StringField(required=True)
    registered_date = mongoengine.DateTimeField(default=datetime.datetime.now) # passing now function, not now value

    documents = mongoengine.EmbeddedDocumentListField(ZW_Document)

    meta = {
        'db_alias':'core',
        'collection':'ZW_Users'
    }

