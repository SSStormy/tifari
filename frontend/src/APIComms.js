
class TifariAPI {

    constructor(endpoint) {
        this.setEndpoint(endpoint);
    }

    getEndpoint() {
        return this.endpoint.url;
    }

    setEndpoint(endpoint) {
        this.endpoint = {};
        this.endpoint.url = endpoint;
        this.endpoint.search = endpoint + "/api/search";
        this.endpoint.tagQueue = endpoint + "/api/tag_queue";
        this.endpoint.addTags = endpoint + "/api/add_tags";
        this.endpoint.removeTags = endpoint + "/api/remove_tags";
        this.endpoint.getAllTags= endpoint + "/api/get_all_tags";
        this.endpoint.getTagQueueSize = endpoint + "/api/tag_queue_size";
        this.endpoint.reloadRoot = endpoint + "/api/reload";
        this.endpoint.image= endpoint + "/";
    }

    getTagQueueSize() {
        try {
            return fetch(this.endpoint.getTagQueueSize, { method: "GET" })
                    .then(results => results.json())
                    .then(payload => payload.tag_queue_size);
        }
        catch(err) {
            console.error(err);
        }
    }


    getAllTags() {
        try {
            return fetch(this.endpoint.getAllTags, { method: "GET" })
                    .then(results => results.json());
        }
        catch(err) {
            console.error(err);
        }
    }

    reloadRoot() {
        try {
            return fetch(this.endpoint.reloadRoot, { method: "GET" });
        }
        catch(err) {
            console.error(err);
        }
    }

    getToBeTaggedList() {
        try {
            return fetch(this.endpoint.tagQueue, { method: "GET" })
                .then(results => results.json())
        }
        catch(err) {
            console.error(err);
        }

    }
    getImageUrl(img) {
        return this.endpoint.image + img.path;
    }

    search(tags) {
        try {
            return fetch(this.endpoint.search, {
                method: "POST",

                body: JSON.stringify({
                    tags,
                    offset: 0,
                    max: 20
                })
            })
            .then(results => results.json());
        } catch(err) {
            console.error(err);
        }
    }

    addTags(tags, image_ids) {
        try {
            return fetch(this.endpoint.addTags, {
                method: "POST",

                body: JSON.stringify({
                    tags,
                    image_ids,
                })
            })
            .then(results => results.json());
        }
        catch(err) {
            console.error(err);
        }
    }

    removeTags(tags, imgs) {
        try {
            return fetch(this.endpoint.removeTags, {
                method: "POST",

                body: JSON.stringify({
                    tag_ids: tags,
                    image_ids: imgs,
                })
            })
            .then(results => results.json());
        }
        catch(err) {
            console.error(err);
        }
    }
}

export default TifariAPI;
