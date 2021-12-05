/// General Handling
/// Update to Loading Button onsubmit
let switchToLoadingButton = (id) => {
    let button = document.getElementById(id)
    button.innerHTML = `<span class="spinner-border spinner-border-sm" role="status" aria-hidden="true"></span>
        <span class="sr-only">Loading...</span>`;
    button.setAttribute("disabled", "");
}

/// Missing Text-to-Speech Handling
/// Removes the speech link after a user saves a phrase for all phrases on the page.
let removeAllSpeechImages = () => {
    speech_link = `<img src="/static/icons/volume-up-fill.svg"></img>`;
    let spans = document.querySelectorAll("span[data-bs-content]");
    const title_attr = "data-bs-original-title";
    for (let i=0; i < spans.length; i++) {
        let new_title = spans[i].getAttribute(title_attr).replace(speech_link, "");
        spans[i].setAttribute(title_attr, new_title);
    }
}

/// Adds alert with error message if speechSynthesis load fails
let replaceButtonGroup = (msg) => {
    const button_group_id = "reader-btn-group";
    let div = document.getElementById(button_group_id);
    if (div != null) {
        div.innerHTML = `<div class="alert alert-primary" role="alert">${msg}</div>`;
        div.setAttribute('aria-label', 'Error loading speechSynthesis, text-to-speech unavailable.');
        div.parentElement.insertBefore(document.createElement('br'), div);
    }
}

/// Loads the appropriate Duey image to id="duey"
let loadDueyImage = (imgName) => {
    let img = document.getElementById("duey");
    if (img != null) {
        img.setAttribute('src', `/static/duey/${imgName}`)
    }
}

/**
 * Performs Text-to-Speech step with given phrase.
 * @param {String} phrase Chinese String to read.
 */
/// TODO: Support different Chinese voice variants (e.g. zh-TW, zh-HK)
let sayPhrase = (phrase, lang='zh-CN') => {
    let utterance = new SpeechSynthesisUtterance(phrase);
    utterance.lang = lang;
    utterance.rate = 0.8;
    return window.speechSynthesis.speak(utterance);
}

/// Handle Hash Changes
/**
 * Sends POST request to /api/update-settings (defined in users.rs).
 * @param {String} hash_string String starting with $
 */
let postUserSetting = (hash_string) => {
    let xhr = new XMLHttpRequest();
    xhr.open("POST", "/api/update-settings");
    xhr.setRequestHeader("Content-type", "application/x-www-form-urlencoded");
    let params = `setting=${hash_string}`;
    xhr.onload = () => {
        if (xhr.status == 202) {
            // change active setting bar, button name
            if (hash_string === 'pinyin') {
                document.getElementById('phonetic-setting').innerHTML = "Use Pinyin";
            } else if (hash_string === 'zhuyin') {
                document.getElementById('phonetic-setting').innerHTML = "Use Zhuyin";
            } else if (hash_string === 'simp') {
                document.getElementById('char-setting').innerHTML = "Use Simplified";
            } else if (hash_string === 'trad') {
                document.getElementById('char-setting').innerHTML = "Use Traditional";
            }
        }
        window.location.reload();
    }      
    xhr.onerror = () => {
        alert(`Error when updating setting. Try again and/or open a Github issue`);
    }
    xhr.send(params);
}

/**
 * Sends POST request to /api/vocab (defined in users.rs).
 * @param {String} hash_string Phrase uid (currently: simplified+raw_pinyin)
 */
let postNewVocab = (hash_string) => {
    let xhr = new XMLHttpRequest();
    xhr.open("POST", "/api/vocab");
    xhr.setRequestHeader("Content-type", "application/x-www-form-urlencoded");
    let params = `phrase_uid=${hash_string}&from_doc_title=${document.title}`;
    xhr.onload = () => {
        if (xhr.status == 202) {
            alert(`Successfully added ${hash_string} to your dictionary!`);
            try { user_saved_uid_list = user_saved_uid_list.concat(hash_string); } 
            finally { switchOffWordVisibility(hash_string); }
            
        } else {
            alert(`Error when adding ${hash_string} to dictionary.\n\nEither you aren't logged-in, you've already saved this phrase from this doc, or you should provide some feedback :-)`);
        }
    }
    xhr.onerror = () => {
        alert(`Error when adding ${hash_string} to dictionary. Try again and/or open a Github issue`);
    }
    xhr.send(params);
}

/**
 * Handles the hash updating logic. 
 */
let parseHashChange = () => {
    if (location.hash) {
        let hash_string = location.hash.substring(1);
        hash_string = decodeURIComponent(hash_string);
        // Remove the hash selector. From: https://stackoverflow.com/a/5298684/13073731
        history.replaceState("", document.title, window.location.pathname + window.location.search);
        history.back();
        // If starts with ~: try Text-to-Speech
        // If starts with $: try User settings update
        // Otherwise       : try to save as UserVocab
        if (hash_string.charAt(0) == '~') {
            hash_string = hash_string.substring(1);
            sayPhrase(hash_string);
        } else if (hash_string.charAt(0) == '$') {
            hash_string = hash_string.substring(1);
            postUserSetting(hash_string);
        } else {
            postNewVocab(hash_string);
            removeDownloadLink(hash_string);
        }
    }
}

/// Set event callback
window.onhashchange = parseHashChange;
window.onload = () => {
    if ('speechSynthesis' in window) {
        let is_mobile = ( /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent) );
        if (is_mobile) {
            removeAllSpeechImages();
            replaceButtonGroup('Note: Text-to-Speech is not currently supported on mobile. To use the feature, try desktop!');
        }
    } else { 
        removeAllSpeechImages();
        replaceButtonGroup('Note: The speechSynthesis API is not detected in your browser. Try using a different and non-mobile browser!');
    }    
}