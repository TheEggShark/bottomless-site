function sendApiRequest() {
    let a = [10, 20, 30, 40];
    fetch("/api/mail", {
        method: "POST",
        headers: {
            "Content-Type": "text/plain"
        },
        body: a,
    });
}