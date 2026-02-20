export interface WebsiteData {
    id: string;
    title: string;
    url: string;
    snippet: string;
    imageUrl: string; // For the hover preview
}

export const MOCK_WEBSITES: WebsiteData[] = [
    {
        id: "1",
        title: "Wikipedia - The Free Encyclopedia",
        url: "https://en.wikipedia.org/wiki/Main_Page",
        snippet: "Wikipedia is a free online encyclopedia, created and edited by volunteers around the world and hosted by the Wikipedia Foundation.",
        imageUrl: "https://upload.wikimedia.org/wikipedia/commons/6/63/Wikipedia-logo.png"
    },
    {
        id: "2",
        title: "MDN Web Docs",
        url: "https://developer.mozilla.org",
        snippet: "The MDN Web Docs site provides information about Open Web technologies including HTML, CSS, and APIs for both Web sites and progressive web apps.",
        imageUrl: "https://developer.mozilla.org/mdn-social-share.cd6c4a5a.png"
    },
    {
        id: "3",
        title: "Stack Overflow",
        url: "https://stackoverflow.com",
        snippet: "Stack Overflow is the largest, most trusted online community for developers to learn, share their programming knowledge, and build their careers.",
        imageUrl: "https://cdn.sstatic.net/Sites/stackoverflow/Img/apple-touch-icon.png?v=c78bd457575a"
    },
    {
        id: "4",
        title: "GitHub: Let's build from here",
        url: "https://github.com",
        snippet: "GitHub is where over 100 million developers shape the future of software, together. Contribute to the open source community",
        imageUrl: "https://github.githubassets.com/images/modules/logos_page/GitHub-Mark.png"
    },
    {
        id: "5",
        title: "Reddit - Dive into anything",
        url: "https://www.reddit.com",
        snippet: "Reddit is a network of communities where people can dive into their interests, hobbies and passions. There's a community for whatever you're interested in.",
        imageUrl: "https://www.redditinc.com/assets/images/site/reddit-logo.png"
    }
];
