from ZW_data_models import zwUser,zwDocument,vocabItem

def createAccount(name, email): # -> Adds annotation to function
    """
    :param name: Desired username (string)
    :param email: Desired email (string)
    :return: zwUser object created
    """
    user = zwUser()
    user.name = name
    user.email = email
    user.save() # _id automatically generated with defaults
    return user

def prevUserID(db, email):
    """
    :param db: database name (string)
    :param email: email address (string)
    :return: _id if user already exists, else 0. Queries via email account
    """
    query = db.users.find({"email": email})
    if len(query)==0:
        return 0 # id not found
    return query['_id']