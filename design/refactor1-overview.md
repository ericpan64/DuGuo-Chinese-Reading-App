# Zhongwen CRM - refactor 1

## Tech Stack
Web
- Backend: Rust (Rocket)
- Frontend: vanilla JS + Bootstrap 5
- Database: mongoDB

## Web Routing

No Login Required (Anon)
- Main: /
- Account signup: /register
- Upload document (anon): /upload
- View other account dashboard: /{username}
- View saved vocab: /{username}/vocab

Login Required
- Edit User dashboard: /u/info
- Edit saved vocab: /u/vocab
- Upload document (user): /u/doc/upload
- View document: /u/doc/{docId}

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
