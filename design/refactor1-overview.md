# Zhongwen CRM - refactor 1

## Current Tech Stack
Web Server
    Backend: Python (Flask)
    Frontend: 
    Database: mongodb

Web Routing
    Main: /

Hosting
    Servers:

## Desired Tech Stack
Web
    Backend: Python (Flask)
    Frontend: plain JS/HTML/CSS
    Database: mongoDB

Web Routing
    Main: /
    Account signup: /register
    Account view: /user/{username}
    Upload document: /upload/{generatedDocId}
    View document: /view/{docId}
    View saved vocab: /vocab/{username}

Hosting
    Servers: AWS

## Refactor 1 Goals
Deploy following functionality:
- Home page loads
- User can upload document in temp session (no login required)
- User can login/create account to save uploaded documents
- Document gets tagged by NLP in backend