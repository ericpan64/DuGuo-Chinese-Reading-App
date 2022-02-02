# DuGuo - Refactor 2

## Refactor Goals

Overall, the goal is to have the app ready for deployment (without the actual deployment details).

Initial Checklist:
- [x] Add Bootstrap for initial UI design. Keep it as lightweight as possible while also looking good!
- [x] Add popup dictionary functionality for (almost) _all_ Chinese rendered on the screen.
    - Popup dictionary not rendered for document preview which imo makes sense
- [x] Add ability to toggle pinyin on/off for a user's saved vocab.
    - [x] See if this can be done _without_ using React
- ~~Add front-end form validation.~~ Worry about this later, based on feedback from MVP
- [x] Add appropriate framework for displaying errors to end-users.
    - Decided on using vanilla JS alerts. Simple though can be annoying depending on the person, we'll see from the feedback
- [x] Add appropriate permissions framework for viewing other user information (default all private).
- [x] Ensure document rendering speeds are reasonable.

## Refactor "Nice-to-Have"s
Both of these are worth considering in the future, scope appropriately!
- [ ] Add ability to export vocab items to a csv (or similar format).
- [ ] Add small library of starter template articles to generate from.