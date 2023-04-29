const encoder = new TextEncoder();
const emailRegex = new RegExp(/^[A-Za-z0-9_!#$%&'*+\/=?`{|}~^.-]+@[A-Za-z0-9.-]+$/, "gm");

function sendMailApiRequest() {
    const email = document.getElementById("email").value;

    if (!emailRegex.test(email)) {
        console.log("not an valid email");
        return;
    }
    

    const email_bytes = encoder.encode(email);
    const email_len = email_bytes.length;
    if (email_len > 255) {
        console.log("email to long");
        return;
    }
    const len_plus_mail = new Uint8Array(email_len + 1);
    len_plus_mail.set([email_len]);
    len_plus_mail.set(email_bytes, 1);

    const message = document.getElementById("message").value;
    const message_bytes = encoder.encode(message);
    const message_len = message_bytes.length;
    if (message_len > 65535) {
        console.log("please stop");
        return;
    }
    const data_16 = new Uint16Array([message_len]);
    const message_len_u8 = new Uint8Array(data_16.buffer);
    let the_big_one = new Uint8Array(message_len_u8.length + len_plus_mail.length + message_len);
    the_big_one.set(len_plus_mail);
    the_big_one.set(message_len_u8, len_plus_mail.length);
    the_big_one.set(message_bytes, len_plus_mail.length + 2);
    // I LOVE DYNAMIC TYPES I LOVE DYNAMIC TYPES I LOVE DYNAMIC TYPES

    fetch("/api/mail", {
        method: "POST",
        headers: {
            "Content-Type": "application/octet-stream",
        },
        body: the_big_one,
    });
}