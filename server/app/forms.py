"""
Author: Martin Kess
Description: Handles forms for user input

Modified to fit new zw Model schema
"""

from flask_wtf import FlaskForm
from wtforms import StringField,BooleanField,PasswordField,ValidationError,TextAreaField,FileField
from wtforms.validators import DataRequired,Email,EqualTo

from models.zwUser import zwUser # Modified

class LoginForm(FlaskForm):
    email = StringField('Email', validators=[DataRequired(), Email()])
    password = PasswordField('Password', validators=[DataRequired()])
    remember_me = BooleanField('Remember me', default=False)

class SignupForm(FlaskForm):
    email = StringField('Email', validators=[DataRequired(), Email()])
    password = PasswordField('Password', validators=[DataRequired(), EqualTo('confirm_password', message='Passwords must match')])
    confirm_password = PasswordField('Confirm Password', validators=[DataRequired()])
    remember_me = BooleanField('Remember me', default=False)

    def validate_email(self, email):
        if zwUser.objects(email=email.data).first():
        #if User.query.filter_by(email=email.data).first():
            raise ValidationError('Email already registered')


class EditDocumentForm(FlaskForm):
    title = StringField('Title', validators=[DataRequired()])
    original_url = StringField('Original URL')
    contents = TextAreaField('Contents')

class VocabFileForm(FlaskForm):
    vocab_file = FileField('Load Vocabulary File')