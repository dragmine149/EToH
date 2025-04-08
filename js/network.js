class Network {
  constructor() {
    this.verbose = new Verbose("Network", "#aa8323");
  }

  /**
  * Request the server for some data. The data is then streamed back to the client for processing.
  * NOTE: The request from the server is better in the 'application/x-ndjson' format.
  * NOTE: This function assumes a bunch of things, be warned.
  * @param {Request} fetch_request The url + data to fetch
  * @param {(v: string) => Promise<any>} data_received The callback function upon retrieving some data
  */
  async requestStream(fetch_request, data_received) {
    this.verbose.log(`received stream request:`, fetch_request);

    // get the data from server
    let response = await fetch(fetch_request);
    if (!response.ok) {
      let errorText = await response.text();
      showNotification(`Failed to fetch badge data. (status: ${response.status} ${response.statusText}\n${errorText})`, true);
      return;
    }

    // create stuff for reading data
    let reader = response.body.getReader();
    let decoder = new TextDecoder();
    let buffer = '';

    // read data
    while (true) {
      let { done, value } = await reader.read();

      buffer += decoder.decode(value, { stream: true });

      // line at a time, making sure to keep the last line in buffer just in case of incompletion.
      let lines = buffer.split('\n');
      // the not done here is to allow the for loop to process the last line once the data has been fully received.
      if (!done) buffer = lines.pop() || "";

      for (let line of lines) {
        // incase a line is incomplete.
        if (!line.trim()) {
          continue;
        }

        // let the user process the data
        await data_received(line);
      }

      // escape the loop.
      if (done) break;
    }
  }
}

let network = new Network();
