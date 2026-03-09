// terser -c -m --module tests/fixtures/private-method/original.js --source-map includeSources -o tests/fixtures/private-method/minified.js
export class ApiConnector {

    #baseUrl = "https://api.example.com/";

    async #makeRequest(options) {
        let response = await fetch(options);
        if (!response.ok) {
            throw new Error("Failed to fetch!");
        }
        return response;
    }

    #buildUrl(path) {
        return this.#baseUrl + path;
    }

    async get(url) {
        return this.#makeRequest({
            url: this.#buildUrl(url),
            method: "GET"
        });
    }
}
