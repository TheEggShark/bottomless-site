const decoder = new TextDecoder()

async function fetch2posts() {
    let res = await fetch("/api/recentBlogPosts?max=2");
    let buffer = await res.arrayBuffer();
    let cbmds = create_cbmd_from_buffer(buffer);
    console.log(cbmds);
}

function create_cbmd_from_buffer(buffer) {
    let cbmds = [];
    let pain = new DataView(buffer);
    let cursor = 0;

    let num_cards = pain.getUint8(cursor, true);
    console.log(num_cards);
    cursor++;

    for (let i = 0; i < num_cards; i++) {
        let first_item_len = pain.getUint16(cursor, true);
        cursor += 2;

        let text_len = pain.getUint8(cursor, true);
        cursor++;
        let text_slice = buffer.slice(cursor, cursor+text_len);
        let title = decoder.decode(text_slice);
        cursor += text_len;

        let intro_len = pain.getUint8(cursor, true);
        cursor++;
        let intro = decoder.decode(buffer.slice(cursor, cursor+intro_len));
        cursor += intro_len;

        let url_len = pain.getUint8(cursor, true);
        cursor++;
        let url_text = decoder.decode(buffer.slice(cursor, cursor + url_len));
        cursor += url_len;

        let publish_ts = get_u64(pain, cursor);
        cursor += 8;
        let publish_date = toDateTime(publish_ts);

        cbmds.push(new Cbmd(title, intro, url_text, publish_date))
    }

    return cbmds;
}

function get_u64(dataview, cursor) {
    const left =  dataview.getUint32(cursor, true);
    const right = dataview.getUint32(cursor+4, true);
  
    // combine the two 32-bit values
    const combined = left + 2**32*right;
    return combined;
}

function toDateTime(secs) {
    var t = new Date(Date.UTC(1970, 0, 1)); // Epoch
    t.setUTCHours(t.getUTCHours() + 6); // all TS were made in CST so this coverts it!
    t.setUTCSeconds(secs);
    return t;
}

fetch2posts();

class Cbmd {
    constructor(title, intro, url, publish_date) {
        this.title = title;
        this.intro = intro;
        this.url = url;
        this.publish_date = publish_date;
    }
}