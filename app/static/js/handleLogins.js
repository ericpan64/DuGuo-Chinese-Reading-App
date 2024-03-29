/// Performs POST request to /api/login (defined in users.rs)
let attemptLogin = (formId) => {
    let xhr = new XMLHttpRequest();
    xhr.open("POST", "/api/login", true);
    xhr.setRequestHeader("Content-type", "application/x-www-form-urlencoded");
    xhr.onload = () => {
        if (xhr.status == 202) {
            alert("Successfully logged in!");
            window.location.href = '/'; // redirect to index
        } else if (xhr.status == 401) {
            alert("Login attempt failed, try again.");
        } else {
            alert("Error when trying to login. Try again and/or open a GitHub issue");
        }
    }
    xhr.onerror = () => {
        alert("Error when trying to login. Try again and/or open a GitHub issue");
    }
    let formData = new FormData(document.getElementById(formId));
    let params = new URLSearchParams(formData);
    console.log(params.toString())
    xhr.send(params.toString());
}

/// Performs POST request to /api/register (defined in users.rs)
let attemptRegister = (formId) => {
    let xhr = new XMLHttpRequest();
    xhr.open("POST", "/api/register");
    xhr.setRequestHeader("Content-type", "application/x-www-form-urlencoded");
    xhr.onload = () => {
        if (xhr.status == 202) {
            alert('Successfully registered!');
            window.location.href = '/'; // redirect to index
        } else if (xhr.status == 422) {
            alert('Registration failed, username and/or email are taken. Try again.');
        } else {
            alert('Error when trying to register. Try again and/or open a GitHub issue');
        }
    }
    xhr.onerror = () => {
        alert("Error when trying to register. Try again and/or open a GitHub issue");
    }
    let formData = new FormData(document.getElementById(formId));
    let params = new URLSearchParams(formData);
    console.log(params.toString())
    xhr.send(params.toString());
}

/// Toggle password visibility in form
let showPassword = (fieldId) => {
    let pw_field = document.getElementById(fieldId)
    if (pw_field.type === "password") {
        pw_field.type = "text"
    } else {
        pw_field.type = "password"
    }
}