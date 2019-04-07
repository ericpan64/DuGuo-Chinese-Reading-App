import os
basedir = os.path.abspath(os.path.dirname(__file__))

CRF_ENABLED = True
SECRET_KEY = 'jdfaiwuhfi9ah49h4398u0rjg0a34'

SQLALCHEMY_DATABASE_URI = 'sqlite:///' + os.path.join(basedir, 'app.db')
SQLALCHEMY_MIGRATE_REPO = os.path.join(basedir, 'db_repository')

POS_SERVER_PORT = 8083
SEGMENT_SERVER_PORT = 8084