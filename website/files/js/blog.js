const decoder = new TextDecoder()

function fetch2posts() {
    let data = fetch("/api/recentBlogPosts?max=2")
    .then((response) => response.arrayBuffer())
    .then((buffer) => {
        console.log(buffer);
        let pain = new Uint8Array(buffer);
        console.log(pain);
        let num_cards = pain[0];
        let first_item_len = pain[1] | pain[2] << 8;
        let text_len = pain[3];
        let text_slice = pain.slice(4, 4+text_len);
        let title = decoder.decode(text_slice);
        console.log(num_cards, first_item_len, text_len, text_slice, title);
    });
}

fetch2posts();