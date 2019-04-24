import mongoengine

class ZW_Document(mongoengine.EmbeddedDocument):
    """ Embedded Document. Only exists within ZW_User Document """
    body_text = mongoengine.StringField(required=True)
    context_Title = mongoengine.StringField(required=True)
    context_URL = mongoengine.StringField(required=True)

    comment = mongoengine.StringField(required=False)
