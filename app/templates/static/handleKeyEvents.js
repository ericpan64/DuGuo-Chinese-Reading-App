
/// Configuration for button text
/// Updates "read-start-stop" button
const DEFAULT_MSG = 'Read Document Aloud';
const PAUSE_MSG = 'Pause Document Reading';
const RESUME_MSG = 'Resume Document Reading';
const START_STOP_BUTTON_ID = 'read-start-stop';
let set_start_stop_button_text = (msg) => { document.getElementById(START_STOP_BUTTON_ID).innerText = msg; }

/// Variables for Document Reader
let is_reading = false;
let span_index = 0;
let spans = document.querySelectorAll("span[data-bs-content]");
let n = spans.length;

/// Listen for key events
document.onkeyup = (event) => {
    // close all active elements
    let active_elements = document.querySelectorAll("span[aria-describedby]");
    for (let i=0; i < active_elements.length; i++) {
        active_elements[i].click();
    }
    if (event.key == 'r') {
        document.getElementById(START_STOP_BUTTON_ID).click();
    } else if (event.key == 'Tab') {
        document.activeElement.click();
    }
}

/// Handle Document Reader
let READER_INTERVAL_MS = 1200;
let read_span = () => {
    if (span_index < n) {
        let e = spans[span_index];
        e.focus();
        let phrase = e.innerText.split('\n')[1].replace('\t', '');
        window.location.hash = `~${phrase}`;
        span_index += 1;
    } else if (span_index == n) {
        reset_reader();
    }
}
let interval_id;
let trigger_reader = () => {
    if (is_reading) {
        is_reading = false;
        set_start_stop_button_text(RESUME_MSG);
        clearInterval(interval_id);        
    } else {
        is_reading = true;
        set_start_stop_button_text(PAUSE_MSG);
        interval_id = setInterval(read_span, READER_INTERVAL_MS);
    }
}
let reset_reader = () => {
    is_reading = false;
    span_index = 0;
    clearInterval(interval_id);
    document.activeElement.blur();
    set_start_stop_button_text(DEFAULT_MSG);
}