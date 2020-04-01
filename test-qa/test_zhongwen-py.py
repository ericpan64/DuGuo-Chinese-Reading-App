import unittest
from app.lib import zhongwen
from app.__init__ import connectMongoDB
from app.models import zwChars as z

class TestZhongwenPy(unittest.TestCase):
    def setUp(self):
        # Start mongoDB on server (manually), then connect
        connectMongoDB()

        # Generate generic test data
        phrase_a_simp = z.zwPhrase(phrase="",pinyin="",definition="",is_simplified=True)

        cedict_a = z.CEDICT(traditional=None,simplified=phrase_a_simp)

        print("hi")

    def test_query_cedict(self):
        print("Complete if this breaks :-P")

    def test_get_pinyin(self):
        print("Complete if this breaks :-P")

    def test_get_tone(self):
        print("Complete if this breaks :-O")

    def test_render_chinese_word(self):
        print("Complete if this breaks :-P")

    def test_generate_html(self):
        print("Complete if this breaks :-3")

    def test_annotate_text(self):
        print("Complete if this breaks :-P")

if __name__ == '__main__':
    unittest.main()