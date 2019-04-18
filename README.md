# Zhongwen Learning-Mode extension

This extension will allow users to interactively learn how to read Chinese with any text-based article on the web by providing word segmentation, pinyin support, and a database of a user's "vocabulary-dictionary" of user-saved phrases.

Input: User selects text in web browser --> right click to access extension
  Implementation of the Context menu is found in the separate "Zhongwen" repo
Output: New tab displaying selected text with pinyin and ability to save articles

This feature will be built on top of the existing Zhongwen Google Chrome extension.

FINAL TO-DO:
1) Get Post Request to Work
1.5) Understand previous code
2) Integrate with previous code
  - Update db to Mongodb
  - Get working instance on local server
3) Figure-out where/how to add pinyin + toggle "Learning Mode"
4) SHIP!

Technical Notes
- Context Menu (Chrome) -- right click menu options
- Sockets -- helps control client/server communication
- Context Manager -- better file handling (handles open/close of database, locks)