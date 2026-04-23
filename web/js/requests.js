/**
 * Utility class for making HTTP requests.
 */
export class Requests {
  /**
   * Creates a new Requests instance.
   *
   * @param {Object} options - The configuration options.
   * @param {string} options.url - The base URL for the request.
   * @param {Record<string, unknown>} [options.params] - Optional query parameters.
   * @param {Array<{key: string, value: string}>} [options.headers] - Optional headers.
   */
  constructor({ url, params, headers }) {
    /** @type {string} */
    this.url = url;
    /** @type {Record<string, unknown>|undefined} */
    this.params = params;
    /** @type {Array<{key: string, value: string}>|undefined} */
    this.headers = headers;
  }

  // ── JSON ──────────────────────────────────────────────────────────────────

  /**
   * Performs a GET request.
   *
   * @returns {Promise<any>} The response data.
   */
  async get() {
    return this.fetch("GET");
  }

  /**
   * Performs a POST request with JSON payload.
   *
   * @param {object} body - The payload to send.
   * @returns {Promise<any>} The response data.
   */
  async post(body) {
    return this.fetch("POST", JSON.stringify(body));
  }

  /**
   * Performs a PATCH request with JSON payload.
   *
   * @param {object} body - The payload to send.
   * @returns {Promise<any>} The response data.
   */
  async patch(body) {
    return this.fetch("PATCH", JSON.stringify(body));
  }

  /**
   * Performs a DELETE request.
   *
   * @returns {Promise<any>} The response data.
   */
  async delete() {
    return this.fetch("DELETE");
  }

  // ── Form ──────────────────────────────────────────────────────────────────

  /**
   * Performs a POST request with form data.
   *
   * @param {HTMLFormElement | FormData} form - The form element or FormData to send.
   * @returns {Promise<any>} The response data.
   */
  async form(form) {
    const body = form instanceof FormData ? form : new FormData(form);
    return this.fetch("POST", body, false);
  }

  // ── Internal ──────────────────────────────────────────────────────────────

  /**
   * Internal fetch wrapper.
   *
   * @param {string} method - The HTTP method to use.
   * @param {BodyInit} [body] - The request body.
   * @param {boolean} [json=true] - Whether the payload is JSON.
   * @returns {Promise<any>} The response data.
   * @private
   */
  async fetch(method, body, json = true) {
    this.mount_params();

    const headers = this.mount_headers();
    if (json && body) headers["Content-Type"] = "application/json";

    const res = await fetch(this.url, { method, headers, body });

    if (!res.ok) throw await res.json().catch(() => res.text());

    return res.json();
  }

  /**
   * Mounts the query parameters into the URL.
   *
   * @private
   */
  mount_params() {
    if (!this.params) return;
    for (const [k, v] of Object.entries(this.params)) {
      const sep = this.url.includes("?") ? "&" : "?";
      this.url += `${sep}${k}=${v}`;
    }
  }

  /**
   * Mounts the headers array into a Record.
   *
   * @returns {Record<string, string>} The mapped headers.
   * @private
   */
  mount_headers() {
    return Object.fromEntries(
      this.headers?.map(({ key, value }) => [key, value ?? ""]) ?? [],
    );
  }
}
