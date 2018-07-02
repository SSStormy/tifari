
class TifariAPI {

    constructor(endpoint, callbackSuccess, callbackErr) {
        this.setEndpoint(endpoint);
        this.callbackSuccess = callbackSuccess;
        this.callbackErr = callbackErr;

        this.errHandle = this.errHandle.bind(this);
        this.onSuccess= this.onSuccess.bind(this);
    }

    getEndpoint() {
        return this.endpoint.url;
    }

    setEndpoint(endpoint) {
        this.endpoint = {};
        this.endpoint.url = endpoint;
        this.endpoint.search = endpoint + "/api/v1/search";
        this.endpoint.tagQueue = endpoint + "/api/v1/tag_queue";
        this.endpoint.addTags = endpoint + "/api/v1/add_tags";
        this.endpoint.removeTags = endpoint + "/api/v1/remove_tags";
        this.endpoint.getAllTags= endpoint + "/api/v1/get_all_tags";
        this.endpoint.getTagQueueSize = endpoint + "/api/v1/tag_queue_size";
        this.endpoint.reloadRoot = endpoint + "/api/v1/reload";
        this.endpoint.config= endpoint + "/api/v1/config";
        this.endpoint.image= endpoint + "/";
        this.endpoint.configStatus = endpoint + "/api/v1/config_status";
    }

    errHandle(err) {
        this.callbackErr();
    }

    onSuccess(val) {
        this.callbackSuccess();
        return val;
    }

    getConfigStatus() {
        return this.doRequest(() => 
            fetch(this.endpoint.configStatus, { method: "GET"})
                .then(results => results.json())
        );
    }

    doRequest(fx) {
        try {
            let ret = fx();
            ret.then(this.onSuccess);
            ret.catch(this.errHandle);
            return ret;
        }
        catch(err) {
            this.errHandle(err);
        }
    }

    getConfig() {
        return this.doRequest(() => 
            fetch(this.endpoint.config, { method: "GET" })
                    .then(results => results.json())
        );
    }

    setConfig(cfg) {
        return this.doRequest(() => 
            fetch(this.endpoint.config, { 
                method: "POST",
                body: JSON.stringify(cfg),
            })
            .then(results => results.json())
        );
    }

    getTagQueueSize() {
        return this.doRequest(() => 
            fetch(this.endpoint.getTagQueueSize, { method: "GET" })
                    .then(results => results.json())
                    .then(payload => payload.tag_queue_size)
        );
    }


    getAllTags() {
        return this.doRequest(() => 
            fetch(this.endpoint.getAllTags, { method: "GET" })
               .then(results => results.json())
        );
    }

    reloadRoot() {
        return this.doRequest(() => 
            fetch(this.endpoint.reloadRoot, { method: "GET" })
        );
    }

    getToBeTaggedList() {
        return this.doRequest(() => 
            fetch(this.endpoint.tagQueue, { method: "GET" })
            .then(results => results.json())
        );
    }
    getImageUrl(img) {
        return this.endpoint.image + img.path;
    }

    search(tags) {
        return this.doRequest(() => 
            fetch(this.endpoint.search, {
                method: "POST",
                body: JSON.stringify(tags)
            })
            .then(results => results.json())
        );
    }

    addTags(tags, image_ids) {
        return this.doRequest(() => 
            fetch(this.endpoint.addTags, {
                method: "POST",

                body: JSON.stringify({
                    tags,
                    image_ids,
                })
            })
            .then(results => results.json())
        );
    }

    removeTags(tags, imgs) {
        return this.doRequest(() => 
            fetch(this.endpoint.removeTags, {
                method: "POST",

                body: JSON.stringify({
                    tag_ids: tags,
                    image_ids: imgs,
                })
            })
            .then(results => results.json())
        );
    }
}

export default TifariAPI;
