
# Notes for Mongoengine
"""
Section 4-1 Tutorial (page 13):
    https://buildmedia.readthedocs.org/media/pdf/mongoengine-odm/latest/mongoengine-odm.pdf


MongoDB Crash Course:

    Building Data Model
        Embed if:
            Data wanted 80% of the time
            Small bounded set
            Answers the most valuable queries
        Integration DB - Central DB for multiple apps
            Difficult to "agree" between apps, data integrity difficults
        Application DB - one per application
            More suited for Document-Database / MongoDB

    Demo App: Example 'Snake BnB'
        Registering multi-server connections, provide:
            username, password, host, port, authentication_source, authentication method, ssl, ssl certs
"""


from mongoengine import *

connect('test-DB') # connects mongoengine to existing mongodb database