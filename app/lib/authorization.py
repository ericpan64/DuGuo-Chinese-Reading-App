"""
Author: Eric Pan
Description: Helper functions interacting with database
"""

from models import zwUser
from bcrypt import gensalt,hashpw

def inputSecScan(input,lang="Python"):
    """
    Sanitizes user input
    :param input: String to parse
    :param lang: Programming language (default: Python)
    :return:
    """
    # TODO - Add input checking for Python code (security)
    if lang == "Python":
        input.replace("\"","") # Removes quotation comments
        input.replace("#","") # Remove comment lines

def createAccount(name, email, password): # -> Adds annotation to function
    """
    Function to create user account
    :param name: Desired username (string)
    :param email: Desired email (string)
    :param password: Password
    :return: zwUser object
    """
    # Generating password
    salt = gensalt()
    pw = hashpw(password,salt)

    # Creating user
    user = zwUser(name=name, email=email, pw_salt=salt, pw_hash=pw)
    user.save() # _id automatically generated with defaults
    return user


def prevUserID(db, email):
    """
    Gets _id if user already exists, else 0. Queries via email account
    :param db: database name (string)
    :param email: email address (string)
    :return: _id or 0
    """
    query = db.users.find({"email": email})
    if len(query)==0:
        return 0 # id not found
    return query['_id']