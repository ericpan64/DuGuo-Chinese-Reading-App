"""
Next-steps (create + test):
- Understand Flask at a low-level (i.e. fundamentals)
- Clean-up Flask code (a LOT of work)
- Understand MongoDB at a low-level (fundamentals)
- Understand HTML at a low-level

"""


from models import zwPhrase, zwWord

word = "點心"
word2 = "ab"

for i in word, word2:
    print(i)

word = zwWord(word="點", is_simplified=True, pinyin="test2")
word2 = zwWord(word="", is_simplified=True, pinyin="test3")
w_list = [word, word2]
zPhrase = zwPhrase(phrase=w_list, is_simplified=True, pinyin="test2test3", definition="Creating phrase")

print(zPhrase.phrase)