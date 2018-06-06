const ENDPOINT_API_SEARCH               = "http://localhost:8001/api/search";
const ENDPOINT_API_TAG_QUEUE            = "http://localhost:8001/api/tag_queue";
const ENDPOINT_API_ADD_TAGS             = "http://localhost:8001/api/add_tags";
const ENDPOINT_API_REMOVE_TAGS          = "http://localhost:8001/api/remove_tags";
const ENDPOINT_API_GET_ALL_TAGS         = "http://localhost:8001/api/get_all_tags";
const ENDPOINT_API_GET_TAG_QUEUE_SIZE   = "http://localhost:8001/api/tag_queue_size";
const ENDPOINT_API_RELOAD_ROOT          = "http://localhost:8001/api/reload";
const ENDPOINT_API_IMAGE                = "http://localhost:8001/";

class TifariAPI {

    static getTagQueueSize() {
        try {
            return fetch(ENDPOINT_API_GET_TAG_QUEUE_SIZE, { method: "GET" })
                    .then(results => results.json())
                    .then(payload => payload.tag_queue_size);
        }
        catch(err) {
            console.error(err);
        }
    }


    static getAllTags() {
        try {
            return fetch(ENDPOINT_API_GET_ALL_TAGS, { method: "GET" })
                    .then(results => results.json());
        }
        catch(err) {
            console.error(err);
        }
    }

    static reloadRoot() {
        try {
            return fetch(ENDPOINT_API_RELOAD_ROOT, { method: "GET" });
        }
        catch(err) {
            console.error(err);
        }
    }

    static getToBeTaggedList() {
        try {
            return fetch(ENDPOINT_API_TAG_QUEUE, { method: "GET" })
                .then(results => results.json())
        }
        catch(err) {
            console.error(err);
        }

    }
    static getImageUrl(img) {
        return ENDPOINT_API_IMAGE + img.path;
    }

    static search(tags) {
        try {
            return fetch(ENDPOINT_API_SEARCH, {
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

    static addTags(img, tags) {
        try {
            return fetch(ENDPOINT_API_ADD_TAGS, {
                method: "POST",

                body: JSON.stringify({
                    tags: tags,
                    image_id: img.id
                })
            })
            .then(results => results.json());
        }
        catch(err) {
            console.error(err);
        }
    }

    static removeTags(tags, imgs) {
        try {
            return fetch(ENDPOINT_API_REMOVE_TAGS, {
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
