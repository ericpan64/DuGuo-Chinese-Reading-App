# Zhongwen CRM - refactor 1

## Tech Stack
Web
- Backend: Rust (Rocket)
- Frontend: plain JS/HTML/CSS
- Database: mongoDB

Web Routing
- Main: /
- Account signup: /register
- Account view: /u/{username}
- View saved vocab: /u/{username}/vocab
- Upload document (anon): /upload
- Upload document (user): /u/upload
- View document: /d/{docId}

Hosting
- Servers: Heroku/Github Pages

## Refactor Goals
Deploy following functionality:
- Home page loads
- User can upload document in temp session (no login required)
    -- Gets flushed daily
- User can login/create account
- User can upload documents that are linked to their account

### Epics (high-level goals)
- Get I/O functional
- Set-up project infrastructure

### User Stories (subset of Epics)
- "What's this? Let me try it out!"
- "This is cool -- let me make an account!"
- "Hm, I can make an account and upload, but why isn't there any functionality?"
