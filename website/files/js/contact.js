const encoder = new TextEncoder();
const emailRegex = new RegExp(/^[A-Za-z0-9_!#$%&'*+\/=?`{|}~^.-]+@[A-Za-z0-9.-]+$/, "m");
const loadingBar = document.getElementById("loadingDots");
const emailSubmitButton = document.getElementById("mailButton");
const emailInput = document.getElementById("email");
const messageInput = document.getElementById("message");
const emailText = document.getElementById("emailText");

function sendMailApiRequest() {
    const email = emailInput.value;
    console.log(email);

    emailText.style.display = "none";
    emailText.innerText = "";
    emailText.className = "";
    emailInput.classList.remove("error-highlight");
    messageInput.classList.remove("error-highlight");

    const is_valid = !emailRegex.test(email);

    if (is_valid) {
        console.log("not an valid email");
        emailInput.classList.add("error-highlight");
        error_text("Not an valid Email");
        return;
    }
    

    const email_bytes = encoder.encode(email);
    const email_len = email_bytes.length;
    if (email_len > 255) {
        console.log("email to long");
        error_text("Email too long");
        return;
    }
    const len_plus_mail = new Uint8Array(email_len + 1);
    len_plus_mail.set([email_len]);
    len_plus_mail.set(email_bytes, 1);

    const message = messageInput.value;
    const message_bytes = encoder.encode(message);
    const message_len = message_bytes.length;
    if (message_len > 2000) {
        console.log("please stop");
        messageInput.classList.add("error-highlight");
        error_text("Message too long");
        return;
    } else if (message_len <= 0) {
        console.log("no message");
        messageInput.classList.add("error-highlight");
        error_text("Must send message");
        return;
    }

    const data_16 = new Uint16Array([message_len]);
    const message_len_u8 = new Uint8Array(data_16.buffer);
    let the_big_one = new Uint8Array(message_len_u8.length + len_plus_mail.length + message_len);
    the_big_one.set(len_plus_mail);
    the_big_one.set(message_len_u8, len_plus_mail.length);
    the_big_one.set(message_bytes, len_plus_mail.length + 2);
    // I LOVE DYNAMIC TYPES I LOVE DYNAMIC TYPES I LOVE DYNAMIC TYPES

    let promise = fetch("/api/mail", {
        method: "POST",
        headers: {
            "Content-Type": "application/octet-stream",
        },
        body: the_big_one,
    });
    emailSubmitButton.style.display = "none";
    loadingBar.style.display = "flex";

    promise.then((response) => {
        loadingBar.style.display = "none";
        emailSubmitButton.style.display = "";
        console.log(response);
        if (response.ok) {
            emailText.classList.add("success-text")
            emailText.innerText = "Email Sent Sucessfully";
            emailText.style.display = "";
            return;
        }

        if (response.status == 429) {
            error_text("Too many requests");
            // Rate limited :)
            return;
        }

        // some other error occured and I dont feel like adding more speccial cases
        error_text("Something went wrong, please try again later");
        return;
    });
}

function error_text(message) {
    emailText.classList.add("failure-text");
    emailText.innerText = message;
    emailText.style.display = "";
}