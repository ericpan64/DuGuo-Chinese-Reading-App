"""
Author: Eric Pan
Description: Run this once to initiate connection to MongoDB
"""

import mongoengine

def global_init(alias,name):
    """
    :param alias: Name referring to specific connection in MongoEngine
    :param name: Name of DB to create in MongoDB
    :return: none
    Run once, this starts mongoDB on default port 27017
    Local copy -- multi-server hosting requires more params (passed in **args)"""
    mongoengine.register_connection(alias=alias, name=name)


if __name__ == "__main__":

    # Make global init to start database
    # ID User's unique session
        # If new user, initiate OAuth process
        # If previous user, then link to previous account
    alias = 'core'
    name = 'zwDB'
    global_init(alias,name)
    client = mongoengine.connect(name) # http://docs.mongoengine.org/guide/connecting.html