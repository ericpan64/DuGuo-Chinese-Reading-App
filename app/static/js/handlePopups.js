/// Enable pop-ups
let popoverTriggerList = [].slice.call(document.querySelectorAll('[data-bs-toggle="popover"]'))
let popoverList = popoverTriggerList.map(function (popoverTriggerEl) {
    return new bootstrap.Popover(popoverTriggerEl)
})

/**
 * Removes the download link after a user saves a phrase.
 * @param {String} uid Phrase uid (currently: simplified+raw_pinyin)
 */
let removeDownloadLink = (uid) => {
    const download_link = ` <a role="button" href="#${uid}"><img src="/static/img/download.svg"></img></a>`;
    let spans = document.getElementsByClassName(uid);
    const title_attr = "data-bs-original-title";
    for (let i=0; i < spans.length; i++) {
        let new_title = spans[i].getAttribute(title_attr).replace(download_link, "");
        spans[i].setAttribute(title_attr, new_title);
    }
}

/// Remove download link for all saved phrases (defined in reader.html.tera)
for (let i=0; i < user_saved_uid_list.length; i++) {
    removeDownloadLink(user_saved_uid_list[i]);
}

/// Closes active popovers (by clicking)
let close_active_popovers = (event) => {
    let active_elements = document.querySelectorAll("span[aria-describedby]");
    for (let i=0; i < active_elements.length; i++) {
        active_elements[i].click();
    }
    if (event.key == 'Tab') {
        document.activeElement.click();
    }
}

/// Popover close triggers
document.onkeyup = close_active_popovers;
document.onscroll = close_active_popovers;
document.oncontextmenu = close_active_popovers;