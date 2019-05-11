"""
Author: Martin Kess
Description: Utility functions. Queries

Parts I changed:
    CEDICT queries adjusted to query NoSQL instead of SQL
        Added CEDICT as a parameter to adjust accordingly
    Added some function documentation
"""

# -*- coding: utf-8 -*-

''' Utility functions for rendering Chinese text. '''

from stanford import segment_text, get_parts_of_speech
from flask_login import current_user
from bs4 import BeautifulSoup
from app import CEDICT # mongoDB collection containing CEDICT dictionary

def get_pinyin(chinese_word):
    """
    :param chinese_word: chinese word
    :return: pinyin array=[word,pinyin,is_chinese]
    """
    if len(chinese_word) == 0:
        return u''

    tokens = []

    current_chinese = None    # ie. "unknown"
    current_token = chinese_word[0]

    for char in chinese_word:
        entry = CEDICT.objects(simplified=char) # Queries simplified Chinese only
        if current_chinese is None:
            # First character
            current_chinese = entry is not None

        elif current_chinese:
            if entry is None:
                tokens.append((current_token, False))
                current_chinese = False
                current_token = char
            else:
                current_chinese = True
                current_token += char

        elif not current_chinese:
            if entry is None:
                current_token += char
                current_chinese = False
            else:
                tokens.append((current_token, False))
                current_chinese = True
                current_token = char

    if len(current_token) > 0:
        tokens.append((current_token, current_chinese))

    res = []
    for word, is_chinese in tokens:
        if is_chinese:
            # Try to get pinyin
            # Not the cleanest below, but it works (converts to dict)
            entry = CEDICT.objects(simplified=word).as_pymongo()[0]
            if entry is not None:
                res.append((word, entry['pinyin'], is_chinese))
            else:
                pinyin = []
                # Otherwise, just do each letter
                # TODO(mkess): could be more clever here... greedy match?
                for char in word:
                    entry = CEDICT.objects(simplified=char).as_pymongo()[0]
                    pinyin.append(entry['pinyin'])
                res.append((word, ' '.join(pinyin), is_chinese))
        else:
            res.append((word, word, is_chinese))

    return res

def get_tone(py):
    tones = '12345'
    for t in tones:
        if py.find(t) != -1:
            return t

    return '0'


def render_chinese_word(chinese_word,pos=''):
    """
    :param chinese_word: Chinese word
    :param pos:
    :return: HTML syntax code for given Chinese
    """
    ''' Generates HTML for the given Chinese word. For example:
    render_chinese_word(u'2009你好')
    <span>
        2009
    </span>
    <span class="word" tabindex="0" data-toggle="popover" data-content="/Hello!/Hey there!/" data-trigger="focus" data-original-title="你好 [ni3 hao3]" html="true">
        <span class="character tone3">你</span>
        <span class="character tone3">好</span>
    </span>

    If no definition can be found, fall back to having no tone and put an error in popover.

    The part of speech is added as the class `pos-XX` (ie. pos-NR for proper nouns) attribute
    of the word span. This way, CSS can properly mark up the text.
    '''

    res = []

    pinyin = get_pinyin(chinese_word)

    for word, py, is_chinese in pinyin:
        if py is None:
            res.append('<span>')
            res.append(word)
            res.append('</span>')
        else:
            entry = CEDICT.objects(simplified=word).as_pymongo()[0]

            definition = None
            if entry is None:
                definition = u'Could not find definition for "{}"'.format(word)
            else:
                definition = entry['definition']

            defn_html = '<ul>'
            defn_html += ''.join('<li>' + defn + '</li>' for defn in definition.split('/') if defn != '')
            defn_html += '</ul>'

            title_html = u'{} [{}]'.format(word, py)

            res.append(u'<span class="word pos-{} {}" tabindex="0" data-word="{}">'.format(pos, '' if is_chinese else 'non-chinese', word))
            if is_chinese:
                word_py = py.split(' ')
                assert len(word_py) == len(word), u'Pinyin mismatch - {} {}'.format(py, word)

                for char, pronunciation in zip(word, word_py):
                    res.append('<span class="character tone{}">'.format(get_tone(pronunciation)))
                    res.append(char)
                    res.append('</span>')
            else:
                for c in word:
                    res.append('<span class="character non-chinese">')
                    res.append(c)
                    res.append('</span>')

            res.append('</span>')

    return '\n'.join(res)

def generate_html(pos_text):
    res = []

    bs = BeautifulSoup.BeautifulStoneSoup(pos_text)
    words = bs.sentence.findAll('word')

    for word in words:
        pos = word['pos']
        if len(word.contents) > 0:
            res.append(render_chinese_word(word.contents[0], pos))

    return ''.join(res)

def annotate_text(data):
    data = data.splitlines()
    processed = []
    for line in data:
        segmented_text = segment_text(line)
        pos_text = get_parts_of_speech(segmented_text)

        processed.append(generate_html(pos_text))

    res = []
    for paragraph in processed:
        res.append('<div class="paragraph">')
        res.append(paragraph)
        res.append('</div>')

    return ''.join(res)

def render_document(doc):
    bs = BeautifulSoup.BeautifulSoup(doc)
    for div in bs.findAll('div'):
        for span in div.findAll('span'):
            if 'word' in span['class'].split():
                data_word = span['data-word']
                if current_user.vocab.filter_by(simplified=data_word).count() > 0:
                    span['class'] = span.get('class', '') + ' in-vocab'
                else:
                    span['class'] = span.get('class', '') + ' not-in-vocab'

    return bs.prettify().decode('utf-8')
