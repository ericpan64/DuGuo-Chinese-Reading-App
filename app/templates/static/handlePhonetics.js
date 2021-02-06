/// Trigger Pinyin Visibility (individual elements)
let switchToHidden = (e) => { e.style.visibility = 'hidden'; }
let switchToVisible = (e) => { e.style.visibility = 'visible'; }
let switchOffWordVisibility = (phrase) => {
    for (word_name of phrase.split('')) {
        yins = document.getElementsByName(word_name).forEach(switchToHidden);
    }
}

/// Trigger Pinyin Visibility (all elements)
let all_phonetics_list = document.getElementsByClassName("phonetic");
let hideSavedPhonetics = (phrase_list) => {
    phrase_list.forEach(x => document.getElementsByName(x).forEach(switchToHidden));
}
let hideAllPhonetics = () => { 
    for (let i = 0; i < all_phonetics_list.length; i++) {
        switchToHidden(all_phonetics_list[i]);
    }   
}
let resetAllPhonetics = () => {
    for (let i = 0; i < all_phonetics_list.length; i++) {
        switchToVisible(all_phonetics_list[i]);
    }   
}