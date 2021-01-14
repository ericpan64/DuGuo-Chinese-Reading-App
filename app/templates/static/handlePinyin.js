/// Trigger Pinyin Visibility
let switchToHidden = (e) => { e.style.visibility = 'hidden'; }
let switchToVisible = (e) => { e.style.visibility = 'visible'; }
let switchOffWordVisibility = (pinyin) => {
    for (word_name of pinyin.split(' ')) {
        yins = document.getElementsByName(word_name).forEach(switchToHidden);
    }
}
let all_pinyin_list = document.getElementsByClassName("pinyin");
let hideSavedPinyin = (phrase_list) => {
    phrase_list.forEach(x => document.getElementsByName(x).forEach(switchToHidden));
}
let hideAllPinyin = () => { 
    for (let i = 0; i < all_pinyin_list.length; i++) {
        switchToHidden(all_pinyin_list[i]);
    }   
}
let resetAllPinyin = () => {
    for (let i = 0; i < all_pinyin_list.length; i++) {
        switchToVisible(all_pinyin_list[i]);
    }   
}