// terser -c -m --module tests/fixtures/private-method/original.js --source-map includeSources -o tests/fixtures/private-method/minified.js
export class ApiConnector {
    async #makeRequest(options) {
        let response = await fetch(options);
        return response;
    }

    #buildUrl(path) {
        return "https://api.example.com/" + path;
    }

    async get(url) {
        return this.#makeRequest({ url: this.#buildUrl(url), method: "GET" });
    }
}
