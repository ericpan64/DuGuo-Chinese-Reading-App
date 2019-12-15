
# Notes for Mongoengine
"""
Section 4-1 Tutorial (page 13):
    https://buildmedia.readthedocs.org/media/pdf/mongoengine-odm/latest/mongoengine-odm.pdf


MongoDB Crash Course:
    Link: https://www.youtube.com/watch?v=E-1xI85Zog8&t=2415s

    Building Data Model
        Embed if:
            Data wanted 80% of the time
            Small bounded set
            Answers the most valuable queries
        Integration DB - Central DB for multiple apps
            Difficult to "agree" between apps, data integrity difficults
        Application DB - one per application
            More suited for Document-Database / MongoDB

    POINTERS:
        ObjectIdField() --> Allows for lookup from document, rather than containing

    Demo App: Example 'Snake BnB'
        Registering multi-server connections, provide:
            username, password, host, port, authentication_source, authentication method, ssl, ssl certs

    Concept: Inserting
        Creating the object creates in Python IDE, saving creates database ID
            object = Class(param1=..., ...)
            object.save --> creates ObjectId
        Can insert multiple objects in a list
            list = []
            ... # add items to list
            Class.objects().insert(list)

    Concept: Querying (direct match)
        object = Class.objects().filter(param=query).first()
            # filter on 1 or more fields, first() picks first (else returns None)

    Concept: Querying (subdocuments)
        # Use __ to separate/navigate subdocument levels
        # Case: Class-A contains Class-B. Want to query objects in Class-A, with param from Class-B
        # Execute Query before returning results (general best practice)
        object-A = Class-A.objects(object-B-in-A__param = ...).all()
        booked_cages = Cage.objects(bookings__guest_snake_id__in=owner.snake_ids).all()

    Concept: Querying (operators)
        # Use ___ afterward a param and before = to do query
        E.g. object = Class.objects(param__gte=int).count()
            or object = Class.objects().filter(param__gte=int)

"""


from mongoengine import *

connect('test-DB') # connects mongoengine to existing mongodb database