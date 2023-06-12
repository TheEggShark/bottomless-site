const miniBlogTemplate = document.getElementById("mini-card-template");
const miniBlogHolder = document.getElementById("blog-card-holder");
const miniBlogError = document.getElementById("mini-card-error");

async function fetch2posts() {
    const res = await fetch("/api/recentBlogPosts?max=2");

    if (!res.ok) {
        return mini_card_error(res);
    }

    const buffer = await res.arrayBuffer();
    const cbmds = create_cbmd_from_buffer(buffer);

    for (let i = 0; i < cbmds.length; i++) {
        const data = cbmds[i];
        const clone = miniBlogTemplate.content.firstElementChild.cloneNode(true);

        const title_text = clone.getElementsByTagName("h3")[0];
        const date_text = clone.getElementsByTagName("p")[0];
        const intro_text = clone.getElementsByTagName("p")[1];

        title_text.innerText = data.title;
        intro_text.innerText = data.intro;
        date_text.innerText = `${data.publish_date.getDate()}/${data.publish_date.getMonth()}/${data.publish_date.getFullYear()}`;

        miniBlogHolder.appendChild(clone);

        console.log(data, clone, title_text, date_text, intro_text);
    }

    const blog_link = document.createElement("a");
    blog_link.href = "/blog";
    blog_link.innerText = "see all posts ->";

    miniBlogHolder.appendChild(blog_link);
}

function mini_card_error(res) {
    console.log(res);
    const template = miniBlogError.content.firstElementChild.cloneNode(true);

    const error_text = template.getElementsByTagName("h3")[0];
    error_text.innerText = `Something Went Wrong (${res.status})`;

    miniBlogHolder.appendChild(template);
}

fetch2posts();