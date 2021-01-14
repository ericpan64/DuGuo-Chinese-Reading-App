// Text-to-Speech
let sayPhrase = (phrase) => {
    let utterance = new SpeechSynthesisUtterance(phrase);
    utterance.lang = 'zh-CN';
    utterance.rate = 0.8;
    return window.speechSynthesis.speak(utterance);
}
/// Handle Hash Changes
let parseHashChange = () => {
    if (location.hash) {
        let py_string = location.hash.substring(1);
        py_string = decodeURIComponent(py_string);
        // Remove the hash selector. From: https://stackoverflow.com/a/5298684/13073731
        history.pushState("", document.title, window.location.pathname + window.location.search);
        // Text-to-Speech if starts with ~
        // Otherwise, Saving Vocab
        if (py_string.charAt(0) == '~') {
            py_string = py_string.substring(1);
            sayPhrase(py_string); // defined in "template.html.tera"
        } else {
            // Send POST request to add vocab string with title, then return result
            // (Rocket handles the cookie reading!)
            let xhr = new XMLHttpRequest();
            xhr.open("POST", "/api/vocab");
            xhr.setRequestHeader("Content-type", "application/x-www-form-urlencoded");
            let params = `saved_phrase=${py_string}&from_doc_title=${document.title}`;
            
            // TODO make sure this works correctly in Sandbox/otherwise
            xhr.onload = () => {
                if (xhr.status == 202) {
                    alert(`Successfully added ${py_string} to your dictionary!`);
                    
                } else {
                    alert(`Error when adding ${py_string} to dictionary.\n\nEither you aren't logged-in, you've already saved this phrase from this doc, or you should provide some feedback :-)`);
                }
                try { user_saved_phrase_list = user_saved_phrase_list.concat(py_string.split('')); } 
                finally { switchOffWordVisibility(py_string); }
            }
            xhr.onerror = () => {
                alert(`Error when adding ${py_string} to dictionary. Try again and/or open a Github issue`);
            }
            xhr.send(params);
        }
    }
}
window.onhashchange = parseHashChange;