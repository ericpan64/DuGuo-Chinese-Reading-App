// Add server response for form submissions. Ref: https://stackoverflow.com/a/47675314
let attemptLogin = () => {
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
            alert("Error when trying to login. Try again and/or open a Github issue");
        }
    }
    xhr.onerror = () => {
        alert("Error when trying to login. Try again and/or open a Github issue");
    }
    let formData = new FormData(document.getElementById("login-form"));
    let params = new URLSearchParams(formData);
    console.log(params.toString())
    xhr.send(params.toString());
}
let getForbiddenChars = (pw) => {
    const invalid_chars = new Set(['<', '>', '!', '(', ')', '{', '}', '"' , '\'', ';', ':', '\\', '*']);
    let res = "";
    for (i in pw) {
        if (invalid_chars.has(pw[i])) {
            res += pw[i];
        }
    }
    return res;
}
let attemptRegister = () => {
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
            alert('Error when trying to register. Try again and/or open a Github issue');
        }
    }
    xhr.onerror = () => {
        alert("Error when trying to register. Try again and/or open a Github issue");
    }
    let formData = new FormData(document.getElementById("register-form"));
    let invalid_chars = getForbiddenChars(formData.get("password"));
    if (invalid_chars) {
        alert(`Registration failed, password contained the following forbidden characters: ${invalid_chars}`);
    }
    let params = new URLSearchParams(formData);
    console.log(params.toString())
    xhr.send(params.toString());
}
/// Toggle password visibility
let showPassword = () => {
    let login_pw = document.getElementById("pw-login")
    let reg_pw = document.getElementById("pw-reg")

    if (login_pw.type == "password") {
        login_pw.type = "text"
        reg_pw.type = "text"
    } else {
        login_pw.type = "password"
        reg_pw.type = "password"
    }
}