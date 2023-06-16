let skip = 0;
let stuff_to_load = true;
const blogCardTemplate = document.getElementById("blog-card");
const blogWrapper = document.getElementById("blog-card-wrapper");
const blogSearchButton = document.getElementById("search-button");
const searchResults = document.getElementById("search-results");
const searchCard = document.getElementById("search-card");
const searchBar = document.getElementById("search-bar");
const innerResults = document.getElementById("inner-results");
const noPosts = document.getElementById("no-posts");
const postError = document.getElementById("post-error");
const noResults = document.getElementById("no-results");
const searchError = document.getElementById("search-error");

let search_results_presnet = false;

searchBar.addEventListener("keypress", (event) => {
    if (event.key == "Enter") {
        event.preventDefault();

        search();
    }
});

window.addEventListener("scroll", () => {
    const {
        scrollTop,
        scrollHeight,
        clientHeight
    } = document.documentElement;

    if ((scrollTop + clientHeight >= scrollHeight) && stuff_to_load) {
        get_posts();
    }
});

addEventListener("resize", (e) => {
    update_results_hieght();
});

get_posts();

async function get_posts() {
    const res = await fetch(`/api/recentBlogPosts?skip=${skip}&max=10`);

    if (!res.ok) {
        console.log(res);
        return post_list_error(res);
    }

    const buffer = await res.arrayBuffer();

    const cmbds = create_cbmd_from_buffer(buffer);

    if (cmbds.length == 0) {
        stuff_to_load = false;
        return;
    }

    noPosts.style.display = "none";

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
        publish_date.innerText = `${data.publish_date.getMonth()+1}/${data.publish_date.getDate()}/${data.publish_date.getFullYear()}`;;
        intro.innerText = data.intro;
        link.href = data.url;
        button.href = data.url;

        blogWrapper.appendChild(clone);
    };
}

function post_list_error(res) {
    noPosts.style.display = "none";
    const clone = postError.content.firstElementChild.cloneNode(true);
    const text = clone.getElementsByTagName("h2")[0];
    if (res.status == 429) {
        text.innerText = "You're making too many requests";
    } else {
        text.innerText = `Something went wrong (${res.status})`;
    }


    blogWrapper.appendChild(clone);
}

async function search() {
    const res = await fetch(`/api/searchBlog?title=${searchBar.value}`);
    innerResults.innerHTML = '';
    if (!res.ok) {
        return search_error(res);
    }

    const buffer = await res.arrayBuffer();
    const cmbds = create_cbmd_from_buffer(buffer);

    innerResults.innerHTML = '';

    if (cmbds.length == 0) {
        return no_results();
    }

    for (let i = 0; i < cmbds.length; i++) {
        const data = cmbds[i];
        const clone = searchCard.content.firstElementChild.cloneNode(true);

        const title = clone.getElementsByTagName("h3")[0];
        const publish_date = clone.getElementsByTagName("p")[0];
        const intro = clone.getElementsByTagName("p")[1];
        const link = clone.getElementsByTagName("a")[0];

        title.innerText = data.title;
        publish_date.innerText = `${data.publish_date.getMonth()+1}/${data.publish_date.getDate()}/${data.publish_date.getFullYear()}`;
        intro.innerText = data.intro;
        link.href = data.url;

        innerResults.appendChild(clone);
    }

    searchResults.style.padding = "20px";
    searchResults.style.maxHeight = searchResults.scrollHeight + "px";
    search_results_presnet = true;
}

function search_error(res) {
    const clone = searchError.content.firstElementChild.cloneNode(true);
    const text = clone.getElementsByTagName("h3")[0];
    
    if (res.status == 429) {
        text.innerText = "You're making too many requests";
    } else {
        text.innerText = `Something went wrong (${res.status})`;
    }

    const ded1 = new Image();
    // image loading messes this up
    ded1.onload = () => {
        searchResults.style.padding = "20px";
        searchResults.style.maxHeight = searchResults.scrollHeight + "px";
    }
    clone.prepend(ded1);
    
    innerResults.appendChild(clone);
    ded1.src = "images/ded1.png";
}

function update_results_hieght() {
    if (search_results_presnet) {
        searchResults.style.maxHeight = searchResults.scrollHeight + "px";
    }
}

function no_results() {
    const clone = noResults.content.firstElementChild.cloneNode(true);
    innerResults.appendChild(clone);
    searchResults.style.padding = "20px";
    searchResults.style.maxHeight = searchResults.scrollHeight + "px";
}

function clear_results() {
    searchResults.style.padding = 0;
    searchResults.style.maxHeight = 0;
    search_results_presnet = false;
}