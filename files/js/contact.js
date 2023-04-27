const encoder = new TextEncoder();

function sendApiRequest() {
    let email = document.getElementById("email").value;
    let email_bytes = encoder.encode(email);
    let email_len = email_bytes.length;
    if (email_len > 255) {
        console.log("email to long");
        return;
    }
    let len_plus_mail = new Uint8Array(email_len + 1);
    len_plus_mail.set([email_len]);
    len_plus_mail.set(email_bytes, 1);

    let message = document.getElementById("message").value;
    let message_bytes = encoder.encode(message);
    let message_len = message_bytes.length;
    if (message_len > 65535) {
        console.log("please stop");
        return;
    }
    data_16 = new Uint16Array([message_len]);
    let message_len_u8 = new Uint8Array(data_16.buffer);
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