/// Settings logic. Updates setting button name and form values.
let urlButtonId = 'url-upload-button'
let cn_phonetics_pinyin = (e) => { e.value = "pinyin"; }
let cn_phonetics_zhuyin = (e) => { e.value = "zhuyin"; }
let cn_type_simp = (e) => { e.value = "simp"; }
let cn_type_trad = (e) => { e.value = "trad"; }
let setType = (type_string) => {
    if (type_string === 'pinyin') {
        document.getElementById('phonetic-setting').innerHTML = "Render Pinyin";
        document.getElementsByName('cn_phonetics').forEach(cn_phonetics_pinyin);
    } else if (type_string === 'zhuyin') {
        document.getElementById('phonetic-setting').innerHTML = "Render Zhuyin";
        document.getElementsByName('cn_phonetics').forEach(cn_phonetics_zhuyin);
    } else if (type_string === 'simp') {
        document.getElementById('char-setting').innerHTML = "Render Simplified";
        document.getElementsByName('cn_type').forEach(cn_type_simp);
    } else if (type_string === 'trad') {
        document.getElementById('char-setting').innerHTML = "Render Traditional";
        document.getElementsByName('cn_type').forEach(cn_type_trad);
    }
}
let processLuckyButton = (id) => {
    urlButtonId = id
    document.getElementById('urlField').value='https:\/\/zh.wikipedia.org/wiki/Special:%E9%9A%8F%E6%9C%BA%E9%A1%B5%E9%9D%A2'
}