/// Trigger Pinyin Visibility
let switchToHidden = (e) => { e.style.visibility = 'hidden'; }
let switchToVisible = (e) => { e.style.visibility = 'visible'; }
let switchOffWordVisibility = (phrase) => {
    for (word_name of phrase.split('')) { // TODO: verify this change works as expected
        yins = document.getElementsByName(word_name).forEach(switchToHidden);
    }
}
// Only one of these lists should be populated at a given instance
let all_pinyin_list = document.getElementsByClassName("pinyin");
let all_zhuyin_list = document.getElementsByClassName("zhuyin");
let hideSavedPhonetics = (phrase_list) => {
    phrase_list.forEach(x => document.getElementsByName(x).forEach(switchToHidden));
}
let hideAllPhonetics = (phonetics_list) => { 
    for (let i = 0; i < phonetics_list.length; i++) {
        switchToHidden(phonetics_list[i]);
    }   
}
let resetAllPhonetics = (phonetics_list) => {
    for (let i = 0; i < phonetics_list.length; i++) {
        switchToVisible(phonetics_list[i]);
    }   
}