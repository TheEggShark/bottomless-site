let skip = 0;
let stuff_to_load = true;
const blogCardTemplate = document.getElementById("blog-card");
const blogWrapper = document.getElementById("blog-card-wrapper");
const blogSearchButton = document.getElementById("search-button");
const searchResults = document.getElementById("search-results");
const searchCard = document.getElementById("search-card");
const searchBar = document.getElementById("search-bar");
const innerResults = document.getElementById("inner-results");

searchBar.addEventListener("keypress", (event) => {
    if (event.key == "Enter") {
        event.preventDefault();

        search();
    }
})


get_posts();

async function get_posts() {
    const res = await fetch(`/api/recentBlogPosts?skip=${skip}&max=10`);

    if (!res.ok) {
        console.log(res);
        return;
    }

    const buffer = await res.arrayBuffer();

    const cmbds = create_cbmd_from_buffer(buffer);

    if (cmbds.length == 0) {
        stuff_to_load = false;
        return;
    }

    skip += cmbds.length;

    for (let i = 0; i < cmbds.length; i++) {
        const data = cmbds[i];

        const clone = blogCardTemplate.content.firstElementChild.cloneNode(true);
        const title = clone.getElementsByTagName("h2")[0];
        const publish_date = clone.getElementsByTagName("p")[0];
        const intro = clone.getElementsByTagName("p")[1];
        const link = clone.getElementsByTagName("a")[0];
        const button = clone.getElementsByTagName("a")[1];

        title.innerText = data.title;
        publish_date.innerText = `${data.publish_date.getDate()}/${data.publish_date.getMonth()}/${data.publish_date.getFullYear()}`;;
        intro.innerText = data.intro;
        link.href = data.url;
        button.href = data.url;

        blogWrapper.appendChild(clone);
    };
}

async function search() {
    const res = await fetch(`/api/searchBlog?title=${searchBar.value}`);
    const buffer = await res.arrayBuffer();
    const cmbds = create_cbmd_from_buffer(buffer);

    innerResults.innerHTML = '';

    for (let i = 0; i < cmbds.length; i++) {
        const data = cmbds[i];
        const clone = searchCard.content.firstElementChild.cloneNode(true);

        const title = clone.getElementsByTagName("h3")[0];
        const publish_date = clone.getElementsByTagName("p")[0];
        const intro = clone.getElementsByTagName("p")[1];
        const link = clone.getElementsByTagName("a")[0];

        title.innerText = data.title;
        publish_date.innerText = `${data.publish_date.getDate()}/${data.publish_date.getMonth()}/${data.publish_date.getFullYear()}`;
        intro.innerText = data.intro;
        link.href = data.url;

        innerResults.appendChild(clone);
    }

    searchResults.style.padding = "10px";
    searchResults.style.maxHeight = searchResults.scrollHeight + "px";
}

function clear_results() {
    searchResults.style.padding = 0;
    searchResults.style.maxHeight = 0;
}

window.addEventListener("scroll", () => {
    const {
        scrollTop,
        scrollHeight,
        clientHeight
    } = document.documentElement;

    if ((scrollTop + clientHeight >= scrollHeight) && stuff_to_load) {
        get_posts();
    }
})