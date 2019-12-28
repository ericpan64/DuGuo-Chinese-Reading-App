from mongoengine import EmbeddedDocument, StringField, DateTimeField
import datetime

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