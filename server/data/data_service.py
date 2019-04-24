from ZW_users import ZW_User
# create account

def create_account(name:str, email:str) -> ZW_User: # -> Adds annotation to function
    user = ZW_User()
    user.name = name
    user.email=email
    user.save() # _id automatically generated with defaults
    return user
# log into account
# register __