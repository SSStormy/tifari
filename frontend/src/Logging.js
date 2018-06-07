function ldebug(msg) {
    if("production" !== process.env.NODE_ENV) {
        if(typeof msg === "string") {
            console.log("[DEBUG] " + msg);
        } else {
            console.log(msg);
        }
    }
}

function assert(expr) {
    if(!expr) throw new Error("Assertion failed");

}

export { ldebug, assert };
