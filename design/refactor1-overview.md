# ~~Zhongwen CRM~~ DuGuo - Refactor 1

## Tech Stack
Web
- Backend: Rust (Rocket)
- ~~Frontend: Bootstrap 5. No JS yet~~
- Database: mongoDB

## Web Routing

No Login Required (Anon)
- Index: /
- Account signup: /login
- Upload document (anon): /sandbox
- View other account dashboard: /u/{username}
- ~~View saved vocab: /u/{username}/vocab~~

Login Required
- Edit User dashboard: ~~/u/info~~ /u/{username}
- Edit saved vocab: ~~/u/vocab~~ /u/{username}
- Upload document (user): ~~/u/doc/upload~~ /u/{username}
- View document: ~~/u/doc/{docId}~~ /u/{username}/{doc_title}

## Refactor Goals
Deploy following functionality:
- Home page loads
- User can upload document in temp session (no login required)
- User can login/create account
- User can upload documents that are linked to their account

### Epics (high-level goals)
- Get I/O functional
- Set-up project infrastructure

### User Stories (subset of Epics)
- "What's this? Let me try it out!"
- "This is ~~cool~~ interesting -- let me make an account!"
- "Hm, I can make an account and upload, but why isn't there any functionality?"
