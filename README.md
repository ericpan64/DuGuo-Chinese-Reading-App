# Zhongwen Learning-Mode extension

This extension will allow users to interactively learn how to read Chinese with any text-based article on the web by providing word segmentation, pinyin support, and a database of a user's "vocabulary-dictionary" of user-saved phrases.

Input: User selects text in web browser --> right click to access extension
  Implementation of the Context menu is found in the separate "Zhongwen" repo
Output: New tab displaying selected text with pinyin and ability to save articles

This feature will be built on top of the existing Zhongwen Google Chrome extension.

=== To-Do (updated 12/15/19) ===

= Flask Server =
* Get running instance with new mongoDB database
--> Fix queries in lib/zhongwen.py (z.CEDICT.objects(simplified=char))
* Figure-out where/how to add pinyin + toggle "Learning Mode"
* Get running Flask instance (mini-PRD)
* Bug fixes
* Migrate to non-flask

= Chrome Extension =
* Find way to get POST request
* Clean-up current errors when running extension
* Figure-out how to add OAuth integration


Misc Technical Notes
- Context Menu (Chrome) -- right click menu options
- Sockets -- helps control client/server communication
- Context Manager -- better file handling (handles open/close of database, locks)