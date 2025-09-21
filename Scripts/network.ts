import { tryCatch } from "./utils";
import { logs } from "./logs";
const CLOUD_URL = 'https://roblox-proxy.dragmine149.workers.dev';

type EarlierBadge = {
  earliest: number,
  data: [RawBadge?, RawBadge?]
}

type RawBadge = {
  badgeId: number,
  date: number,
}

class Network {
  /**
  * Lets the server process the two badges and return the earliest.
  * @param user_id The userid to get the data for
  * @param old_badge The first "ol" badge
  * @param new_badge The second "new" badge
  * @returns {Promise<{
    earliest: number,
    data: [{ badgeId: number; date: number; }, { badgeId: number; date: number; }]
  }>}
  */
  async getEarlierBadge(user_id: number, old_badge: number, new_badge: number): Promise<EarlierBadge> {
    const url = `${CLOUD_URL}/badges/${user_id}/earliest/${old_badge}/${new_badge}`;
    logs.log(`Sending network request to ${url}`, `network`, 0);
    const response = await tryCatch(fetch(url));

    if (response.error) {
      logs.log(`Failed to fetch badge data. (status: ${response.error.message} ${response.error.cause})`, `network`, 100);
      return {
        earliest: -1, data: []
      }
    }

    if (!response.data.ok) {
      logs.log(`Failed to fetch badge data. (status: ${response.data.status} ${response.data.statusText})`, `network`, 100);
      return {
        earliest: -1, data: []
      }
    }

    const data = await tryCatch(response.data.json() as Promise<EarlierBadge>);
    if (data.error) {
      logs.log(`Failed to fetch badge data. (status: ${response.data.status} ${response.data.statusText})`, `network`, 100);
      return {
        earliest: -1, data: []
      }
    }

    logs.log(`Data received successfully!`)
    return data.data;
  }

  /**
  * Request the server for some data. The data is then streamed back to the client for processing.
  * NOTE: The request from the server is better in the 'application/x-ndjson' format.
  * NOTE: This function assumes a bunch of things, be warned.
  * @param fetch_request The url + data to fetch
  * @param data_received The callback function upon retrieving some data
  */
  async requestStream(fetch_request: Request, data_received: (v: string) => Promise<void>) {
    logs.log(`received stream request: ${fetch_request.url}`, `network`, 0);

    // get the data from server
    const response = await tryCatch(fetch(fetch_request));
    if (response.error) {
      logs.log(`Failed to fetch badge data. (status: ${response.error.message} ${response.error.cause})`, `network`, 100);
      return;
    }

    if (!response.data.ok) {
      logs.log(`Failed to fetch badge data. (status: ${response.data.status} ${response.data.statusText})`, `network`, 100);
      return;
    }

    logs.log(`Creating streamer and starting reading`, `network`, 10);
    // create stuff for reading data
    const reader = (response.data.body as ReadableStream<any>).getReader();
    const decoder = new TextDecoder();
    let buffer = '';

    // read data
    while (true) {
      const { done, value } = await reader.read();

      buffer += decoder.decode(value, { stream: true });

      // line at a time, making sure to keep the last line in buffer just in case of incompletion.
      const lines = buffer.split('\n');
      // the not done here is to allow the for loop to process the last line once the data has been fully received.
      if (!done) buffer = lines.pop() || "";

      for (const line of lines) {
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
    logs.log(`Stream is finished.`, `network`, 100);
  }

  /**
  * Keeps retrying the request forever every 2 seconds until it is successful (when response.ok is true)
  * @param request The request to retry.
  * @returns The successful request.
  */
  async retryTilResult(request: Request) {
    let response: Response = new Response(null, { status: 400 });
    logs.log(`Attempting ${request.url} until sucesed.`, `network`, 0);
    while (!response.ok) {
      response = await fetch(request);
      if (!response.ok) {
        await new Promise((r) => setTimeout(r, 2000));
      }
    }
    logs.log(`${request.url} success!`, `network`, 100);
    return response;
  }
}

const network = new Network();

export { network, CLOUD_URL };
export type { RawBadge };
