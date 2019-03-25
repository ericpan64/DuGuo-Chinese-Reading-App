# Zhongwen Learning-Mode extension

This extension will allow users to interactively learn how to read Chinese with any text-based article on the web by providing word segmentation, pinyin support, and a database of a user's "vocabulary-dictionary" of user-saved phrases.

Input: User selects text in web browser --> right click to access extension
Output: New tab displaying selected text with pinyin and ability to save articles

This feature will be built on top of the existing Zhongwen Google Chrome extension.

3/24/19 To-Do:
- Get initial database architecture (e.g. collections, objects saved, etc.)
- Create local prototype of "learning mode" functionality
  - 1) Integrate NLP segmenter of words
  - 2) Integrate parts-of-speech identifier of words
  - 3) Integrate pinyin support over Words
  - 4) Have pinyin be toggle-able depending if word is saved in dictionary or notes
      Identify a good look-up key to use in the DB
  - 5) Give users ability to add words to DB as necessary
- Create general "saved articles" page (home page)
    Figure out what code/tables will be needed
    Use current Word Dictionary in Zhongwen repo as guided
- Add text-highlighting option to existing right-click widget
    Make sure it pulls text correctly
- Test using local machine as server
    Once successful, think about remote hosting options / integration
- Create user account workflow (OAuth)
    Figure out user authentication with db
- Create some back-end analysis framework with existing data
    Start with radical tagging and user analysis
