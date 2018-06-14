function ldebug(msg) {
    if("production" !== process.env.NODE_ENV) {
        if(typeof msg === "string") {
            console.log("[DEBUG] " + msg);
        } else {
            console.log(msg);
        }
    }
}
export { ldebug };
