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