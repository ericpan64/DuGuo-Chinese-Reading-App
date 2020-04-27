# Zhongwen CRM - refactor 1

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
    Servers: Heroku/Github Pages

## Refactor 1 Goals
Deploy following functionality:
- Home page loads
- User can upload document in temp session (no login required)
    -- Saved to MongoDB database
    -- Gets flushed daily
- User can login/create account to save uploaded documents
- UI is passable

### Epics (high-level goal)
- Get I/O functional

### User Stories (subset of Epics)
- "What's this? Let me try it out!
- "This is cool -- I let me make an account!"